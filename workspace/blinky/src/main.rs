#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main,
};

#[main]
fn main() -> ! {
    // Initialize RTT for console output
    rtt_init_print!();

    let peripherals = esp_hal::init(esp_hal::Config::default());

    rprintln!("esp32-c3 is booting!");

    // Set GPIO8 as an output, and set its state low initially.
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    led.set_high();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a loop.
    let delay = Delay::new();

    loop {
        led.toggle();
        delay.delay_millis(500);
        rprintln!("status: {:?}", led.output_level());
    }
}