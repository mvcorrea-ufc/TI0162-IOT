//! # Result Type Aliases and Utilities
//!
//! This module provides convenient Result type aliases for different error categories
//! and utility functions for working with IoT results in embedded environments.

use crate::error::{IoTError, SensorError, NetworkError, HardwareError, ConfigError, SystemError};

/// Main result type for IoT operations
pub type IoTResult<T> = Result<T, IoTError>;

/// Result type for sensor operations
pub type SensorResult<T> = Result<T, SensorError>;

/// Result type for network operations  
pub type NetworkResult<T> = Result<T, NetworkError>;

/// Result type for hardware operations
pub type HardwareResult<T> = Result<T, HardwareError>;

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Result type for system operations
pub type SystemResult<T> = Result<T, SystemError>;

/// Extension trait for Result types to add context and conversion utilities
pub trait IoTResultExt<T> {
    /// Add context to an error result
    fn with_context(self, context: &str) -> IoTResult<T>;

    /// Convert to IoTResult with context
    fn into_iot_result(self, context: &str) -> IoTResult<T>;

    /// Log error using RTT if available (no-op in release builds without RTT)
    fn log_error(self, operation: &str) -> Self;
}

impl<T> IoTResultExt<T> for IoTResult<T> {
    fn with_context(self, context: &str) -> IoTResult<T> {
        self.map_err(|e| e.with_context(context))
    }

    fn into_iot_result(self, _context: &str) -> IoTResult<T> {
        self
    }

    fn log_error(self, operation: &str) -> Self {
        if let Err(ref _e) = self {
            #[cfg(feature = "debug")]
            {
                // In debug builds with RTT support, log the error
                // Note: This requires rtt_target dependency when debug feature is enabled
                // rtt_target::rprintln!("[ERROR] {}: {} (Code: {})", operation, e, e.error_code());
                // For now, we'll use a placeholder since RTT setup varies by application
                let _ = (operation, e); // Suppress unused warnings
            }
            #[cfg(not(feature = "debug"))]
            {
                // In release builds, this is a no-op
                let _ = operation; // Suppress unused variable warning
            }
        }
        self
    }
}

// Note: We'll provide specific implementations for common error types instead
// of a blanket implementation to avoid conflicts

/// Utility functions for working with results
pub mod utils {
    use super::*;
    use crate::error::ErrorMessage;

    /// Convert a generic error to IoTError with context
    pub fn to_iot_error<E>(error: E, context: &str) -> IoTError
    where
        E: Into<IoTError>,
    {
        error.into().with_context(context)
    }

    /// Create an error result with context
    pub fn error_result<T>(error: IoTError, context: &str) -> IoTResult<T> {
        Err(error.with_context(context))
    }

    /// Safely convert string to ErrorMessage
    pub fn safe_error_message(msg: &str) -> ErrorMessage {
        crate::error::utils::error_message(msg)
    }

    /// Execute two operations in sequence, stopping at first error
    pub fn chain_two_results<T1, T2, F1, F2>(op1: F1, op2: F2) -> IoTResult<(T1, T2)>
    where
        F1: FnOnce() -> IoTResult<T1>,
        F2: FnOnce() -> IoTResult<T2>,
    {
        let result1 = op1()?;
        let result2 = op2()?;
        Ok((result1, result2))
    }

    /// Execute operation with automatic error context
    pub fn with_operation_context<T, F>(operation_name: &str, f: F) -> IoTResult<T>
    where
        F: FnOnce() -> IoTResult<T>,
    {
        f().with_context(operation_name)
    }
}

/// Macro for convenient error handling with context
#[macro_export]
macro_rules! iot_try {
    ($expr:expr, $context:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err($crate::error::IoTError::from(err).with_context($context)),
        }
    };
}

/// Macro for early return with IoT error
#[macro_export]
macro_rules! iot_bail {
    ($error:expr) => {
        return Err($error.into())
    };
    ($error:expr, $context:expr) => {
        return Err($error.into().with_context($context))
    };
}

/// Macro for ensuring conditions with IoT errors
#[macro_export]
macro_rules! iot_ensure {
    ($condition:expr, $error:expr) => {
        if !($condition) {
            $crate::iot_bail!($error);
        }
    };
    ($condition:expr, $error:expr, $context:expr) => {
        if !($condition) {
            $crate::iot_bail!($error, $context);
        }
    };
}

// Tests are in tests/error_tests.rs to avoid no_std compilation issues