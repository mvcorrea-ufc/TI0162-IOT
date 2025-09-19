//! BME280 Algorithm Tests
//!
//! This library contains host-only tests for BME280 algorithms and business logic.
//! It has no embedded dependencies and runs on standard Rust targets.

#![no_std]

// Re-export core types for testing
pub use core::{fmt, mem};

/// BME280 Constants for testing
pub mod constants {
    pub const BME280_I2C_ADDR_PRIMARY: u8 = 0x76;
    pub const BME280_I2C_ADDR_SECONDARY: u8 = 0x77;
    pub const BME280_CHIP_ID_REG: u8 = 0xD0;
    pub const BME280_EXPECTED_CHIP_ID: u8 = 0x60;
}

/// BME280 calibration data structure
#[derive(Debug, Default, Clone)]
pub struct CalibrationData {
    // Temperature calibration
    pub dig_t1: u16,
    pub dig_t2: i16,
    pub dig_t3: i16,
    
    // Pressure calibration
    pub dig_p1: u16,
    pub dig_p2: i16,
    pub dig_p3: i16,
    pub dig_p4: i16,
    pub dig_p5: i16,
    pub dig_p6: i16,
    pub dig_p7: i16,
    pub dig_p8: i16,
    pub dig_p9: i16,
    
    // Humidity calibration
    pub dig_h1: u8,
    pub dig_h2: i16,
    pub dig_h3: u8,
    pub dig_h4: i16,
    pub dig_h5: i16,
    pub dig_h6: i8,
}

/// Measurement results
#[derive(Debug, Clone, PartialEq)]
pub struct Measurements {
    pub temperature: f32,  // in Celsius
    pub pressure: f32,     // in hPa
    pub humidity: f32,     // in %
}

/// BME280 algorithm implementations for testing
pub mod algorithms {
    use super::CalibrationData;

    /// Temperature compensation using BME280 datasheet algorithm
    pub fn compensate_temperature(adc_t: i32, calib: &CalibrationData) -> (f32, i32) {
        let var1 = ((((adc_t >> 3) - ((calib.dig_t1 as i32) << 1))) * (calib.dig_t2 as i32)) >> 11;
        let var2 = (((((adc_t >> 4) - (calib.dig_t1 as i32)) * 
                     ((adc_t >> 4) - (calib.dig_t1 as i32))) >> 12) * 
                    (calib.dig_t3 as i32)) >> 14;
        
        let t_fine = var1 + var2;
        let temperature = (t_fine * 5 + 128) >> 8;
        
        (temperature as f32 / 100.0, t_fine)
    }

    /// Pressure compensation using BME280 datasheet algorithm
    pub fn compensate_pressure(adc_p: i32, t_fine: i32, calib: &CalibrationData) -> f32 {
        let mut var1: i64 = (t_fine as i64) - 128000;
        let mut var2: i64 = var1 * var1 * (calib.dig_p6 as i64);
        var2 += (var1 * (calib.dig_p5 as i64)) << 17;
        var2 += (calib.dig_p4 as i64) << 35;
        var1 = ((var1 * var1 * (calib.dig_p3 as i64)) >> 8) + 
               ((var1 * (calib.dig_p2 as i64)) << 12);
        var1 = (((1i64 << 47) + var1)) * (calib.dig_p1 as i64) >> 33;

        if var1 == 0 {
            return 0.0; // Avoid division by zero
        }

        let mut p: i64 = 1048576 - (adc_p as i64);
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((calib.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((calib.dig_p7 as i64) << 4);
        
        // Convert from Pa to hPa
        (p as f32) / 25600.0
    }

    /// Simplified humidity compensation using linear mapping
    pub fn compensate_humidity(adc_h: i32) -> f32 {
        let humidity_raw_min = 30000.0;
        let humidity_raw_max = 65000.0;
        
        let normalized = (adc_h as f32 - humidity_raw_min) / (humidity_raw_max - humidity_raw_min);
        let basic_humidity = normalized * 100.0;
        
        // Clamp to valid range
        basic_humidity.max(0.0).min(100.0)
    }
}