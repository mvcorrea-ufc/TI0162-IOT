//! Standardized MQTT Message Formats
//! 
//! This module defines the unified message formats that ALL ESP32-C3 IoT applications
//! must use to achieve 100% functionality parity. These structures replace the 
//! inconsistent message formats currently used across different applications.

// Use heapless for no_std JSON formatting
use serde::{Deserialize, Serialize};
use serde_json_core;

/// Standard sensor reading format - MANDATORY for all applications
/// 
/// This structure ensures consistent field ordering and content across
/// synchronous (main-nodeps) and asynchronous (main-min, main-app) applications.
/// 
/// JSON Output Format:
/// ```json
/// {
///   "temperature": 22.45,
///   "humidity": 65.2, 
///   "pressure": 1013.25,
///   "timestamp": 1234567890,
///   "device_id": "esp32-c3-nodeps",
///   "reading_count": 123
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardSensorReading {
    /// Temperature in Celsius (always first field)
    pub temperature: f32,
    /// Humidity percentage (always second field)  
    pub humidity: f32,
    /// Pressure in hPa (always third field)
    pub pressure: f32,
    /// Timestamp in milliseconds since boot (NEW - for synchronization)
    pub timestamp: u64,
    /// Device identifier distinguishing applications (NEW - for identification)
    pub device_id: &'static str,
    /// Sequential reading counter (STANDARDIZED - was "reading" vs "count")
    pub reading_count: u32,
}

impl StandardSensorReading {
    /// Create new standardized sensor reading
    pub fn new(
        temperature: f32,
        humidity: f32, 
        pressure: f32,
        timestamp: u64,
        device_id: &'static str,
        reading_count: u32,
    ) -> Self {
        Self {
            temperature,
            humidity,
            pressure,
            timestamp,
            device_id,
            reading_count,
        }
    }
    
    /// Convert to standardized JSON format using heapless (no_std compatible)
    #[allow(dead_code)]
    pub fn to_standard_json_deprecated(&self) -> &'static str {
        "Use to_json_bytes() instead for no_std compatibility"
    }
    
    /// Convert to standardized JSON using serde (no_std compatible)
    pub fn to_json_bytes(&self) -> Result<heapless::Vec<u8, 256>, serde_json_core::ser::Error> {
        serde_json_core::to_vec(self)
    }
}

/// Legacy message formats for backward compatibility
#[derive(Debug, Clone, Copy)]
pub enum LegacyFormat {
    /// main-nodeps original format with different field ordering
    MainNodeps,
    /// main-min/main-app original format without metadata
    MainMinApp,
}

/// Standard heartbeat message format - MANDATORY for all applications
/// 
/// JSON Output Format:
/// ```json
/// {
///   "status": "ping",
///   "timestamp": 1234567890,
///   "device_id": "esp32-c3-nodeps",
///   "sequence": 42
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardHeartbeat {
    /// Status indicator (always "ping" for heartbeat)
    pub status: &'static str,
    /// Timestamp in milliseconds since boot
    pub timestamp: u64,
    /// Device identifier distinguishing applications
    pub device_id: &'static str,
    /// Heartbeat sequence number
    pub sequence: u32,
}

impl StandardHeartbeat {
    /// Create new standardized heartbeat
    pub fn new(timestamp: u64, device_id: &'static str, sequence: u32) -> Self {
        Self {
            status: "ping",
            timestamp,
            device_id,
            sequence,
        }
    }
    
    /// Convert to standardized JSON format using heapless (no_std compatible)
    #[allow(dead_code)]
    pub fn to_standard_json_deprecated(&self) -> &'static str {
        "Use to_json_bytes() instead for no_std compatibility"
    }
    
    /// Convert to legacy simple format for backward compatibility
    pub fn to_legacy_simple(&self) -> &'static str {
        "ping"
    }
}

/// Standard device status message format - MANDATORY for all applications
/// 
/// JSON Output Format:
/// ```json
/// {
///   "status": "online",
///   "uptime_seconds": 1234,
///   "free_heap_bytes": 32768,
///   "wifi_rssi_dbm": -45,
///   "sensor_readings": 42,
///   "timestamp": 1234567890,
///   "device_id": "esp32-c3-nodeps",
///   "architecture": "sync"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDeviceStatus {
    /// Device status: "online", "degraded", "error"
    pub status: &'static str,
    /// System uptime in seconds
    pub uptime_seconds: u32,
    /// Available heap memory in bytes
    pub free_heap_bytes: u32,
    /// WiFi signal strength in dBm
    pub wifi_rssi_dbm: i8,
    /// Total sensor readings completed
    pub sensor_readings: u32,
    /// Timestamp in milliseconds since boot
    pub timestamp: u64,
    /// Device identifier distinguishing applications
    pub device_id: &'static str,
    /// Architecture type for debugging
    pub architecture: &'static str,
}

impl StandardDeviceStatus {
    /// Create new standardized device status
    pub fn new(
        status: &'static str,
        uptime_seconds: u32,
        free_heap_bytes: u32,
        wifi_rssi_dbm: i8,
        sensor_readings: u32,
        timestamp: u64,
        device_id: &'static str,
        architecture: &'static str,
    ) -> Self {
        Self {
            status,
            uptime_seconds,
            free_heap_bytes,
            wifi_rssi_dbm,
            sensor_readings,
            timestamp,
            device_id,
            architecture,
        }
    }
    
    /// Convert to standardized JSON format using heapless (no_std compatible)
    #[allow(dead_code)]
    pub fn to_standard_json_deprecated(&self) -> &'static str {
        "Use to_json_bytes() instead for no_std compatibility"
    }
}

/// Device architecture identification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IoTArchitecture {
    /// Synchronous blocking architecture (main-nodeps)
    Synchronous,
    /// Asynchronous minimal architecture (main-min) 
    AsyncMinimal,
    /// Asynchronous full-featured architecture (main-app)
    AsyncFull,
}

impl IoTArchitecture {
    /// Get device ID for this architecture
    pub const fn get_device_id(self) -> &'static str {
        match self {
            IoTArchitecture::Synchronous => "esp32-c3-nodeps",
            IoTArchitecture::AsyncMinimal => "esp32-c3-min",
            IoTArchitecture::AsyncFull => "esp32-c3-full",
        }
    }
    
    /// Get architecture string for status messages
    pub const fn get_architecture_string(self) -> &'static str {
        match self {
            IoTArchitecture::Synchronous => "sync",
            IoTArchitecture::AsyncMinimal => "async-min",
            IoTArchitecture::AsyncFull => "async-full",
        }
    }
    
    /// Get MQTT topic suffix for this architecture
    pub const fn get_topic_suffix(self) -> &'static str {
        match self {
            IoTArchitecture::Synchronous => "sync",
            IoTArchitecture::AsyncMinimal => "async",
            IoTArchitecture::AsyncFull => "full",
        }
    }
}

/// Standard MQTT topic hierarchy
pub struct StandardTopics {
    /// Base topic prefix (esp32c3)
    pub prefix: &'static str,
    /// Sensor data topic
    pub sensor: &'static str,
    /// Heartbeat topic  
    pub heartbeat: &'static str,
    /// Device status topic
    pub status: &'static str,
}

impl StandardTopics {
    /// Get standard topics for an architecture
    pub fn for_architecture(arch: IoTArchitecture) -> Self {
        let _suffix = arch.get_topic_suffix();
        Self {
            prefix: "esp32c3",
            sensor: match arch {
                IoTArchitecture::Synchronous => "esp32c3/sensor/bme280/sync",
                IoTArchitecture::AsyncMinimal => "esp32c3/sensor/bme280/async", 
                IoTArchitecture::AsyncFull => "esp32c3/sensor/bme280/full",
            },
            heartbeat: match arch {
                IoTArchitecture::Synchronous => "esp32c3/heartbeat/sync",
                IoTArchitecture::AsyncMinimal => "esp32c3/heartbeat/async",
                IoTArchitecture::AsyncFull => "esp32c3/heartbeat/full", 
            },
            status: match arch {
                IoTArchitecture::Synchronous => "esp32c3/status/sync",
                IoTArchitecture::AsyncMinimal => "esp32c3/status/async",
                IoTArchitecture::AsyncFull => "esp32c3/status/full",
            },
        }
    }
}

/// Timestamp provider trait for different architectures
pub trait TimestampProvider {
    /// Get current timestamp in milliseconds since boot
    fn get_timestamp_ms(&self) -> u64;
}

/// Synchronous timestamp provider for main-nodeps
pub struct SyncTimestampProvider {
    loop_count: u32,
    loop_delay_ms: u32,
}

impl SyncTimestampProvider {
    /// Create a new synchronous timestamp provider with specified loop delay
    pub fn new(loop_delay_ms: u32) -> Self {
        Self {
            loop_count: 0,
            loop_delay_ms,
        }
    }
    
    /// Increment the loop counter for timestamp calculation
    pub fn increment_loop(&mut self) {
        self.loop_count = self.loop_count.wrapping_add(1);
    }
}

impl TimestampProvider for SyncTimestampProvider {
    fn get_timestamp_ms(&self) -> u64 {
        (self.loop_count as u64) * (self.loop_delay_ms as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_standard_sensor_reading() {
        let reading = StandardSensorReading::new(
            22.45, 65.2, 1013.25, 1234567890, "esp32-c3-test", 123
        );
        
        let json = reading.to_standard_json().unwrap();
        assert!(json.contains("\"temperature\":22.45"));
        assert!(json.contains("\"humidity\":65.2"));
        assert!(json.contains("\"pressure\":1013.25"));
        assert!(json.contains("\"timestamp\":1234567890"));
        assert!(json.contains("\"device_id\":\"esp32-c3-test\""));
        assert!(json.contains("\"reading_count\":123"));
    }
    
    #[test]
    fn test_legacy_format_compatibility() {
        let reading = StandardSensorReading::new(
            22.45, 65.2, 1013.25, 1234567890, "esp32-c3-test", 123
        );
        
        let nodeps_legacy = reading.to_legacy_json(LegacyFormat::MainNodeps);
        assert_eq!(
            nodeps_legacy,
            "{\"temperature\":22.45,\"pressure\":1013.25,\"humidity\":65.2,\"reading\":123}"
        );
        
        let min_app_legacy = reading.to_legacy_json(LegacyFormat::MainMinApp);
        assert_eq!(
            min_app_legacy,
            "{\"temperature\":22.45,\"humidity\":65.2,\"pressure\":1013.25}"
        );
    }
    
    #[test]
    fn test_architecture_identification() {
        assert_eq!(IoTArchitecture::Synchronous.get_device_id(), "esp32-c3-nodeps");
        assert_eq!(IoTArchitecture::AsyncMinimal.get_device_id(), "esp32-c3-min");
        assert_eq!(IoTArchitecture::AsyncFull.get_device_id(), "esp32-c3-full");
        
        assert_eq!(IoTArchitecture::Synchronous.get_architecture_string(), "sync");
        assert_eq!(IoTArchitecture::AsyncMinimal.get_architecture_string(), "async-min");
        assert_eq!(IoTArchitecture::AsyncFull.get_architecture_string(), "async-full");
    }
    
    #[test]
    fn test_standard_topics() {
        let topics = StandardTopics::for_architecture(IoTArchitecture::Synchronous);
        assert_eq!(topics.sensor, "esp32c3/sensor/bme280/sync");
        assert_eq!(topics.heartbeat, "esp32c3/heartbeat/sync");
        assert_eq!(topics.status, "esp32c3/status/sync");
    }
    
    #[test]
    fn test_timestamp_provider() {
        let mut provider = SyncTimestampProvider::new(50);
        assert_eq!(provider.get_timestamp_ms(), 0);
        
        provider.increment_loop();
        assert_eq!(provider.get_timestamp_ms(), 50);
        
        provider.increment_loop();
        assert_eq!(provider.get_timestamp_ms(), 100);
    }
}