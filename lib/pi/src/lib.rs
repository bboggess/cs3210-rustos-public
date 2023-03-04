#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(never_type)]
#![feature(ptr_offset_from)]
#![no_std]

pub mod atags;
pub mod common;
pub mod gpio;
pub mod timer;
pub mod uart;
