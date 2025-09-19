//! Serial Console Embassy - Interactive serial interface for ESP32-C3 IoT system
//! 
//! Provides a command-line interface via UART for:
//! - System information display
//! - WiFi credentials configuration
//! - MQTT broker configuration  
//! - Real-time system monitoring
//! - Module status checking

#![no_std]

pub mod console;
pub mod commands;
pub mod config;

// IoT Container trait implementation (optional feature)
#[cfg(feature = "container")]
mod trait_impl;

pub use console::SerialConsole;
pub use commands::{Command, CommandHandler};
pub use config::{SystemConfig, WiFiCredentials, MqttConfig};

// Re-export container integration when available
#[cfg(feature = "container")]
pub use trait_impl::{ConsoleContainerAdapter, create_container_console, create_container_console_with_config};