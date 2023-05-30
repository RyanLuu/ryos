use crate::{csr::SSTATUS_SPP, csr_read, csr_read_field};

#[repr(C)]
pub struct TrapFrame {
    pub regs: [u64; 32],
    pub satp: u64,
    pub trap_stack: *mut u8,
    pub hartid: u64,
}

const INTERRUPT: u64 = 1 << 63;
#[repr(u64)]
#[derive(Debug)]
pub enum SCause {
    InstAddrMisaligned = 0,
    InstAccessFault = 1,
    InstIllegal = 2,
    Breakpoint = 3,
    LoadAddrMisaligned = 4,
    LoadAccessFault = 5,
    StoreAMOAddrMisaligned = 6,
    StoreAMOAccessFault = 7,
    EnvCallFromUMode = 8,
    EnvCallFromSMode = 9,
    InstPageFault = 12,
    LoadPageFault = 13,
    StoreAMOPageFault = 15,

    SSoftwareInterrupt = INTERRUPT | 1,
    STimerInterrupt = INTERRUPT | 5,
    SExternalInterrupt = INTERRUPT | 9,
}

impl SCause {
    pub fn should_panic(&self) -> bool {
        match self {
            SCause::InstAddrMisaligned
            | SCause::InstAccessFault
            | SCause::InstIllegal
            | SCause::LoadAddrMisaligned
            | SCause::LoadAccessFault
            | SCause::StoreAMOAddrMisaligned
            | SCause::StoreAMOAccessFault
            | SCause::InstPageFault
            | SCause::LoadPageFault
            | SCause::StoreAMOPageFault => true,
            _ => false,
        }
    }
}

#[no_mangle]
extern "C" fn kernel_trap() {
    unsafe {
        let epc: u64 = csr_read!(sepc);
        let status: u64 = csr_read!(sstatus);
        let cause: SCause = core::mem::transmute(csr_read!(scause));

        if csr_read_field!(sstatus, SSTATUS_SPP) == 0 {
            panic!("trap originated from user mode");
        }
        if cause.should_panic() {
            panic!("Kernel trap 0x{:08x} {:064b} {:?}", epc, status, cause);
        }
    }
}
