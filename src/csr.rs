/// Macros and constants for managing Control and Status Registers

/// Read u64 value from CSR
#[macro_export]
macro_rules! csr_read {
    ($csr:ident) => ({
        let value: u64;
        core::arch::asm!(concat!("csrr {val}, ", stringify!($csr)), val = out(reg) value);
        value
    });
}

/// Write u64 value to CSR
#[macro_export]
macro_rules! csr_write {
    ($csr:ident, $value:expr) => ({
        let value: u64 = $value;
        core::arch::asm!(concat!("csrw ", stringify!($csr), ", {val}"), val = in(reg) value);
    });
}

/// Bitwise or an arbitrary number of values
#[macro_export]
macro_rules! or {
    ($bit:expr) => ($bit);
    ($bit:expr, $($rest:expr),*) => ($bit | crate::or!($($rest),*));
}

/// Set some number of bits of a CSR to 1
#[macro_export]
macro_rules! csr_set_bits {
    ($csr:ident, $($bits:expr),*) => {{
        csr_write!($csr, (crate::csr_read!($csr) | crate::or!($($bits),*)));
    }};
}

/// Set some number of bits of a CSR to 0
#[macro_export]
macro_rules! csr_clear_bits {
    ($csr:ident, $($bits:expr),*) => {{
        csr_write!($csr, (crate::csr_read!($csr) & !crate::or!($($bits),*)));
    }};
}

/// Read the value from the CSR named by $csr positioned inside the bitmask $mask
/// only supports contiguous bitmasks (i.e. all 1s are connected)
#[macro_export]
macro_rules! csr_read_field {
    ($csr:ident, $mask:expr) => {{
        let mask: u64 = $mask;
        assert!(mask.leading_zeros() + mask.trailing_zeros() + mask.count_ones() == 64); // check that bitmask is contiguous
        (csr_read!($csr) & mask) >> mask.trailing_zeros()
    }};
}

/// Write $value into the CSR named by $csr positioned inside the bitmask $mask
/// only supports contiguous bitmasks (i.e. all 1s are connected)
#[macro_export]
macro_rules! csr_write_field {
    ($csr:ident, $mask:expr, $value:expr) => {{
        let value: u64 = $value;
        let mask: u64 = $mask;
        assert!(mask.leading_zeros() + mask.trailing_zeros() + mask.count_ones() == 64); // check that bitmask is contiguous
        assert!(value.leading_zeros() >= mask.count_zeros()); // check that value fits into mask
        csr_write!(
            $csr,
            (crate::csr_read!($csr) & !mask) | (value << mask.trailing_zeros())
        );
    }};
}

// MACHINE

// 3.1.6 Machine Status Registers
pub const MSTATUS_MPP: u64 = 0b11 << 11;
pub const MSTATUS_MPP_M: u64 = 3;
pub const MSTATUS_MPP_S: u64 = 1;
pub const MSTATUS_MPP_U: u64 = 0;

// 3.7.1 Physical Memory Protection CSRs
pub const PMPCFG_A: u64 = 0b11 << 3;
pub const PMPCFG_A_OFF: u64 = 0;
pub const PMPCFG_A_TOR: u64 = 1;
pub const PMPCFG_A_NA4: u64 = 2;
pub const PMPCFG_A_NAPOT: u64 = 3;
pub const PMPCFG_X: u64 = 1 << 2;
pub const PMPCFG_W: u64 = 1 << 1;
pub const PMPCFG_R: u64 = 1 << 0;

// SUPERVISOR

// 5.1.1 Supervisor Status Register
pub const SSTATUS_SPP: u64 = 1 << 8;

// 5.1.3 Supervisor Interrupt Registers
pub const SIE_SEIE: u64 = 1 << 9;
pub const SIE_STIE: u64 = 1 << 5;
pub const SIE_SSIE: u64 = 1 << 1;

// 5.1.11 Supervisor Address Translation and Protection Register
pub const SATP_MODE: u64 = 0b1111 << 60;
pub const SATP_MODE_SV39: u64 = 8;
pub const SATP_MODE_SV48: u64 = 9;
pub const SATP_MODE_SV57: u64 = 10;
pub const SATP_PPN: u64 = 0xfffffffffff << 0;
