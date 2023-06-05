use core::mem::variant_count;

use crate::kmem::PLIC_BASE;
use crate::mmio::MMIORegister;
use crate::reg_read;

/// Manages the Platform Level Interrupt Controller
/// https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic.adoc

type PlicRegister = MMIORegister<PLIC_BASE, u32>;

const PRIORITY0: PlicRegister = PlicRegister::write_only(0x0000);
const PENDING0: PlicRegister = PlicRegister::new(0x1000);
const ENABLE00: PlicRegister = PlicRegister::new(0x2000);
const THRESHOLD0: PlicRegister = PlicRegister::write_only(0x20_0000);
const CLAIM0: PlicRegister = PlicRegister::read_only(0x20_0004);
const COMPLETE0: PlicRegister = PlicRegister::write_only(0x20_0004);

#[derive(Clone, Copy)]
pub enum PlicPrivilege {
    Machine = 0,
    Supervisor = 1,
}

impl PlicPrivilege {
    fn context(&self) -> usize {
        unsafe {
            let context =
                (reg_read!(tp) as usize * variant_count::<PlicPrivilege>()) + *self as usize;
            assert!(context < 15872);
            context
        }
    }
}

pub fn enable(privilege: PlicPrivilege, irq: u32) {
    assert!(irq != 0 && irq < 1024);
    let (reg, bit) = (irq / 32, irq % 32);
    assert!(reg < 32 && bit < 32);
    unsafe {
        let enable = ENABLE00
            .byte_add(0x80 * privilege.context())
            .add(reg as usize);
        enable.write(enable.read() | (1 << bit))
    }
}

pub fn set_priority(irq: u32, prio: u32) {
    assert!(irq != 0 && irq < 1024);
    assert!(prio < 8);
    unsafe { PRIORITY0.add(irq as usize).write(prio) }
}

pub fn set_threshold(privilege: PlicPrivilege, threshold: u32) {
    assert!(threshold < 8);
    unsafe {
        THRESHOLD0
            .byte_add(0x1000 * privilege.context())
            .write(threshold)
    }
}

pub fn claim(privilege: PlicPrivilege) -> u32 {
    unsafe { CLAIM0.byte_add(0x1000 * privilege.context()).read() }
}

pub fn complete(privilege: PlicPrivilege, irq: u32) {
    unsafe { COMPLETE0.add(0x1000 * privilege.context()).write(irq) }
}
