//! # BME280 with Hardware Abstraction Layer Integration
//!
//! This example demonstrates how to use the BME280 sensor with the new
//! hardware abstraction layer, enabling platform-independent code and testing.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use iot_hal::{HardwarePlatform, HardwareConfig};
use bme280_embassy::BME280;
use iot_common::IoTError;

// Platform-specific imports
#[cfg(feature = "esp32c3")]
use iot_hal::esp32c3::Esp32C3Platform;

#[cfg(feature = "mock")]
use iot_hal::mock::MockPlatform;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

/// Main sensor reading task using hardware abstraction
#[embassy_executor::task]
async fn sensor_task() -> Result<(), IoTError> {
    rprintln!("Starting BME280 sensor with Hardware Abstraction Layer");

    // Initialize hardware platform (ESP32-C3 or mock for testing)
    let mut platform = initialize_platform().await?;
    
    rprintln!("Platform initialized: {}", platform.platform_info());

    // Check platform health
    if !platform.is_healthy().await {
        rprintln!("WARNING: Platform health check failed");
    }

    // Create BME280 sensor using abstracted I2C interface
    let i2c = platform.get_i2c();
    let mut sensor = BME280::new(i2c);

    // Verify sensor connection
    rprintln!("Checking BME280 sensor connectivity...");
    match sensor.check_id().await {
        Ok(true) => {
            rprintln!("✓ BME280 sensor detected successfully");
        }
        Ok(false) => {
            rprintln!("✗ Device found but not BME280 sensor");
            return Ok(()); // Exit gracefully
        }
        Err(e) => {
            rprintln!("✗ Sensor communication failed: {:?}", e);
            return Ok(()); // Exit gracefully
        }
    }

    // Initialize sensor for measurements
    rprintln!("Initializing BME280 sensor...");
    sensor.init().await?;
    rprintln!("✓ BME280 sensor initialized successfully");

    // Get status LED for visual feedback
    let led = platform.get_status_led();

    // Main measurement loop
    rprintln!("Starting environmental monitoring (10-second intervals)");
    let mut reading_count = 0u32;

    loop {
        // Flash LED to indicate measurement
        led.set_high().await?;
        Timer::after(Duration::from_millis(50)).await;
        led.set_low().await?;

        // Read environmental measurements
        match sensor.read_measurements().await {
            Ok(measurements) => {
                reading_count += 1;
                rprintln!(
                    "[{}] T={:.2}°C, H={:.1}%RH, P={:.1}hPa",
                    reading_count,
                    measurements.temperature,
                    measurements.humidity,
                    measurements.pressure
                );

                // Validate measurements (basic sanity check)
                if measurements.temperature < -40.0 || measurements.temperature > 85.0 {
                    rprintln!("WARNING: Temperature out of range");
                }
                if measurements.humidity < 0.0 || measurements.humidity > 100.0 {
                    rprintln!("WARNING: Humidity out of range");
                }
                if measurements.pressure < 300.0 || measurements.pressure > 1100.0 {
                    rprintln!("WARNING: Pressure out of range");
                }
            }
            Err(e) => {
                rprintln!("ERROR: Failed to read measurements: {:?}", e);
                
                // Flash LED rapidly to indicate error
                for _ in 0..5 {
                    led.set_high().await?;
                    Timer::after(Duration::from_millis(100)).await;
                    led.set_low().await?;
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        }

        // Wait before next measurement
        Timer::after(Duration::from_secs(10)).await;
    }
}

/// Initialize hardware platform based on target
async fn initialize_platform() -> Result<impl HardwarePlatform, IoTError> {
    let config = HardwareConfig::default();
    
    #[cfg(feature = "esp32c3")]
    {
        rprintln!("Initializing ESP32-C3 hardware platform");
        Esp32C3Platform::initialize_with_config(config).await
    }
    
    #[cfg(feature = "mock")]
    {
        rprintln!("Initializing mock hardware platform for testing");
        MockPlatform::initialize_with_config(config).await
    }
    
    #[cfg(not(any(feature = "esp32c3", feature = "mock")))]
    {
        compile_error!("Must enable either 'esp32c3' or 'mock' feature");
    }
}

/// Platform health monitoring task
#[embassy_executor::task]
async fn health_monitor_task() {
    let mut platform = initialize_platform().await.unwrap();
    
    loop {
        let healthy = platform.is_healthy().await;
        if !healthy {
            rprintln!("HEALTH: Platform health check failed");
        }
        
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize RTT for debugging
    rtt_init_print!();
    
    rprintln!("=== BME280 Hardware Abstraction Layer Example ===");
    rprintln!("Platform: {}", env!("CARGO_CFG_TARGET_ARCH"));
    
    // Spawn sensor monitoring task
    spawner.spawn(sensor_task()).ok();
    
    // Spawn health monitoring task
    spawner.spawn(health_monitor_task()).ok();
    
    rprintln!("All tasks spawned successfully");
    
    // Main loop
    loop {
        Timer::after(Duration::from_secs(60)).await;
        rprintln!("System running - hardware abstraction layer active");
    }
}