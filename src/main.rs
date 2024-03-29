#![no_main]
#![no_std]
#![feature(
    panic_info_message,         // message() in panic handler
    int_roundings,              // div_floor(), div_ceil()
    pointer_byte_offsets,       // byte_offset(), byte_add(), byte_sub()
    const_pointer_byte_offsets, // byte_offset(), byte_add(), byte_sub()
    variant_count,              // variant_count<T>()
    const_maybe_uninit_zeroed,
)]

extern "C" {
    fn kernel_vec();
}

use crate::csr::{
    MSTATUS_MPP, MSTATUS_MPP_S, PMPCFG_A, PMPCFG_A_TOR, PMPCFG_R, PMPCFG_W, PMPCFG_X, SIE_SEIE,
    SIE_SSIE, SIE_STIE, SSTATUS_SIE,
};
use core::arch::asm;

#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {{
        crate::print!("{:>6}: ", file!().rsplit('/').next().unwrap().strip_suffix(".rs").unwrap());
        crate::println!($($args)*)
    }};
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        csr_write!(sie, 0);
    }
    print_sync!("Aborting: ");
    if let Some(p) = info.location() {
        println_sync!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println_sync!("no information available.");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

/// ENTRY POINT
#[no_mangle]
extern "C" fn kinit() {
    unsafe {
        // disable paging until the MMU is initialized
        csr_write!(satp, 0u64);

        // allow supervisor interrupts
        csr_set_bits!(sstatus, SSTATUS_SIE);

        // delegate interrupts and exceptions to supervisor
        csr_write!(medeleg, 0xffffu64);
        csr_write!(mideleg, 0xffffu64);
        csr_set_bits!(sie, SIE_SEIE, SIE_STIE, SIE_SSIE);

        // set interrupt vector
        csr_write!(stvec, kernel_vec as u64);

        // allow supervisor to access all memory
        csr_write!(pmpaddr0, (1 << 54) - 1);
        csr_write_field!(pmpcfg0, PMPCFG_A, PMPCFG_A_TOR);
        csr_set_bits!(pmpcfg0, PMPCFG_R, PMPCFG_W, PMPCFG_X);

        // write mhartid into tp
        let hartid: u64 = csr_read!(mhartid);
        reg_write!(tp, hartid);

        // switch to supervisor mode upon mret
        csr_write_field!(mstatus, MSTATUS_MPP, MSTATUS_MPP_S);

        // jump to main upon mret
        csr_write!(mepc, main as u64);

        asm!("mret");
    }
}

fn main() {
    // Main should initialize all sub-systems and get
    // ready to start scheduling. The last thing this
    // should do is start the timer.

    crate::uart::init();
    crate::kmem::init();
    crate::mmu::init();
    crate::virtio::init();

    // Now test println! macro!
    debug!("Initialized hart {}", unsafe { reg_read!(tp) });
    println!();
    println!("===========");
    println!("RyOS v0.1.0");
    println!("===========");
    println!();

    unsafe {
        let mtimecmp = 0x0200_4000 as *mut u64;
        let mtime = 0x0200_bff8 as *const u64;
        mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
    }

    // Test reading from UART
    loop {}
}

pub mod asm;
pub mod cpu;
pub mod csr;
pub mod kmem;
pub mod mmio;
pub mod mmu;
pub mod plic;
pub mod proc;
pub mod reg;
pub mod string;
pub mod term;
pub mod trap;
pub mod uart;
pub mod util;
pub mod virtio;
