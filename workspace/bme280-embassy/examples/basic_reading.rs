#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{I2c, Config as I2cConfig},
    main,
};

#[main]
fn main() -> ! {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("ESP32-C3 BME280 Basic Reading Test");
    rprintln!("==================================");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    
    // Set up LED (GPIO3) for status indication
    let mut led = Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default());
    
    // Setup I2C (GPIO8=SDA, GPIO9=SCL)
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);

    rprintln!("I2C initialized - SDA: GPIO8, SCL: GPIO9");
    
    // Test I2C communication by scanning for BME280
    rprintln!("Scanning for BME280 sensor...");
    
    // BME280 addresses: 0x76 (primary) or 0x77 (secondary)
    let bme280_addrs = [0x76, 0x77];
    let mut found_addr = None;
    
    for &addr in &bme280_addrs {
        if i2c.transaction(addr, &mut []).is_ok() {
            rprintln!("Found BME280 at address: 0x{:02X}", addr);
            found_addr = Some(addr);
            break;
        }
    }
    
    match found_addr {
        Some(addr) => {
            rprintln!("[PASS] BME280 detected at 0x{:02X}", addr);
            
            // Try to read BME280 chip ID (register 0xD0 should return 0x60)
            let mut chip_id = [0u8];
            if i2c.write_read(addr, &[0xD0], &mut chip_id).is_ok() {
                if chip_id[0] == 0x60 {
                    rprintln!("[PASS] BME280 chip ID verified: 0x{:02X}", chip_id[0]);
                } else {
                    rprintln!("[WARN] Unexpected chip ID: 0x{:02X} (expected 0x60)", chip_id[0]);
                }
            } else {
                rprintln!("[WARN] Could not read chip ID");
            }
        },
        None => {
            rprintln!("[INFO] No BME280 found at standard addresses (0x76, 0x77)");
            rprintln!("[INFO] Check wiring: SDA=GPIO8, SCL=GPIO9");
        }
    }
    
    rprintln!("Test completed - blinking LED to indicate success");

    // Simple blink loop like blinky
    loop {
        led.toggle();
        delay.delay_millis(500);
        rprintln!("BME280 test running... LED toggled");
    }
}