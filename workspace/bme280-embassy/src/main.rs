#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use esp_hal::{
    gpio::{Level, Output},
    i2c::master::{I2c, Config as I2cConfig},
    timer::timg::TimerGroup,
    Async,
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use bme280_embassy::BME280;

// BME280 I2C address (primary)
const BME280_ADDRESS: u8 = 0x76;

// GPIO pin definitions for I2C
const SDA_PIN: u8 = 8;  // GPIO8 for SDA
const SCL_PIN: u8 = 9;  // GPIO9 for SCL
const LED_PIN: u8 = 3;  // GPIO3 for LED indicator

#[embassy_executor::task]
async fn sensor_task(mut i2c: I2c<'static, Async>) {
    rprintln!("Starting BME280 sensor task...");
    
    // Initialize BME280 sensor
    let mut bme280 = BME280::new(&mut i2c);

    // Check if BME280 is present and responding
    match bme280.check_id().await {
        Ok(true) => rprintln!("BME280 initialized successfully!"),
        Ok(false) => {
            rprintln!("Wrong sensor detected!");
            return;
        },
        Err(_) => {
            rprintln!("Failed to communicate with BME280 sensor!");
            return;
        },
    }

    // Try to read chip ID register directly for debugging
    match bme280.read_chip_id_raw().await {
        Ok(id) => rprintln!("Chip ID: 0x{:02X} (expected: 0x60)", id),
        Err(_) => rprintln!("Failed to read chip ID"),
    }

    // Initialize the BME280 for measurements
    match bme280.init().await {
        Ok(_) => rprintln!("BME280 sensor initialized for measurements"),
        Err(_) => {
            rprintln!("Failed to initialize BME280!");
            return;
        }
    }

    // Show calibration data for debugging
    let cal = bme280.get_calibration_debug();
    rprintln!("Calibration data loaded:");
    rprintln!("  T1={}, T2={}, T3={}", cal.dig_t1, cal.dig_t2, cal.dig_t3);
    rprintln!("  P1={}, P2={}, P3={}", cal.dig_p1, cal.dig_p2, cal.dig_p3);
    rprintln!("  H1={}, H2={}, H3={}, H4={}, H5={}, H6={}", 
              cal.dig_h1, cal.dig_h2, cal.dig_h3, cal.dig_h4, cal.dig_h5, cal.dig_h6);

    rprintln!("BME280 configured and ready!");

    // Main sensor reading loop
    loop {
        // First check raw data
        match bme280.read_raw_data().await {
            Ok((raw_temp, raw_press, raw_hum)) => {
                rprintln!("Raw Data: T={}, P={}, H={}", raw_temp, raw_press, raw_hum);
                
                // Debug humidity calculation
                let cal = bme280.get_calibration_debug();
                if raw_hum > 0 {
                    rprintln!("Debug H: raw_hum={}, H1={}, H2={}, H3={}, H4={}, H5={}, H6={}", 
                              raw_hum, cal.dig_h1, cal.dig_h2, cal.dig_h3, cal.dig_h4, cal.dig_h5, cal.dig_h6);
                    // Add more detailed debug
                    rprintln!("  t_fine={}, expected H3 should be ~19-100", 3100); // approx for 31C
                }
            }
            Err(_) => {
                rprintln!("Error reading raw data!");
            }
        }
        
        match bme280.read_measurements().await {
            Ok(measurements) => {
                rprintln!("BME280 Readings:");
                rprintln!("  Temperature: {:.2} °C", measurements.temperature);
                rprintln!("  Humidity:    {:.1} %", measurements.humidity);
                rprintln!("  Pressure:    {:.2} hPa", measurements.pressure);
                rprintln!("  Expected for Fortaleza: T=26-30°C, H=68-84%, P=~1012hPa");
                rprintln!("  ------------------------");
            }
            Err(_) => {
                rprintln!("Error reading BME280 sensor data!");
            }
        }

        // Wait 2 seconds before next reading
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    rprintln!("Starting LED heartbeat task...");
    
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(100)).await;
        led.set_low();
        Timer::after(Duration::from_millis(900)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize RTT for console output (following blinky pattern)
    rtt_init_print!();
    
    rprintln!("ESP32-C3 BME280 Weather Station with Embassy");
    rprintln!("============================================");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver with timer
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Set up LED (GPIO3) for status indication
    let led = Output::new(peripherals.GPIO3, Level::Low);

    // Set up I2C in async mode (GPIO8=SDA, GPIO9=SCL)
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9)
        .into_async();

    rprintln!("Hardware initialized:");
    rprintln!("  I2C SDA: GPIO{}", SDA_PIN);
    rprintln!("  I2C SCL: GPIO{}", SCL_PIN);  
    rprintln!("  LED:     GPIO{}", LED_PIN);
    rprintln!("  BME280:  0x{:02X}", BME280_ADDRESS);

    // Spawn Embassy tasks
    spawner.spawn(sensor_task(i2c)).ok();
    spawner.spawn(led_task(led)).ok();

    rprintln!("Embassy tasks spawned successfully!");
    
    // Main task can do other work or just wait
    loop {
        Timer::after(Duration::from_secs(10)).await;
        rprintln!("System running... ({}s uptime)", 
                embassy_time::Instant::now().as_secs());
    }
}