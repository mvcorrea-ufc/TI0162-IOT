//! BME280 Algorithm Tests
//!
//! Comprehensive tests for BME280 sensor algorithms running on host (x86_64).
//! These tests validate the mathematical algorithms without hardware dependencies.

use bme280_tests::{
    algorithms::*,
    constants::*,
    CalibrationData, 
    Measurements
};

#[test]
fn test_bme280_constants() {
    // Verify BME280 constants are correct
    assert_eq!(BME280_I2C_ADDR_PRIMARY, 0x76);
    assert_eq!(BME280_I2C_ADDR_SECONDARY, 0x77);
    assert_eq!(BME280_CHIP_ID_REG, 0xD0);
    assert_eq!(BME280_EXPECTED_CHIP_ID, 0x60);
    
    // Verify I2C addresses are valid 7-bit addresses
    assert!(BME280_I2C_ADDR_PRIMARY <= 0x7F);
    assert!(BME280_I2C_ADDR_SECONDARY <= 0x7F);
    assert_ne!(BME280_I2C_ADDR_PRIMARY, BME280_I2C_ADDR_SECONDARY);
}

#[test]
fn test_temperature_compensation_algorithm() {
    // Create realistic calibration data from BME280 datasheet example
    let calibration = CalibrationData {
        dig_t1: 27504,
        dig_t2: 26435,
        dig_t3: -1000,
        ..Default::default()
    };

    // Test case 1: Room temperature (~23°C)
    let adc_t = 519888; // Raw ADC value for ~23°C
    let (temp_celsius, t_fine) = compensate_temperature(adc_t, &calibration);
    
    // Should be approximately 23°C
    assert!(temp_celsius >= 20.0 && temp_celsius <= 26.0);
    assert!(t_fine > 0 && t_fine < 200000);

    // Test case 2: Cold temperature
    let adc_cold = 400000;
    let (temp_cold, _) = compensate_temperature(adc_cold, &calibration);
    
    // Test case 3: Hot temperature
    let adc_hot = 600000;
    let (temp_hot, _) = compensate_temperature(adc_hot, &calibration);
    
    // Verify temperature increases with ADC value for BME280
    assert!(temp_cold < temp_celsius);
    assert!(temp_celsius < temp_hot);
    
    // Test extreme values
    let adc_very_cold = 300000;
    let (temp_very_cold, _) = compensate_temperature(adc_very_cold, &calibration);
    assert!(temp_very_cold < temp_cold);
}

#[test]
fn test_pressure_compensation_algorithm() {
    // Create realistic calibration data
    let calibration = CalibrationData {
        dig_t1: 27504, dig_t2: 26435, dig_t3: -1000,
        dig_p1: 36477, dig_p2: -10685, dig_p3: 3024, dig_p4: 2855,
        dig_p5: 140, dig_p6: -7, dig_p7: 15500, dig_p8: -14600, dig_p9: 6000,
        ..Default::default()
    };

    let t_fine = 128066; // From temperature compensation
    let adc_p = 415148;  // Raw pressure reading

    let pressure_hpa = compensate_pressure(adc_p, t_fine, &calibration);

    // Should be around standard atmospheric pressure
    assert!(pressure_hpa >= 800.0 && pressure_hpa <= 1200.0);
    assert!(pressure_hpa >= 950.0 && pressure_hpa <= 1100.0);

    // Test division by zero protection with zero calibration
    let zero_calib = CalibrationData {
        dig_p1: 0, // This will cause division by zero
        ..calibration
    };
    
    let zero_pressure = compensate_pressure(adc_p, t_fine, &zero_calib);
    assert_eq!(zero_pressure, 0.0);

    // Test pressure variation with different ADC values
    let pressure_low = compensate_pressure(300000, t_fine, &calibration);
    let pressure_high = compensate_pressure(500000, t_fine, &calibration);
    
    // Different ADC values should give different pressures
    assert!(pressure_low != pressure_high);
}

#[test]
fn test_humidity_compensation() {
    // Test boundary conditions
    assert_eq!(compensate_humidity(30000), 0.0);   // Minimum
    assert_eq!(compensate_humidity(65000), 100.0); // Maximum

    // Test middle value (should be ~50%)
    let mid_humidity = compensate_humidity(47500);
    assert!((mid_humidity - 50.0).abs() < 1.0);

    // Test clamping behavior
    assert_eq!(compensate_humidity(20000), 0.0);   // Below minimum -> 0%
    assert_eq!(compensate_humidity(80000), 100.0); // Above maximum -> 100%

    // Test typical indoor humidity range
    let indoor = compensate_humidity(45000);
    assert!(indoor >= 30.0 && indoor <= 70.0);

    // Test monotonicity within valid range
    let hum_30k = compensate_humidity(30000);
    let hum_40k = compensate_humidity(40000);
    let hum_50k = compensate_humidity(50000);
    let hum_60k = compensate_humidity(60000);
    let hum_65k = compensate_humidity(65000);
    
    assert!(hum_30k <= hum_40k);
    assert!(hum_40k <= hum_50k);
    assert!(hum_50k <= hum_60k);
    assert!(hum_60k <= hum_65k);
}

#[test]
fn test_calibration_data_structure() {
    let calibration = CalibrationData {
        dig_t1: 27504, dig_t2: 26435, dig_t3: -1000,
        dig_p1: 36477, dig_p2: -10685, dig_p3: 3024, dig_p4: 2855,
        dig_p5: 140, dig_p6: -7, dig_p7: 15500, dig_p8: -14600, dig_p9: 6000,
        dig_h1: 75, dig_h2: 367, dig_h3: 0, dig_h4: 301, dig_h5: 50, dig_h6: 30,
    };

    // Verify structure is memory-efficient for embedded systems
    assert!(core::mem::size_of::<CalibrationData>() <= 64);
    assert!(core::mem::align_of::<CalibrationData>() <= 8);

    // Verify typical ranges
    assert!(calibration.dig_t1 > 0 && calibration.dig_t1 < 50000);
    assert!(calibration.dig_p1 > 0 && calibration.dig_p1 < 50000);
    assert!(calibration.dig_h1 > 0 && calibration.dig_h1 < 200);

    // Some values can be negative (this is normal for BME280)
    assert!(calibration.dig_t3 < 0);
    assert!(calibration.dig_p2 < 0);

    // Test default initialization
    let default_calib = CalibrationData::default();
    assert_eq!(default_calib.dig_t1, 0);
    assert_eq!(default_calib.dig_p1, 0);
    assert_eq!(default_calib.dig_h1, 0);

    // Test cloning
    let cloned = calibration.clone();
    assert_eq!(cloned.dig_t1, calibration.dig_t1);
    assert_eq!(cloned.dig_p1, calibration.dig_p1);
    assert_eq!(cloned.dig_h1, calibration.dig_h1);
}

#[test]
fn test_measurements_structure() {
    let measurements = Measurements {
        temperature: 23.5,
        pressure: 1013.25,
        humidity: 65.2,
    };

    // Verify BME280 operating ranges
    assert!(measurements.temperature >= -40.0 && measurements.temperature <= 85.0);
    assert!(measurements.pressure >= 300.0 && measurements.pressure <= 1100.0);
    assert!(measurements.humidity >= 0.0 && measurements.humidity <= 100.0);

    // Verify structure efficiency
    assert!(core::mem::size_of::<Measurements>() <= 16);
    assert!(core::mem::align_of::<Measurements>() <= 4);

    // Test cloning and comparison
    let cloned = measurements.clone();
    assert_eq!(measurements, cloned);

    // Test that values are finite
    assert!(measurements.temperature.is_finite());
    assert!(measurements.pressure.is_finite());
    assert!(measurements.humidity.is_finite());
}

#[test]
fn test_chip_id_validation() {
    // Test valid chip ID
    assert_eq!(BME280_EXPECTED_CHIP_ID, 0x60);

    // Common chip IDs that should NOT match BME280
    let invalid_ids = [
        0x58, // BMP280
        0x61, // BME680
        0xFF, // Invalid
        0x00, // No response
        0x55, // Random
    ];

    for &invalid_id in &invalid_ids {
        assert_ne!(invalid_id, BME280_EXPECTED_CHIP_ID);
    }
}

#[test]
fn test_raw_data_validation() {
    // BME280 specific invalid values (when sensor not ready)
    const INVALID_TEMP_PRESS: i32 = 0x80000;
    const INVALID_HUMIDITY: i32 = 0x8000;

    // Function to validate raw sensor data
    fn is_valid_raw_data(temp: i32, press: i32, hum: i32) -> bool {
        // BME280 specific invalid conditions
        if temp == INVALID_TEMP_PRESS || press == INVALID_TEMP_PRESS || hum == INVALID_HUMIDITY {
            return false;
        }
        // Raw values should be non-negative for BME280
        if temp < 0 || press < 0 || hum < 0 {
            return false;
        }
        true
    }

    // Test valid data (avoid using values that match invalid markers)
    assert!(is_valid_raw_data(519888, 415148, 30000));  // Changed humidity from 32768 to 30000

    // Test invalid data (sensor not ready)
    assert!(!is_valid_raw_data(INVALID_TEMP_PRESS, 415148, 30000));
    assert!(!is_valid_raw_data(519888, INVALID_TEMP_PRESS, 30000));
    assert!(!is_valid_raw_data(519888, 415148, INVALID_HUMIDITY));

    // Test negative values (should not happen with BME280)
    assert!(!is_valid_raw_data(-1, 415148, 30000));
    assert!(!is_valid_raw_data(519888, -1, 30000));
    assert!(!is_valid_raw_data(519888, 415148, -1));
}

#[test]
fn test_algorithm_edge_cases() {
    let calibration = CalibrationData {
        dig_t1: 27504, dig_t2: 26435, dig_t3: -1000,
        dig_p1: 36477, dig_p2: -10685, dig_p3: 3024, dig_p4: 2855,
        dig_p5: 140, dig_p6: -7, dig_p7: 15500, dig_p8: -14600, dig_p9: 6000,
        dig_h1: 75, dig_h2: 367, dig_h3: 0, dig_h4: 301, dig_h5: 50, dig_h6: 30,
    };

    // Test temperature algorithm with extreme values
    let (temp_min, t_fine_min) = compensate_temperature(0, &calibration);
    let (temp_max, t_fine_max) = compensate_temperature(1048575, &calibration); // 20-bit max
    
    // Extreme values should still produce reasonable results
    assert!(temp_min.is_finite());
    assert!(temp_max.is_finite());
    assert!(temp_min < temp_max); // Should still be monotonic
    assert!(t_fine_min != t_fine_max);

    // Test pressure algorithm with extreme t_fine values
    let pressure_cold = compensate_pressure(415148, -50000, &calibration);
    let pressure_hot = compensate_pressure(415148, 200000, &calibration);
    
    assert!(pressure_cold.is_finite());
    assert!(pressure_hot.is_finite());
    assert!(pressure_cold >= 0.0);
    assert!(pressure_hot >= 0.0);

    // Test humidity with extreme values
    let hum_zero = compensate_humidity(0);
    let hum_max = compensate_humidity(65535); // 16-bit max
    
    assert_eq!(hum_zero, 0.0);
    assert_eq!(hum_max, 100.0);
}

#[test]
fn test_performance_characteristics() {
    // Test that algorithms are efficient for embedded systems
    
    let calibration = CalibrationData {
        dig_t1: 27504, dig_t2: 26435, dig_t3: -1000,
        dig_p1: 36477, dig_p2: -10685, dig_p3: 3024, dig_p4: 2855,
        dig_p5: 140, dig_p6: -7, dig_p7: 15500, dig_p8: -14600, dig_p9: 6000,
        dig_h1: 75, dig_h2: 367, dig_h3: 0, dig_h4: 301, dig_h5: 50, dig_h6: 30,
    };

    // Perform multiple calculations to verify consistency
    for i in 0..100 {
        let adc_temp = 500000 + i * 100;
        let adc_press = 400000 + i * 50;
        let adc_hum = 40000 + i * 100;

        let (temp, t_fine) = compensate_temperature(adc_temp, &calibration);
        let pressure = compensate_pressure(adc_press, t_fine, &calibration);
        let humidity = compensate_humidity(adc_hum);

        // All results should be finite and reasonable
        assert!(temp.is_finite());
        assert!(pressure.is_finite());
        assert!(humidity.is_finite());
        
        assert!(temp >= -40.0 && temp <= 85.0);
        assert!(pressure >= 300.0 && pressure <= 1100.0);
        assert!(humidity >= 0.0 && humidity <= 100.0);
    }
}

#[test]
fn test_json_serialization_compatibility() {
    // Test that measurements can be serialized for MQTT
    let measurements = Measurements {
        temperature: 23.5,
        pressure: 1013.25,
        humidity: 65.2,
    };

    // Simple JSON-like formatting (for MQTT payloads)
    let json_str = format!(
        "{{\"temperature\":{:.1},\"humidity\":{:.1},\"pressure\":{:.2}}}",
        measurements.temperature, measurements.humidity, measurements.pressure
    );

    // Verify JSON contains expected values
    assert!(json_str.contains("\"temperature\":23.5"));
    assert!(json_str.contains("\"humidity\":65.2"));
    assert!(json_str.contains("\"pressure\":1013.25"));

    // Verify reasonable payload size for MQTT
    assert!(json_str.len() < 256);
    assert!(json_str.len() > 10);

    // Test with extreme values
    let extreme = Measurements {
        temperature: -25.7,
        pressure: 850.12,
        humidity: 95.8,
    };

    let extreme_json = format!(
        "{{\"temperature\":{:.1},\"humidity\":{:.1},\"pressure\":{:.2}}}",
        extreme.temperature, extreme.humidity, extreme.pressure
    );

    assert!(extreme_json.contains("-25.7"));
    assert!(extreme_json.contains("850.12"));
    assert!(extreme_json.contains("95.8"));
}

#[test]
fn test_memory_layout_efficiency() {
    // Verify structures are packed efficiently for embedded systems
    
    // CalibrationData should be reasonably sized
    assert!(core::mem::size_of::<CalibrationData>() <= 64);
    assert!(core::mem::align_of::<CalibrationData>() <= 8);
    
    // Measurements should be compact
    assert!(core::mem::size_of::<Measurements>() <= 16);
    assert!(core::mem::align_of::<Measurements>() <= 4);
    
    // Test that we're not wasting memory with padding
    let calib_size = core::mem::size_of::<CalibrationData>();
    let measurements_size = core::mem::size_of::<Measurements>();
    
    // Should be reasonable for ESP32-C3 with 48KB RAM
    let total_sensor_memory = calib_size + measurements_size + 128; // + working space
    assert!(total_sensor_memory <= 256); // Should use less than 256 bytes total
}