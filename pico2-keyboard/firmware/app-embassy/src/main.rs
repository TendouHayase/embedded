#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    gpio::{Input, Level, Output, Pull},
    peripherals::{PIN_15, PIN_25},
};
use embassy_time::Timer;
use panic_probe as _;

#[embassy_executor::task]
async fn blink_led(mut button: Input<'static>, mut led: Output<'static>) {
    led.set_inversion(true);

    let mut state = button.get_level();

    loop {
        let current_state = button.get_level();

        if state != current_state {
            let mut count: u8 = 0;
            while count <= 20 {
                Timer::after_micros(500).await;
                if button.get_level() != current_state {
                    break;
                } else {
                    count += 1;
                }
            }

            if count > 20 {
                led.set_level(current_state);
                state = current_state;
            }
        }
        if state == Level::High {
            button.wait_for_low().await;
        } else {
            button.wait_for_high().await;
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let pin25: Peri<'static, PIN_25> = p.PIN_25;
    let pin15: Peri<'static, PIN_15> = p.PIN_15;
    let led = Output::new(pin25, Level::Low);
    let btn = Input::new(pin15, Pull::Up);

    spawner.spawn(blink_led(btn, led).unwrap())
}
