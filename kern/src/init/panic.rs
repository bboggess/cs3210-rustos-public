use crate::console::kprintln;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("         ¯\\_(ツ)_/¯");
    kprintln!("---------- PANIC ----------");
    kprintln!("");
    kprintln!("{}", info);

    loop {}
}
