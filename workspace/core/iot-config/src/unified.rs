//! Unified Configuration Structure
//! 
//! Central configuration system that replaces scattered configurations across the codebase.

extern crate alloc;
#[allow(unused_imports)]
use alloc::{string::String, format};
#[allow(unused_imports)]
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use heapless::String as HeaplessString;
#[allow(unused_imports)]
use iot_common::{IoTResult, IoTError};

use crate::{ConfigResult, ConfigError, create_bounded_string};

/// Main system configuration containing all subsystem settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTSystemConfig {
    pub wifi: WiFiConfig,
    pub mqtt: MqttConfig,
    pub sensor: SensorConfig,
    pub console: ConsoleConfig,
    pub system: SystemConfig,
    pub storage: StorageConfig,
    pub features: FeatureFlags,
    pub hardware: HardwareConfig,
}

/// WiFi network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiConfig {
    pub ssid: HeaplessString<32>,
    pub password: HeaplessString<64>,
    pub timeout_seconds: u32,
    pub retry_attempts: u8,
    pub auto_reconnect: bool,
}

/// MQTT broker and topic configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_ip: HeaplessString<16>,
    pub broker_port: u16,
    pub client_id: HeaplessString<32>,
    pub topic_prefix: HeaplessString<32>,
    pub sensor_topic: HeaplessString<64>,
    pub status_topic: HeaplessString<64>,
    pub heartbeat_topic: HeaplessString<64>,
    pub sensor_interval_secs: u16,
    pub heartbeat_interval_secs: u16,
    pub status_interval_secs: u16,
}

/// Sensor configuration (primarily BME280)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub i2c_address: u8,
    pub reading_interval_secs: u32,
    pub calibration_enabled: bool,
    pub temperature_offset: f32,
    pub humidity_offset: f32,
    pub pressure_offset: f32,
}

/// Console configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    pub enabled: bool,
    pub baud_rate: u32,
    pub command_timeout_ms: u32,
    pub history_size: u8,
    pub prompt: HeaplessString<16>,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub performance_monitoring: bool,
    pub status_led_enabled: bool,
    pub debug_output: bool,
    pub heap_size: usize,
    pub task_stack_size: usize,
    pub watchdog_timeout_secs: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub flash_offset: u32,
    pub backup_enabled: bool,
    pub wear_leveling: bool,
    pub compression: bool,
}

/// Hardware pin and peripheral configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub i2c_sda_pin: u8,
    pub i2c_scl_pin: u8,
    pub status_led_pin: u8,
    pub i2c_frequency_hz: u32,
}

/// Feature flags for compile-time optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub wifi_enabled: bool,
    pub mqtt_enabled: bool,
    pub console_enabled: bool,
    pub performance_enabled: bool,
    pub container_enabled: bool,
    pub storage_enabled: bool,
}

impl Default for IoTSystemConfig {
    fn default() -> Self {
        Self {
            wifi: WiFiConfig::default(),
            mqtt: MqttConfig::default(),
            sensor: SensorConfig::default(),
            console: ConsoleConfig::default(),
            system: SystemConfig::default(),
            storage: StorageConfig::default(),
            features: FeatureFlags::default(),
            hardware: HardwareConfig::default(),
        }
    }
}

impl Default for WiFiConfig {
    fn default() -> Self {
        Self {
            ssid: HeaplessString::new(),
            password: HeaplessString::new(),
            timeout_seconds: 10,
            retry_attempts: 3,
            auto_reconnect: true,
        }
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_ip: create_bounded_string("192.168.1.100", "broker_ip").unwrap_or_default(),
            broker_port: 1883,
            client_id: create_bounded_string("esp32c3-iot", "client_id").unwrap_or_default(),
            topic_prefix: create_bounded_string("esp32", "topic_prefix").unwrap_or_default(),
            sensor_topic: create_bounded_string("esp32/sensor/bme280", "sensor_topic").unwrap_or_default(),
            status_topic: create_bounded_string("esp32/status", "status_topic").unwrap_or_default(),
            heartbeat_topic: create_bounded_string("esp32/heartbeat", "heartbeat_topic").unwrap_or_default(),
            sensor_interval_secs: 30,
            heartbeat_interval_secs: 60,
            status_interval_secs: 120,
        }
    }
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            i2c_address: 0x76, // BME280 primary address
            reading_interval_secs: 30,
            calibration_enabled: true,
            temperature_offset: 0.0,
            humidity_offset: 0.0,
            pressure_offset: 0.0,
        }
    }
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            baud_rate: 115200,
            command_timeout_ms: 5000,
            history_size: 10,
            prompt: create_bounded_string("esp32> ", "prompt").unwrap_or_default(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            performance_monitoring: true,
            status_led_enabled: true,
            debug_output: cfg!(debug_assertions),
            heap_size: 64 * 1024, // 64KB heap
            task_stack_size: 4096,
            watchdog_timeout_secs: 30,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            flash_offset: 0x310000, // 3MB offset in flash
            backup_enabled: true,
            wear_leveling: true,
            compression: false,
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            i2c_sda_pin: 8,
            i2c_scl_pin: 9,
            status_led_pin: 3,
            i2c_frequency_hz: 100_000,
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            wifi_enabled: cfg!(feature = "wifi"),
            mqtt_enabled: cfg!(feature = "mqtt"),
            console_enabled: cfg!(feature = "console"),
            performance_enabled: cfg!(feature = "performance"),
            container_enabled: cfg!(feature = "container"),
            storage_enabled: cfg!(feature = "storage"),
        }
    }
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: IoTSystemConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: IoTSystemConfig::default(),
        }
    }

    pub fn wifi_credentials(mut self, ssid: &str, password: &str) -> ConfigResult<Self> {
        self.config.wifi.ssid = create_bounded_string(ssid, "WiFi SSID")?;
        self.config.wifi.password = create_bounded_string(password, "WiFi password")?;
        Ok(self)
    }

    pub fn mqtt_broker(mut self, ip: &str, port: u16) -> ConfigResult<Self> {
        self.config.mqtt.broker_ip = create_bounded_string(ip, "MQTT broker IP")?;
        self.config.mqtt.broker_port = port;
        Ok(self)
    }

    pub fn mqtt_client_id(mut self, client_id: &str) -> ConfigResult<Self> {
        self.config.mqtt.client_id = create_bounded_string(client_id, "MQTT client ID")?;
        Ok(self)
    }

    pub fn sensor_i2c_address(mut self, address: u8) -> Self {
        self.config.sensor.i2c_address = address;
        self
    }

    pub fn build(self) -> IoTSystemConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment variable loading with fallbacks
impl IoTSystemConfig {
    /// Load configuration from environment variables with defaults
    /// Note: This is primarily for compile-time environment loading via env!() macro
    pub fn from_environment() -> ConfigResult<Self> {
        let config = Self::default();

        // Note: Environment loading is limited in no_std
        // For runtime environment loading, use external tools or pre-processing
        
        Ok(config)
    }

    /// Serialize configuration to JSON bytes
    pub fn to_json_bytes(&self) -> ConfigResult<heapless::Vec<u8, 2048>> {
        serde_json_core::to_vec(self)
            .map_err(|_| ConfigError::SerializationError("Failed to serialize config to JSON".into()))
    }

    /// Deserialize configuration from JSON bytes
    pub fn from_json_bytes(data: &[u8]) -> ConfigResult<Self> {
        serde_json_core::from_slice(data)
            .map_err(|_| ConfigError::SerializationError("Failed to deserialize config from JSON".into()))
            .map(|(config, _)| config)
    }

    /// Get embedded default configuration for minimal systems
    pub fn default_embedded() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IoTSystemConfig::default();
        assert_eq!(config.mqtt.broker_port, 1883);
        assert_eq!(config.sensor.i2c_address, 0x76);
        assert!(config.system.performance_monitoring);
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .wifi_credentials("test_ssid", "test_password")
            .unwrap()
            .mqtt_broker("192.168.1.200", 1884)
            .unwrap()
            .sensor_i2c_address(0x77)
            .build();

        assert_eq!(config.wifi.ssid.as_str(), "test_ssid");
        assert_eq!(config.mqtt.broker_port, 1884);
        assert_eq!(config.sensor.i2c_address, 0x77);
    }

    #[test]
    fn test_json_serialization() {
        let config = IoTSystemConfig::default();
        let json_bytes = config.to_json_bytes().unwrap();
        let deserialized = IoTSystemConfig::from_json_bytes(&json_bytes).unwrap();
        
        assert_eq!(config.mqtt.broker_port, deserialized.mqtt.broker_port);
    }
}