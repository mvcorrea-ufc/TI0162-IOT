//! Zero-dependency BME280 Driver - PURE BLOCKING
//! Direct I2C register access for maximum performance

use esp_hal::i2c::master::{I2c, Operation};
use rtt_target::rprintln;
use crate::config::NodepsConfig;

/// BME280 Register Addresses - MINIMAL SET
const BME280_CHIP_ID_REG: u8 = 0xD0;
const BME280_RESET_REG: u8 = 0xE0;
const BME280_CTRL_HUM_REG: u8 = 0xF2;
const BME280_CTRL_MEAS_REG: u8 = 0xF4;
const BME280_CONFIG_REG: u8 = 0xF5;
const BME280_PRESS_MSB_REG: u8 = 0xF7;

/// BME280 Expected Chip ID
const BME280_CHIP_ID: u8 = 0x60;

/// BME280 Reset Command
const BME280_RESET_CMD: u8 = 0xB6;

/// BME280 Calibration Data Registers
const BME280_CALIB_00_REG: u8 = 0x88; // dig_T1 LSB
const BME280_CALIB_25_REG: u8 = 0xA1; // dig_H1
const BME280_CALIB_26_REG: u8 = 0xE1; // dig_H2 LSB

/// Sensor Data Structure
#[derive(Debug, Clone, Copy)]
pub struct SensorData {
    pub temperature: f32,  // °C
    pub pressure: f32,     // hPa
    pub humidity: f32,     // %RH
}

/// BME280 Calibration Data
#[derive(Debug)]
struct CalibrationData {
    // Temperature calibration
    dig_t1: u16,
    dig_t2: i16,
    dig_t3: i16,
    
    // Pressure calibration
    dig_p1: u16,
    dig_p2: i16,
    dig_p3: i16,
    dig_p4: i16,
    dig_p5: i16,
    dig_p6: i16,
    dig_p7: i16,
    dig_p8: i16,
    dig_p9: i16,
    
    // Humidity calibration
    dig_h1: u8,
    dig_h2: i16,
    dig_h3: u8,
    dig_h4: i16,
    dig_h5: i16,
    dig_h6: i8,
}

/// BME280 Driver
pub struct Bme280 {
    calibration: CalibrationData,
    t_fine: i32, // Used for pressure and humidity compensation
}

impl Bme280 {
    /// Initialize BME280 sensor with direct register access
    pub fn new_blocking(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<Self, &'static str> {
        rprintln!("[BME280] Initializing BME280 sensor...");
        
        // Check chip ID
        let mut chip_id = [0u8; 1];
        if let Err(_) = i2c.transaction(
            NodepsConfig::bme280_address(),
            &mut [
                Operation::Write(&[BME280_CHIP_ID_REG]),
                Operation::Read(&mut chip_id),
            ],
        ) {
            return Err("Failed to read chip ID");
        }
        
        if chip_id[0] != BME280_CHIP_ID {
            rprintln!("[BME280] ERROR: Invalid chip ID: 0x{:02X} (expected 0x{:02X})", chip_id[0], BME280_CHIP_ID);
            return Err("Invalid chip ID");
        }
        
        rprintln!("[BME280] Chip ID verified: 0x{:02X}", chip_id[0]);
        
        // Reset sensor
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_RESET_REG, BME280_RESET_CMD]) {
            return Err("Failed to reset sensor");
        }
        
        // Wait for reset to complete
        blocking_delay_ms(10);
        
        // Read calibration data
        let calibration = Self::read_calibration_data(i2c)?;
        
        // Configure sensor for normal mode
        // Humidity oversampling x1
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_CTRL_HUM_REG, 0x01]) {
            return Err("Failed to configure humidity");
        }
        
        // Initial configuration: Temperature oversampling x1, Pressure oversampling x1, Sleep mode
        // We'll use forced mode for each measurement
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_CTRL_MEAS_REG, 0x24]) {
            return Err("Failed to configure measurement");
        }
        
        // Standby time 1000ms, Filter off, 3-wire SPI disabled
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_CONFIG_REG, 0xA0]) {
            return Err("Failed to configure sensor");
        }
        
        rprintln!("[BME280] Sensor configured and ready");
        
        Ok(Self {
            calibration,
            t_fine: 0,
        })
    }
    
    /// Read calibration data from sensor
    fn read_calibration_data(i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<CalibrationData, &'static str> {
        let mut calib_00_25 = [0u8; 26]; // T1-P9 (0x88-0xA1)
        let mut calib_26_41 = [0u8; 16]; // H1-H6 (0xE1-0xF0)
        
        // Read first calibration block
        if let Err(_) = i2c.transaction(
            NodepsConfig::bme280_address(),
            &mut [
                Operation::Write(&[BME280_CALIB_00_REG]),
                Operation::Read(&mut calib_00_25),
            ],
        ) {
            return Err("Failed to read calibration data block 1");
        }
        
        // Read H1
        let mut h1 = [0u8; 1];
        if let Err(_) = i2c.transaction(
            NodepsConfig::bme280_address(),
            &mut [
                Operation::Write(&[BME280_CALIB_25_REG]),
                Operation::Read(&mut h1),
            ],
        ) {
            return Err("Failed to read H1 calibration");
        }
        
        // Read second calibration block
        if let Err(_) = i2c.transaction(
            NodepsConfig::bme280_address(),
            &mut [
                Operation::Write(&[BME280_CALIB_26_REG]),
                Operation::Read(&mut calib_26_41),
            ],
        ) {
            return Err("Failed to read calibration data block 2");
        }
        
        // Parse calibration data
        let dig_t1 = u16::from_le_bytes([calib_00_25[0], calib_00_25[1]]);
        let dig_t2 = i16::from_le_bytes([calib_00_25[2], calib_00_25[3]]);
        let dig_t3 = i16::from_le_bytes([calib_00_25[4], calib_00_25[5]]);
        
        let dig_p1 = u16::from_le_bytes([calib_00_25[6], calib_00_25[7]]);
        let dig_p2 = i16::from_le_bytes([calib_00_25[8], calib_00_25[9]]);
        let dig_p3 = i16::from_le_bytes([calib_00_25[10], calib_00_25[11]]);
        let dig_p4 = i16::from_le_bytes([calib_00_25[12], calib_00_25[13]]);
        let dig_p5 = i16::from_le_bytes([calib_00_25[14], calib_00_25[15]]);
        let dig_p6 = i16::from_le_bytes([calib_00_25[16], calib_00_25[17]]);
        let dig_p7 = i16::from_le_bytes([calib_00_25[18], calib_00_25[19]]);
        let dig_p8 = i16::from_le_bytes([calib_00_25[20], calib_00_25[21]]);
        let dig_p9 = i16::from_le_bytes([calib_00_25[22], calib_00_25[23]]);
        
        let dig_h1 = h1[0];
        let dig_h2 = i16::from_le_bytes([calib_26_41[0], calib_26_41[1]]);
        let dig_h3 = calib_26_41[2];
        let dig_h4 = ((calib_26_41[3] as i16) << 4) | ((calib_26_41[4] as i16) & 0x0F);
        let dig_h5 = ((calib_26_41[5] as i16) << 4) | ((calib_26_41[4] as i16) >> 4);
        let dig_h6 = calib_26_41[6] as i8;
        
        Ok(CalibrationData {
            dig_t1, dig_t2, dig_t3,
            dig_p1, dig_p2, dig_p3, dig_p4, dig_p5, dig_p6, dig_p7, dig_p8, dig_p9,
            dig_h1, dig_h2, dig_h3, dig_h4, dig_h5, dig_h6,
        })
    }
    
    /// Read sensor data
    pub fn read_data_blocking(&mut self, i2c: &mut I2c<'_, esp_hal::Blocking>) -> Result<SensorData, &'static str> {
        // Configure humidity oversampling first (must be done before ctrl_meas)
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_CTRL_HUM_REG, 0x01]) {
            return Err("Failed to configure humidity");
        }
        
        // Force measurement: temp x1, press x1, forced mode
        if let Err(_) = i2c.write(NodepsConfig::bme280_address(), &[BME280_CTRL_MEAS_REG, 0x25]) {
            return Err("Failed to trigger measurement");
        }
        
        // Wait for measurement to complete (BME280 datasheet: max 9.3ms for forced mode)
        blocking_delay_ms(15);
        
        // Read raw data (pressure, temperature, humidity)
        let mut raw_data = [0u8; 8];
        if let Err(_) = i2c.transaction(
            NodepsConfig::bme280_address(),
            &mut [
                Operation::Write(&[BME280_PRESS_MSB_REG]),
                Operation::Read(&mut raw_data),
            ],
        ) {
            return Err("Failed to read sensor data");
        }
        
        // Parse raw values
        let raw_pressure = ((raw_data[0] as u32) << 12) | ((raw_data[1] as u32) << 4) | ((raw_data[2] as u32) >> 4);
        let raw_temperature = ((raw_data[3] as u32) << 12) | ((raw_data[4] as u32) << 4) | ((raw_data[5] as u32) >> 4);
        let raw_humidity = ((raw_data[6] as u32) << 8) | (raw_data[7] as u32);
        
        // Compensate temperature
        let temperature = self.compensate_temperature(raw_temperature);
        
        // Compensate pressure
        let pressure = self.compensate_pressure(raw_pressure);
        
        // Compensate humidity
        let humidity = self.compensate_humidity(raw_humidity);
        
        Ok(SensorData {
            temperature,
            pressure,
            humidity,
        })
    }
    
    /// Temperature compensation (returns °C)
    fn compensate_temperature(&mut self, raw_temp: u32) -> f32 {
        let var1 = (raw_temp as i32 / 16384 - self.calibration.dig_t1 as i32 / 1024) * self.calibration.dig_t2 as i32;
        let var2 = (raw_temp as i32 / 131072 - self.calibration.dig_t1 as i32 / 8192) *
                  (raw_temp as i32 / 131072 - self.calibration.dig_t1 as i32 / 8192) *
                  self.calibration.dig_t3 as i32 / 16;
        
        self.t_fine = var1 + var2;
        (self.t_fine * 5 + 128) as f32 / 25600.0
    }
    
    /// Pressure compensation (returns hPa)
    fn compensate_pressure(&self, raw_press: u32) -> f32 {
        let mut var1 = self.t_fine as i64 - 128000;
        let mut var2 = var1 * var1 * self.calibration.dig_p6 as i64;
        var2 = var2 + ((var1 * self.calibration.dig_p5 as i64) << 17);
        var2 = var2 + ((self.calibration.dig_p4 as i64) << 35);
        var1 = ((var1 * var1 * self.calibration.dig_p3 as i64) >> 8) + ((var1 * self.calibration.dig_p2 as i64) << 12);
        var1 = (((1i64 << 47) + var1) * self.calibration.dig_p1 as i64) >> 33;
        
        if var1 == 0 {
            return 0.0; // Avoid division by zero
        }
        
        let mut p = 1048576 - raw_press as i64;
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = (self.calibration.dig_p9 as i64 * (p >> 13) * (p >> 13)) >> 25;
        var2 = (self.calibration.dig_p8 as i64 * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((self.calibration.dig_p7 as i64) << 4);
        
        p as f32 / 25600.0
    }
    
    /// Humidity compensation (returns %RH)
    fn compensate_humidity(&self, raw_hum: u32) -> f32 {
        let var_h = self.t_fine - 76800;
        let var_h = (raw_hum as i32 - (self.calibration.dig_h4 as i32 * 64 + self.calibration.dig_h5 as i32 / 16384 * var_h)) *
                   (self.calibration.dig_h2 as i32 / 65536 * (1 + self.calibration.dig_h6 as i32 / 67108864 * var_h *
                   (1 + self.calibration.dig_h3 as i32 / 67108864 * var_h)));
        let var_h = var_h * (1 - self.calibration.dig_h1 as i32 * var_h / 524288);
        
        if var_h > 100 * 32768 {
            100.0
        } else if var_h < 0 {
            0.0
        } else {
            var_h as f32 / 32768.0
        }
    }
}

/// Simple blocking delay
fn blocking_delay_ms(ms: u32) {
    let cycles = ms * 240_000; // ESP32-C3 at 240MHz
    for _ in 0..cycles {
        unsafe { 
            let dummy: u32 = 0;
            core::ptr::read_volatile(&dummy); 
        }
    }
}