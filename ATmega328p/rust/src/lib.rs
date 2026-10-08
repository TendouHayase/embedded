#![no_std]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

pub mod sync;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
