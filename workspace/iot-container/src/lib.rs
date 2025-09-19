//! # IoT Container - Dependency Injection for ESP32-C3 IoT Systems
//!
//! A comprehensive dependency injection container designed for embedded IoT systems.
//! This container enables clean architecture by decoupling business logic from 
//! concrete implementations, enabling comprehensive testing and flexible deployment.
//!
//! ## Features
//!
//! - **Trait-Based Architecture**: All components implement well-defined trait interfaces
//! - **Configuration-Driven**: Components created based on system configuration
//! - **Async/Await Support**: Full Embassy async runtime integration
//! - **Mock Implementations**: Complete test doubles for all traits
//! - **No-std Compatible**: Works in embedded environments without heap allocation
//! - **Type Safety**: Compile-time dependency validation
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     IoT Container                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │  SensorReader  │ NetworkManager │ MessagePublisher │ Console │
//! ├─────────────────────────────────────────────────────────────┤
//! │               Hardware Abstraction Layer                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use iot_container::{IoTContainer, SystemConfiguration, ComponentFactory};
//! use iot_hal::Esp32C3Platform;
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) -> ! {
//!     // Load configuration
//!     let config = SystemConfiguration::from_env()
//!         .unwrap_or_else(|_| SystemConfiguration::default());
//!     
//!     // Initialize hardware platform
//!     let mut platform = Esp32C3Platform::initialize().await
//!         .expect("Failed to initialize hardware platform");
//!     
//!     // Create components using factory
//!     let sensor = ComponentFactory::create_sensor(&mut platform, &config.sensor).await
//!         .expect("Failed to create sensor");
//!         
//!     let network = ComponentFactory::create_network_manager(&mut platform, &config.wifi).await
//!         .expect("Failed to create network manager");
//!         
//!     let publisher = ComponentFactory::create_message_publisher(&network, &config.mqtt).await
//!         .expect("Failed to create message publisher");
//!         
//!     let console = ComponentFactory::create_console(&mut platform, &config.console).await
//!         .expect("Failed to create console");
//!     
//!     // Create and run IoT container
//!     let mut container = IoTContainer::new(
//!         platform,
//!         sensor,
//!         network, 
//!         publisher,
//!         console,
//!         config,
//!     ).await.expect("Failed to create IoT container");
//!     
//!     // Run the system with dependency injection
//!     container.run_system().await.expect("System execution failed");
//! }
//! ```
//!
//! ## Testing with Mocks
//!
//! ```rust
//! use iot_container::{IoTContainer, mocks::*};
//!
//! #[tokio::test]
//! async fn test_system_integration() {
//!     let platform = MockPlatform::new();
//!     let sensor = MockSensorReader::new();
//!     let network = MockNetworkManager::new(); 
//!     let publisher = MockMessagePublisher::new();
//!     let console = MockConsoleInterface::new();
//!     let config = SystemConfiguration::test_config();
//!     
//!     let mut container = IoTContainer::new(
//!         platform, sensor, network, publisher, console, config
//!     ).await.unwrap();
//!     
//!     // Test system behavior with controlled mocks
//!     container.run_single_cycle().await.unwrap();
//! }
//! ```

#![no_std]

// Only require alloc for mock implementations and std testing
#[cfg(any(feature = "mock", test))]
extern crate alloc;

// Core modules
pub mod traits;
pub mod container;
pub mod factory;
pub mod config;

// Mock implementations for testing
#[cfg(feature = "mock")]
pub mod mocks;

// Re-export main types
pub use traits::{
    SensorReader, NetworkManager, MessagePublisher, ConsoleInterface,
    Measurements, ConnectionInfo, SensorData, DeviceStatus
};
pub use container::IoTContainer;
pub use factory::ComponentFactory;
pub use config::{
    SystemConfiguration, SensorConfig, WiFiConfig, MqttConfig, 
    ConsoleConfig, LogLevel, OperatingMode
};

// Re-export mock implementations when available
#[cfg(feature = "mock")]
pub use mocks::{
    MockSensorReader, MockNetworkManager, MockMessagePublisher, 
    MockConsoleInterface, MockPlatform
};

// Re-export commonly used types for convenience
pub use embassy_executor::Spawner;
pub use embassy_time::{Duration, Timer};
pub use iot_common::{IoTError, IoTResult};
pub use iot_hal::HardwarePlatform;

/// Current version of the iot-container library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of measurement samples buffered in container
pub const MAX_MEASUREMENT_BUFFER: usize = 16;

/// Default system operation interval in seconds
pub const DEFAULT_OPERATION_INTERVAL_SECS: u64 = 30;

/// Maximum retry attempts for failed operations
pub const MAX_RETRY_ATTEMPTS: u32 = 3;