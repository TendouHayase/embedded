#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_bare_metal as _;

use core::{
    ptr::{read_volatile, write_volatile},
    sync::atomic::{AtomicU16, Ordering::Relaxed},
};

const THRESHOLD: u16 = 500;
const COUNTER_OFF_MASK: u8 = 0b0001000;
const PRESCALER_CLOCK_MASK: u8 = 0b0001011;
const OCFA_MASK: u8 = 1 << 1;
const OCIEA_MASK: u8 = 1 << 1;
const GLOBAL_INTERRUPT_ENABLE_MASK: u8 = 1 << 7;

static CNT: AtomicU16 = AtomicU16::new(0);

// 타이머 인터럽트
#[unsafe(export_name = "__vector_11")]
pub extern "avr-interrupt" fn __vector_11() {
    CNT.store(CNT.load(Relaxed).wrapping_add(1), Relaxed);
}

fn init() {
    let ddrb: *mut u8 = 0x24 as *mut u8;
    const MASK: u8 = 1 << 5;
    let ddrb_value = unsafe { read_volatile(ddrb) };
    unsafe { write_volatile(ddrb, ddrb_value | MASK) };
}

fn interrupt_init() {
    let sreg = 0x5F as *mut u8;

    unsafe {
        write_volatile(sreg, read_volatile(sreg) | GLOBAL_INTERRUPT_ENABLE_MASK);
    };
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

fn led_change() {
    let portb = 0x25 as *mut u8;
    const MASK: u8 = 1 << 5;
    unsafe { write_volatile(portb, read_volatile(portb) ^ MASK) };
}

fn enable_timer() {
    let timsk1 = 0x6F as *mut u8;
    unsafe {
        write_volatile(timsk1, read_volatile(timsk1) | OCIEA_MASK);
    };
}

#[unsafe(export_name = "main")]
pub extern "C" fn main() -> ! {
    init();
    cnt_init();
    let mut before_time: u16 = 0;
    interrupt_init();
    enable_timer();
    loop {
        let cnt = CNT.load(Relaxed);
        if cnt.wrapping_sub(before_time) >= THRESHOLD {
            before_time = cnt;
            led_change();
        }
    }
}
