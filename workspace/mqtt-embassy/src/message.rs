//! MQTT message types and JSON serialization
//!
//! Defines data structures for sensor readings and device status,
//! with JSON serialization support for MQTT payloads.

use serde::{Deserialize, Serialize};
use heapless::String;

/// MQTT message structure
#[derive(Debug, Clone)]
pub struct MqttMessage<'a> {
    pub topic: &'a str,
    pub payload: &'a [u8],
    pub qos: u8,
    pub retain: bool,
}

impl<'a> MqttMessage<'a> {
    /// Create a new MQTT message with QoS 0 and no retain
    pub fn new(topic: &'a str, payload: &'a [u8]) -> Self {
        Self {
            topic,
            payload,
            qos: 0,
            retain: false,
        }
    }
    
    /// Set QoS level (0, 1, or 2)
    pub fn with_qos(mut self, qos: u8) -> Self {
        self.qos = qos;
        self
    }
    
    /// Set retain flag
    pub fn with_retain(mut self, retain: bool) -> Self {
        self.retain = retain;
        self
    }
}

/// BME280 sensor data structure matching the project specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorData {
    pub temperature: f32,  // °C
    pub humidity: f32,     // %
    pub pressure: f32,     // hPa
}

impl SensorData {
    /// Create new sensor data
    pub fn new(temperature: f32, humidity: f32, pressure: f32) -> Self {
        Self {
            temperature,
            humidity,
            pressure,
        }
    }
    
    /// Serialize to JSON string (max 256 bytes)
    pub fn to_json(&self) -> Result<String<256>, &'static str> {
        serde_json_core::to_string(self).map_err(|_| "JSON serialization failed")
    }
}

/// Device status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub status: String<32>,     // "online", "offline", "error"
    pub uptime: u32,            // seconds
    pub free_heap: u32,         // bytes
    pub wifi_rssi: i8,          // dBm
}

impl DeviceStatus {
    /// Create new device status
    pub fn new(status: &str, uptime: u32, free_heap: u32, wifi_rssi: i8) -> Self {
        let mut status_str = String::new();
        status_str.push_str(status).ok();
        
        Self {
            status: status_str,
            uptime,
            free_heap,
            wifi_rssi,
        }
    }
    
    /// Serialize to JSON string (max 256 bytes)
    pub fn to_json(&self) -> Result<String<256>, &'static str> {
        serde_json_core::to_string(self).map_err(|_| "JSON serialization failed")
    }
}

/// Complete MQTT payload matching CLAUDE.md specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttPayload {
    pub timestamp: String<32>,   // ISO 8601 format
    pub sensor: String<16>,      // "BME280"
    pub data: SensorData,
}

impl MqttPayload {
    /// Create new MQTT payload with current sensor data
    pub fn new(sensor_data: SensorData) -> Self {
        let mut sensor_name = String::new();
        sensor_name.push_str("BME280").ok();
        
        let mut timestamp = String::new();
        // For now, use a simple timestamp - in real implementation would use RTC
        timestamp.push_str("2025-01-15T10:30:00Z").ok();
        
        Self {
            timestamp,
            sensor: sensor_name,
            data: sensor_data,
        }
    }
    
    /// Create with custom timestamp
    pub fn with_timestamp(mut self, timestamp: &str) -> Self {
        self.timestamp.clear();
        self.timestamp.push_str(timestamp).ok();
        self
    }
    
    /// Serialize to JSON string (max 512 bytes) 
    pub fn to_json(&self) -> Result<String<512>, &'static str> {
        serde_json_core::to_string(self).map_err(|_| "JSON serialization failed")
    }
}