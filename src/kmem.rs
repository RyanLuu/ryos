/// Page-level physical memory allocation
///
/// The kernel heap is divided into 4K pages and managed by a free list composed of the pages
/// themselves. Pages can be allocated and freed one at a time using `kalloc()` and `kfree()`.
use core::ptr::null_mut;

// expose memory layout constants defined in mem.s
extern "C" {
    pub static TEXT_START: u64;
    pub static TEXT_END: u64;
    pub static RODATA_START: u64;
    pub static RODATA_END: u64;
    pub static DATA_START: u64;
    pub static DATA_END: u64;
    pub static BSS_START: u64;
    pub static BSS_END: u64;
    pub static STACK_START: u64;
    pub static STACK_END: u64;
    pub static HEAP_START: u64;
    pub static HEAP_END: u64;
}

// more memory layout constants, found in the .dts file generated by `./qemu-ryos.sh -d`
pub const CLINT_BASE: u64 = 0x0200_0000;
pub const PLIC_BASE: u64 = 0x0C00_0000;
pub const UART_BASE: u64 = 0x1000_0000;
pub const VIRTIO_BASES: [u64; 8] = [
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
    0x1000_1000,
];

pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_OFFSET_MASK: u64 = PAGE_SIZE - 1;
pub const PAGE_NUMBER_MASK: u64 = !PAGE_OFFSET_MASK;

/// Round address down to the nearest PAGE_SIZE
#[macro_export]
macro_rules! page_floor {
    ($p:expr) => {
        $p & crate::kmem::PAGE_NUMBER_MASK
    };
}

/// Round address up to the nearest PAGE_SIZE
#[macro_export]
macro_rules! page_ceil {
    ($p:expr) => {
        ($p + crate::kmem::PAGE_OFFSET_MASK) & crate::kmem::PAGE_NUMBER_MASK
    };
}

/// Get the page number of address
#[macro_export]
macro_rules! page_number {
    ($p:expr) => {
        $p >> 12
    };
}

// the first size_of(FreePage) bytes of each
// free page is used as a node in the free list
struct FreePage {
    next: *mut FreePage,
}

static mut FREE_LIST: *mut FreePage = null_mut();
static mut NUM_PAGES_ALLOCED: u32 = 0;
static mut INITIALIZED: bool = false;

/// Initialize the kernel's heap memory so that
/// all of the pages are ready to be allocated.
pub fn init() {
    unsafe {
        FREE_LIST = null_mut();
        let mut ptr = page_ceil!(HEAP_START) as *mut FreePage;
        let heap_end: *mut FreePage = HEAP_END as *mut FreePage;
        let mut num_heap_pages: u32 = 0;
        while ptr.byte_add(PAGE_SIZE as usize) <= heap_end {
            (*ptr).next = FREE_LIST;
            FREE_LIST = ptr;
            ptr = ptr.byte_add(PAGE_SIZE as usize);
            num_heap_pages += 1;
        }
        assert!(num_heap_pages == ((HEAP_END - HEAP_START) / PAGE_SIZE) as u32);

        debug!("Initializing page allocator");
        debug!("  text: 0x{:x}..0x{:x}", TEXT_START, TEXT_END);
        debug!("rodata: 0x{:x}..0x{:x}", RODATA_START, RODATA_END);
        debug!("  data: 0x{:x}..0x{:x}", DATA_START, DATA_END);
        debug!("   bss: 0x{:x}..0x{:x}", BSS_START, BSS_END);
        debug!(" stack: 0x{:x}..0x{:x}", STACK_START, STACK_END);
        debug!("  heap: 0x{:x}..0x{:x}", HEAP_START, HEAP_END);
        debug!("        ({} pages)", num_heap_pages);
        INITIALIZED = true;
    }
}

pub fn initialized() -> bool {
    unsafe { INITIALIZED }
}

/// Allocate one physical page of memory.
/// Returns null_mut() if no pages are left.
pub fn kalloc() -> *mut u8 {
    unsafe {
        if FREE_LIST.is_null() {
            null_mut()
        } else {
            let ptr = FREE_LIST;
            FREE_LIST = (*ptr).next;
            NUM_PAGES_ALLOCED += 1;
            ptr as *mut u8
        }
    }
}

/// Free the physical page of memory pointed to by ptr.
/// Typically used with kalloc(). For example,
///
/// let page = kalloc();
/// // ... use page ...
/// kfree(page);
pub fn kfree(ptr: *mut u8) {
    let ptr_value = ptr as u64;
    unsafe {
        if (ptr_value % PAGE_SIZE != 0) || (ptr_value < HEAP_START) || (ptr_value >= HEAP_END) {
            panic!("kfree")
        }
        let freed_page = ptr as *mut FreePage;
        (*freed_page).next = FREE_LIST;
        FREE_LIST = freed_page;
        NUM_PAGES_ALLOCED -= 1;
    }
}
