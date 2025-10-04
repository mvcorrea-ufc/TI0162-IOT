//! Simple BME280 Driver for Raw Value Reading
//! Based on direct register access without complex calibration

use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;

// BME280 constants
const BME280_I2C_ADDR_PRIMARY: u8 = 0x76;
const BME280_CHIP_ID_REG: u8 = 0xD0;
const BME280_EXPECTED_CHIP_ID: u8 = 0x60;

// Control registers
const BME280_CTRL_MEAS_REG: u8 = 0xF4;
const BME280_CTRL_HUM_REG: u8 = 0xF2;
const BME280_CONFIG_REG: u8 = 0xF5;

// Register addresses for reading sensor data
const BME280_TEMP_MSB_REG: u8 = 0xFA;
const BME280_PRESS_MSB_REG: u8 = 0xF7;
const BME280_HUM_MSB_REG: u8 = 0xFD;

// Calibration data registers
const BME280_CALIB_00_REG: u8 = 0x88; // dig_T1 LSB
const BME280_CALIB_25_REG: u8 = 0xA1; // dig_H1
const BME280_CALIB_26_REG: u8 = 0xE1; // dig_H2 LSB

#[derive(Debug)]
pub struct Measurements {
    pub temperature: f32,  // in Celsius
    pub pressure: f32,     // in hPa
    pub humidity: f32,     // in %
}

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

pub struct SimpleBME280<'a> {
    i2c: &'a mut I2c<'a, Blocking>,
    address: u8,
    calib_data: Option<CalibrationData>,
}

impl<'a> SimpleBME280<'a> {
    /// Create a new SimpleBME280 driver instance
    pub fn new(i2c: &'a mut I2c<'a, Blocking>, address: u8) -> Self {
        Self { 
            i2c, 
            address,
            calib_data: None,
        }
    }

    /// Check if the connected device is a BME280 by verifying its chip ID
    pub fn check_id(&mut self) -> Result<bool, &'static str> {
        match self.read_register(BME280_CHIP_ID_REG) {
            Ok(id) => Ok(id == BME280_EXPECTED_CHIP_ID),
            Err(_) => Err("Failed to read chip ID"),
        }
    }

    /// Initialize sensor with basic configuration and read calibration data
    pub fn init(&mut self) -> Result<(), &'static str> {
        // Read calibration data from sensor
        self.read_calibration_data()?;
        
        // BME280 CRITICAL: Humidity register must be written before CTRL_MEAS
        // Configure humidity oversampling (001 = 1x)
        self.write_register(BME280_CTRL_HUM_REG, 0x01)?;
        
        // Configure temperature and pressure oversampling + sleep mode first
        // Bits 7:5 = pressure oversampling (001 = 1x)
        // Bits 4:2 = temperature oversampling (001 = 1x)  
        // Bits 1:0 = mode (00 = sleep mode)
        self.write_register(BME280_CTRL_MEAS_REG, 0b00100100)?;
        
        // Configure standby time and filter
        self.write_register(BME280_CONFIG_REG, 0x00)?;
        
        // Small delay to ensure registers are written
        esp_hal::delay::Delay::new().delay_millis(10);
        
        Ok(())
    }

    /// Read calibration data from sensor registers
    fn read_calibration_data(&mut self) -> Result<(), &'static str> {
        // Read temperature and pressure calibration (registers 0x88-0x9F)
        let mut calib_00_25 = [0u8; 26];
        self.read_registers(BME280_CALIB_00_REG, &mut calib_00_25)?;
        
        // Read humidity calibration part 1 (register 0xA1)
        let dig_h1 = self.read_register(BME280_CALIB_25_REG)?;
        
        // Read humidity calibration part 2 (registers 0xE1-0xE7)
        let mut calib_26_32 = [0u8; 7];
        self.read_registers(BME280_CALIB_26_REG, &mut calib_26_32)?;
        
        // Parse calibration data
        let calib_data = CalibrationData {
            // Temperature calibration
            dig_t1: u16::from_le_bytes([calib_00_25[0], calib_00_25[1]]),
            dig_t2: i16::from_le_bytes([calib_00_25[2], calib_00_25[3]]),
            dig_t3: i16::from_le_bytes([calib_00_25[4], calib_00_25[5]]),
            
            // Pressure calibration
            dig_p1: u16::from_le_bytes([calib_00_25[6], calib_00_25[7]]),
            dig_p2: i16::from_le_bytes([calib_00_25[8], calib_00_25[9]]),
            dig_p3: i16::from_le_bytes([calib_00_25[10], calib_00_25[11]]),
            dig_p4: i16::from_le_bytes([calib_00_25[12], calib_00_25[13]]),
            dig_p5: i16::from_le_bytes([calib_00_25[14], calib_00_25[15]]),
            dig_p6: i16::from_le_bytes([calib_00_25[16], calib_00_25[17]]),
            dig_p7: i16::from_le_bytes([calib_00_25[18], calib_00_25[19]]),
            dig_p8: i16::from_le_bytes([calib_00_25[20], calib_00_25[21]]),
            dig_p9: i16::from_le_bytes([calib_00_25[22], calib_00_25[23]]),
            
            // Humidity calibration
            dig_h1,
            dig_h2: i16::from_le_bytes([calib_26_32[0], calib_26_32[1]]),
            dig_h3: calib_26_32[2],
            dig_h4: (i16::from(calib_26_32[3]) << 4) | (i16::from(calib_26_32[4]) & 0x0F),
            dig_h5: (i16::from(calib_26_32[5]) << 4) | (i16::from(calib_26_32[4]) >> 4),
            dig_h6: calib_26_32[6] as i8,
        };
        
        self.calib_data = Some(calib_data);
        Ok(())
    }

    /// Trigger a forced measurement
    pub fn force_measurement(&mut self) -> Result<(), &'static str> {
        // Write to CTRL_MEAS to trigger forced mode measurement
        // Bits 7:5 = pressure oversampling (001 = 1x)
        // Bits 4:2 = temperature oversampling (001 = 1x)  
        // Bits 1:0 = mode (01 = forced mode)
        self.write_register(BME280_CTRL_MEAS_REG, 0b00100101)?;
        
        // Small delay to ensure measurement starts
        esp_hal::delay::Delay::new().delay_millis(5);
        
        Ok(())
    }

    /// Read a single register
    fn read_register(&mut self, reg: u8) -> Result<u8, &'static str> {
        let mut data = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut data)
            .map_err(|_| "I2C read failed")?;
        Ok(data[0])
    }

    /// Write to a register
    fn write_register(&mut self, reg: u8, value: u8) -> Result<(), &'static str> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(|_| "I2C write failed")?;
        Ok(())
    }

    /// Read multiple registers
    fn read_registers(&mut self, reg: u8, data: &mut [u8]) -> Result<(), &'static str> {
        self.i2c
            .write_read(self.address, &[reg], data)
            .map_err(|_| "I2C read failed")?;
        Ok(())
    }

    /// Read raw temperature data (20 bits)
    pub fn read_raw_temperature(&mut self) -> Result<i32, &'static str> {
        let mut buffer = [0u8; 3];
        self.read_registers(BME280_TEMP_MSB_REG, &mut buffer)?;
        
        // Temperature data is 20 bits (MSB, LSB, XLSB[7:4])
        let raw_temp = (i32::from(buffer[0]) << 12) | 
                       (i32::from(buffer[1]) << 4) | 
                       (i32::from(buffer[2]) >> 4);
        
        Ok(raw_temp)
    }

    /// Read raw pressure data (20 bits)
    pub fn read_raw_pressure(&mut self) -> Result<i32, &'static str> {
        let mut buffer = [0u8; 3];
        self.read_registers(BME280_PRESS_MSB_REG, &mut buffer)?;
        
        // Pressure data is 20 bits (MSB, LSB, XLSB[7:4])
        let raw_press = (i32::from(buffer[0]) << 12) | 
                        (i32::from(buffer[1]) << 4) | 
                        (i32::from(buffer[2]) >> 4);
        
        Ok(raw_press)
    }

    /// Read raw humidity data (16 bits)
    pub fn read_raw_humidity(&mut self) -> Result<i32, &'static str> {
        let mut buffer = [0u8; 2];
        self.read_registers(BME280_HUM_MSB_REG, &mut buffer)?;
        
        // Humidity data is 16 bits (MSB, LSB)
        let raw_hum = (i32::from(buffer[0]) << 8) | 
                      i32::from(buffer[1]);
        
        Ok(raw_hum)
    }

    /// Read all measurements as raw values
    pub fn read_raw_data(&mut self) -> Result<(i32, i32, i32), &'static str> {
        let temperature = self.read_raw_temperature()?;
        let pressure = self.read_raw_pressure()?;
        let humidity = self.read_raw_humidity()?;
        
        Ok((temperature, pressure, humidity))
    }
    
    /// Read calibrated measurements using proper BME280 compensation algorithms
    pub fn read_measurements(&mut self) -> Result<Measurements, &'static str> {
        // Ensure calibration data is available
        if self.calib_data.is_none() {
            return Err("Calibration data not loaded");
        }
        
        // First trigger a measurement
        self.force_measurement()?;
        
        // Wait for measurement to complete (BME280 datasheet: max 112.8ms for all measurements)
        esp_hal::delay::Delay::new().delay_millis(150);
        
        let (raw_temp, raw_press, raw_hum) = self.read_raw_data()?;
        
        // Check for invalid readings (0x80000 for temp/pressure, 0x8000 for humidity)
        if raw_temp == 0x80000 || raw_press == 0x80000 {
            return Err("Invalid sensor readings - temperature/pressure not ready");
        }
        
        // Debug output to check calibration
        // rprintln!("DEBUG: Raw T=0x{:X} P=0x{:X} H=0x{:X}", raw_temp, raw_press, raw_hum);
        // rprintln!("DEBUG: dig_T1={} dig_T2={} dig_T3={}", calib_data.dig_t1, calib_data.dig_t2, calib_data.dig_t3);
        
        // Get calibration data and apply compensation algorithms
        let calib_data = self.calib_data.as_ref().unwrap(); // Safe because we checked above
        let temperature = Self::compensate_temperature(raw_temp, calib_data);
        let pressure = Self::compensate_pressure(raw_press, calib_data, raw_temp);
        
        // Handle humidity - might be invalid on BMP280 or due to configuration
        let humidity = if raw_hum == 0x8000 {
            0.0 // Set to 0 for BMP280 or invalid humidity reading
        } else {
            Self::compensate_humidity(raw_hum, calib_data, raw_temp)
        };
        
        Ok(Measurements {
            temperature,
            pressure,
            humidity,
        })
    }

    /// Compensate temperature using BME280 algorithm (32-bit fixed point)
    fn compensate_temperature(raw_temp: i32, calib: &CalibrationData) -> f32 {
        let var1 = ((raw_temp >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32) >> 11;
        let var2 = (((raw_temp >> 4) - (calib.dig_t1 as i32)) * 
                   ((raw_temp >> 4) - (calib.dig_t1 as i32)) >> 12) * (calib.dig_t3 as i32) >> 14;
        let t_fine = var1 + var2;
        let temperature = (t_fine * 5 + 128) >> 8;
        temperature as f32 / 100.0
    }

    /// Compensate pressure using BME280 algorithm (64-bit fixed point)
    fn compensate_pressure(raw_press: i32, calib: &CalibrationData, raw_temp: i32) -> f32 {
        // First calculate t_fine for pressure compensation
        let var1 = ((raw_temp >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32) >> 11;
        let var2 = (((raw_temp >> 4) - (calib.dig_t1 as i32)) * 
                   ((raw_temp >> 4) - (calib.dig_t1 as i32)) >> 12) * (calib.dig_t3 as i32) >> 14;
        let t_fine = var1 + var2;

        let mut var1_p = (t_fine as i64) - 128000;
        let mut var2_p = var1_p * var1_p * (calib.dig_p6 as i64);
        var2_p = var2_p + ((var1_p * (calib.dig_p5 as i64)) << 17);
        var2_p = var2_p + ((calib.dig_p4 as i64) << 35);
        var1_p = ((var1_p * var1_p * (calib.dig_p3 as i64)) >> 8) + ((var1_p * (calib.dig_p2 as i64)) << 12);
        var1_p = (((1_i64 << 47) + var1_p) * (calib.dig_p1 as i64)) >> 33;

        if var1_p == 0 {
            return 0.0; // avoid exception caused by division by zero
        }

        let mut p = 1048576 - raw_press as i64;
        p = (((p << 31) - var2_p) * 3125) / var1_p;
        var1_p = ((calib.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2_p = ((calib.dig_p8 as i64) * p) >> 19;
        p = ((p + var1_p + var2_p) >> 8) + ((calib.dig_p7 as i64) << 4);

        (p as f32) / 256.0 / 100.0 // Convert to hPa
    }

    /// Compensate humidity using BME280 algorithm (32-bit fixed point)
    fn compensate_humidity(raw_hum: i32, calib: &CalibrationData, raw_temp: i32) -> f32 {
        // First calculate t_fine for humidity compensation
        let var1 = ((raw_temp >> 3) - ((calib.dig_t1 as i32) << 1)) * (calib.dig_t2 as i32) >> 11;
        let var2 = (((raw_temp >> 4) - (calib.dig_t1 as i32)) * 
                   ((raw_temp >> 4) - (calib.dig_t1 as i32)) >> 12) * (calib.dig_t3 as i32) >> 14;
        let t_fine = var1 + var2;

        let mut v_x1_u32r = t_fine - 76800;
        v_x1_u32r = (((raw_hum << 14) - ((calib.dig_h4 as i32) << 20) - ((calib.dig_h5 as i32) * v_x1_u32r)) +
                    16384) >> 15;
        v_x1_u32r = (v_x1_u32r * (((((((v_x1_u32r * (calib.dig_h6 as i32)) >> 10) *
                    (((v_x1_u32r * (calib.dig_h3 as i32)) >> 11) + 32768)) >> 10) + 2097152) *
                    (calib.dig_h2 as i32) + 8192) >> 14));
        v_x1_u32r = v_x1_u32r - (((((v_x1_u32r >> 15) * (v_x1_u32r >> 15)) >> 7) *
                                ((calib.dig_h1 as i32) as i32)) >> 4);
        v_x1_u32r = if v_x1_u32r < 0 { 0 } else { v_x1_u32r };
        v_x1_u32r = if v_x1_u32r > 419430400 { 419430400 } else { v_x1_u32r };
        let humidity = v_x1_u32r >> 12;

        humidity as f32 / 1024.0
    }
}