use core::fmt::Display;

use crate::kmem::PLIC_BASE;

/// Simple wrapper struct for MMIO registers

pub struct MMIORegister<const BASE: u64, T> {
    address: *mut T,
    r: bool,
    w: bool,
}

impl<const BASE: u64, T: Display> MMIORegister<BASE, T> {
    pub const fn new(offset: u64) -> Self {
        Self {
            address: (BASE + offset) as *mut T,
            r: true,
            w: true,
        }
    }

    pub const fn read_only(offset: u64) -> Self {
        Self {
            address: (BASE + offset) as *mut T,
            r: true,
            w: false,
        }
    }

    pub const fn write_only(offset: u64) -> Self {
        Self {
            address: (BASE + offset) as *mut T,
            r: false,
            w: true,
        }
    }

    pub const fn add(&self, i: usize) -> Self {
        Self {
            address: unsafe { self.address.add(i) },
            r: self.r,
            w: self.w,
        }
    }

    pub const fn byte_add(&self, i: usize) -> Self {
        Self {
            address: unsafe { self.address.byte_add(i) },
            r: self.r,
            w: self.w,
        }
    }

    pub unsafe fn write(&self, value: T) {
        assert!(self.w);
        self.address.write_volatile(value)
    }

    pub unsafe fn read(&self) -> T {
        assert!(self.r);
        self.address.read_volatile()
    }
}
