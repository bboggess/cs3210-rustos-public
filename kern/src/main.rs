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
use core::fmt::Write;
use pi::uart::MiniUart;

unsafe fn kmain() -> ! {
    let mut uart = MiniUart::new();

    loop {
        let byte = uart.read_byte();
        uart.write_byte(byte);
    }
}
