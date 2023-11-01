use core::mem::MaybeUninit;

use crate::{
    kmem::{kalloc, kfree, PAGE_SIZE},
    mmu::{PageTable, PTE_R, PTE_USER, PTE_W, PTE_X},
};

#[macro_export]
macro_rules! proc {
    () => {
        crate::proc::PROCS.assume_init_mut()[crate::cpu!().current_proc]
    };
}

pub struct TrapFrame {
    pub kernel_satp: u64,
    pub kernel_sp: u64,
    pub epc: u64,
    pub kernel_hartid: u64,
    pub regs: [u64; 32],
}

pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Dead,
}

pub struct Process {
    pub frame: MaybeUninit<TrapFrame>,
    pub stack: MaybeUninit<[*mut u8; STACK_PAGES as usize]>,
    pub pc: u64,
    pub pid: u16,
    pub root: *mut PageTable,
    pub state: ProcessState,
}

pub const NPROC: usize = 64;
pub static mut PROCS: MaybeUninit<[Process; NPROC]> = MaybeUninit::zeroed();

pub const STACK_ADDR: u64 = 0x1_0000_0000;
pub const STACK_PAGES: u64 = 4;
pub const PROC_STARTING_ADDR: u64 = 0x2000_0000;

static mut NEXT_PID: u16 = 1;

impl Process {
    // TODO: upgrade executable from function to ELF
    pub fn new(func: fn()) -> Self {
        let mut new_proc = Process {
            frame: MaybeUninit::zeroed(),
            stack: MaybeUninit::zeroed(),
            pc: PROC_STARTING_ADDR,
            pid: unsafe { NEXT_PID },
            root: kalloc() as *mut PageTable,
            state: ProcessState::Waiting,
        };
        unsafe {
            NEXT_PID += 1;
        }
        unsafe {
            new_proc.frame.assume_init_mut().regs[2] = STACK_ADDR + PAGE_SIZE * 1;
        }

        // set up memory mappings
        let pt;
        unsafe {
            pt = &mut *new_proc.root;
        }
        for i in 0..STACK_PAGES {
            let vaddr = STACK_ADDR + i * PAGE_SIZE;
            let paddr = kalloc() as u64;
            assert!(paddr != 0); // TODO: handle alloc failure
            pt.map(vaddr, paddr, PTE_USER | PTE_R | PTE_W, 0);
        }
        pt.map(PROC_STARTING_ADDR, func as u64, PTE_USER | PTE_R | PTE_X, 0);

        new_proc
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        // free stack
        for page in unsafe { self.stack.assume_init() } {
            kfree(page)
        }
        unsafe { &mut *self.root }.free();
    }
}
