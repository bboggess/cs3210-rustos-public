use core::fmt;
use core::time::Duration;

use shim::const_assert_size;
use shim::io;

use volatile::prelude::*;
use volatile::{ReadVolatile, Reserved, Volatile};

use crate::common::IO_BASE;
use crate::gpio::{Function, Gpio};
use crate::timer;

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IO: Volatile<u32>,
    IER: Volatile<u32>,
    IIR: Volatile<u32>,
    LCR: Volatile<u32>,
    MCR: Volatile<u32>,
    LSR: ReadVolatile<u32>,
    MSR: ReadVolatile<u32>,
    SCRATCH: Volatile<u32>,
    CNTL: Volatile<u32>,
    STAT: ReadVolatile<u32>,
    BAUD: Volatile<u32>,
}

const_assert_size!(Registers, 0x7E21506C - 0x7E215040);

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // Set data size to 8 bits
        registers.LCR.or_mask(3);

        // Set baud rate. Keep in mind that the baud rate is calculated
        // as sys_clock_freq / (8 * (register_value + 1))
        registers.BAUD.write(270);

        // turn on GPIO pins
        let tx_pin = Gpio::new(14).into_alt(Function::Alt5);
        let rx_pin = Gpio::new(15).into_alt(Function::Alt5);

        // enable the TX and RX
        registers.CNTL.or_mask(3);

        MiniUart {
            registers,
            timeout: None,
        }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        while !self.registers.LSR.has_mask(LsrStatus::TxAvailable as u32) {
            continue;
        }

        self.registers.IO.write(byte as u32);
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        self.registers.LSR.has_mask(LsrStatus::DataReady as u32)
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        let end_time = self.timeout.map(|timeout| timeout + timer::current_time());

        while !self.has_byte() {
            let is_timed_out = end_time.map_or(false, |end_time| timer::current_time() >= end_time);

            if is_timed_out {
                return Err(());
            }
        }

        Ok(())
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {
            continue;
        }

        self.registers.IO.read() as u8
    }
}

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for &byte in s.as_bytes() {
            if byte == b'\n' {
                // Must write a carriage return before any newlines
                self.write_byte(b'\r');
            }

            self.write_byte(byte);
        }

        Ok(())
    }
}

mod uart_io {
    use super::io;
    use super::MiniUart;
    use shim::ioerr;
    use volatile::prelude::*;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.

    impl io::Read for MiniUart {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.wait_for_byte().is_err() {
                return ioerr!(TimedOut, "Timed out waiting for first byte");
            }

            let mut num_bytes_read = 0;
            while num_bytes_read < buf.len() && self.has_byte() {
                buf[num_bytes_read] = self.read_byte();
                num_bytes_read += 1;
            }

            Ok(num_bytes_read)
        }
    }

    impl io::Write for MiniUart {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for &byte in buf {
                self.write_byte(byte);
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            // Wait for the transmit FIFO buffer to empty
            while !self.registers.LSR.has_mask(1 << 6) {
                continue;
            }

            Ok(())
        }
    }
}
