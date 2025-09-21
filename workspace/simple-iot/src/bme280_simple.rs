//! Minimal BME280 Console Test - Custom Implementation
//! Direct register access for raw sensor readings

#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use esp_hal::{
    delay::Delay,
    i2c::master::{I2c, Config as I2cConfig},
    main,
};

use simple_iot::{SimpleBME280, Measurements};

#[main]
fn main() -> ! {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("🌡️  ESP32-C3 BME280 Minimal Console Test (Custom Implementation)");
    rprintln!("================================================================");
    rprintln!("Testing direct register access for raw sensor readings");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    
    // Initialize I2C - same setup that worked for detection
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);

    rprintln!("I2C initialized - SDA: GPIO8, SCL: GPIO9");
    
    // Extended stabilization
    delay.delay_millis(2000);
    rprintln!("I2C bus stabilized (2000ms delay)");
    
    // Create custom BME280 sensor 
    rprintln!("\\n🔧 Initializing custom BME280 implementation...");
    
    let mut sensor = SimpleBME280::new(&mut i2c, 0x76);
    
    // Check if sensor is present
    match sensor.check_id() {
        Ok(true) => {
            rprintln!("✅ BME280 chip ID verified (0x60)");
        }
        Ok(false) => {
            rprintln!("❌ Wrong chip ID - not a BME280");
            loop {
                delay.delay_millis(5000);
                rprintln!("💀 Wrong chip - check sensor type");
            }
        }
        Err(e) => {
            rprintln!("❌ Failed to read chip ID: {}", e);
            loop {
                delay.delay_millis(5000);
                rprintln!("💀 Chip ID read failed - check wiring");
            }
        }
    }
    
    // Initialize sensor with custom configuration
    match sensor.init() {
        Ok(_) => {
            rprintln!("✅ Custom BME280 initialized successfully!");
            rprintln!("📋 Configuration: 1x oversampling, forced mode");
        }
        Err(e) => {
            rprintln!("❌ Custom BME280 initialization failed: {}", e);
            
            loop {
                delay.delay_millis(5000);
                rprintln!("💀 Custom init failed - check hardware");
            }
        }
    }
    
    // Additional stabilization
    delay.delay_millis(1000);
    rprintln!("🕐 Post-initialization stabilization complete");
    
    rprintln!("\\n🚀 Starting sensor reading loop...");
    rprintln!("=====================================");
    
    let mut reading_count = 0;
    
    // Main sensor reading loop
    loop {
        reading_count += 1;
        
        rprintln!("\\n📊 Reading #{} (every 3 seconds)", reading_count);
        rprintln!("--------------------------------");
        
        // Method 1: Try custom measurements with forced mode
        match sensor.read_measurements() {
            Ok(measurements) => {
                rprintln!("✅ Custom read_measurements() SUCCESS");
                
                rprintln!("🌡️  Temperature: {:.2}°C", measurements.temperature);
                rprintln!("📊 Pressure: {:.2} hPa", measurements.pressure);
                
                if measurements.humidity > 0.0 {
                    rprintln!("💧 Humidity: {:.2}% (BME280)", measurements.humidity);
                    rprintln!("📈 REAL BME280 DATA: T={:.1}°C, H={:.1}%, P={:.1}hPa", 
                             measurements.temperature, measurements.humidity, measurements.pressure);
                } else {
                    rprintln!("💧 Humidity: N/A (BMP280 or disabled)");
                    rprintln!("📈 REAL BMP280 DATA: T={:.1}°C, P={:.1}hPa", 
                             measurements.temperature, measurements.pressure);
                }
                
                rprintln!("✅ Values are properly calibrated using BME280 algorithms");
            }
            Err(e) => {
                rprintln!("❌ Custom read_measurements() FAILED: {}", e);
                
                // Method 2: Try raw value reading for debugging
                rprintln!("🔄 Trying raw value reads...");
                
                match sensor.read_raw_data() {
                    Ok((raw_temp, raw_press, raw_hum)) => {
                        rprintln!("✅ Raw values read successfully:");
                        rprintln!("   Raw Temperature: 0x{:X} ({})", raw_temp, raw_temp);
                        rprintln!("   Raw Pressure: 0x{:X} ({})", raw_press, raw_press);
                        rprintln!("   Raw Humidity: 0x{:X} ({})", raw_hum, raw_hum);
                        
                        // Check for invalid readings
                        if raw_temp == 0x80000 {
                            rprintln!("⚠️  Temperature reading invalid (0x80000)");
                        }
                        if raw_press == 0x80000 {
                            rprintln!("⚠️  Pressure reading invalid (0x80000)");
                        }
                        if raw_hum == 0x8000 {
                            rprintln!("⚠️  Humidity reading invalid (0x8000)");
                        }
                    }
                    Err(e) => {
                        rprintln!("❌ Raw value reads also failed: {}", e);
                        rprintln!("🔍 This indicates I2C communication issues");
                    }
                }
            }
        }
        
        // Wait 3 seconds before next reading
        delay.delay_millis(3000);
    }
}