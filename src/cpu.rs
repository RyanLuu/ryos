use core::mem::MaybeUninit;

#[macro_export]
macro_rules! cpu {
    () => {
        crate::cpu::CPUS.assume_init_mut()[crate::csr_read!(tp) as usize]
    };
}

pub struct CPU {
    pub current_proc: usize,
}

pub static mut CPUS: MaybeUninit<[CPU; 4]> = MaybeUninit::zeroed();
