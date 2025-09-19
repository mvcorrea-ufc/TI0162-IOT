//! # Hardware Abstraction Traits
//!
//! Core trait definitions for hardware platform abstraction.
//! These traits enable dependency injection, testing, and platform portability.

use embedded_hal_async::i2c::I2c;
use embedded_io_async::{Read, Write};
use embedded_hal::digital::OutputPin;
use embassy_time::Duration;
use iot_common::IoTError;
use async_trait::async_trait;

/// Core hardware platform abstraction trait
/// 
/// This trait defines the interface for accessing hardware resources in a
/// platform-independent way. Implementations provide actual hardware access
/// (ESP32-C3) or mock interfaces (testing).
/// 
/// # Design Principles
/// 
/// - **Zero-cost abstractions**: No runtime overhead in release builds
/// - **Memory efficient**: All resources allocated statically or on stack
/// - **Real-time compatible**: No blocking operations in async methods
/// - **Error consistent**: All operations return IoTError for unified handling
/// 
/// # Usage
/// 
/// ```rust
/// use iot_hal::{HardwarePlatform, HardwareConfig};
/// 
/// async fn initialize_system() -> Result<(), IoTError> {
///     // Platform-agnostic initialization
///     let mut platform = Platform::initialize().await?;
///     
///     // Access hardware resources through traits
///     let i2c = platform.get_i2c();
///     let (tx, rx) = platform.get_console();
///     let led = platform.get_status_led();
///     let timer = platform.get_timer();
///     
///     Ok(())
/// }
/// ```
/// 
/// # Implementation Requirements
/// 
/// Implementors must ensure:
/// - Thread safety for embedded async contexts
/// - Proper resource cleanup on drop
/// - Hardware state management
/// - Error propagation with context
#[async_trait(?Send)]
pub trait HardwarePlatform {
    /// I2C bus interface for sensor communication
    type I2cBus: I2cInterface;
    
    /// UART transmitter for console output
    type UartTx: UartTxInterface;
    
    /// UART receiver for console input
    type UartRx: UartRxInterface;
    
    /// GPIO pin interface for status indicators
    type GpioPin: GpioInterface;
    
    /// Timer interface for delays and timeouts
    type Timer: TimerInterface;
    
    /// WiFi interface for network connectivity
    type WiFi: WiFiInterface;

    /// Initialize hardware platform with default configuration
    /// 
    /// This method performs complete hardware initialization including:
    /// - Peripheral configuration
    /// - Clock setup  
    /// - Pin assignment
    /// - Resource allocation
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - Platform initialized successfully
    /// * `Err(IoTError)` - Hardware initialization failed
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// let platform = Esp32C3Platform::initialize().await?;
    /// ```
    async fn initialize() -> Result<Self, IoTError> 
    where 
        Self: Sized;

    /// Initialize hardware platform with custom configuration
    /// 
    /// Allows customization of hardware settings like pin assignments,
    /// clock frequencies, and communication parameters.
    /// 
    /// # Arguments
    /// 
    /// * `config` - Hardware configuration parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Self)` - Platform initialized with config
    /// * `Err(IoTError)` - Configuration invalid or initialization failed
    async fn initialize_with_config(config: crate::HardwareConfig) -> Result<Self, IoTError>
    where 
        Self: Sized;

    /// Get mutable reference to I2C bus interface
    /// 
    /// Provides access to the I2C bus for sensor communication.
    /// The interface supports async operations and proper error handling.
    /// 
    /// # Returns
    /// 
    /// Mutable reference to I2C interface
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// let i2c = platform.get_i2c();
    /// let mut buffer = [0u8; 4];
    /// i2c.read(0x76, &mut buffer).await?;
    /// ```
    fn get_i2c(&mut self) -> &mut Self::I2cBus;

    /// Get console UART interfaces for bidirectional communication
    /// 
    /// Returns both transmitter and receiver for console communication.
    /// Typically used for command-line interfaces and debugging.
    /// 
    /// # Returns
    /// 
    /// Tuple of (transmitter, receiver) interfaces
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// let (tx, rx) = platform.get_console();
    /// tx.write(b"Hello, World!\n").await?;
    /// ```
    fn get_console(&mut self) -> (&mut Self::UartTx, &mut Self::UartRx);

    /// Get status LED GPIO interface
    /// 
    /// Provides access to status LED for visual feedback.
    /// Supports digital output operations for on/off control.
    /// 
    /// # Returns
    /// 
    /// Mutable reference to GPIO interface
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// let led = platform.get_status_led();
    /// led.set_high().await?;  // Turn on LED
    /// led.set_low().await?;   // Turn off LED
    /// ```
    fn get_status_led(&mut self) -> &mut Self::GpioPin;

    /// Get timer interface for delays and timeouts
    /// 
    /// Provides non-blocking timer functionality for delays,
    /// timeouts, and periodic operations.
    /// 
    /// # Returns
    /// 
    /// Mutable reference to timer interface
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// let timer = platform.get_timer();
    /// timer.delay(Duration::from_millis(100)).await;
    /// ```
    fn get_timer(&mut self) -> &mut Self::Timer;

    /// Get WiFi interface for network connectivity
    /// 
    /// Provides access to WiFi hardware for network operations.
    /// Supports connection management and status monitoring.
    /// 
    /// # Returns
    /// 
    /// Mutable reference to WiFi interface
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// let wifi = platform.get_wifi();
    /// wifi.connect("MyNetwork", "password").await?;
    /// ```
    fn get_wifi(&mut self) -> &mut Self::WiFi;

    /// Check if platform is properly initialized
    /// 
    /// Verifies that all hardware resources are available and responsive.
    /// Useful for system health checks and diagnostics.
    /// 
    /// # Returns
    /// 
    /// * `true` - All hardware interfaces operational
    /// * `false` - One or more interfaces not responding
    async fn is_healthy(&mut self) -> bool;

    /// Get platform identification information
    /// 
    /// Returns human-readable platform information for diagnostics
    /// and version tracking.
    /// 
    /// # Returns
    /// 
    /// String slice with platform identification
    fn platform_info(&self) -> &'static str;
}

/// I2C interface abstraction for sensor communication
/// 
/// Provides async I2C operations with unified error handling.
/// Supports both read and write operations with proper addressing.
pub trait I2cInterface {
    /// Read data from I2C device
    /// 
    /// # Arguments
    /// 
    /// * `address` - 7-bit I2C device address
    /// * `buffer` - Buffer to store read data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Read operation successful
    /// * `Err(IoTError)` - Communication failure
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), IoTError>;

    /// Write data to I2C device
    /// 
    /// # Arguments
    /// 
    /// * `address` - 7-bit I2C device address
    /// * `data` - Data to write
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Write operation successful
    /// * `Err(IoTError)` - Communication failure
    async fn write(&mut self, address: u8, data: &[u8]) -> Result<(), IoTError>;

    /// Write then read from I2C device (common sensor pattern)
    /// 
    /// # Arguments
    /// 
    /// * `address` - 7-bit I2C device address
    /// * `write_data` - Data to write (typically register address)
    /// * `read_buffer` - Buffer to store read data
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Transaction successful
    /// * `Err(IoTError)` - Communication failure
    async fn write_read(&mut self, address: u8, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), IoTError>;
}

/// UART transmitter interface for output operations
pub trait UartTxInterface {
    /// Write data to UART
    /// 
    /// # Arguments
    /// 
    /// * `data` - Data to transmit
    /// 
    /// # Returns
    /// 
    /// * `Ok(bytes_written)` - Number of bytes successfully written
    /// * `Err(IoTError)` - Transmission failure
    async fn write(&mut self, data: &[u8]) -> Result<usize, IoTError>;

    /// Flush transmit buffer
    /// 
    /// Ensures all buffered data is transmitted before returning.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Buffer flushed successfully
    /// * `Err(IoTError)` - Flush operation failed
    async fn flush(&mut self) -> Result<(), IoTError>;
}

/// UART receiver interface for input operations
pub trait UartRxInterface {
    /// Read data from UART
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - Buffer to store received data
    /// 
    /// # Returns
    /// 
    /// * `Ok(bytes_read)` - Number of bytes successfully read
    /// * `Err(IoTError)` - Reception failure or timeout
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoTError>;

    /// Check if data is available for reading
    /// 
    /// # Returns
    /// 
    /// * `true` - Data available in receive buffer
    /// * `false` - No data available
    fn available(&self) -> bool;
}

/// GPIO interface for digital pin control
pub trait GpioInterface {
    /// Set pin to high level
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Pin set high successfully
    /// * `Err(IoTError)` - GPIO operation failed
    async fn set_high(&mut self) -> Result<(), IoTError>;

    /// Set pin to low level
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Pin set low successfully  
    /// * `Err(IoTError)` - GPIO operation failed
    async fn set_low(&mut self) -> Result<(), IoTError>;

    /// Toggle pin state
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Pin toggled successfully
    /// * `Err(IoTError)` - GPIO operation failed
    async fn toggle(&mut self) -> Result<(), IoTError>;

    /// Get current pin state
    /// 
    /// # Returns
    /// 
    /// * `Ok(true)` - Pin is high
    /// * `Ok(false)` - Pin is low
    /// * `Err(IoTError)` - GPIO read failed
    async fn is_high(&self) -> Result<bool, IoTError>;
}

/// Timer interface for delays and timeouts
pub trait TimerInterface {
    /// Asynchronous delay
    /// 
    /// # Arguments
    /// 
    /// * `duration` - How long to delay
    /// 
    /// # Returns
    /// 
    /// Returns when delay period has elapsed
    async fn delay(&mut self, duration: Duration);

    /// Get current timestamp
    /// 
    /// # Returns
    /// 
    /// Current timestamp for timing calculations
    fn now(&self) -> embassy_time::Instant;

    /// Create a deadline for timeout operations
    /// 
    /// # Arguments
    /// 
    /// * `duration` - Timeout duration from now
    /// 
    /// # Returns
    /// 
    /// Deadline instant for timeout checks
    fn deadline(&self, duration: Duration) -> embassy_time::Instant;
}

/// WiFi interface for network connectivity
#[async_trait(?Send)]
pub trait WiFiInterface {
    /// Connect to WiFi network
    /// 
    /// # Arguments
    /// 
    /// * `ssid` - Network name
    /// * `password` - Network password
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connected successfully
    /// * `Err(IoTError)` - Connection failed
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), IoTError>;

    /// Disconnect from WiFi network
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Disconnected successfully
    /// * `Err(IoTError)` - Disconnect failed
    async fn disconnect(&mut self) -> Result<(), IoTError>;

    /// Check if connected to WiFi
    /// 
    /// # Returns
    /// 
    /// * `true` - Connected to network
    /// * `false` - Not connected
    fn is_connected(&self) -> bool;

    /// Get assigned IP address
    /// 
    /// # Returns
    /// 
    /// * `Some(ip)` - Current IP address
    /// * `None` - Not connected or no IP assigned
    fn get_ip_address(&self) -> Option<core::net::IpAddr>;

    /// Get signal strength
    /// 
    /// # Returns
    /// 
    /// Signal strength in dBm (negative values, closer to 0 is stronger)
    fn get_signal_strength(&self) -> i8;

    /// Get connection information
    /// 
    /// # Returns
    /// 
    /// Detailed connection status and configuration
    fn get_connection_info(&self) -> Option<WiFiConnectionInfo>;
}

/// WiFi connection information structure
#[derive(Debug, Clone)]
pub struct WiFiConnectionInfo {
    /// Assigned IP address
    pub ip_address: core::net::IpAddr,
    /// Network gateway address
    pub gateway: Option<core::net::IpAddr>,
    /// Network subnet mask
    pub netmask: Option<core::net::IpAddr>,
    /// Connected network SSID
    pub ssid: heapless::String<32>,
    /// Signal strength in dBm
    pub signal_strength: i8,
    /// Connection uptime in seconds
    pub uptime_seconds: u32,
}