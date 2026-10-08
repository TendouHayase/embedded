#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

use avr_bare_metal::{
    self as _,
    sync::{Delay, run, spawn},
};

const COUNTER_OFF_MASK: u8 = 0b0001000;
const PRESCALER_CLOCK_MASK: u8 = 0b0001011;
const OCFA_MASK: u8 = 1 << 1;
const OCIEA_MASK: u8 = 1 << 1;
const GLOBAL_INTERRUPT_ENABLE_MASK: u8 = 1 << 7;

fn led_init() {
    let ddrb: *mut u8 = 0x24 as *mut u8;
    const MASK: u8 = 1 << 5;
    let ddrb_value = unsafe { read_volatile(ddrb) };
    unsafe { write_volatile(ddrb, ddrb_value | MASK) };
}

fn cnt_init() {
    let ocr1ah = 0x89 as *mut u8;
    let ocr1al = 0x88 as *mut u8;
    let tcnt1l = 0x84 as *mut u8;
    let tcnt1h = 0x85 as *mut u8;
    let tccr1b = 0x81 as *mut u8;
    let tifr1 = 0x36 as *mut u8;

    unsafe {
        write_volatile(tccr1b, COUNTER_OFF_MASK);

        write_volatile(tcnt1h, 0);
        write_volatile(tcnt1l, 0);

        const REPEAT_CNT: [u8; 2] = 249u16.to_be_bytes();

        write_volatile(ocr1ah, REPEAT_CNT[0]);
        write_volatile(ocr1al, REPEAT_CNT[1]);

        write_volatile(tifr1, OCFA_MASK);
        write_volatile(tccr1b, PRESCALER_CLOCK_MASK);
    }
}

fn interrupt_init() {
    let sreg = 0x5F as *mut u8;

    unsafe {
        write_volatile(sreg, read_volatile(sreg) | GLOBAL_INTERRUPT_ENABLE_MASK);
    };
}

fn enable_sleep() {
    const SMCR: *mut u8 = 0x53 as *mut u8;
    unsafe { write_volatile(SMCR, 1 << 0) };
}

fn led_change(mask: u8) {
    let portb = 0x25 as *mut u8;
    unsafe { write_volatile(portb, read_volatile(portb) ^ mask) };
}

fn enable_timer() {
    let timsk1 = 0x6F as *mut u8;
    unsafe {
        write_volatile(timsk1, read_volatile(timsk1) | OCIEA_MASK);
    };
}

async fn blink(mask: u8, period_ms: u16) {
    loop {
        led_change(mask);
        Delay::ms(period_ms).await;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    led_init();
    cnt_init();
    interrupt_init();
    enable_timer();
    enable_sleep();
    spawn(blink(1 << 5, 1000));
    run()
}
