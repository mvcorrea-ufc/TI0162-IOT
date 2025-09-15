#![no_std]

//! # MQTT Embassy
//! 
//! MQTT client module using Embassy async framework for ESP32-C3.
//! 
//! This module provides async MQTT publishing capabilities with JSON data serialization,
//! designed to work seamlessly with the wifi-embassy network stack.

extern crate alloc;

pub mod mqtt_client;
pub mod message;

// Re-export main types
pub use mqtt_client::{MqttClient, MqttConfig, MqttError};
pub use message::{MqttMessage, SensorData, DeviceStatus};

// Re-export Embassy types for convenience
pub use embassy_executor::Spawner;
pub use embassy_time::{Duration, Timer};