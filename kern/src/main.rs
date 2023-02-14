#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;
use core::time::Duration;
use pi::gpio::Gpio;

unsafe fn kmain() -> ! {
    let mut gpio_pin = Gpio::new(16).into_output();

    let mut state = 0;
    loop {
        if state == 0 {
            gpio_pin.set();
        } else {
            gpio_pin.clear();
        }

        state = (state + 1) % 2;
        pi::timer::spin_sleep(Duration::from_millis(500));
    }
}
