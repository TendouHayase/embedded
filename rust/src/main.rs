#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(export_name = "main")]
pub extern "C" fn main() {
    let ddrb: *mut u8 = 0x24 as *mut u8;
    let portb: *mut u8 = 0x25 as *mut u8;
    const MASK: u8 = 1 << 5;
    let ddrb_value = unsafe { read_volatile(ddrb) };
    unsafe { write_volatile(ddrb, ddrb_value | MASK) };

    loop {
        let portb_value = unsafe { read_volatile(portb) };
        unsafe { write_volatile(portb, portb_value ^ MASK) };
        delay();
    }
}

fn delay() {
    let mut i: u16 = 0;
    while i < 100 {
        let mut j: u16 = 0;
        while j < 10000 {
            let j_value = unsafe { read_volatile(&j) };
            unsafe { write_volatile(&mut j as *mut u16, j_value + 1) };
        }
        let i_value = unsafe { read_volatile(&i) };
        unsafe { write_volatile(&mut i as *mut u16, i_value + 1) };
    }
}
