use core::arch::global_asm;

// Pull assembly files into a .rs file so that we can build
// everything together without any extra toolchain steps

global_asm!(include_str!("asm/boot.s"));
global_asm!(include_str!("asm/mem.s"));
global_asm!(include_str!("asm/trap.s"));
