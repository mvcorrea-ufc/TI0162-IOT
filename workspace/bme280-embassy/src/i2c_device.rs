use iot_hal::I2cInterface;
use iot_common::IoTError;

pub struct I2cDevice<'a, I2C>
where
    I2C: I2cInterface,
{
    i2c: &'a mut I2C,
    address: u8,
}

impl<'a, I2C> I2cDevice<'a, I2C>
where
    I2C: I2cInterface,
{
    /// Create a new I2C device instance
    pub fn new(i2c: &'a mut I2C, address: u8) -> Self {
        Self {
            i2c,
            address,
        }
    }

    /// Configure with a different I2C address
    pub fn with_address(mut self, address: u8) -> Self {
        self.address = address;
        self
    }

    /// Get current address
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Set a new address
    pub fn set_address(&mut self, address: u8) {
        self.address = address;
    }

    /// Read a single register (async)
    pub async fn read_register(&mut self, reg: u8) -> Result<u8, IoTError> {
        let mut data = [0u8; 1];
        self.i2c.write_read(self.address, &[reg], &mut data).await?;
        Ok(data[0])
    }

    /// Read multiple registers (async)
    pub async fn read_registers(&mut self, reg: u8, data: &mut [u8]) -> Result<(), IoTError> {
        self.i2c.write_read(self.address, &[reg], data).await?;
        Ok(())
    }

    /// Write to a register (async)
    pub async fn write_register(&mut self, reg: u8, value: u8) -> Result<(), IoTError> {
        self.i2c.write(self.address, &[reg, value]).await?;
        Ok(())
    }

    /// Write multiple values to a register (async)
    pub async fn write_registers(&mut self, reg: u8, values: &[u8]) -> Result<(), IoTError> {
        let mut buffer = [0u8; 32]; // Adjust size as needed
        buffer[0] = reg;
        
        let length = values.len().min(buffer.len() - 1);
        buffer[1..=length].copy_from_slice(&values[0..length]);
        
        self.i2c.write(self.address, &buffer[0..=length]).await?;
        Ok(())
    }
}