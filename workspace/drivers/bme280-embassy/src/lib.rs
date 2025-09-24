//! # BME280 Embassy Driver
//!
//! A complete async driver for the BME280 environmental sensor using the Embassy framework.
//! This driver provides accurate temperature, humidity, and pressure measurements with
//! proper calibration compensation.
//!
//! ## Features
//!
//! - **Async/Await Support**: All I/O operations are non-blocking using Embassy
//! - **Automatic Calibration**: Reads and applies sensor calibration coefficients
//! - **Dual Address Support**: Supports both 0x76 and 0x77 I2C addresses
//! - **Accurate Compensation**: Uses official BME280 algorithms for data compensation
//! - **Error Handling**: Comprehensive error handling with context preservation
//! - **No-std Compatible**: Works in embedded environments without heap allocation
//!
//! ## Quick Start
//!
//! ```no_run
//! use bme280_embassy::{BME280, Measurements};
//! use esp_hal::i2c::master::{I2c, Config as I2cConfig};
//! use embassy_time::{Timer, Duration};
//!
//! #[embassy_executor::task]
//! async fn sensor_task() {
//!     let peripherals = esp_hal::init(esp_hal::Config::default());
//!     
//!     let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
//!         .unwrap()
//!         .with_sda(peripherals.GPIO8)
//!         .with_scl(peripherals.GPIO9);
//!         
//!     let mut sensor = BME280::new(&mut i2c, 0x76);
//!     
//!     // Initialize sensor
//!     sensor.init().await.unwrap();
//!     
//!     loop {
//!         match sensor.read_measurements().await {
//!             Ok(measurements) => {
//!                 println!("Temperature: {:.2}°C", measurements.temperature);
//!                 println!("Humidity: {:.2}%RH", measurements.humidity);
//!                 println!("Pressure: {:.2} hPa", measurements.pressure);
//!             }
//!             Err(e) => {
//!                 println!("Sensor error: {:?}", e);
//!             }
//!         }
//!         
//!         Timer::after(Duration::from_secs(10)).await;
//!     }
//! }
//! ```
//!
//! ## Hardware Setup
//!
//! Connect the BME280 sensor to your ESP32-C3:
//!
//! | ESP32-C3 Pin | BME280 Pin | Description |
//! |--------------|------------|-------------|
//! | GPIO8        | SDA        | I2C Data    |
//! | GPIO9        | SCL        | I2C Clock   |
//! | 3.3V         | VCC        | Power       |
//! | GND          | GND        | Ground      |
//!
//! **Important**: Most BME280 modules have built-in pull-up resistors. External pull-ups are usually not needed.
//!
//! ## Sensor Specifications
//!
//! - **Temperature Range**: -40°C to +85°C (±1°C accuracy)
//! - **Humidity Range**: 0-100% RH (±3% accuracy)
//! - **Pressure Range**: 300-1100 hPa (±1 hPa accuracy)
//! - **I2C Addresses**: 0x76 (primary), 0x77 (secondary)
//! - **Supply Voltage**: 1.8V - 3.6V (3.3V recommended)
//! - **Current Consumption**: 3.4μA @ 1Hz humidity and temperature
//!
//! ## Integration with IoT Common
//!
//! This driver integrates with the unified error handling system:
//!
//! ```no_run
//! use iot_common::{IoTResult, IoTError};
//! 
//! async fn sensor_with_error_handling() -> IoTResult<f32> {
//!     let mut sensor = BME280::new(&mut i2c, 0x76);
//!     
//!     let measurements = sensor.read_measurements().await
//!         .map_err(|e| IoTError::from(e))?;
//!     
//!     Ok(measurements.temperature)
//! }
//! ```

#![no_std]

mod i2c_device;
mod bme280;

// Re-export types that should be accessible to users
pub use bme280::{BME280, Measurements, CalibrationData};
pub use i2c_device::I2cDevice;