#![feature(asm)]
#![feature(global_asm)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

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
            asm!("nop" :::: "volatile");
        }
    }
}

/// Sets the specified bit (zero indexed) at the address given by
/// the pointer. Does not do any kind of bounds checking on bit.
fn set_bit_at_addr(addr: *mut u32, bit: usize) {
    unsafe {
        let cur_val = addr.read_volatile();
        let new_val = cur_val | (1 << bit);
        addr.write_volatile(new_val);
    }
}

/// Turns on a given GPIO pin for writing. Does not do any
/// bounds checking on the pin number.
fn set_gpio_output_pin(pin: usize) {
    let bit_num = (pin - 10) * 3;
    set_bit_at_addr(GPIO_FSEL1, bit_num);
}

/// Turns on a given GPIO pin. Does not do any bounds checking on
/// the pin number.
fn set_gpio_pin(pin: usize) {
    set_bit_at_addr(GPIO_SET0, pin);
}

/// Turns off a given GPIO pin. Does not do any bounds checking on
/// the pin number.
fn clear_gpio_pin(pin: usize) {
    set_bit_at_addr(GPIO_CLR0, pin);
}

unsafe fn kmain() -> ! {
    let output_pin = 16;

    set_gpio_output_pin(output_pin);

    let mut state = 0;
    loop {
        if state == 0 {
            set_gpio_pin(output_pin);
        } else {
            clear_gpio_pin(output_pin);
        }

        state = (state + 1) % 2;
        spin_sleep_ms(1000);
    }
}
