use crate::{
    cpu,
    csr::SSTATUS_SPP,
    csr_read, csr_read_field,
    plic::{self, PlicPrivilege},
    proc, uart,
};

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

pub const UART_IRQ: u32 = 0x0a;

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
        match cause {
            SCause::EnvCallFromUMode => {
                debug!("EnvCall from User mode!");
                handle_syscall();
            }
            SCause::SExternalInterrupt => {
                // handle external interrupts until there are none left
                loop {
                    let irq: u32 = plic::claim(PlicPrivilege::Supervisor);
                    match irq {
                        0 => {
                            // no pending interrupts
                            return;
                        }
                        1..=8 => {}
                        UART_IRQ => {
                            // either received a byte or transmit buffer is empty
                            uart::handle_intr();
                        }
                        _ => {
                            debug!("Unexpected PLIC IRQ {}", irq)
                        }
                    }
                    plic::complete(PlicPrivilege::Supervisor, irq);
                }
            }
            _ => {}
        }
    }
}

fn handle_syscall() {
    unsafe {
        debug!("Syscall {}", proc!().frame.assume_init_ref().regs[17]);
    }
}
