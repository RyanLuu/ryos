use crate::kmem::UART_BASE;

/// UART routines and driver
/// Reference: http://byterunner.com/16550.html
use core::fmt::Error;
use core::fmt::Write;

const BAUD_RATE: usize = 2_400;

// UART registers
const RHR: usize = 0; // receive holding register (for input bytes)
const THR: usize = 0; // transmit holding register (for output bytes)
const IER: usize = 1; // interrupt enable register
const IER_RX_ENABLE: u8 = 1 << 0;
const IER_TX_ENABLE: u8 = 1 << 1;
const FCR: usize = 2; // FIFO control register
const FCR_FIFO_ENABLE: u8 = 1 << 0;
const FCR_FIFO_RESET: u8 = 0b11 << 1; // clear the content of the two FIFOs
const LCR: usize = 3; // line control register
const LCR_EIGHT_BITS: u8 = 0b11 << 0;
const LCR_BAUD_LATCH: u8 = 1 << 7; // special mode to set baud rate (DLAB)
const LSR: usize = 5; // line status register
const LSR_RX_READY: u8 = 1 << 0; // input is waiting to be read from RHR
const LSR_TX_IDLE: u8 = 1 << 5; // THR can accept another character to send

pub enum UartWriteMode {
    SYNC, // synchronous
    INTR, // interrupt
}

impl Write for UartWriteMode {
    fn write_str(&mut self, out: &str) -> Result<(), Error> {
        match self {
            UartWriteMode::SYNC => {
                for c in out.bytes() {
                    unsafe {
                        while (reg(LSR).read() & LSR_TX_IDLE) == 0 {}
                        reg(THR).write(c);
                    }
                }
            }
            UartWriteMode::INTR => {
                for c in out.bytes() {
                    unsafe {
                        reg(THR).write(c);
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn init() {
    unsafe {
        // disable interrupts
        reg(IER).write(0x00);

        // clear and enable the FIFOs
        reg(FCR).write(FCR_FIFO_RESET | FCR_FIFO_ENABLE);

        // Enable receiver buffer interrupts, which is at bit index
        // 0 of the interrupt enable register (IER at offset 1).
        // enable transmitter empty and receiver ready interrupts
        reg(IER).write(IER_TX_ENABLE | IER_RX_ENABLE);

        // Calculate divisor = ceil(clock frequency / (16 * baud rate))
        // The device tree dump generated by `./qemu-ryos.sh -d` shows that the
        // 16550 UART device has a clock frequency of 3,686,400 Hz
        const CLOCK_HZ: usize = 3_686_400;
        const BAUD_OUT: usize = 16 * BAUD_RATE;
        const DIVISOR: usize = CLOCK_HZ.div_ceil(BAUD_OUT);
        if DIVISOR > 0xFFFF || CLOCK_HZ < BAUD_OUT {
            panic!("Invalid UART baud rate");
        }

        let divisor_lsb: u8 = (DIVISOR & 0xff).try_into().unwrap();
        let divisor_msb: u8 = (DIVISOR >> 8).try_into().unwrap();

        // Prepare to send divisor
        reg(LCR).write(LCR_BAUD_LATCH);

        // Write divisor to DLL and DLM
        reg(0).write(divisor_lsb);
        reg(1).write(divisor_msb);

        // release baud latch and set word length to 8 bits, no parity
        reg(LCR).write(LCR_EIGHT_BITS);
    }
}

pub fn get() -> Option<u8> {
    unsafe {
        if reg(LSR).read() & LSR_RX_READY == 0 {
            // The DR bit is 0, meaning no data
            None
        } else {
            // The DR bit is 1, meaning data!
            Some(reg(RHR).read())
        }
    }
}

unsafe fn reg(i: usize) -> UartRegister {
    UartRegister {
        address: (UART_BASE as *mut u8).add(i),
    }
}

struct UartRegister {
    address: *mut u8,
}

impl UartRegister {
    unsafe fn write(&mut self, value: u8) {
        self.address.write_volatile(value)
    }

    unsafe fn read(&mut self) -> u8 {
        self.address.read_volatile()
    }
}
