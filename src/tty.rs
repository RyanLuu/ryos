use crate::uart;

pub fn handle_byte(byte: u8) {
    match byte {
        0x08 | 0x7F => {
            // backspace or del
            uart::put(0x08);
            uart::put(0x20);
            uart::put(0x08);
        }
        0x0A | 0x0D => {
            // line feed or carriage return, just convert to CRLF to avoid problems
            uart::put(0x0D);
            uart::put(0x0A);
        }
        _ => {
            uart::put(byte);
        }
    }
}
