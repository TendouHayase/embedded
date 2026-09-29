#![no_std]
#![no_main]

use core::panic::PanicInfo;

fn main() {}

struct PanicHandler {}

impl PanicHandler {
    fn new() -> Self {
        PanicHandler {}
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let host_stderr = PanicHandler::new();

    loop {}
}
