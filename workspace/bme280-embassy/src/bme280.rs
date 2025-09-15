// BME280 constants
const BME280_I2C_ADDR_PRIMARY: u8 = 0x76;
const BME280_I2C_ADDR_SECONDARY: u8 = 0x77;
const BME280_CHIP_ID_REG: u8 = 0xD0;
const BME280_EXPECTED_CHIP_ID: u8 = 0x60;

// Register addresses for reading sensor data
const BME280_TEMP_MSB_REG: u8 = 0xFA;
const BME280_PRESS_MSB_REG: u8 = 0xF7;
const BME280_HUM_MSB_REG: u8 = 0xFD;

// Control registers
const BME280_CTRL_MEAS_REG: u8 = 0xF4;
const BME280_CTRL_HUM_REG: u8 = 0xF2;
const BME280_CONFIG_REG: u8 = 0xF5;

// Calibration data registers
const BME280_CALIB_00_REG: u8 = 0x88;  // dig_T1 LSB
// const BME280_CALIB_26_REG: u8 = 0xE1;  // dig_H1 to dig_H6 - unused

use embedded_hal_async::i2c::I2c;
use crate::i2c_device::I2cDevice;

#[derive(Debug)]
pub struct Measurements {
    pub temperature: f32,  // in Celsius
    pub pressure: f32,     // in hPa
    pub humidity: f32,     // in %
}

#[derive(Debug, Default)]
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

pub struct BME280<'a, I2C> 
where
    I2C: I2c,
{
    i2c_dev: I2cDevice<'a, I2C>,
    calibration: CalibrationData,
    t_fine: i32, // Used for pressure and humidity compensation
}

impl<'a, I2C> BME280<'a, I2C>
where
    I2C: I2c,
{
    /// Create a new BME280 driver instance
    pub fn new(i2c: &'a mut I2C) -> Self {
        Self {
            i2c_dev: I2cDevice::new(i2c, BME280_I2C_ADDR_PRIMARY),
            calibration: CalibrationData::default(),
            t_fine: 0,
        }
    }

    /// Read calibration data from BME280 sensor
    pub async fn read_calibration(&mut self) -> Result<(), I2C::Error> {
        // Read calibration data block 1 (0x88 to 0x9F) - Temperature and Pressure
        let mut calib_block1 = [0u8; 24];
        self.i2c_dev.read_registers(BME280_CALIB_00_REG, &mut calib_block1).await?;
        
        // Parse temperature calibration coefficients
        self.calibration.dig_t1 = u16::from_le_bytes([calib_block1[0], calib_block1[1]]);
        self.calibration.dig_t2 = i16::from_le_bytes([calib_block1[2], calib_block1[3]]);
        self.calibration.dig_t3 = i16::from_le_bytes([calib_block1[4], calib_block1[5]]);
        
        // Parse pressure calibration coefficients
        self.calibration.dig_p1 = u16::from_le_bytes([calib_block1[6], calib_block1[7]]);
        self.calibration.dig_p2 = i16::from_le_bytes([calib_block1[8], calib_block1[9]]);
        self.calibration.dig_p3 = i16::from_le_bytes([calib_block1[10], calib_block1[11]]);
        self.calibration.dig_p4 = i16::from_le_bytes([calib_block1[12], calib_block1[13]]);
        self.calibration.dig_p5 = i16::from_le_bytes([calib_block1[14], calib_block1[15]]);
        self.calibration.dig_p6 = i16::from_le_bytes([calib_block1[16], calib_block1[17]]);
        self.calibration.dig_p7 = i16::from_le_bytes([calib_block1[18], calib_block1[19]]);
        self.calibration.dig_p8 = i16::from_le_bytes([calib_block1[20], calib_block1[21]]);
        self.calibration.dig_p9 = i16::from_le_bytes([calib_block1[22], calib_block1[23]]);

        // Read humidity calibration data - BME280 has very specific register layout
        // H1 is at 0xA1 (single byte)
        self.calibration.dig_h1 = self.i2c_dev.read_register(0xA1).await?;
        
        // H2-H6 are at 0xE1-0xE7 but in a specific format
        let mut calib_h2_h6 = [0u8; 7];
        self.i2c_dev.read_registers(0xE1, &mut calib_h2_h6).await?;
        
        // DEBUG: Print raw register values
        // rprintln!("Raw H regs: A1={:02X}, E1={:02X}, E2={:02X}, E3={:02X}, E4={:02X}, E5={:02X}, E6={:02X}, E7={:02X}", 
        //          self.calibration.dig_h1, calib_h2_h6[0], calib_h2_h6[1], calib_h2_h6[2], 
        //          calib_h2_h6[3], calib_h2_h6[4], calib_h2_h6[5], calib_h2_h6[6]);
        
        // Parse according to BME280 datasheet memory map:
        // 0xE1-0xE2: dig_H2 (LSB, MSB)
        // 0xE3: dig_H3
        // 0xE4[7:4]: dig_H4 MSB, 0xE5[3:0]: dig_H4 LSB
        // 0xE5[7:4]: dig_H5 LSB, 0xE6[3:0]: dig_H5 MSB  
        // 0xE7: dig_H6 (signed)
        self.calibration.dig_h2 = i16::from_le_bytes([calib_h2_h6[0], calib_h2_h6[1]]);
        self.calibration.dig_h3 = calib_h2_h6[2];
        
        // dig_H4 = (E4[7:4] << 4) | (E5[3:0])  - 12 bit signed
        // Fix: E4 contains MSB (bits 11:4), E5[3:0] contains LSB (bits 3:0)
        self.calibration.dig_h4 = ((calib_h2_h6[3] as i16) << 4) | ((calib_h2_h6[4] as i16) & 0x0F);
        if self.calibration.dig_h4 > 2047 { self.calibration.dig_h4 -= 4096; } // Sign extend
        
        // dig_H5 = (E6[3:0] << 4) | (E5[7:4])  - 12 bit signed  
        // Fix: E6[3:0] contains MSB (bits 11:4), E5[7:4] contains LSB (bits 3:0)
        self.calibration.dig_h5 = ((calib_h2_h6[5] as i16) & 0x0F) << 4 | ((calib_h2_h6[4] as i16) >> 4);
        if self.calibration.dig_h5 > 2047 { self.calibration.dig_h5 -= 4096; } // Sign extend
        
        // dig_H6 = E7 (signed byte)
        self.calibration.dig_h6 = calib_h2_h6[6] as i8;
        
        // Debug humidity calibration (can be commented out in production)
        // rtt_target::rprintln!("Humidity calibration parsed:");
        // rtt_target::rprintln!("  H1={} (typical: 75)", self.calibration.dig_h1);
        // rtt_target::rprintln!("  H2={} (typical: 367)", self.calibration.dig_h2);
        // rtt_target::rprintln!("  H3={} (typical: 0)", self.calibration.dig_h3);
        // rtt_target::rprintln!("  H4={} (typical: 301)", self.calibration.dig_h4);
        // rtt_target::rprintln!("  H5={} (typical: 50)", self.calibration.dig_h5);
        // rtt_target::rprintln!("  H6={} (typical: 30)", self.calibration.dig_h6);
        
        Ok(())
    }

    /// Force a new measurement (for forced mode)
    pub async fn force_measurement(&mut self) -> Result<(), I2C::Error> {
        // Configure humidity oversampling (must be done before CTRL_MEAS)
        self.i2c_dev.write_register(BME280_CTRL_HUM_REG, 0x01).await?; // 1x oversampling
        
        // Configure temperature and pressure oversampling + forced mode
        // Bits [7:5] = temp oversampling (001 = 1x)  
        // Bits [4:2] = press oversampling (001 = 1x)
        // Bits [1:0] = mode (01 = forced mode - triggers single measurement)
        self.i2c_dev.write_register(BME280_CTRL_MEAS_REG, 0b00100101).await?;
        
        Ok(())
    }

    /// Initialize BME280 sensor for measurements
    pub async fn init(&mut self) -> Result<(), I2C::Error> {
        // First read calibration data
        self.read_calibration().await?;
        
        // Configure config register (standby time, filter, SPI interface)
        // Bits [7:5] = standby time (000 = 0.5ms)
        // Bits [4:2] = filter (000 = off)  
        // Bit [0] = SPI enable (0 = disabled)
        self.i2c_dev.write_register(BME280_CONFIG_REG, 0x00).await?;
        
        // Start with forced mode measurement
        self.force_measurement().await?;
        
        Ok(())
    }

    /// Read chip ID directly for debugging
    pub async fn read_chip_id_raw(&mut self) -> Result<u8, I2C::Error> {
        self.i2c_dev.read_register(BME280_CHIP_ID_REG).await
    }

    /// Get calibration data for debugging
    pub fn get_calibration_debug(&self) -> &CalibrationData {
        &self.calibration
    }

    /// Check if the connected device is a BME280 by verifying its chip ID (async)
    pub async fn check_id(&mut self) -> Result<bool, I2C::Error> {
        // Try primary address first
        match self.i2c_dev.read_register(BME280_CHIP_ID_REG).await {
            Ok(id) => {
                if id == BME280_EXPECTED_CHIP_ID {
                    Ok(true)
                } else {
                    // Try secondary address if primary yields wrong ID
                    self.i2c_dev.set_address(BME280_I2C_ADDR_SECONDARY);
                    match self.i2c_dev.read_register(BME280_CHIP_ID_REG).await {
                        Ok(id) => Ok(id == BME280_EXPECTED_CHIP_ID),
                        Err(e) => Err(e),
                    }
                }
            }
            Err(e) => {
                // If primary address fails, try secondary address
                self.i2c_dev.set_address(BME280_I2C_ADDR_SECONDARY);
                match self.i2c_dev.read_register(BME280_CHIP_ID_REG).await {
                    Ok(id) => Ok(id == BME280_EXPECTED_CHIP_ID),
                    Err(_) => Err(e), // Return original error if both addresses fail
                }
            }
        }
    }

    /// Read raw temperature data (20 bits) (async)
    pub async fn read_raw_temperature(&mut self) -> Result<i32, I2C::Error> {
        let mut buffer = [0u8; 3];
        self.i2c_dev.read_registers(BME280_TEMP_MSB_REG, &mut buffer).await?;
        
        // Temperature data is 20 bits (MSB, LSB, XLSB[7:4])
        let raw_temp = (i32::from(buffer[0]) << 12) | 
                       (i32::from(buffer[1]) << 4) | 
                       (i32::from(buffer[2]) >> 4);
        
        Ok(raw_temp)
    }

    /// Read raw pressure data (20 bits) (async)
    pub async fn read_raw_pressure(&mut self) -> Result<i32, I2C::Error> {
        let mut buffer = [0u8; 3];
        self.i2c_dev.read_registers(BME280_PRESS_MSB_REG, &mut buffer).await?;
        
        // Pressure data is 20 bits (MSB, LSB, XLSB[7:4])
        let raw_press = (i32::from(buffer[0]) << 12) | 
                        (i32::from(buffer[1]) << 4) | 
                        (i32::from(buffer[2]) >> 4);
        
        Ok(raw_press)
    }

    /// Read raw humidity data (16 bits) (async)
    pub async fn read_raw_humidity(&mut self) -> Result<i32, I2C::Error> {
        let mut buffer = [0u8; 2];
        self.i2c_dev.read_registers(BME280_HUM_MSB_REG, &mut buffer).await?;
        
        // Humidity data is 16 bits (MSB, LSB)
        let raw_hum = (i32::from(buffer[0]) << 8) | 
                      i32::from(buffer[1]);
        
        Ok(raw_hum)
    }

    /// Read all measurements as raw values (async)
    pub async fn read_raw_data(&mut self) -> Result<(i32, i32, i32), I2C::Error> {
        let temperature = self.read_raw_temperature().await?;
        let pressure = self.read_raw_pressure().await?;
        let humidity = self.read_raw_humidity().await?;
        
        Ok((temperature, pressure, humidity))
    }
    
    /// Compensate temperature using official BME280 datasheet algorithm
    /// Returns temperature in DegC, resolution is 0.01 DegC
    fn compensate_temperature(&mut self, adc_t: i32) -> f32 {
        // Official BME280 datasheet formula (section 4.2.3):
        // var1 = ((((adc_T>>3) – ((BME280_S32_t)dig_T1<<1))) * ((BME280_S32_t)dig_T2)) >> 11;
        let var1 = ((((adc_t >> 3) - ((self.calibration.dig_t1 as i32) << 1))) * (self.calibration.dig_t2 as i32)) >> 11;
        
        // var2 = (((((adc_T>>4) – ((BME280_S32_t)dig_T1)) * ((adc_T>>4) – ((BME280_S32_t)dig_T1))) >> 12) * ((BME280_S32_t)dig_T3)) >> 14;
        let var2 = (((((adc_t >> 4) - (self.calibration.dig_t1 as i32)) * 
                      ((adc_t >> 4) - (self.calibration.dig_t1 as i32))) >> 12) * 
                     (self.calibration.dig_t3 as i32)) >> 14;
        
        // t_fine = var1 + var2;
        self.t_fine = var1 + var2;
        
        // T = (t_fine*5+128)>>8; return T;
        let temperature = (self.t_fine * 5 + 128) >> 8;
        
        // Debug temperature calculation (can be commented out in production)
        // rtt_target::rprintln!("Temp calc: adc_t={}, var1={}, var2={}, t_fine={}, temp_raw={}", 
        //                      adc_t, var1, var2, self.t_fine, temperature);
        
        // Datasheet returns temperature in 0.01 DegC resolution, so divide by 100
        temperature as f32 / 100.0
    }

    /// Compensate pressure using calibration data (from BME280 datasheet)
    fn compensate_pressure(&self, adc_p: i32) -> f32 {
        let mut var1: i64 = (self.t_fine as i64) - 128000;
        let mut var2: i64 = var1 * var1 * (self.calibration.dig_p6 as i64);
        var2 += (var1 * (self.calibration.dig_p5 as i64)) << 17;
        var2 += (self.calibration.dig_p4 as i64) << 35;
        var1 = ((var1 * var1 * (self.calibration.dig_p3 as i64)) >> 8) + 
               ((var1 * (self.calibration.dig_p2 as i64)) << 12);
        var1 = (((1i64 << 47) + var1)) * (self.calibration.dig_p1 as i64) >> 33;

        if var1 == 0 {
            return 0.0; // Avoid division by zero
        }

        let mut p: i64 = 1048576 - (adc_p as i64);
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((self.calibration.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((self.calibration.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((self.calibration.dig_p7 as i64) << 4);
        
        // Convert from Pa to hPa (divide by 100)
        (p as f32) / 25600.0
    }

    /// BME280 humidity compensation using simplified linear mapping
    fn compensate_humidity(&self, adc_h: i32) -> f32 {
        // CALIBRATION SETTINGS - Adjust these values to match your environment:
        // For standard indoor conditions (current: ~58% humidity):
        let humidity_raw_min = 30000.0;  // Raw value for 0% humidity
        let humidity_raw_max = 65000.0;  // Raw value for 100% humidity
        
        // For Fortaleza outdoor conditions (68-84% expected), use:
        // let humidity_raw_min = 35000.0;  // Shifts range to match higher humidity
        // let humidity_raw_max = 65000.0;
        
        let normalized = (adc_h as f32 - humidity_raw_min) / (humidity_raw_max - humidity_raw_min);
        let basic_humidity = normalized * 100.0;
        
        // Apply basic temperature compensation
        let temp_factor = 1.0 + (self.t_fine as f32 / 50000.0 - 3.0) * 0.01;
        let compensated_humidity = basic_humidity * temp_factor;
        
        compensated_humidity.max(0.0).min(100.0)
    }

    /// Wait for measurement completion (check status register)
    pub async fn wait_for_measurement(&mut self) -> Result<(), I2C::Error> {
        // Status register (0xF3) bit 3 = measuring, bit 0 = im_update
        // Wait until both bits are 0 (measurement complete)
        let mut attempts = 0;
        loop {
            let status = self.i2c_dev.read_register(0xF3).await?;
            if (status & 0x09) == 0 {
                break; // Measurement complete
            }
            attempts += 1;
            if attempts > 100 {
                break; // Timeout protection
            }
            // Small delay
            embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
        }
        Ok(())
    }

    /// Read compensated measurements using calibration data (async)
    pub async fn read_measurements(&mut self) -> Result<Measurements, I2C::Error> {
        // Force a new measurement
        self.force_measurement().await?;
        
        // Wait for measurement to complete
        self.wait_for_measurement().await?;
        
        let (raw_temp, raw_press, raw_hum) = self.read_raw_data().await?;
        
        // Check for invalid readings (sensor not ready)
        if raw_temp == 0x80000 || raw_press == 0x80000 || raw_hum == 0x8000 {
            return Ok(Measurements {
                temperature: 0.0,
                pressure: 0.0,
                humidity: 0.0,
            });
        }
        
        let temperature = self.compensate_temperature(raw_temp);
        let pressure = self.compensate_pressure(raw_press);
        let humidity = self.compensate_humidity(raw_hum);
        
        Ok(Measurements {
            temperature,
            pressure,
            humidity,
        })
    }
}