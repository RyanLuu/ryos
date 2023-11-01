use crate::csr::{SATP_MODE, SATP_MODE_SV39, SATP_PPN};
use crate::kmem::{
    self, kalloc, kfree, BSS_END, BSS_START, CLINT_BASE, DATA_END, DATA_START, PAGE_SIZE,
    PLIC_BASE, RODATA_END, RODATA_START, STACK_END, STACK_START, TEXT_END, TEXT_START, UART_BASE,
    VIRTIO_BASES,
};
use crate::{csr_write, csr_write_field, page_ceil, page_floor, page_number};

/// Sv39 memory management unit.
use core::{mem::size_of, ptr::null_mut};

/// Convert physical address to PPN field of PTE
macro_rules! paddr2pte {
    ($paddr:expr) => {
        (($paddr as u64) >> 12) << 10
    };
}

/// Convert PPN field of PTE to physical address
macro_rules! pte2paddr {
    ($pte:expr) => {
        ($pte >> 10) << 12
    };
}

const PAGE_TABLE_SIZE: usize = 512; // number of PTEs in a PageTable
static mut PAGE_TABLE: *mut PageTable = null_mut(); // root PageTable

pub struct PageTable {
    pub entries: [PTE; PAGE_TABLE_SIZE],
}

// page table entry
pub struct PTE {
    entry: u64,
}

static mut INITIALIZED: bool = false;

pub fn init() {
    unsafe {
        debug!("Initializing Sv39 page table");
        assert!(size_of::<PageTable>() as u64 <= PAGE_SIZE);
        PAGE_TABLE = kalloc() as *mut PageTable;
        PAGE_TABLE.write_bytes(0x00, 1);

        if !kmem::initialized() {
            debug!("kmem must be initialized before mmu");
            return;
        }

        debug!("adding mappings for kernel memory allocations");
        (*PAGE_TABLE).map_range(TEXT_START, TEXT_START, TEXT_END, PTE_R | PTE_X);
        (*PAGE_TABLE).map_range(RODATA_START, RODATA_START, RODATA_END, PTE_R | PTE_X);
        (*PAGE_TABLE).map_range(DATA_START, DATA_START, DATA_END, PTE_R | PTE_W);
        (*PAGE_TABLE).map_range(BSS_START, BSS_START, BSS_END, PTE_R | PTE_W);
        (*PAGE_TABLE).map_range(STACK_START, STACK_START, STACK_END, PTE_R | PTE_W);
        (*PAGE_TABLE).map(UART_BASE, UART_BASE, PTE_R | PTE_W, 0);
        for base in VIRTIO_BASES {
            (*PAGE_TABLE).map_range(base, base, base + 0x1000, PTE_R | PTE_W);
        }
        (*PAGE_TABLE).map_range(CLINT_BASE, CLINT_BASE, CLINT_BASE + 0x1_0000, PTE_R | PTE_W);
        (*PAGE_TABLE).map_range(PLIC_BASE, PLIC_BASE, PLIC_BASE + 0x40_0000, PTE_R | PTE_W);

        // update SATP to enable virtual memory
        csr_write_field!(satp, SATP_MODE, SATP_MODE_SV39);
        csr_write_field!(satp, SATP_PPN, page_number!(PAGE_TABLE as u64));

        INITIALIZED = true;
    }
}

pub fn initialized() -> bool {
    unsafe { INITIALIZED }
}

/// Two-level Sv39 page table
impl PageTable {
    /// Upserts a mapping from a virtual address to a physical address.
    ///
    /// Properties of the newly mapped page can be set via `flags`. One of PTE_R, PTE_W, and
    /// PTE_X must be set. The size of the page is controlled via `level`: 0 for 4KiB, 1 for
    /// 2MiB, and 2 for 1GiB.
    pub fn map(&mut self, vaddr: u64, paddr: u64, flags: u64, level: usize) {
        assert!(
            (flags & PTE_PBMT == 0) &&  // Svpbmt not implemented
            (flags & PTE_RESERVED == 0) && // reserved for future standard use
            (flags & PTE_PPN == 0) // flags should not specify PPN
        );
        assert!(level == 0 || level == 1 || level == 2); // level is valid
        assert!(flags & PTE_RWX != 0); // flags indicate leaf

        // extract virtual page numbers from vaddr
        let vpn = [
            (vaddr >> 12) & 0x01ff, // vaddr[20:12] (9 bits)
            (vaddr >> 21) & 0x01ff, // vaddr[29:21] (9 bits)
            (vaddr >> 30) & 0x01ff, // vaddr[38:30] (9 bits)
        ];
        let mut pte = &mut self.entries[vpn[2] as usize];
        // navigate to new leaf position
        for l in (level..2).rev() {
            if !pte.is_valid() {
                let page = kalloc();
                unsafe {
                    (page as *mut PageTable).write_bytes(0x00, 1);
                }
                pte.set_ppn(paddr2pte!(page)).validate();
            }
            let entry = pte2paddr!(pte.get_ppn()) as *mut PTE;
            pte = unsafe { entry.add(vpn[l] as usize).as_mut().unwrap() };
        }

        // check if leaf is already mapped
        let old_ppn = pte.get_ppn();
        let new_ppn = paddr2pte!(paddr);
        if old_ppn != 0 && old_ppn != new_ppn {
            debug!(
                "Overwriting vaddr 0x{:x} mapping 0x{:x} -> 0x{:x}",
                vaddr,
                pte2paddr!(old_ppn),
                paddr
            );
        }

        // set leaf value
        pte.set_ppn(new_ppn).set_flags(flags).validate();
    }

    /// Add the necessary 4KB page mappings to map the address range `[paddr, paddr + len)` to [vaddr, vaddr + len)
    fn map_range(&mut self, mut vaddr: u64, paddr_start: u64, paddr_end: u64, flags: u64) {
        assert!(paddr_end > paddr_start);
        let paddr_range = page_floor!(paddr_start)..page_ceil!(paddr_end);
        for paddr in paddr_range.step_by(PAGE_SIZE as usize) {
            self.map(vaddr, paddr, flags, 0);
            vaddr += PAGE_SIZE;
        }
    }

    /// Convert a virtual address to a physical address.
    fn lookup(&self, vaddr: u64) -> Option<u64> {
        // extract virtual page numbers from vaddr
        let vpn = [
            (vaddr >> 12) & 0x01ff, // vaddr[20:12] (9 bits)
            (vaddr >> 21) & 0x01ff, // vaddr[29:21] (9 bits)
            (vaddr >> 30) & 0x01ff, // vaddr[38:30] (9 bits)
        ];
        let mut pte = &self.entries[vpn[2] as usize];
        // navigate to new leaf position
        for l in (0..=2).rev() {
            if !pte.is_valid() {
                return None;
            }
            if pte.is_leaf() {
                let offset_mask: u64 = !(!0 << (12 + 9 * l));
                let page = pte2paddr!(pte.get_ppn());
                return Some(page | (vaddr & offset_mask));
            }
            let entry = pte2paddr!(pte.get_ppn()) as *mut PTE;
            pte = unsafe { entry.add(vpn[l - 1] as usize).as_mut().unwrap() };
        }
        None
    }

    pub fn free(&mut self) {
        for i in 0..PAGE_TABLE_SIZE {
            let pte = &self.entries[i];
            if pte.is_valid() && !pte.is_leaf() {
                let child = pte2paddr!(pte.get_ppn()) as *mut PageTable;
                unsafe {
                    (*child).free();
                }
            }
        }
        kfree(self as *mut _ as *mut u8);
    }
}

pub const PTE_VALID: u64 = 1 << 0;
pub const PTE_R: u64 = 1 << 1;
pub const PTE_W: u64 = 1 << 2;
pub const PTE_X: u64 = 1 << 3;
pub const PTE_USER: u64 = 1 << 4;
pub const _PTE_GLOBAL: u64 = 1 << 5;
pub const _PTE_ACCESSED: u64 = 1 << 6;
pub const _PTE_DIRTY: u64 = 1 << 7;
pub const _PTE_RSW: u64 = 0b11 << 8;
pub const PTE_PPN: u64 = 0xfffffffffff << 10;
pub const PTE_RESERVED: u64 = 0b111_1111 << 54;
pub const PTE_PBMT: u64 = 0b11 << 61;
pub const _PTE_NAPOT: u64 = 1 << 63;
pub const PTE_RWX: u64 = PTE_R | PTE_W | PTE_X;

/// Page table entry
impl PTE {
    pub fn validate(&mut self) {
        self.entry |= PTE_VALID;
    }

    pub fn is_valid(&self) -> bool {
        self.entry & PTE_VALID != 0
    }

    pub fn is_leaf(&self) -> bool {
        self.entry & PTE_RWX != 0
    }

    pub fn set_ppn(&mut self, ppn: u64) -> &mut Self {
        assert!(ppn & !PTE_PPN == 0);
        self.entry = (self.entry & !PTE_PPN) | ppn;
        self
    }

    pub fn get_ppn(&self) -> u64 {
        self.entry & PTE_PPN
    }

    pub fn set_flags(&mut self, flags: u64) -> &mut Self {
        self.entry |= flags;
        self
    }
}
