//! # IoT Common Library
//!
//! A unified error handling and common utilities library for ESP32-C3 IoT systems.
//! This library provides a consistent error handling framework across all IoT modules
//! while maintaining `no_std` compatibility for embedded environments.
//!
//! ## Features
//!
//! - **Unified Error Types**: Consistent error handling across all IoT modules
//! - **No-std Compatible**: Works in embedded environments without heap allocation
//! - **Error Context**: Preserves error context for debugging without heap allocation
//! - **Error Conversion**: Automatic conversion from module-specific errors
//! - **RTT Debugging**: Support for Real-Time Transfer debugging
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iot_common::{IoTResult, IoTError, SensorError};
//!
//! fn read_sensor() -> IoTResult<f32> {
//!     // Your sensor reading logic
//!     match sensor.read() {
//!         Ok(value) => Ok(value),
//!         Err(_) => Err(IoTError::Sensor(SensorError::ReadFailure("I2C timeout")))
//!     }
//! }
//! ```

#![no_std]
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]

pub mod error;
pub mod result;
pub mod standard_messages;
pub mod standard_timing;
pub mod standard_config;

#[cfg(feature = "testing")]
pub mod testing;

// Re-export main types for convenience
pub use error::{
    IoTError, SensorError, NetworkError, HardwareError, 
    ConfigError, SystemError, ErrorContext, ErrorCode
};
pub use result::{IoTResult, SensorResult, NetworkResult, HardwareResult};
pub use standard_messages::{
    StandardSensorReading, StandardHeartbeat, StandardDeviceStatus,
    IoTArchitecture, StandardTopics, TimestampProvider, SyncTimestampProvider,
    LegacyFormat
};
pub use standard_timing::{
    StandardTimingConfig, SyncTimingCycles, TimingEvent, TimingManager
};
pub use standard_config::{
    StandardIoTConfig, NetworkConfig, MqttConfig, FeatureFlags, HardwareConfig,
    SystemConfig, ConfigOverrides, ConfigBuilder, Feature
};

#[cfg(feature = "embassy")]
pub use standard_timing::AsyncTimingDurations;

/// Current version of the iot-common library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum length for error messages in embedded environment
pub const MAX_ERROR_MESSAGE_LEN: usize = 64;

/// Maximum depth for error context chain
pub const MAX_ERROR_CONTEXT_DEPTH: usize = 4;