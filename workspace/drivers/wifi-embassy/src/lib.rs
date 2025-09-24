//! # WiFi Embassy - Robust WiFi Connectivity for ESP32-C3
//!
//! A production-ready WiFi connectivity module using the Embassy async framework.
//! This module provides reliable WiFi connection management with automatic
//! reconnection, DHCP support, and comprehensive error handling.
//!
//! ## Features
//!
//! - **Robust WiFi Connectivity**: Automatic connection with retry logic
//! - **DHCP Support**: Automatic IP address acquisition and management
//! - **Embassy Integration**: Full async/await support with Embassy framework
//! - **Automatic Reconnection**: Graceful handling of network disconnections
//! - **Connection Monitoring**: Real-time status verification and reporting
//! - **Network Stack Access**: Provides embassy-net stack for TCP/UDP operations
//! - **Error Handling**: Comprehensive error reporting with context
//! - **Dual Address Support**: Automatic detection of both I2C addresses
//!
//! ## Quick Start
//!
//! ```no_run
//! use wifi_embassy::{WiFiManager, WiFiConfig};
//! use embassy_executor::Spawner;
//!
//! #[esp_hal_embassy::main]
//! async fn main(spawner: Spawner) {
//!     let peripherals = esp_hal::init(esp_hal::Config::default());
//!     
//!     // Initialize Embassy before WiFi
//!     esp_hal_embassy::init(peripherals.TIMG1);
//!     
//!     let wifi_config = WiFiConfig {
//!         ssid: "YourNetwork",
//!         password: "YourPassword",
//!     };
//!     
//!     let wifi_manager = WiFiManager::new(
//!         spawner,
//!         peripherals.TIMG0,
//!         peripherals.WIFI,
//!         peripherals.RNG,
//!         wifi_config,
//!     ).await.unwrap();
//!     
//!     // WiFi is now connected and ready for use
//!     let stack = wifi_manager.get_stack();
//!     
//!     // Use stack for network operations...
//! }
//! ```
//!
//! ## Network Requirements
//!
//! - **Frequency**: 2.4GHz WiFi network (ESP32-C3 doesn't support 5GHz)
//! - **Security**: WPA2-Personal or WPA3-Personal
//! - **DHCP**: Must be enabled on the router
//! - **Connectivity**: Stable internet connection for full functionality
//!
//! ## Integration with Other Modules
//!
//! This module is designed to work seamlessly with other Embassy-based modules:
//!
//! ```no_run
//! use wifi_embassy::WiFiManager;
//! use mqtt_embassy::MqttClient;
//! use bme280_embassy::BME280;
//!
//! // Complete IoT system integration
//! let wifi_manager = WiFiManager::new(/* ... */).await?;
//! let stack = wifi_manager.get_stack();
//! 
//! let mqtt_client = MqttClient::new(stack).await?;
//! let mut sensor = BME280::new(&mut i2c);
//! 
//! // Publish sensor data via WiFi and MQTT
//! let measurements = sensor.read_measurements().await?;
//! mqtt_client.publish("sensors/temperature", &measurements).await?;
//! ```
//!
//! ## Error Handling
//!
//! The module provides comprehensive error handling through the [`WiFiError`] enum:
//!
//! ```no_run
//! use wifi_embassy::{WiFiManager, WiFiError};
//!
//! match WiFiManager::new(/* ... */).await {
//!     Ok(manager) => {
//!         println!("WiFi connected successfully");
//!         // Use the manager...
//!     }
//!     Err(WiFiError::Connection(msg)) => {
//!         println!("Connection failed: {}", msg);
//!         // Handle connection error...
//!     }
//!     Err(e) => {
//!         println!("WiFi error: {}", e);
//!         // Handle other errors...
//!     }
//! }
//! ```
//!
//! ## Configuration
//!
//! WiFi credentials are typically loaded from environment variables in `.cargo/config.toml`:
//!
//! ```toml
//! [env]
//! WIFI_SSID = "YourNetworkName"
//! WIFI_PASSWORD = "YourNetworkPassword"
//! ```
//!
//! Then use in code:
//!
//! ```rust
//! let wifi_config = WiFiConfig {
//!     ssid: env!("WIFI_SSID"),
//!     password: env!("WIFI_PASSWORD"),
//! };
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Connection Time**: Typically 3-5 seconds for initial connection
//! - **Memory Usage**: ~8KB heap allocation for network stack
//! - **Power Consumption**: Varies with WiFi activity (avg ~100mA during transmission)
//! - **Reconnection**: Automatic with exponential backoff
//!
//! ## Troubleshooting
//!
//! Common issues and solutions:
//!
//! 1. **Connection Timeout**: Verify SSID and password, ensure 2.4GHz network
//! 2. **DHCP Failure**: Check router DHCP settings and available IP addresses
//! 3. **Embassy Not Initialized**: Call `esp_hal_embassy::init()` before creating WiFiManager
//! 4. **Memory Issues**: Ensure sufficient heap allocation (72KB minimum)
//!
//! See the module documentation and examples for detailed troubleshooting guides.

#![no_std]

pub mod wifi_manager;

// IoT Container trait implementation (optional feature)
#[cfg(feature = "container")]
mod trait_impl;

// Re-export main types for convenient access
pub use wifi_manager::{WiFiManager, WiFiConfig, ConnectionInfo, WiFiError};

// Re-export container integration when available
#[cfg(feature = "container")]
pub use trait_impl::{WiFiContainerAdapter, create_container_network_manager, create_container_network_manager_with_interval};

// Re-export commonly used Embassy types
pub use embassy_executor::Spawner;
pub use embassy_time::{Duration, Timer};
pub use embassy_net::Stack;