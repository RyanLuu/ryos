/// Macros for reading/writing to general purpose registers

#[macro_export]
macro_rules! reg_read {
    ($reg:ident) => {{
        let value: u64;
        core::arch::asm!(concat!("mv {val}, ", stringify!($reg)), val = out(reg) value);
        value
    }};
}

#[macro_export]
macro_rules! reg_write {
    ($reg:ident, $value:expr) => ({
        let value: u64 = $value;
        core::arch::asm!(concat!("mv ", stringify!($reg), ", {val}"), val = in(reg) value);
    });
}
