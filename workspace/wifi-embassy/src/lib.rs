#![no_std]
#![doc = include_str!("../README.md")]

//! # WiFi Embassy
//! 
//! WiFi connectivity module using Embassy async framework for ESP32-C3.
//! 
//! This module provides robust WiFi connectivity with DHCP, automatic reconnection,
//! and seamless integration with the Embassy async ecosystem.

pub mod wifi_manager;

// Re-export main types
pub use wifi_manager::{WiFiManager, WiFiConfig, ConnectionInfo, WiFiError};

// Re-export Embassy types for convenience
pub use embassy_executor::Spawner;
pub use embassy_time::{Duration, Timer};