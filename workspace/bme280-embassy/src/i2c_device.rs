//! I2C Device Abstraction Layer
//! 
//! This module provides a clean abstraction for I2C communication with the BME280 sensor.
//! It isolates all I2C operations and provides a consistent interface for the BME280 driver.

use esp_hal::i2c::master::I2c;
use esp_hal::Blocking;
use iot_common::{IoTError, error::utils::error_message};

/// I2C device abstraction for BME280 communication
/// 
/// This structure encapsulates I2C communication with the BME280 sensor,
/// providing a clean interface for register read/write operations.
pub struct I2cDevice<'a> {
    i2c: &'a mut I2c<'a, Blocking>,
    address: u8,
}

impl<'a> I2cDevice<'a> {
    /// Creates a new I2C device abstraction
    /// 
    /// # Arguments
    /// 
    /// * `i2c` - Mutable reference to esp-hal I2C interface
    /// * `address` - I2C slave address of the device
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use bme280_embassy::I2cDevice;
    /// use esp_hal::i2c::master::{I2c, Config as I2cConfig};
    /// 
    /// let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
    ///     .unwrap()
    ///     .with_sda(peripherals.GPIO8)
    ///     .with_scl(peripherals.GPIO9);
    ///     
    /// let device = I2cDevice::new(&mut i2c, 0x76);
    /// ```
    pub fn new(i2c: &'a mut I2c<'a, Blocking>, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Changes the I2C address for this device
    /// 
    /// This is useful when scanning for devices at multiple addresses
    /// or when the device address is configurable.
    /// 
    /// # Arguments
    /// 
    /// * `address` - New I2C slave address
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use bme280_embassy::I2cDevice;
    /// # let mut device: I2cDevice = unimplemented!();
    /// // Try primary address first
    /// device.set_address(0x76);
    /// // If that fails, try secondary address
    /// device.set_address(0x77);
    /// ```
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }

    /// Gets the current I2C address
    /// 
    /// # Returns
    /// 
    /// The current I2C slave address
    pub fn get_address(&self) -> u8 {
        self.address
    }

    /// Reads a single register from the device
    /// 
    /// This method performs a write-read transaction to read one byte
    /// from the specified register address.
    /// 
    /// # Arguments
    /// 
    /// * `register` - Register address to read from
    /// 
    /// # Returns
    /// 
    /// * `Ok(value)` - The byte value read from the register
    /// * `Err(IoTError)` - I2C communication failure
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use bme280_embassy::I2cDevice;
    /// # let mut device: I2cDevice = unimplemented!();
    /// // Read BME280 chip ID register
    /// let chip_id = device.read_register(0xD0).await?;
    /// assert_eq!(chip_id, 0x60); // Expected BME280 chip ID
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn read_register(&mut self, register: u8) -> Result<u8, IoTError> {
        let mut data = [0u8; 1];
        self.i2c
            .write_read(self.address, &[register], &mut data)
            .map_err(|_| IoTError::sensor(iot_common::SensorError::I2CError(error_message("I2C read register failed"))))?;
        Ok(data[0])
    }

    /// Writes a single register on the device
    /// 
    /// This method performs a write transaction to set the value
    /// of the specified register.
    /// 
    /// # Arguments
    /// 
    /// * `register` - Register address to write to
    /// * `value` - Byte value to write to the register
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Write operation successful
    /// * `Err(IoTError)` - I2C communication failure
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use bme280_embassy::I2cDevice;
    /// # let mut device: I2cDevice = unimplemented!();
    /// // Configure BME280 humidity oversampling
    /// device.write_register(0xF2, 0x01).await?; // 1x oversampling
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn write_register(&mut self, register: u8, value: u8) -> Result<(), IoTError> {
        self.i2c
            .write(self.address, &[register, value])
            .map_err(|_| IoTError::sensor(iot_common::SensorError::I2CError(error_message("I2C write register failed"))))
    }

    /// Reads multiple registers from the device
    /// 
    /// This method performs a write-read transaction to read a block
    /// of consecutive registers starting from the specified address.
    /// 
    /// # Arguments
    /// 
    /// * `start_register` - Starting register address
    /// * `buffer` - Mutable buffer to store the read data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Read operation successful, data stored in buffer
    /// * `Err(IoTError)` - I2C communication failure
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use bme280_embassy::I2cDevice;
    /// # let mut device: I2cDevice = unimplemented!();
    /// // Read BME280 calibration data (24 bytes starting at 0x88)
    /// let mut calib_data = [0u8; 24];
    /// device.read_registers(0x88, &mut calib_data).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// 
    /// # Notes
    /// 
    /// - The buffer size determines how many consecutive registers are read
    /// - The device must support sequential register reading
    /// - BME280 supports this for all calibration and data registers
    pub async fn read_registers(&mut self, start_register: u8, buffer: &mut [u8]) -> Result<(), IoTError> {
        self.i2c
            .write_read(self.address, &[start_register], buffer)
            .map_err(|_| IoTError::sensor(iot_common::SensorError::I2CError(error_message("I2C read registers failed"))))
    }

    /// Tests if the device responds at the current I2C address
    /// 
    /// This method performs a simple I2C transaction to check if
    /// a device is present and responding at the configured address.
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Device responds at the current address
    /// * `Ok(false)` - Device does not respond (but no I2C bus error)
    /// * `Err(IoTError)` - I2C bus error or communication failure
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// # use bme280_embassy::I2cDevice;
    /// # let mut device: I2cDevice = unimplemented!();
    /// // Check if BME280 is present at 0x76
    /// device.set_address(0x76);
    /// if device.ping().await? {
    ///     println!("BME280 found at 0x76");
    /// } else {
    ///     // Try alternative address
    ///     device.set_address(0x77);
    ///     if device.ping().await? {
    ///         println!("BME280 found at 0x77");
    ///     }
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// 
    /// # Notes
    /// 
    /// - This is useful for device discovery and address scanning
    /// - The transaction is minimal to avoid side effects
    /// - Some devices may respond differently to ping operations
    pub async fn ping(&mut self) -> Result<bool, IoTError> {
        match self.i2c.transaction(self.address, &mut []) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // Device not responding, but not necessarily an error
        }
    }
}