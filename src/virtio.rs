use core::{
    mem::{size_of, MaybeUninit},
    ptr::null_mut,
    sync::atomic::AtomicPtr,
};

use crate::{
    kmem::{kalloc, PAGE_SIZE, VIRTIO_BASES},
    mmio::MMIODevice,
    string::memset,
};

/// Driver for VirtIO over MMIO. Supports block devices.
/// https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.pdf

#[repr(C)]
struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
struct Available {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; VIRTIO_QUEUE_LEN],
}

#[repr(C)]
struct UsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
struct Used {
    pub flags: u16,
    pub idx: u16,
    pub ring: [UsedElem; VIRTIO_QUEUE_LEN],
}

#[repr(C)]
struct Queue {
    pub num: usize,
    pub desc: *mut Descriptor,
    pub avail: *mut Available,
    pub used: *mut Used,
}

struct Device {
    pub queue: Queue,
}

#[repr(u32)]
enum VirtIODeviceId {
    Network = 1,
    Block = 2,
    Console = 3,
    Entropy = 4,
    Balloon = 5,
    SCSI = 8,
    GPU = 16,
    Input = 18,
    Crypto = 20,
    Socket = 19,
    FS = 26,
    RPMB = 28,
    IOMMU = 23,
    Sound = 25,
    Memory = 24,
    I2C = 34,
    SCMI = 32,
    GPIO = 41,
    PMEM = 27,
}

static mut VIRTIO_DEVICES: [MaybeUninit<Device>; 8] =
    unsafe { MaybeUninit::uninit().assume_init() };

const VIRTIO_MAGIC: u32 = 0x74_72_69_76;
const VIRTIO_VERSION: u32 = 2;
const VIRTIO_QUEUE_LEN: usize = 8; // use a constant queue length for all devices for simplicity

// 2.1 Device Status Field
const STATUS_ACKNOWLEDGE: u32 = 1;
const STATUS_DRIVER: u32 = 2;
const STATUS_FAILED: u32 = 128;
const STATUS_FEATURES_OK: u32 = 8;
const STATUS_DRIVER_OK: u32 = 4;
const STATUS_DEVICE_NEEDS_RESET: u32 = 64;

// 5.2.3 Feature bits (block device)
const VIRTIO_BLK_F_RO: u32 = 1 << 5;
const VIRTIO_BLK_F_CONFIG_WCE: u32 = 1 << 11;
const VIRTIO_BLK_F_MQ: u32 = 1 << 12;

pub fn init() {
    assert!(size_of::<Descriptor>() == 16);
    assert!(size_of::<Available>() <= PAGE_SIZE as usize);
    assert!(size_of::<Used>() <= PAGE_SIZE as usize);
    for (i, addr) in VIRTIO_BASES.iter().enumerate() {
        let mmio = MMIODevice::<u32>::new(addr.clone());
        let magic: u32;
        let version: u32;
        let device_id: u32;
        unsafe {
            magic = mmio.reg_r(0x000).read();
            version = mmio.reg_r(0x004).read();
            device_id = mmio.reg_r(0x008).read();
        }
        if (magic == VIRTIO_MAGIC) && (version == VIRTIO_VERSION) && (device_id != 0) {
            match unsafe { core::mem::transmute::<u32, VirtIODeviceId>(device_id) } {
                VirtIODeviceId::Block => setup_block_device(mmio, i),
                _ => debug!("Unknown/unsupported VirtIO device {}", device_id),
            }
        }
    }
}

fn setup_block_device(mmio: MMIODevice<u32>, index: usize) {
    // 3.1.1 Driver Requirements: Device Initialization
    unsafe {
        let device_features_reg = mmio.reg_r(0x010);
        let driver_features_reg = mmio.reg_w(0x020);
        let queue_sel_reg = mmio.reg_w(0x030);
        let queue_num_max_reg = mmio.reg_r(0x034);
        let queue_num_reg = mmio.reg_w(0x038);
        let queue_ready_reg = mmio.reg_rw(0x044);
        let status_reg = mmio.reg_rw(0x070);
        let queue_desc_l_reg = mmio.reg_w(0x080);
        let queue_desc_h_reg = mmio.reg_w(0x084);
        let queue_driver_l_reg = mmio.reg_w(0x090);
        let queue_driver_h_reg = mmio.reg_w(0x094);
        let queue_device_l_reg = mmio.reg_w(0x0a0);
        let queue_device_h_reg = mmio.reg_w(0x0a4);

        macro_rules! virtio_fail {
            ($($args:tt)*) => {
                debug!($($args)*);
                status_reg.write(STATUS_FAILED);
                return;
            };
        }

        let mut status = 0;
        status_reg.write(status); // reset device
        status |= STATUS_ACKNOWLEDGE;
        status_reg.write(status);
        status |= STATUS_DRIVER;
        status_reg.write(status);
        let device_features = device_features_reg.read();
        let driver_features =
            device_features & !VIRTIO_BLK_F_RO & !VIRTIO_BLK_F_CONFIG_WCE & !VIRTIO_BLK_F_MQ; // negotiate features
        driver_features_reg.write(driver_features);
        status |= STATUS_FEATURES_OK;
        status_reg.write(status);
        if status_reg.read() & STATUS_FEATURES_OK == 0 {
            // device rejected driver features
            virtio_fail!("Features not supported");
        }
        // 4.2.3.2 Virtqueue Configuration
        queue_sel_reg.write(0);
        if queue_ready_reg.read() != 0 {
            virtio_fail!("Queue is in use");
        }
        let queue_num_max = queue_num_max_reg.read();
        if queue_num_max == 0 {
            virtio_fail!("Queue is not available");
        }
        if queue_num_max < VIRTIO_QUEUE_LEN as u32 {
            virtio_fail!("Queue length {} not supported", VIRTIO_QUEUE_LEN);
        }

        // allocate and zero queue memory
        let queue = Queue {
            num: VIRTIO_QUEUE_LEN,
            desc: kalloc() as *mut Descriptor,
            avail: kalloc() as *mut Available,
            used: kalloc() as *mut Used,
        };
        if queue.desc == null_mut() || queue.avail == null_mut() || queue.used == null_mut() {
            virtio_fail!("Queue length {} not supported", VIRTIO_QUEUE_LEN);
        }
        memset(queue.desc, 0, PAGE_SIZE as usize);
        memset(queue.avail, 0, PAGE_SIZE as usize);
        memset(queue.used, 0, PAGE_SIZE as usize);

        queue_desc_l_reg.write((queue.desc as u64 & 0xFFFF_FFFF) as u32);
        queue_desc_h_reg.write((queue.desc as u64 >> 32) as u32);
        queue_driver_l_reg.write((queue.avail as u64 & 0xFFFF_FFFF) as u32);
        queue_driver_h_reg.write((queue.avail as u64 >> 32) as u32);
        queue_device_l_reg.write((queue.used as u64 & 0xFFFF_FFFF) as u32);
        queue_device_h_reg.write((queue.used as u64 >> 32) as u32);

        let device = Device { queue };
        VIRTIO_DEVICES[index].write(device);

        queue_num_reg.write(VIRTIO_QUEUE_LEN as u32);

        queue_ready_reg.write(0x1);

        status |= STATUS_DRIVER_OK;
        status_reg.write(status);
    }
}
