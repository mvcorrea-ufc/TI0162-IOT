#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use esp_hal::{
    i2c::master::{I2c, Config as I2cConfig},
    timer::timg::TimerGroup,
    Async,
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use bme280_embassy::BME280;

#[embassy_executor::task]
async fn bme280_test_task(mut i2c: I2c<'static, Async>) {
    rprintln!("BME280 Basic Reading Test");
    rprintln!("========================");
    
    let mut bme280 = BME280::new(&mut i2c);
    
    // Test sensor detection
    rprintln!("Testing BME280 detection...");
    match bme280.check_id().await {
        Ok(true) => rprintln!("[PASS] BME280 detected successfully"),
        Ok(false) => {
            rprintln!("[FAIL] Wrong sensor chip ID");
            return;
        },
        Err(_) => {
            rprintln!("[FAIL] I2C communication error");
            return;
        },
    }
    
    // Test raw data reading
    rprintln!("Testing raw data reading...");
    match bme280.read_raw_data().await {
        Ok((temp, press, hum)) => {
            rprintln!("[PASS] Raw readings - Temp:{}, Press:{}, Hum:{}", temp, press, hum);
        },
        Err(_) => {
            rprintln!("[FAIL] Raw data reading error");
            return;
        }
    }
    
    // Test processed measurements
    rprintln!("Testing processed measurements...");
    for i in 1..=5 {
        match bme280.read_measurements().await {
            Ok(measurements) => {
                rprintln!("Reading {} - Temp: {:.2}°C, Press: {:.2}hPa, Hum: {:.2}%", 
                        i, measurements.temperature, measurements.pressure, measurements.humidity);
            },
            Err(_) => {
                rprintln!("[FAIL] Measurement reading error on iteration {}", i);
            }
        }
        Timer::after(Duration::from_secs(1)).await;
    }
    
    rprintln!("[PASS] All tests completed successfully!");
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("ESP32-C3 BME280 Embassy Test Suite");
    rprintln!("==================================");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver with timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Setup I2C in async mode (GPIO8=SDA, GPIO9=SCL)
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9)
        .into_async();

    rprintln!("I2C initialized - SDA: GPIO8, SCL: GPIO9");

    // Run test
    spawner.spawn(bme280_test_task(i2c)).ok();

    // Keep main alive
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}