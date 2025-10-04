//! # Hardware Abstraction Layer Errors
//!
//! Error types and utilities for hardware abstraction layer operations.
//! Integrates with the unified IoT error handling system.

use iot_common::{IoTError, HardwareError, error::IoTErrorKind};
use core::fmt;

/// Hardware abstraction layer result type
pub type HardwareResult<T> = Result<T, HardwareError>;

/// Hardware platform initialization errors
#[derive(Debug, Clone)]
pub enum PlatformError {
    /// Hardware initialization failed
    InitializationFailed(&'static str),
    
    /// Configuration validation failed
    InvalidConfiguration(&'static str),
    
    /// Required peripheral not available
    PeripheralUnavailable(&'static str),
    
    /// Pin configuration conflict
    PinConflict(&'static str),
    
    /// Clock configuration failed
    ClockConfigurationFailed(&'static str),
    
    /// Resource allocation failed
    ResourceAllocationFailed(&'static str),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::InitializationFailed(msg) => {
                write!(f, "Platform initialization failed: {}", msg)
            }
            PlatformError::InvalidConfiguration(msg) => {
                write!(f, "Invalid platform configuration: {}", msg)
            }
            PlatformError::PeripheralUnavailable(msg) => {
                write!(f, "Peripheral unavailable: {}", msg)
            }
            PlatformError::PinConflict(msg) => {
                write!(f, "Pin configuration conflict: {}", msg)
            }
            PlatformError::ClockConfigurationFailed(msg) => {
                write!(f, "Clock configuration failed: {}", msg)
            }
            PlatformError::ResourceAllocationFailed(msg) => {
                write!(f, "Resource allocation failed: {}", msg)
            }
        }
    }
}

/// I2C communication errors
#[derive(Debug, Clone)]
pub enum I2cError {
    /// I2C bus initialization failed
    InitializationFailed(&'static str),
    
    /// Device not responding
    DeviceNotResponding(u8), // device address
    
    /// Bus arbitration lost
    ArbitrationLost,
    
    /// Not acknowledged by device
    NotAcknowledged(u8), // device address
    
    /// Bus timeout
    Timeout,
    
    /// Invalid device address
    InvalidAddress(u8),
    
    /// Buffer overflow
    BufferOverflow,
    
    /// Hardware fault
    HardwareFault(&'static str),
}

impl fmt::Display for I2cError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            I2cError::InitializationFailed(msg) => {
                write!(f, "I2C initialization failed: {}", msg)
            }
            I2cError::DeviceNotResponding(addr) => {
                write!(f, "I2C device at address 0x{:02X} not responding", addr)
            }
            I2cError::ArbitrationLost => {
                write!(f, "I2C arbitration lost")
            }
            I2cError::NotAcknowledged(addr) => {
                write!(f, "I2C device at address 0x{:02X} did not acknowledge", addr)
            }
            I2cError::Timeout => {
                write!(f, "I2C operation timeout")
            }
            I2cError::InvalidAddress(addr) => {
                write!(f, "Invalid I2C address: 0x{:02X}", addr)
            }
            I2cError::BufferOverflow => {
                write!(f, "I2C buffer overflow")
            }
            I2cError::HardwareFault(msg) => {
                write!(f, "I2C hardware fault: {}", msg)
            }
        }
    }
}

/// UART communication errors
#[derive(Debug, Clone)]
pub enum UartError {
    /// UART initialization failed
    InitializationFailed(&'static str),
    
    /// Frame error (invalid start/stop bits)
    FrameError,
    
    /// Parity error
    ParityError,
    
    /// Buffer overrun
    Overrun,
    
    /// Break condition detected
    Break,
    
    /// Transmit timeout
    TransmitTimeout,
    
    /// Receive timeout
    ReceiveTimeout,
    
    /// Buffer full
    BufferFull,
    
    /// Hardware fault
    HardwareFault(&'static str),
}

impl fmt::Display for UartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UartError::InitializationFailed(msg) => {
                write!(f, "UART initialization failed: {}", msg)
            }
            UartError::FrameError => {
                write!(f, "UART frame error")
            }
            UartError::ParityError => {
                write!(f, "UART parity error")
            }
            UartError::Overrun => {
                write!(f, "UART buffer overrun")
            }
            UartError::Break => {
                write!(f, "UART break condition")
            }
            UartError::TransmitTimeout => {
                write!(f, "UART transmit timeout")
            }
            UartError::ReceiveTimeout => {
                write!(f, "UART receive timeout")
            }
            UartError::BufferFull => {
                write!(f, "UART buffer full")
            }
            UartError::HardwareFault(msg) => {
                write!(f, "UART hardware fault: {}", msg)
            }
        }
    }
}

/// GPIO operation errors
#[derive(Debug, Clone)]
pub enum GpioError {
    /// GPIO initialization failed
    InitializationFailed(&'static str),
    
    /// Invalid pin number
    InvalidPin(u8),
    
    /// Pin already in use
    PinInUse(u8),
    
    /// Unsupported operation for pin mode
    UnsupportedOperation,
    
    /// Hardware fault
    HardwareFault(&'static str),
}

impl fmt::Display for GpioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GpioError::InitializationFailed(msg) => {
                write!(f, "GPIO initialization failed: {}", msg)
            }
            GpioError::InvalidPin(pin) => {
                write!(f, "Invalid GPIO pin: {}", pin)
            }
            GpioError::PinInUse(pin) => {
                write!(f, "GPIO pin {} already in use", pin)
            }
            GpioError::UnsupportedOperation => {
                write!(f, "Unsupported GPIO operation")
            }
            GpioError::HardwareFault(msg) => {
                write!(f, "GPIO hardware fault: {}", msg)
            }
        }
    }
}

/// WiFi operation errors
#[derive(Debug, Clone)]
pub enum WiFiError {
    /// WiFi initialization failed
    InitializationFailed(&'static str),
    
    /// Connection to network failed
    ConnectionFailed(&'static str),
    
    /// Authentication failed
    AuthenticationFailed,
    
    /// Network not found
    NetworkNotFound,
    
    /// Connection timeout
    ConnectionTimeout,
    
    /// Connection lost
    ConnectionLost,
    
    /// DHCP failed
    DhcpFailed,
    
    /// Invalid credentials
    InvalidCredentials,
    
    /// Hardware fault
    HardwareFault(&'static str),
}

impl fmt::Display for WiFiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WiFiError::InitializationFailed(msg) => {
                write!(f, "WiFi initialization failed: {}", msg)
            }
            WiFiError::ConnectionFailed(msg) => {
                write!(f, "WiFi connection failed: {}", msg)
            }
            WiFiError::AuthenticationFailed => {
                write!(f, "WiFi authentication failed")
            }
            WiFiError::NetworkNotFound => {
                write!(f, "WiFi network not found")
            }
            WiFiError::ConnectionTimeout => {
                write!(f, "WiFi connection timeout")
            }
            WiFiError::ConnectionLost => {
                write!(f, "WiFi connection lost")
            }
            WiFiError::DhcpFailed => {
                write!(f, "WiFi DHCP failed")
            }
            WiFiError::InvalidCredentials => {
                write!(f, "WiFi invalid credentials")
            }
            WiFiError::HardwareFault(msg) => {
                write!(f, "WiFi hardware fault: {}", msg)
            }
        }
    }
}

/// Convert platform errors to IoT errors
impl From<PlatformError> for IoTError {
    fn from(error: PlatformError) -> Self {
        use iot_common::error::utils::error_message;
        
        let message = match error {
            PlatformError::InitializationFailed(_msg) => {
                error_message("Platform initialization failed")
            }
            PlatformError::InvalidConfiguration(_msg) => {
                error_message("Platform configuration invalid")
            }
            PlatformError::PeripheralUnavailable(_msg) => {
                error_message("Peripheral unavailable")
            }
            PlatformError::PinConflict(_msg) => {
                error_message("Pin configuration conflict")
            }
            PlatformError::ClockConfigurationFailed(_msg) => {
                error_message("Clock configuration failed")
            }
            PlatformError::ResourceAllocationFailed(_msg) => {
                error_message("Resource allocation failed")
            }
        };
        
        IoTError::hardware(HardwareError::ClockError(message))
    }
}

/// Convert I2C errors to IoT errors
impl From<I2cError> for IoTError {
    fn from(error: I2cError) -> Self {
        use iot_common::error::utils::error_message;
        
        let message = match error {
            I2cError::InitializationFailed(_msg) => {
                error_message("I2C initialization failed")
            }
            I2cError::DeviceNotResponding(_addr) => {
                error_message("I2C device not responding")
            }
            I2cError::ArbitrationLost => {
                error_message("I2C arbitration lost")
            }
            I2cError::NotAcknowledged(_addr) => {
                error_message("I2C device NACK")
            }
            I2cError::Timeout => {
                error_message("I2C timeout")
            }
            I2cError::InvalidAddress(_addr) => {
                error_message("I2C invalid address")
            }
            I2cError::BufferOverflow => {
                error_message("I2C buffer overflow")
            }
            I2cError::HardwareFault(_msg) => {
                error_message("I2C hardware fault")
            }
        };
        
        IoTError::hardware(HardwareError::SPIError(message))
    }
}

/// Convert UART errors to IoT errors
impl From<UartError> for IoTError {
    fn from(error: UartError) -> Self {
        use iot_common::error::utils::error_message;
        
        let message = match error {
            UartError::InitializationFailed(_msg) => {
                error_message("UART initialization failed")
            }
            UartError::FrameError => {
                error_message("UART frame error")
            }
            UartError::ParityError => {
                error_message("UART parity error")
            }
            UartError::Overrun => {
                error_message("UART overrun")
            }
            UartError::Break => {
                error_message("UART break")
            }
            UartError::TransmitTimeout => {
                error_message("UART TX timeout")
            }
            UartError::ReceiveTimeout => {
                error_message("UART RX timeout")
            }
            UartError::BufferFull => {
                error_message("UART buffer full")
            }
            UartError::HardwareFault(_msg) => {
                error_message("UART hardware fault")
            }
        };
        
        IoTError::hardware(HardwareError::SPIError(message))
    }
}

/// Convert GPIO errors to IoT errors
impl From<GpioError> for IoTError {
    fn from(error: GpioError) -> Self {
        use iot_common::error::utils::error_message;
        
        let message = match error {
            GpioError::InitializationFailed(_msg) => {
                error_message("GPIO initialization failed")
            }
            GpioError::InvalidPin(_pin) => {
                error_message("GPIO invalid pin")
            }
            GpioError::PinInUse(_pin) => {
                error_message("GPIO pin in use")
            }
            GpioError::UnsupportedOperation => {
                error_message("GPIO unsupported operation")
            }
            GpioError::HardwareFault(_msg) => {
                error_message("GPIO hardware fault")
            }
        };
        
        IoTError::hardware(HardwareError::GPIOError(message))
    }
}

/// Convert WiFi errors to IoT errors  
impl From<WiFiError> for IoTError {
    fn from(error: WiFiError) -> Self {
        use iot_common::{NetworkError, error::utils::error_message};
        
        let network_error = match error {
            WiFiError::InitializationFailed(_msg) => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi initialization failed"))
            }
            WiFiError::ConnectionFailed(_msg) => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi connection failed"))
            }
            WiFiError::AuthenticationFailed => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi auth failed"))
            }
            WiFiError::NetworkNotFound => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi network not found"))
            }
            WiFiError::ConnectionTimeout => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi timeout"))
            }
            WiFiError::ConnectionLost => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi connection lost"))
            }
            WiFiError::DhcpFailed => {
                NetworkError::WiFiConnectionFailed(error_message("DHCP failed"))
            }
            WiFiError::InvalidCredentials => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi invalid credentials"))
            }
            WiFiError::HardwareFault(_msg) => {
                NetworkError::WiFiConnectionFailed(error_message("WiFi hardware fault"))
            }
        };
        
        IoTError::network(network_error)
    }
}

/// Utility functions for hardware error handling
pub mod utils {
    use super::*;

    /// Create a platform initialization error
    pub fn platform_init_error(_message: &str) -> IoTError {
        PlatformError::InitializationFailed(
            // Convert to static str - in real implementation this would use
            // a bounded string or static string pool
            "Platform initialization failed"
        ).into()
    }

    /// Create an I2C communication error for a specific device
    pub fn i2c_device_error(device_address: u8, _operation: &str) -> IoTError {
        I2cError::DeviceNotResponding(device_address).into()
    }

    /// Create a GPIO pin configuration error
    pub fn gpio_pin_error(pin: u8, _reason: &str) -> IoTError {
        GpioError::InvalidPin(pin).into()
    }

    /// Create a UART communication error
    pub fn uart_communication_error(_operation: &str) -> IoTError {
        UartError::HardwareFault("UART communication error").into()
    }

    /// Create a WiFi connection error
    pub fn wifi_connection_error(_reason: &str) -> IoTError {
        WiFiError::ConnectionFailed("WiFi connection failed").into()
    }

    /// Check if error is recoverable
    pub fn is_recoverable_error(error: &IoTError) -> bool {
        match &error.kind() {
            IoTErrorKind::Hardware(hw_error) => {
                match hw_error {
                    HardwareError::SPIError(_) | HardwareError::UARTError(_) => true,  // Communication errors often recoverable
                    HardwareError::GPIOError(_) | HardwareError::ClockError(_) => false, // Configuration errors usually not recoverable
                    _ => false,
                }
            }
            IoTErrorKind::Network(_) => true, // Network errors often recoverable
            _ => false,
        }
    }

    /// Get error recovery suggestion
    pub fn get_recovery_suggestion(error: &IoTError) -> &'static str {
        match &error.kind() {
            IoTErrorKind::Hardware(hw_error) => {
                match hw_error {
                    HardwareError::SPIError(_) | HardwareError::UARTError(_) => "Check hardware connections and power",
                    HardwareError::GPIOError(_) | HardwareError::ClockError(_) => "Check configuration and restart system",
                    _ => "Check hardware connections",
                }
            }
            IoTErrorKind::Network(_) => "Check network connectivity and credentials",
            _ => "Consult system documentation",
        }
    }
}