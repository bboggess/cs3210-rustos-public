#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use core::arch::asm;
#[cfg(not(test))]
mod init;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 6000) {
        unsafe {
            asm!("nop");
        }
    }
}

fn set_gpio_output_pin(pin: u32) {
    unsafe {
        GPIO_FSEL1.write_volatile(1 << pin);
    }
}

fn set_gpio_pin(pin: u32) {
    unsafe {
        GPIO_SET0.write_volatile(1 << pin);
    }
}

fn clear_gpio_pin(pin: u32) {
    unsafe {
        GPIO_CLR0.write_volatile(1 << pin);
    }
}

unsafe fn kmain() -> ! {
    // FIXME: STEP 1: Set GPIO Pin 16 as output.
    set_gpio_output_pin(18);
    // FIXME: STEP 2: Continuously set and clear GPIO 16.
    let mut state = 0;
    loop {
        if state == 0 {
            set_gpio_pin(16);
        } else {
            clear_gpio_pin(16);
        }

        state = (state + 1) % 2;
        spin_sleep_ms(1000);
    }
}
