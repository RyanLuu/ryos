#![no_main]
#![no_std]
#![feature(panic_info_message, int_roundings, pointer_byte_offsets)]

extern "C" {
    fn kernel_vec();
}

use crate::csr::{
    MSTATUS_MPP, MSTATUS_MPP_S, PMPCFG_A, PMPCFG_A_TOR, PMPCFG_R, PMPCFG_W, PMPCFG_X, SIE_SEIE,
    SIE_SSIE, SIE_STIE,
};
use core::arch::asm;

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print {
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::uart::UartWriteMode::SYNC, $($args)+);
			});
}
#[macro_export]
macro_rules! println {
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
			print!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			print!(concat!($fmt, "\r\n"), $($args)+)
			});
}
#[macro_export]
macro_rules! debug {
	($($args:tt)+) => ({
            print!("{:>4}: ", file!().rsplit_once('/').map(|s| s.1).unwrap_or(file!()).strip_suffix(".rs").unwrap_or(file!()));
			println!($($args)*)
			});
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
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

    crate::kmem::init();
    crate::mmu::init();
    crate::uart::init();

    // Now test println! macro!
    println!();
    println!("===========");
    println!("RyOS v0.1.0");
    println!("===========");
    println!();

    // Test reading from UART
    loop {
        if let Some(c) = uart::get() {
            match c {
                8 | 127 => {
                    // backspace or del
                    print!("{}{}{}", '\x08', ' ', '\x08');
                }
                10 | 13 => {
                    // line feed or carriage return
                    println!();
                }
                _ => {
                    print!("{}", c as char);
                }
            }
        }
    }
}

pub mod asms;
pub mod csr;
pub mod kmem;
pub mod mmu;
pub mod trap;
pub mod uart;
