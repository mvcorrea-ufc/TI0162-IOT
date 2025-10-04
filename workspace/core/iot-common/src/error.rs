//! # Unified Error Handling for IoT System
//!
//! This module provides a comprehensive error handling system for the ESP32-C3 IoT system
//! that maintains `no_std` compatibility while providing rich error context for debugging.
//!
//! ## Design Principles
//!
//! - **No heap allocation**: All error messages use bounded `heapless::String`
//! - **Error context preservation**: Chain errors without losing information
//! - **Consistent error codes**: Programmatic error handling with numeric codes
//! - **Real-time friendly**: Minimal performance overhead in error paths
//! - **RTT debugging support**: Formatted error output for debugging

use heapless::String;
use core::fmt;
use core::str::FromStr;

use crate::{MAX_ERROR_MESSAGE_LEN, MAX_ERROR_CONTEXT_DEPTH};

/// Bounded string type for error messages in embedded environment
pub type ErrorMessage = String<MAX_ERROR_MESSAGE_LEN>;

/// Error code type for programmatic error handling
pub type ErrorCode = u16;

/// Error context chain for tracking error propagation
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error context messages (most recent first)
    contexts: heapless::Vec<ErrorMessage, MAX_ERROR_CONTEXT_DEPTH>,
}

impl ErrorContext {
    /// Create new empty error context
    pub fn new() -> Self {
        Self {
            contexts: heapless::Vec::new(),
        }
    }

    /// Add context to the error chain
    pub fn add_context(&mut self, context: &str) {
        if let Ok(msg) = ErrorMessage::from_str(context) {
            let _ = self.contexts.insert(0, msg); // Insert at front (most recent first)
        }
    }

    /// Get context messages (most recent first)
    pub fn contexts(&self) -> &[ErrorMessage] {
        &self.contexts
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, context) in self.contexts.iter().enumerate() {
            if i > 0 {
                write!(f, " <- ")?;
            }
            write!(f, "{}", context)?;
        }
        Ok(())
    }
}

/// Sensor-related errors
#[derive(Debug, Clone)]
pub enum SensorError {
    /// Sensor initialization failed
    InitializationFailed(ErrorMessage),
    /// I2C communication error
    I2CError(ErrorMessage),
    /// Invalid sensor data received
    InvalidData(ErrorMessage),
    /// Sensor calibration error
    CalibrationError(ErrorMessage),
    /// Sensor not responding
    NotResponding(ErrorMessage),
    /// Invalid sensor configuration
    InvalidConfiguration(ErrorMessage),
}

impl SensorError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match self {
            SensorError::InitializationFailed(_) => 1001,
            SensorError::I2CError(_) => 1002,
            SensorError::InvalidData(_) => 1003,
            SensorError::CalibrationError(_) => 1004,
            SensorError::NotResponding(_) => 1005,
            SensorError::InvalidConfiguration(_) => 1006,
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        "Sensor"
    }
}

impl fmt::Display for SensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SensorError::InitializationFailed(msg) => write!(f, "Sensor initialization failed: {}", msg),
            SensorError::I2CError(msg) => write!(f, "I2C communication error: {}", msg),
            SensorError::InvalidData(msg) => write!(f, "Invalid sensor data: {}", msg),
            SensorError::CalibrationError(msg) => write!(f, "Sensor calibration error: {}", msg),
            SensorError::NotResponding(msg) => write!(f, "Sensor not responding: {}", msg),
            SensorError::InvalidConfiguration(msg) => write!(f, "Invalid sensor configuration: {}", msg),
        }
    }
}

/// Network-related errors
#[derive(Debug, Clone)]
pub enum NetworkError {
    /// WiFi connection failed
    WiFiConnectionFailed(ErrorMessage),
    /// WiFi configuration error
    WiFiConfigurationError(ErrorMessage),
    /// DHCP configuration failed
    DHCPFailed(ErrorMessage),
    /// TCP connection failed
    TCPConnectionFailed(ErrorMessage),
    /// Network timeout
    Timeout(ErrorMessage),
    /// DNS resolution failed
    DNSResolutionFailed(ErrorMessage),
    /// Network hardware initialization failed
    HardwareInitFailed(ErrorMessage),
}

impl NetworkError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match self {
            NetworkError::WiFiConnectionFailed(_) => 2001,
            NetworkError::WiFiConfigurationError(_) => 2002,
            NetworkError::DHCPFailed(_) => 2003,
            NetworkError::TCPConnectionFailed(_) => 2004,
            NetworkError::Timeout(_) => 2005,
            NetworkError::DNSResolutionFailed(_) => 2006,
            NetworkError::HardwareInitFailed(_) => 2007,
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        "Network"
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::WiFiConnectionFailed(msg) => write!(f, "WiFi connection failed: {}", msg),
            NetworkError::WiFiConfigurationError(msg) => write!(f, "WiFi configuration error: {}", msg),
            NetworkError::DHCPFailed(msg) => write!(f, "DHCP failed: {}", msg),
            NetworkError::TCPConnectionFailed(msg) => write!(f, "TCP connection failed: {}", msg),
            NetworkError::Timeout(msg) => write!(f, "Network timeout: {}", msg),
            NetworkError::DNSResolutionFailed(msg) => write!(f, "DNS resolution failed: {}", msg),
            NetworkError::HardwareInitFailed(msg) => write!(f, "Network hardware initialization failed: {}", msg),
        }
    }
}

/// Hardware-related errors
#[derive(Debug, Clone)]
pub enum HardwareError {
    /// GPIO configuration error
    GPIOError(ErrorMessage),
    /// SPI communication error
    SPIError(ErrorMessage),
    /// UART communication error
    UARTError(ErrorMessage),
    /// Timer configuration error
    TimerError(ErrorMessage),
    /// Interrupt configuration error
    InterruptError(ErrorMessage),
    /// Power management error
    PowerError(ErrorMessage),
    /// Clock configuration error
    ClockError(ErrorMessage),
}

impl HardwareError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match self {
            HardwareError::GPIOError(_) => 3001,
            HardwareError::SPIError(_) => 3002,
            HardwareError::UARTError(_) => 3003,
            HardwareError::TimerError(_) => 3004,
            HardwareError::InterruptError(_) => 3005,
            HardwareError::PowerError(_) => 3006,
            HardwareError::ClockError(_) => 3007,
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        "Hardware"
    }
}

impl fmt::Display for HardwareError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HardwareError::GPIOError(msg) => write!(f, "GPIO error: {}", msg),
            HardwareError::SPIError(msg) => write!(f, "SPI error: {}", msg),
            HardwareError::UARTError(msg) => write!(f, "UART error: {}", msg),
            HardwareError::TimerError(msg) => write!(f, "Timer error: {}", msg),
            HardwareError::InterruptError(msg) => write!(f, "Interrupt error: {}", msg),
            HardwareError::PowerError(msg) => write!(f, "Power management error: {}", msg),
            HardwareError::ClockError(msg) => write!(f, "Clock configuration error: {}", msg),
        }
    }
}

/// Configuration-related errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Invalid configuration parameter
    InvalidParameter(ErrorMessage),
    /// Missing required configuration
    MissingConfiguration(ErrorMessage),
    /// Configuration parsing error
    ParsingError(ErrorMessage),
    /// Configuration validation error
    ValidationError(ErrorMessage),
    /// Configuration file not found
    ConfigNotFound(ErrorMessage),
}

impl ConfigError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match self {
            ConfigError::InvalidParameter(_) => 4001,
            ConfigError::MissingConfiguration(_) => 4002,
            ConfigError::ParsingError(_) => 4003,
            ConfigError::ValidationError(_) => 4004,
            ConfigError::ConfigNotFound(_) => 4005,
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        "Configuration"
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidParameter(msg) => write!(f, "Invalid configuration parameter: {}", msg),
            ConfigError::MissingConfiguration(msg) => write!(f, "Missing required configuration: {}", msg),
            ConfigError::ParsingError(msg) => write!(f, "Configuration parsing error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Configuration validation error: {}", msg),
            ConfigError::ConfigNotFound(msg) => write!(f, "Configuration not found: {}", msg),
        }
    }
}

/// System-related errors
#[derive(Debug, Clone)]
pub enum SystemError {
    /// Out of memory
    OutOfMemory(ErrorMessage),
    /// Task creation failed
    TaskCreationFailed(ErrorMessage),
    /// Resource unavailable
    ResourceUnavailable(ErrorMessage),
    /// System initialization failed
    InitializationFailed(ErrorMessage),
    /// Watchdog timeout
    WatchdogTimeout(ErrorMessage),
    /// Stack overflow
    StackOverflow(ErrorMessage),
    /// System panic
    Panic(ErrorMessage),
}

impl SystemError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match self {
            SystemError::OutOfMemory(_) => 5001,
            SystemError::TaskCreationFailed(_) => 5002,
            SystemError::ResourceUnavailable(_) => 5003,
            SystemError::InitializationFailed(_) => 5004,
            SystemError::WatchdogTimeout(_) => 5005,
            SystemError::StackOverflow(_) => 5006,
            SystemError::Panic(_) => 5007,
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        "System"
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::OutOfMemory(msg) => write!(f, "Out of memory: {}", msg),
            SystemError::TaskCreationFailed(msg) => write!(f, "Task creation failed: {}", msg),
            SystemError::ResourceUnavailable(msg) => write!(f, "Resource unavailable: {}", msg),
            SystemError::InitializationFailed(msg) => write!(f, "System initialization failed: {}", msg),
            SystemError::WatchdogTimeout(msg) => write!(f, "Watchdog timeout: {}", msg),
            SystemError::StackOverflow(msg) => write!(f, "Stack overflow: {}", msg),
            SystemError::Panic(msg) => write!(f, "System panic: {}", msg),
        }
    }
}

/// Main IoT error type that encompasses all error categories
#[derive(Debug, Clone)]
pub struct IoTError {
    /// The specific error variant
    kind: IoTErrorKind,
    /// Error context chain
    context: ErrorContext,
}

/// Internal error kind enumeration
#[derive(Debug, Clone)]
pub enum IoTErrorKind {
    /// Sensor-related errors
    Sensor(SensorError),
    /// Network-related errors
    Network(NetworkError),
    /// Hardware-related errors
    Hardware(HardwareError),
    /// Configuration-related errors
    Configuration(ConfigError),
    /// System-related errors
    System(SystemError),
}

impl IoTError {
    /// Create a new IoT error
    pub fn new(kind: IoTErrorKind) -> Self {
        Self {
            kind,
            context: ErrorContext::new(),
        }
    }

    /// Create a sensor error
    pub fn sensor(error: SensorError) -> Self {
        Self::new(IoTErrorKind::Sensor(error))
    }

    /// Create a network error
    pub fn network(error: NetworkError) -> Self {
        Self::new(IoTErrorKind::Network(error))
    }

    /// Create a hardware error
    pub fn hardware(error: HardwareError) -> Self {
        Self::new(IoTErrorKind::Hardware(error))
    }

    /// Create a configuration error
    pub fn configuration(error: ConfigError) -> Self {
        Self::new(IoTErrorKind::Configuration(error))
    }

    /// Create a system error
    pub fn system(error: SystemError) -> Self {
        Self::new(IoTErrorKind::System(error))
    }

    /// Add context to this error
    pub fn with_context(mut self, context: &str) -> Self {
        self.context.add_context(context);
        self
    }

    /// Get the error kind
    pub fn kind(&self) -> &IoTErrorKind {
        &self.kind
    }

    /// Get error context
    pub fn context(&self) -> &ErrorContext {
        &self.context
    }

    /// Get error code for programmatic handling
    pub fn error_code(&self) -> ErrorCode {
        match &self.kind {
            IoTErrorKind::Sensor(e) => e.error_code(),
            IoTErrorKind::Network(e) => e.error_code(),
            IoTErrorKind::Hardware(e) => e.error_code(),
            IoTErrorKind::Configuration(e) => e.error_code(),
            IoTErrorKind::System(e) => e.error_code(),
        }
    }

    /// Get error category name
    pub fn category(&self) -> &'static str {
        match &self.kind {
            IoTErrorKind::Sensor(e) => e.category(),
            IoTErrorKind::Network(e) => e.category(),
            IoTErrorKind::Hardware(e) => e.category(),
            IoTErrorKind::Configuration(e) => e.category(),
            IoTErrorKind::System(e) => e.category(),
        }
    }

    /// Check if this is a sensor error
    pub fn is_sensor_error(&self) -> bool {
        matches!(self.kind, IoTErrorKind::Sensor(_))
    }

    /// Check if this is a network error
    pub fn is_network_error(&self) -> bool {
        matches!(self.kind, IoTErrorKind::Network(_))
    }

    /// Check if this is a hardware error
    pub fn is_hardware_error(&self) -> bool {
        matches!(self.kind, IoTErrorKind::Hardware(_))
    }

    /// Check if this is a configuration error
    pub fn is_configuration_error(&self) -> bool {
        matches!(self.kind, IoTErrorKind::Configuration(_))
    }

    /// Check if this is a system error
    pub fn is_system_error(&self) -> bool {
        matches!(self.kind, IoTErrorKind::System(_))
    }
}

impl fmt::Display for IoTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the main error
        match &self.kind {
            IoTErrorKind::Sensor(e) => write!(f, "{}", e)?,
            IoTErrorKind::Network(e) => write!(f, "{}", e)?,
            IoTErrorKind::Hardware(e) => write!(f, "{}", e)?,
            IoTErrorKind::Configuration(e) => write!(f, "{}", e)?,
            IoTErrorKind::System(e) => write!(f, "{}", e)?,
        }

        // Add context if available
        if !self.context.is_empty() {
            write!(f, " [Context: {}]", self.context)?;
        }

        Ok(())
    }
}

/// Helper macro for creating errors with string literals
/// 
/// This macro provides a convenient way to create IoT errors with compile-time checking.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// let error = iot_error!(sensor, i2c_error, "I2C timeout");
/// let error = iot_error!(network, wifi_failed, "Connection lost");
/// ```
#[macro_export]
macro_rules! iot_error {
    (sensor, init_failed, $msg:expr) => {
        $crate::IoTError::sensor($crate::SensorError::InitializationFailed(
            $crate::error::utils::error_message($msg)
        ))
    };
    (sensor, i2c_error, $msg:expr) => {
        $crate::IoTError::sensor($crate::SensorError::I2CError(
            $crate::error::utils::error_message($msg)
        ))
    };
    (sensor, invalid_data, $msg:expr) => {
        $crate::IoTError::sensor($crate::SensorError::InvalidData(
            $crate::error::utils::error_message($msg)
        ))
    };
    (network, wifi_failed, $msg:expr) => {
        $crate::IoTError::network($crate::NetworkError::WiFiConnectionFailed(
            $crate::error::utils::error_message($msg)
        ))
    };
    (network, tcp_failed, $msg:expr) => {
        $crate::IoTError::network($crate::NetworkError::TCPConnectionFailed(
            $crate::error::utils::error_message($msg)
        ))
    };
    (hardware, gpio_error, $msg:expr) => {
        $crate::IoTError::hardware($crate::HardwareError::GPIOError(
            $crate::error::utils::error_message($msg)
        ))
    };
}

// Conversion traits for existing module errors
// Note: These will need to be updated when the actual error types are available
// Currently implemented as placeholder types for the common patterns we observed

/// Placeholder for BME280 I2C errors (will be replaced with actual embedded-hal errors)
pub trait I2CErrorTrait: fmt::Debug {}

/// Convert I2C errors to IoT sensor errors
impl<E> From<E> for IoTError 
where 
    E: I2CErrorTrait + fmt::Display,
{
    fn from(_error: E) -> Self {
        // Use a simple error message since format! is not available in no_std
        let msg = utils::error_message("I2C communication error");
        IoTError::sensor(SensorError::I2CError(msg))
    }
}

/// Conversion from WiFi-specific errors
pub mod wifi_conversions {
    use super::*;
    
    /// Convert WiFi errors (from wifi-embassy module)
    pub fn from_wifi_error(error_type: &str, message: &str) -> IoTError {
        let msg = utils::error_message(message);
        let network_error = match error_type {
            "HardwareInit" => NetworkError::HardwareInitFailed(msg),
            "Configuration" => NetworkError::WiFiConfigurationError(msg),
            "Connection" => NetworkError::WiFiConnectionFailed(msg),
            "Dhcp" => NetworkError::DHCPFailed(msg),
            _ => NetworkError::WiFiConnectionFailed(msg),
        };
        IoTError::network(network_error)
    }
}

/// Conversion from MQTT-specific errors
pub mod mqtt_conversions {
    use super::*;
    
    /// Convert MQTT errors (from mqtt-embassy module)  
    pub fn from_mqtt_error(error_type: &str, message: &str) -> IoTError {
        let msg = utils::error_message(message);
        let network_error = match error_type {
            "ConnectionFailed" => NetworkError::TCPConnectionFailed(msg),
            "ProtocolError" => NetworkError::TCPConnectionFailed(msg), // MQTT protocol is over TCP
            "IoError" => NetworkError::TCPConnectionFailed(msg),
            "SerializationError" => {
                // This is more of a system error
                return IoTError::system(SystemError::InitializationFailed(msg));
            },
            _ => NetworkError::TCPConnectionFailed(msg),
        };
        IoTError::network(network_error)
    }
}

/// Conversion from serial console errors
pub mod console_conversions {
    use super::*;
    
    /// Convert console/UART errors to hardware errors
    pub fn from_uart_error(message: &str) -> IoTError {
        let msg = utils::error_message(message);
        IoTError::hardware(HardwareError::UARTError(msg))
    }
}

/// Convert from embedded-hal errors
#[cfg(feature = "embedded-hal")]
pub mod embedded_hal_conversions {
    use super::*;
    use embedded_hal::i2c;
    
    impl From<i2c::ErrorKind> for IoTError {
        fn from(error: i2c::ErrorKind) -> Self {
            let msg = match error {
                i2c::ErrorKind::Bus => "I2C bus error",
                i2c::ErrorKind::ArbitrationLoss => "I2C arbitration loss", 
                i2c::ErrorKind::NoAcknowledge(_) => "I2C no acknowledge",
                i2c::ErrorKind::Overrun => "I2C overrun",
                _ => "Unknown I2C error",
            };
            IoTError::sensor(SensorError::I2CError(utils::error_message(msg)))
        }
    }
}

/// Convert nb::Error types (common in embedded)
impl<E> From<nb::Error<E>> for IoTError
where
    E: fmt::Debug + fmt::Display,
{
    fn from(error: nb::Error<E>) -> Self {
        match error {
            nb::Error::WouldBlock => {
                IoTError::system(SystemError::ResourceUnavailable(
                    utils::error_message("Operation would block")
                ))
            }
            nb::Error::Other(_e) => {
                // Use a generic error message since format! is not available in no_std
                let msg = utils::error_message("Hardware operation failed");
                IoTError::hardware(HardwareError::GPIOError(msg))
            }
        }
    }
}

/// Utility functions for error creation
pub mod utils {
    use super::*;

    /// Create error message from string slice
    pub fn error_message(msg: &str) -> ErrorMessage {
        let mut result = ErrorMessage::new();
        
        if msg.len() <= MAX_ERROR_MESSAGE_LEN {
            // Message fits, use it as-is
            let _ = result.push_str(msg);
        } else {
            // Message too long, truncate and add ellipsis
            let truncated_len = MAX_ERROR_MESSAGE_LEN.saturating_sub(3);
            let truncated = &msg[..truncated_len];
            let _ = result.push_str(truncated);
            let _ = result.push_str("...");
        }
        
        result
    }

    /// Create sensor error with message
    pub fn sensor_error(error_type: &str, msg: &str) -> IoTError {
        let error_msg = error_message(msg);
        let sensor_error = match error_type {
            "init_failed" => SensorError::InitializationFailed(error_msg),
            "i2c_error" => SensorError::I2CError(error_msg),
            "invalid_data" => SensorError::InvalidData(error_msg),
            "calibration_error" => SensorError::CalibrationError(error_msg),
            "not_responding" => SensorError::NotResponding(error_msg),
            "invalid_config" => SensorError::InvalidConfiguration(error_msg),
            _ => SensorError::InitializationFailed(error_msg),
        };
        IoTError::sensor(sensor_error)
    }

    /// Create network error with message
    pub fn network_error(error_type: &str, msg: &str) -> IoTError {
        let error_msg = error_message(msg);
        let network_error = match error_type {
            "wifi_failed" => NetworkError::WiFiConnectionFailed(error_msg),
            "wifi_config" => NetworkError::WiFiConfigurationError(error_msg),
            "dhcp_failed" => NetworkError::DHCPFailed(error_msg),
            "tcp_failed" => NetworkError::TCPConnectionFailed(error_msg),
            "timeout" => NetworkError::Timeout(error_msg),
            "dns_failed" => NetworkError::DNSResolutionFailed(error_msg),
            "hw_init_failed" => NetworkError::HardwareInitFailed(error_msg),
            _ => NetworkError::WiFiConnectionFailed(error_msg),
        };
        IoTError::network(network_error)
    }
}