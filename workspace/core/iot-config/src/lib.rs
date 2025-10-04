#![no_std]

//! IoT Configuration Management
//! 
//! Unified configuration system for ESP32-C3 IoT applications.
//! Provides type-safe configuration with validation, persistence, and feature flags.

extern crate alloc;
#[allow(unused_imports)]
use alloc::{string::String, format};
#[allow(unused_imports)]
use core::str::FromStr;

#[allow(unused_imports)]
use iot_common::{IoTResult, IoTError};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use heapless::String as HeaplessString;

pub mod unified;
pub mod validation;
pub mod feature_flags;
pub mod embedded;

pub use unified::*;
pub use validation::*;
pub use feature_flags::*;
pub use embedded::*;

/// Configuration error types specific to configuration management
#[derive(Debug, Clone)]
pub enum ConfigError {
    InvalidValue(String),
    ValidationFailed(String),
    SerializationError(String),
    StorageError(String),
    FeatureNotEnabled(String),
}

impl From<ConfigError> for IoTError {
    fn from(error: ConfigError) -> Self {
        use iot_common::error::ErrorMessage;
        match error {
            ConfigError::InvalidValue(msg) => {
                let err_msg = ErrorMessage::from_str(&msg).unwrap_or_default();
                IoTError::configuration(iot_common::ConfigError::InvalidParameter(err_msg))
            }
            ConfigError::ValidationFailed(msg) => {
                let err_msg = ErrorMessage::from_str(&msg).unwrap_or_default();
                IoTError::configuration(iot_common::ConfigError::ValidationError(err_msg))  
            }
            ConfigError::SerializationError(msg) => {
                let err_msg = ErrorMessage::from_str(&msg).unwrap_or_default();
                IoTError::configuration(iot_common::ConfigError::ParsingError(err_msg))
            }
            ConfigError::StorageError(msg) => {
                let err_msg = ErrorMessage::from_str(&msg).unwrap_or_default();
                IoTError::hardware(iot_common::HardwareError::GPIOError(err_msg))
            }
            ConfigError::FeatureNotEnabled(msg) => {
                let err_msg = ErrorMessage::from_str(&msg).unwrap_or_default();
                IoTError::configuration(iot_common::ConfigError::MissingConfiguration(err_msg))
            }
        }
    }
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Utility for creating heapless strings safely
pub fn create_heapless_string<const N: usize>(s: &str) -> ConfigResult<HeaplessString<N>> {
    HeaplessString::from_str(s).map_err(|_| ConfigError::InvalidValue(
        format!("String '{}' too long for buffer size {}", s, N)
    ))
}

/// Utility for validating and creating bounded strings
pub fn create_bounded_string<const N: usize>(s: &str, field_name: &str) -> ConfigResult<HeaplessString<N>> {
    if s.is_empty() {
        return Err(ConfigError::InvalidValue(format!("{} cannot be empty", field_name)));
    }
    if s.len() > N {
        return Err(ConfigError::InvalidValue(format!("{} too long (max {} chars)", field_name, N)));
    }
    create_heapless_string(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_heapless_string_creation() {
        let result = create_heapless_string::<32>("test");
        assert!(result.is_ok());
        
        let long_string = "a".repeat(100);
        let result = create_heapless_string::<32>(&long_string);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_bounded_string_validation() {
        let result = create_bounded_string::<32>("valid", "test_field");
        assert!(result.is_ok());
        
        let result = create_bounded_string::<32>("", "test_field");
        assert!(result.is_err());
    }
}