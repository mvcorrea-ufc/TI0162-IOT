//! BME280 Environmental Sensor Driver
//! 
//! A complete working implementation based on the proven simple-iot module.
//! This implementation uses the I2cDevice abstraction layer for clean architecture
//! while maintaining compatibility with Phase 2 performance optimization goals.

use crate::i2c_device::I2cDevice;
use iot_common::{IoTError, error::utils::error_message};

/// BME280 register addresses
const BME280_CHIP_ID_REG: u8 = 0xD0;
const BME280_RESET_REG: u8 = 0xE0;
const BME280_CTRL_HUM_REG: u8 = 0xF2;
const BME280_STATUS_REG: u8 = 0xF3;
const BME280_CTRL_MEAS_REG: u8 = 0xF4;
const BME280_CONFIG_REG: u8 = 0xF5;

/// BME280 data registers  
const BME280_PRESS_MSB_REG: u8 = 0xF7;
const BME280_TEMP_MSB_REG: u8 = 0xFA;
const BME280_HUM_MSB_REG: u8 = 0xFD;

/// BME280 calibration registers
const BME280_CALIB_T1_LSB_REG: u8 = 0x88;
const BME280_CALIB_H1_REG: u8 = 0xA1;
const BME280_CALIB_H2_LSB_REG: u8 = 0xE1;

/// Expected chip IDs
const BME280_CHIP_ID: u8 = 0x60;
const BMP280_CHIP_ID: u8 = 0x58;

/// Environmental sensor measurements
#[derive(Debug, Clone, PartialEq)]
pub struct Measurements {
    /// Temperature in degrees Celsius
    pub temperature: f32,
    /// Atmospheric pressure in hectopascals (hPa)
    pub pressure: f32,
    /// Relative humidity as percentage (0.0 for BMP280)
    pub humidity: f32,
}

/// BME280 calibration coefficients
#[derive(Debug, Default, Clone)]
pub struct CalibrationData {
    // Temperature coefficients
    pub dig_t1: u16,
    pub dig_t2: i16,
    pub dig_t3: i16,
    
    // Pressure coefficients
    pub dig_p1: u16,
    pub dig_p2: i16,
    pub dig_p3: i16,
    pub dig_p4: i16,
    pub dig_p5: i16,
    pub dig_p6: i16,
    pub dig_p7: i16,
    pub dig_p8: i16,
    pub dig_p9: i16,
    
    // Humidity coefficients
    pub dig_h1: u8,
    pub dig_h2: i16,
    pub dig_h3: u8,
    pub dig_h4: i16,
    pub dig_h5: i16,
    pub dig_h6: i8,
}

/// BME280 environmental sensor driver using I2C abstraction layer
/// 
/// This implementation is based on the proven working simple-iot module
/// and uses the I2cDevice abstraction for clean architecture and testability.
/// Compatible with Phase 2 performance optimization and HAL abstraction goals.
pub struct BME280<'a> {
    i2c_dev: I2cDevice<'a>,
    calib_data: Option<CalibrationData>,
}

impl<'a> BME280<'a> {
    /// Creates a new BME280 driver instance
    /// 
    /// # Arguments
    /// 
    /// * `i2c_dev` - I2C device abstraction configured for BME280 communication
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use bme280_embassy::{BME280, I2cDevice};
    /// use esp_hal::i2c::master::{I2c, Config as I2cConfig};
    /// 
    /// let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
    ///     .unwrap()
    ///     .with_sda(peripherals.GPIO8)
    ///     .with_scl(peripherals.GPIO9);
    ///     
    /// let i2c_dev = I2cDevice::new(&mut i2c, 0x76);
    /// let mut sensor = BME280::new(i2c_dev);
    /// ```
    pub fn new(i2c_dev: I2cDevice<'a>) -> Self {
        Self {
            i2c_dev,
            calib_data: None,
        }
    }

    /// Initialize the BME280 sensor
    /// 
    /// This method performs complete sensor initialization:
    /// 1. Verifies sensor presence and chip ID
    /// 2. Reads factory calibration coefficients 
    /// 3. Configures sensor registers for optimal operation
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Sensor successfully initialized
    /// * `Err(IoTError)` - Initialization failed with error details
    pub async fn init(&mut self) -> Result<(), IoTError> {
        // Check chip ID to verify sensor presence
        let chip_id = self.i2c_dev.read_register(BME280_CHIP_ID_REG).await?;
            
        match chip_id {
            BME280_CHIP_ID => {
                // BME280 detected - humidity sensor available
            }
            BMP280_CHIP_ID => {
                // BMP280 detected - no humidity sensor
            }
            _ => {
                // Invalid chip ID - return error with message
                return Err(IoTError::sensor(iot_common::SensorError::InitializationFailed(error_message("Invalid BME280 chip ID"))));
            }
        }

        // Read calibration data
        self.read_calibration_data().await?;

        // Reset sensor
        self.i2c_dev.write_register(BME280_RESET_REG, 0xB6).await?;

        // Wait for reset to complete
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;

        // Configure sensor for forced mode measurements
        self.configure_sensor().await?;

        Ok(())
    }

    /// Read compensated environmental measurements
    /// 
    /// This method performs a complete measurement cycle:
    /// 1. Triggers forced measurement
    /// 2. Waits for completion  
    /// 3. Reads raw data
    /// 4. Applies calibration compensation
    /// 
    /// # Returns
    /// 
    /// * `Ok(Measurements)` - Compensated temperature, pressure, and humidity
    /// * `Err(IoTError)` - Measurement failed
    pub async fn read_measurements(&mut self) -> Result<Measurements, IoTError> {
        // Add delay before triggering measurement (hardware stabilization)
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        
        // Trigger forced measurement
        self.force_measurement().await?;

        // Wait for measurement completion
        self.wait_for_measurement().await?;

        // Read raw sensor data
        let (raw_temp, raw_press, raw_hum) = self.read_raw_data().await?;

        // Apply calibration compensation
        let calib = self.calib_data.as_ref()
            .ok_or_else(|| IoTError::sensor(iot_common::SensorError::InitializationFailed(error_message("Calibration data not available"))))?;
            
        let (temperature, t_fine) = Self::compensate_temperature(raw_temp, calib);
        let pressure = Self::compensate_pressure(raw_press, t_fine, calib);
        let humidity = Self::compensate_humidity(raw_hum, t_fine, calib);

        Ok(Measurements {
            temperature,
            pressure,
            humidity,
        })
    }

    /// Get calibration data for debugging
    pub fn get_calibration_data(&self) -> Option<&CalibrationData> {
        self.calib_data.as_ref()
    }

    // Private implementation methods

    async fn read_calibration_data(&mut self) -> Result<(), IoTError> {
        let mut calib = CalibrationData::default();

        // Read temperature and pressure calibration (0x88-0x9F)
        let mut calib_tp = [0u8; 24];
        self.i2c_dev.read_registers(BME280_CALIB_T1_LSB_REG, &mut calib_tp).await?;

        // Parse temperature coefficients
        calib.dig_t1 = u16::from_le_bytes([calib_tp[0], calib_tp[1]]);
        calib.dig_t2 = i16::from_le_bytes([calib_tp[2], calib_tp[3]]);
        calib.dig_t3 = i16::from_le_bytes([calib_tp[4], calib_tp[5]]);

        // Parse pressure coefficients
        calib.dig_p1 = u16::from_le_bytes([calib_tp[6], calib_tp[7]]);
        calib.dig_p2 = i16::from_le_bytes([calib_tp[8], calib_tp[9]]);
        calib.dig_p3 = i16::from_le_bytes([calib_tp[10], calib_tp[11]]);
        calib.dig_p4 = i16::from_le_bytes([calib_tp[12], calib_tp[13]]);
        calib.dig_p5 = i16::from_le_bytes([calib_tp[14], calib_tp[15]]);
        calib.dig_p6 = i16::from_le_bytes([calib_tp[16], calib_tp[17]]);
        calib.dig_p7 = i16::from_le_bytes([calib_tp[18], calib_tp[19]]);
        calib.dig_p8 = i16::from_le_bytes([calib_tp[20], calib_tp[21]]);
        calib.dig_p9 = i16::from_le_bytes([calib_tp[22], calib_tp[23]]);

        // Read humidity calibration H1 (0xA1)
        calib.dig_h1 = self.i2c_dev.read_register(BME280_CALIB_H1_REG).await?;

        // Read humidity calibration H2-H6 (0xE1-0xE7)
        let mut calib_h = [0u8; 7];
        self.i2c_dev.read_registers(BME280_CALIB_H2_LSB_REG, &mut calib_h).await?;

        // Parse humidity coefficients (complex bit manipulation)
        calib.dig_h2 = i16::from_le_bytes([calib_h[0], calib_h[1]]);
        calib.dig_h3 = calib_h[2];
        calib.dig_h4 = ((calib_h[3] as i16) << 4) | ((calib_h[4] as i16) & 0x0F);
        calib.dig_h5 = ((calib_h[5] as i16) << 4) | ((calib_h[4] as i16) >> 4);
        calib.dig_h6 = calib_h[6] as i8;

        // Sign extend 12-bit values
        if calib.dig_h4 > 2047 { calib.dig_h4 -= 4096; }
        if calib.dig_h5 > 2047 { calib.dig_h5 -= 4096; }

        self.calib_data = Some(calib);
        Ok(())
    }

    async fn configure_sensor(&mut self) -> Result<(), IoTError> {
        // Configure humidity oversampling (must be done before CTRL_MEAS)
        self.i2c_dev.write_register(BME280_CTRL_HUM_REG, 0x01).await?; // 1x oversampling

        // Configure config register (standby time, filter, SPI disable)
        self.i2c_dev.write_register(BME280_CONFIG_REG, 0x00).await?; // Standby 0.5ms, filter off

        Ok(())
    }

    async fn force_measurement(&mut self) -> Result<(), IoTError> {
        // Configure humidity oversampling (must be done before CTRL_MEAS)
        self.i2c_dev.write_register(BME280_CTRL_HUM_REG, 0x01).await?;
        
        // Configure temperature and pressure oversampling + forced mode
        // Bits [7:5] = temp oversampling (001 = 1x)
        // Bits [4:2] = press oversampling (001 = 1x)  
        // Bits [1:0] = mode (01 = forced mode)
        self.i2c_dev.write_register(BME280_CTRL_MEAS_REG, 0b00100101).await
    }

    async fn wait_for_measurement(&mut self) -> Result<(), IoTError> {
        // Wait for measurement completion (status register bit 3 = measuring)
        // Give the sensor more time for the measurement cycle
        embassy_time::Timer::after(embassy_time::Duration::from_millis(20)).await;
        
        // Check status register a few times, but don't rely on it completely
        for _i in 0..10 {
            if let Ok(status) = self.i2c_dev.read_register(BME280_STATUS_REG).await {
                if (status & 0x08) == 0 {
                    return Ok(()); // Measurement complete
                }
            }
            // If we can't read status, give it more time and continue
            embassy_time::Timer::after(embassy_time::Duration::from_millis(5)).await;
        }
        
        // Even if status check fails, proceed with reading - some sensors work this way
        Ok(())
    }

    async fn read_raw_data(&mut self) -> Result<(i32, i32, i32), IoTError> {
        // Read pressure (0xF7-0xF9)
        let mut press_data = [0u8; 3];
        self.i2c_dev.read_registers(BME280_PRESS_MSB_REG, &mut press_data).await?;
        let raw_press = ((press_data[0] as i32) << 12) | 
                       ((press_data[1] as i32) << 4) | 
                       ((press_data[2] as i32) >> 4);

        // Read temperature (0xFA-0xFC)
        let mut temp_data = [0u8; 3];
        self.i2c_dev.read_registers(BME280_TEMP_MSB_REG, &mut temp_data).await?;
        let raw_temp = ((temp_data[0] as i32) << 12) | 
                      ((temp_data[1] as i32) << 4) | 
                      ((temp_data[2] as i32) >> 4);

        // Read humidity (0xFD-0xFE)
        let mut hum_data = [0u8; 2];
        self.i2c_dev.read_registers(BME280_HUM_MSB_REG, &mut hum_data).await?;
        let raw_hum = ((hum_data[0] as i32) << 8) | (hum_data[1] as i32);

        Ok((raw_temp, raw_press, raw_hum))
    }

    // Official BME280 compensation algorithms from datasheet

    fn compensate_temperature(adc_t: i32, calib: &CalibrationData) -> (f32, i32) {
        let var1 = (((adc_t >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32)) >> 11;
        let var2 = (((((adc_t >> 4) - (calib.dig_t1 as i32)) * 
                      ((adc_t >> 4) - (calib.dig_t1 as i32))) >> 12) * 
                     (calib.dig_t3 as i32)) >> 14;
        
        let t_fine = var1 + var2;
        let temperature = (t_fine * 5 + 128) >> 8;
        
        (temperature as f32 / 100.0, t_fine)
    }

    fn compensate_pressure(adc_p: i32, t_fine: i32, calib: &CalibrationData) -> f32 {
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
        
        (p as f32) / 25600.0
    }

    fn compensate_humidity(adc_h: i32, t_fine: i32, calib: &CalibrationData) -> f32 {
        if adc_h == 0x8000 {
            return 0.0; // Invalid reading (BMP280 or measurement not ready)
        }

        let v_x1_u32r = t_fine - 76800;
        
        let v_x1_u32r = (adc_h << 14) - ((calib.dig_h4 as i32) << 20) - 
                        ((calib.dig_h5 as i32) * v_x1_u32r) + 16384;
        let v_x1_u32r = v_x1_u32r >> 15;
        
        let v_x1_u32r = v_x1_u32r * (((((v_x1_u32r * (calib.dig_h6 as i32)) >> 10) * 
                        (((v_x1_u32r * (calib.dig_h3 as i32)) >> 11) + 32768)) >> 10) + 2097152);
        let v_x1_u32r = ((v_x1_u32r + 8192) >> 14) * (calib.dig_h2 as i32);
        let v_x1_u32r = v_x1_u32r - (((((v_x1_u32r >> 15) * (v_x1_u32r >> 15)) >> 7) * 
                        (calib.dig_h1 as i32)) >> 4);
        
        let v_x1_u32r = if v_x1_u32r < 0 { 0 } else { v_x1_u32r };
        let v_x1_u32r = if v_x1_u32r > 419430400 { 419430400 } else { v_x1_u32r };
        
        (v_x1_u32r >> 12) as f32 / 1024.0
    }
}