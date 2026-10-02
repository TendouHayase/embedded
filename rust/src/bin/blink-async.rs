#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use avr_bare_metal as _;

#[unsafe(export_name = "__vector_3")]
pub extern "avr-interrupt" fn timer1_comp_a_interrupt() {}

#[unsafe(export_name = "__vector_11")]
pub extern "avr-interrupt" fn pcint0_interrupt() {}

#[unsafe(export_name = "main")]
pub extern "C" fn main() {}
