use core::marker::PhantomData;

pub trait Permission {}
pub trait Readable {}
pub trait Writeable {}

pub struct RPerm {}
impl Permission for RPerm {}
impl Readable for RPerm {}

pub struct WPerm {}
impl Permission for WPerm {}
impl Writeable for WPerm {}

pub struct RWPerm {}
impl Permission for RWPerm {}
impl Readable for RWPerm {}
impl Writeable for RWPerm {}

/// Simple wrapper struct for MMIO registers

pub struct MMIODevice<T> {
    base: u64,
    _phantom: PhantomData<T>,
}

impl<T> MMIODevice<T> {
    pub const fn new(base: u64) -> Self {
        Self {
            base,
            _phantom: PhantomData,
        }
    }
}

pub struct MMIORegister<T, P: Permission> {
    address: *mut T,
    _phantom: PhantomData<P>,
}

impl<T> MMIODevice<T> {
    pub const fn reg<P: Permission>(&self, offset: u64) -> MMIORegister<T, P> {
        MMIORegister {
            address: (self.base + offset) as *mut T,
            _phantom: PhantomData,
        }
    }

    pub const fn reg_rw(&self, offset: u64) -> MMIORegister<T, RWPerm> {
        MMIORegister {
            address: (self.base + offset) as *mut T,
            _phantom: PhantomData,
        }
    }

    pub const fn reg_r(&self, offset: u64) -> MMIORegister<T, RPerm> {
        MMIORegister {
            address: (self.base + offset) as *mut T,
            _phantom: PhantomData,
        }
    }

    pub const fn reg_w(&self, offset: u64) -> MMIORegister<T, WPerm> {
        MMIORegister {
            address: (self.base + offset) as *mut T,
            _phantom: PhantomData,
        }
    }
}

impl<T, P: Permission> MMIORegister<T, P> {
    pub const fn add(&self, i: usize) -> Self {
        Self {
            address: unsafe { self.address.add(i) },
            _phantom: PhantomData,
        }
    }

    pub const fn byte_add(&self, i: usize) -> Self {
        Self {
            address: unsafe { self.address.byte_add(i) },
            _phantom: PhantomData,
        }
    }
}

impl<T, P: Permission + Writeable> MMIORegister<T, P> {
    pub unsafe fn write(&self, value: T) {
        self.address.write_volatile(value)
    }
}

impl<T, P: Permission + Readable> MMIORegister<T, P> {
    pub unsafe fn read(&self) -> T {
        self.address.read_volatile()
    }
}
