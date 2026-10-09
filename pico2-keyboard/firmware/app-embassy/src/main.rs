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
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pin: Peri<'static, PIN_25> = p.PIN_25;
    let led = Output::new(pin, Level::Low);

    static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

    spawn_core1(
        p.CORE1,
        unsafe {
            static mut CORE1_STACK: Stack<4096> = Stack::new();
            (&raw mut CORE1_STACK).as_mut().unwrap()
        },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| spawner.spawn(blink_led(led).unwrap()))
        },
    );
}
