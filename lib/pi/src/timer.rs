use crate::common::IO_BASE;
use core::time::Duration;

use volatile::prelude::*;
use volatile::{ReadVolatile, Volatile};

/// The base address for the ARM system timer registers.
const TIMER_REG_BASE: usize = IO_BASE + 0x3000;

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    CS: Volatile<u32>,
    CLO: ReadVolatile<u32>,
    CHI: ReadVolatile<u32>,
    COMPARE: [Volatile<u32>; 4],
}

/// The Raspberry Pi ARM system timer.
pub struct Timer {
    registers: &'static mut Registers,
}

impl Timer {
    /// Returns a new instance of `Timer`.
    pub fn new() -> Timer {
        Timer {
            registers: unsafe { &mut *(TIMER_REG_BASE as *mut Registers) },
        }
    }

    /// Reads the system timer's counter and returns Duration.
    /// `CLO` and `CHI` together can represent the number of elapsed microseconds.
    pub fn read(&self) -> Duration {
        let registers = &self.registers;
        let mut high_word = registers.CHI.read();
        let mut low_word = registers.CLO.read();

        // Cannot read both registers atomically, so if the high register turns over
        // right after we read CHI but right before we read CLO, we will be way off.
        // Double check CHI -- if it changed, can't keep the original values.
        let check_val = registers.CHI.read();
        if high_word != check_val {
            low_word = registers.CLO.read();
            high_word = check_val;
        }

        let time_in_micros = ((high_word as u64) << 32) | (low_word as u64);
        Duration::from_micros(time_in_micros)
    }
}

/// Returns current time.
pub fn current_time() -> Duration {
    Timer::new().read()
}

/// Spins until `t` duration have passed.
pub fn spin_sleep(t: Duration) {
    let timer = Timer::new();
    let start_time = timer.read();

    loop {
        let cur_time = timer.read();
        let delta = cur_time - start_time;

        if delta >= t {
            break;
        }
    }
}
