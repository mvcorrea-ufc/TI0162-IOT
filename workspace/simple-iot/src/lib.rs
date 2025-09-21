//! Simple IoT ESP32-C3 Library
//! 
//! This library provides common functionality for ESP32-C3 IoT applications,
//! including a working BME280/BMP280 sensor implementation.

#![no_std]

pub mod bme280;

// Re-export the main types for easy access
pub use bme280::{SimpleBME280, Measurements};