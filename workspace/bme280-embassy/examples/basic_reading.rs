#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Timer, Duration};

use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    i2c::master::{I2c, Config as I2cConfig},
};

use bme280_embassy::{BME280, I2cDevice};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("🌡️ ESP32-C3 BME280 Embassy - Basic Reading Test");
    rprintln!("================================================");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    
    let delay = Delay::new();
    
    // Set up LED (GPIO3) for status indication
    let mut led = Output::new(peripherals.GPIO3, Level::Low, OutputConfig::default());
    
    // Setup I2C (GPIO8=SDA, GPIO9=SCL)
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);

    rprintln!("✅ I2C initialized - SDA: GPIO8, SCL: GPIO9");
    
    // Test I2C communication by scanning for BME280
    rprintln!("🔍 Scanning for BME280 sensor...");
    
    // Create I2C device abstraction for BME280
    let i2c_device = I2cDevice::new(&mut i2c, 0x76);
    let mut sensor = BME280::new(i2c_device);
    
    // Try to initialize the sensor
    match sensor.init().await {
        Ok(()) => {
            rprintln!("✅ BME280 sensor initialized successfully!");
            
            // Display calibration data for debugging
            if let Some(calib) = sensor.get_calibration_data() {
                rprintln!("📊 Calibration data loaded:");
                rprintln!("   Temperature: T1={}, T2={}, T3={}", calib.dig_t1, calib.dig_t2, calib.dig_t3);
                rprintln!("   Pressure: P1={}, P2={}, P3={}", calib.dig_p1, calib.dig_p2, calib.dig_p3);
                rprintln!("   Humidity: H1={}, H2={}, H3={}", calib.dig_h1, calib.dig_h2, calib.dig_h3);
            }
            
            // Start measurement loop
            rprintln!("🚀 Starting measurement loop...");
            
            loop {
                led.toggle();
                
                match sensor.read_measurements().await {
                    Ok(measurements) => {
                        rprintln!("📊 BME280 Measurements:");
                        rprintln!("   🌡️  Temperature: {:.2}°C", measurements.temperature);
                        rprintln!("   💧 Humidity: {:.2}% RH", measurements.humidity);
                        rprintln!("   🌊 Pressure: {:.2} hPa", measurements.pressure);
                        rprintln!("   ✅ Reading successful");
                    }
                    Err(e) => {
                        rprintln!("❌ Sensor error: {:?}", e);
                    }
                }
                
                // Wait 5 seconds between readings
                Timer::after(Duration::from_secs(5)).await;
            }
        }
        Err(e) => {
            rprintln!("❌ BME280 initialization failed: {:?}", e);
            rprintln!("💡 Check wiring: SDA=GPIO8, SCL=GPIO9, VCC=3.3V, GND=GND");
            rprintln!("💡 Try different I2C address: 0x77 instead of 0x76");
            
            // Error blink pattern
            loop {
                for _ in 0..3 {
                    led.set_high();
                    delay.delay_millis(200);
                    led.set_low();
                    delay.delay_millis(200);
                }
                delay.delay_millis(1000);
            }
        }
    }
}