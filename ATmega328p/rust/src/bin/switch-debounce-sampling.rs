#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_bare_metal as _;

use core::ptr::{read_volatile, write_volatile};

const OCIEA_MASK: u8 = 1 << 1;
const OCFA_MASK: u8 = 1 << 1;
const INPUT_MASK: u8 = 1 << 4;
const OUTPUT_MASK: u8 = 1 << 3;
const GLOBAL_INTERRUPT_ENABLE_MASK: u8 = 1 << 7;

#[unsafe(export_name = "__vector_3")]
pub extern "avr-interrupt" fn pcint0_interrupt() {
    enable_timer();
}

#[unsafe(export_name = "__vector_11")]
pub extern "avr-interrupt" fn timer1_comp_a_interrupt() {
    static mut MATCHING_CNT: u8 = 0;
    static mut INPUT_BEFORE: u8 = 0;
    const MATCHING_MAX_CNT: u8 = 20;
    let pinb = 0x23 as *mut u8;

    let input: u8 = unsafe { read_volatile(pinb) & INPUT_MASK };

    if input == unsafe { INPUT_BEFORE } {
        unsafe { MATCHING_CNT += 1 };
    } else {
        unsafe {
            INPUT_BEFORE = input;
            MATCHING_CNT = 0;
        }
    }

    if unsafe { MATCHING_CNT >= MATCHING_MAX_CNT } {
        if input != 0 {
            led_on();
        } else {
            led_off();
        }
        disable_timer();
        unsafe { MATCHING_CNT = 0 };
    }
}

fn enable_timer() {
    let timsk1 = 0x6F as *mut u8;
    unsafe { write_volatile(timsk1, read_volatile(timsk1) | OCIEA_MASK) };
}

fn disable_timer() {
    let timsk1 = 0x6F as *mut u8;
    let tifr1 = 0x36 as *mut u8;
    unsafe {
        write_volatile(timsk1, read_volatile(timsk1) & !OCFA_MASK);
        write_volatile(tifr1, OCIEA_MASK);
    }
}

fn led_on() {
    let portb = 0x25 as *mut u8;
    unsafe { write_volatile(portb, read_volatile(portb) | OUTPUT_MASK) };
}

fn led_off() {
    let portb = 0x25 as *mut u8;
    unsafe {
        write_volatile(portb, read_volatile(portb) & !OUTPUT_MASK);
    }
}

fn interrupt_init() {
    let pcmsk0 = 0x6B as *mut u8;
    let sreg = 0x5F as *mut u8;
    let pcicr = 0x68 as *mut u8;
    unsafe {
        write_volatile(pcmsk0, read_volatile(pcmsk0) | INPUT_MASK);
        write_volatile(sreg, read_volatile(sreg) | GLOBAL_INTERRUPT_ENABLE_MASK);
        write_volatile(pcicr, read_volatile(pcicr) | 1);
    }
}

fn counter_init() {
    let ocr1ah = 0x89 as *mut u8;
    let ocr1al = 0x88 as *mut u8;
    let tcnt1l = 0x84 as *mut u8;
    let tcnt1h = 0x85 as *mut u8;
    let tccr1b = 0x81 as *mut u8;
    let tifr1 = 0x36 as *mut u8;

    const COUNTER_OFF_MASK: u8 = 0b00001000;
    const PRESCALER_CLOCK_MASK: u8 = 0b0001011;

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

fn gpio_init() {
    let ddrb = 0x24 as *mut u8;
    let portb = 0x25 as *mut u8;

    unsafe {
        write_volatile(ddrb, read_volatile(ddrb) | OUTPUT_MASK);
        write_volatile(ddrb, read_volatile(ddrb) & !INPUT_MASK);
        write_volatile(portb, read_volatile(portb) & !OUTPUT_MASK);
    }
}

#[unsafe(export_name = "main")]
pub extern "C" fn main() -> ! {
    gpio_init();
    counter_init();
    interrupt_init();

    loop {}
}
