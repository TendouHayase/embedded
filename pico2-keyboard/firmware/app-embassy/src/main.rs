#![no_std]
#![no_main]

use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    multicore::{Stack, spawn_core1},
    peripherals::PIN_25,
};
use embassy_time::Timer;
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

#[embassy_executor::task]
async fn blink_led(mut led: Output<'static>) {
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pin: Peri<'static, PIN_25> = p.PIN_25;
    let led = Output::new(pin, Level::Low);

    spawner.spawn(blink_led(led).unwrap())
}
