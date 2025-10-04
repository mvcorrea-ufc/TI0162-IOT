//! # IoT Hardware Abstraction Layer (HAL)
//!
//! A comprehensive hardware abstraction layer for IoT systems that enables:
//! - **Platform Portability**: Support for multiple hardware platforms
//! - **Dependency Injection**: Clean separation of business logic from hardware
//! - **Testing**: Mock implementations for comprehensive unit testing
//! - **Configuration**: Hardware setup through configuration rather than code
//!
//! ## Architecture Overview
//!
//! This HAL provides abstract interfaces for common IoT hardware components:
//! - I2C buses for sensors
//! - UART interfaces for serial communication  
//! - GPIO pins for status indicators
//! - Timer functionality for delays
//! - WiFi connectivity for networking
//!
//! ## Platform Support
//!
//! ### ESP32-C3 Platform
//! Production implementation using ESP-HAL for real hardware:
//! ```rust
//! use iot_hal::{HardwarePlatform, esp32c3::Esp32C3Platform};
//! 
//! let platform = Esp32C3Platform::initialize().await?;
//! let i2c_bus = platform.get_i2c();
//! ```
//!
//! ### Mock Platform  
//! Testing implementation for unit tests and simulation:
//! ```rust
//! use iot_hal::{HardwarePlatform, mock::MockPlatform};
//! 
//! let platform = MockPlatform::initialize().await?;
//! // Use for testing without real hardware
//! ```
//!
//! ## Usage Examples
//!
//! ### Basic Platform Initialization
//! ```rust
//! use iot_hal::{HardwarePlatform, HardwareConfig};
//!
//! #[cfg(feature = "esp32c3")]
//! use iot_hal::esp32c3::Esp32C3Platform;
//! 
//! #[cfg(feature = "mock")]
//! use iot_hal::mock::MockPlatform;
//!
//! async fn initialize_hardware() -> Result<impl HardwarePlatform, iot_common::IoTError> {
//!     let config = HardwareConfig::default();
//!     
//!     #[cfg(feature = "esp32c3")]
//!     let platform = Esp32C3Platform::initialize_with_config(config).await?;
//!     
//!     #[cfg(feature = "mock")]
//!     let platform = MockPlatform::initialize_with_config(config).await?;
//!     
//!     Ok(platform)
//! }
//! ```
//!
//! ### Using Hardware Interfaces
//! ```rust
//! async fn use_hardware(mut platform: impl HardwarePlatform) -> Result<(), iot_common::IoTError> {
//!     // Get I2C interface for sensors
//!     let i2c = platform.get_i2c();
//!     
//!     // Get UART for console
//!     let (uart_tx, uart_rx) = platform.get_console();
//!     
//!     // Get status LED
//!     let led = platform.get_status_led();
//!     
//!     // Get timer for delays
//!     let timer = platform.get_timer();
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - `esp32c3` (default): ESP32-C3 platform implementation
//! - `mock`: Mock platform for testing
//!
//! ## Error Handling
//!
//! All hardware operations return `Result<T, IoTError>` for consistent error handling
//! across the system. Hardware-specific errors are wrapped in `IoTError::Hardware`.

#![no_std]

// Require alloc for async_trait Box usage and mock features
extern crate alloc;

// Core modules
pub mod traits;
pub mod config;
pub mod error;

// Platform implementations
#[cfg(feature = "esp32c3")]
pub mod esp32c3;

#[cfg(feature = "mock")]
pub mod mock;

// Re-export core types
pub use traits::{HardwarePlatform, I2cInterface, UartTxInterface, UartRxInterface, GpioInterface, TimerInterface, WiFiInterface, WiFiConnectionInfo};
pub use config::{HardwareConfig, I2cConfig, UartConfig, WiFiConfig as HalWiFiConfig};
pub use error::{HardwareResult};

// Re-export platform implementations
#[cfg(feature = "esp32c3")]
pub use esp32c3::Esp32C3Platform;

#[cfg(feature = "mock")]
pub use mock::MockPlatform;

// Version and metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");