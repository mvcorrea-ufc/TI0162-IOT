//! Standardized Configuration Management
//! 
//! This module provides unified configuration structures that work across
//! all ESP32-C3 IoT applications while maintaining their architectural differences.

use crate::standard_messages::IoTArchitecture;
use crate::standard_timing::StandardTimingConfig;

/// Unified IoT system configuration that works across all architectures
#[derive(Debug, Clone)]
pub struct StandardIoTConfig {
    /// Device identification
    pub device_id: &'static str,
    
    /// Architecture type (sync, async-min, async-full)
    pub architecture: IoTArchitecture,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// MQTT broker configuration
    pub mqtt: MqttConfig,
    
    /// Timing configuration
    pub timing: StandardTimingConfig,
    
    /// Feature flags for conditional compilation
    pub features: FeatureFlags,
    
    /// Hardware pin configuration
    pub hardware: HardwareConfig,
    
    /// System-level configuration
    pub system: SystemConfig,
}

/// Network configuration (WiFi)
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// WiFi network SSID
    pub wifi_ssid: &'static str,
    /// WiFi network password
    pub wifi_password: &'static str,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u32,
    /// Number of retry attempts
    pub retry_attempts: u8,
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
}

/// MQTT broker and messaging configuration
#[derive(Debug, Clone)]
pub struct MqttConfig {
    /// MQTT broker IP address
    pub broker_ip: [u8; 4],
    /// MQTT broker port
    pub broker_port: u16,
    /// MQTT client identifier
    pub client_id: &'static str,
    /// MQTT topic prefix
    pub topic_prefix: &'static str,
    /// MQTT Quality of Service level
    pub qos: u8,
    /// MQTT keep alive interval in seconds
    pub keep_alive_secs: u16,
}

/// Feature flags for different application capabilities
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    /// Console command interface (main-app only)
    pub console_enabled: bool,
    
    /// Performance monitoring and analysis (main-app only)
    pub performance_monitoring_enabled: bool,
    
    /// Status LED control (main-app only)
    pub status_led_enabled: bool,
    
    /// Runtime configuration changes (main-app only)
    pub runtime_config_enabled: bool,
    
    /// WiFi connectivity (can be disabled for testing)
    pub wifi_enabled: bool,
    
    /// MQTT publishing (can be disabled for testing)
    pub mqtt_enabled: bool,
}

/// Hardware pin and peripheral configuration
#[derive(Debug, Clone)]
pub struct HardwareConfig {
    /// I2C SDA pin for BME280 sensor
    pub i2c_sda_pin: u8,
    
    /// I2C SCL pin for BME280 sensor
    pub i2c_scl_pin: u8,
    
    /// Status LED pin (if available)
    pub status_led_pin: u8,
    
    /// I2C frequency in Hz
    pub i2c_frequency_hz: u32,
    
    /// BME280 I2C address
    pub bme280_address: u8,
}

/// System-level configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// Heap size for memory allocation
    pub heap_size_bytes: usize,
    
    /// Enable debug output via RTT
    pub debug_output: bool,
    
    /// Watchdog timeout (if enabled)
    pub watchdog_timeout_secs: u32,
    
    /// Task stack size for async applications
    pub task_stack_size_bytes: usize,
}

impl StandardIoTConfig {
    /// Create configuration for main-nodeps (synchronous)
    pub fn for_main_nodeps() -> Self {
        Self {
            device_id: IoTArchitecture::Synchronous.get_device_id(),
            architecture: IoTArchitecture::Synchronous,
            network: NetworkConfig {
                wifi_ssid: Self::get_wifi_ssid(),
                wifi_password: Self::get_wifi_password(),
                connection_timeout_secs: 30,
                retry_attempts: 3,
                auto_reconnect: true,
            },
            mqtt: MqttConfig {
                broker_ip: [10, 10, 10, 210],
                broker_port: 1883,
                client_id: "esp32-c3-nodeps",
                topic_prefix: "esp32c3",
                qos: 0,
                keep_alive_secs: 60,
            },
            timing: StandardTimingConfig::for_architecture(IoTArchitecture::Synchronous),
            features: FeatureFlags {
                console_enabled: false,
                performance_monitoring_enabled: false,
                status_led_enabled: false,
                runtime_config_enabled: false,
                wifi_enabled: true,
                mqtt_enabled: true,
            },
            hardware: HardwareConfig::default(),
            system: SystemConfig {
                heap_size_bytes: 72 * 1024, // 72KB for WiFi operations
                debug_output: true,
                watchdog_timeout_secs: 30,
                task_stack_size_bytes: 4096,
            },
        }
    }
    
    /// Create configuration for main-min (async minimal)
    pub fn for_main_min() -> Self {
        Self {
            device_id: IoTArchitecture::AsyncMinimal.get_device_id(),
            architecture: IoTArchitecture::AsyncMinimal,
            network: NetworkConfig {
                wifi_ssid: Self::get_wifi_ssid(),
                wifi_password: Self::get_wifi_password(),
                connection_timeout_secs: 30,
                retry_attempts: 3,
                auto_reconnect: true,
            },
            mqtt: MqttConfig {
                broker_ip: [10, 10, 10, 210],
                broker_port: 1883,
                client_id: "esp32-c3-min",
                topic_prefix: "esp32c3",
                qos: 0,
                keep_alive_secs: 60,
            },
            timing: StandardTimingConfig::for_architecture(IoTArchitecture::AsyncMinimal),
            features: FeatureFlags {
                console_enabled: false,
                performance_monitoring_enabled: false,
                status_led_enabled: false,
                runtime_config_enabled: false,
                wifi_enabled: true,
                mqtt_enabled: true,
            },
            hardware: HardwareConfig::default(),
            system: SystemConfig {
                heap_size_bytes: 64 * 1024, // 64KB standard
                debug_output: true,
                watchdog_timeout_secs: 30,
                task_stack_size_bytes: 4096,
            },
        }
    }
    
    /// Create configuration for main-app (async full-featured)
    pub fn for_main_app() -> Self {
        Self {
            device_id: IoTArchitecture::AsyncFull.get_device_id(),
            architecture: IoTArchitecture::AsyncFull,
            network: NetworkConfig {
                wifi_ssid: Self::get_wifi_ssid(),
                wifi_password: Self::get_wifi_password(),
                connection_timeout_secs: 30,
                retry_attempts: 3,
                auto_reconnect: true,
            },
            mqtt: MqttConfig {
                broker_ip: [10, 10, 10, 210],
                broker_port: 1883,
                client_id: "esp32-c3-full",
                topic_prefix: "esp32c3",
                qos: 0,
                keep_alive_secs: 60,
            },
            timing: StandardTimingConfig::for_architecture(IoTArchitecture::AsyncFull),
            features: FeatureFlags {
                console_enabled: true,
                performance_monitoring_enabled: true,
                status_led_enabled: true,
                runtime_config_enabled: true,
                wifi_enabled: true,
                mqtt_enabled: true,
            },
            hardware: HardwareConfig::default(),
            system: SystemConfig {
                heap_size_bytes: 64 * 1024, // 64KB standard
                debug_output: true,
                watchdog_timeout_secs: 30,
                task_stack_size_bytes: 4096,
            },
        }
    }
    
    /// Get WiFi SSID from environment or default
    fn get_wifi_ssid() -> &'static str {
        // Try environment variable first, fall back to hardcoded
        option_env!("WIFI_SSID").unwrap_or("FamiliaFeliz-2Ghz")
    }
    
    /// Get WiFi password from environment or default
    fn get_wifi_password() -> &'static str {
        // Try environment variable first, fall back to hardcoded
        option_env!("WIFI_PASSWORD").unwrap_or("ines#sara")
    }
    
    /// Validate configuration for consistency and constraints
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate network configuration
        if self.network.wifi_ssid.is_empty() {
            return Err(ConfigError::InvalidValue("WiFi SSID cannot be empty"));
        }
        
        if self.network.wifi_password.is_empty() {
            return Err(ConfigError::InvalidValue("WiFi password cannot be empty"));
        }
        
        if self.network.connection_timeout_secs == 0 {
            return Err(ConfigError::InvalidValue("Connection timeout must be > 0"));
        }
        
        // Validate MQTT configuration
        if self.mqtt.broker_port == 0 {
            return Err(ConfigError::InvalidValue("MQTT broker port must be > 0"));
        }
        
        if self.mqtt.client_id.is_empty() {
            return Err(ConfigError::InvalidValue("MQTT client ID cannot be empty"));
        }
        
        if self.mqtt.qos > 2 {
            return Err(ConfigError::InvalidValue("MQTT QoS must be 0, 1, or 2"));
        }
        
        // Validate timing configuration
        self.timing.validate()
            .map_err(ConfigError::TimingError)?;
        
        // Validate system configuration
        if self.system.heap_size_bytes < 32 * 1024 {
            return Err(ConfigError::InvalidValue("Heap size must be >= 32KB"));
        }
        
        if self.system.heap_size_bytes > 128 * 1024 {
            return Err(ConfigError::InvalidValue("Heap size must be <= 128KB"));
        }
        
        // Validate feature consistency
        if self.features.console_enabled && self.architecture != IoTArchitecture::AsyncFull {
            return Err(ConfigError::FeatureConflict("Console only available in main-app"));
        }
        
        if self.features.performance_monitoring_enabled && self.architecture != IoTArchitecture::AsyncFull {
            return Err(ConfigError::FeatureConflict("Performance monitoring only available in main-app"));
        }
        
        Ok(())
    }
    
    /// Get MQTT topics for this configuration
    pub fn get_mqtt_topics(&self) -> crate::standard_messages::StandardTopics {
        crate::standard_messages::StandardTopics::for_architecture(self.architecture)
    }
    
    /// Check if feature is enabled
    pub fn is_feature_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::Console => self.features.console_enabled,
            Feature::PerformanceMonitoring => self.features.performance_monitoring_enabled,
            Feature::StatusLed => self.features.status_led_enabled,
            Feature::RuntimeConfig => self.features.runtime_config_enabled,
            Feature::WiFi => self.features.wifi_enabled,
            Feature::Mqtt => self.features.mqtt_enabled,
        }
    }
    
    /// Create configuration with custom overrides
    pub fn with_overrides(mut self, overrides: ConfigOverrides) -> Self {
        if let Some(heap_size) = overrides.heap_size_bytes {
            self.system.heap_size_bytes = heap_size;
        }
        
        if let Some(debug) = overrides.debug_output {
            self.system.debug_output = debug;
        }
        
        if let Some(wifi_enabled) = overrides.wifi_enabled {
            self.features.wifi_enabled = wifi_enabled;
        }
        
        if let Some(mqtt_enabled) = overrides.mqtt_enabled {
            self.features.mqtt_enabled = mqtt_enabled;
        }
        
        self
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            i2c_sda_pin: 8,         // GPIO8
            i2c_scl_pin: 9,         // GPIO9
            status_led_pin: 3,      // GPIO3
            i2c_frequency_hz: 100_000, // 100kHz
            bme280_address: 0x76,   // BME280 primary address
        }
    }
}

/// Configuration override options for testing and customization
#[derive(Debug, Default)]
pub struct ConfigOverrides {
    /// Override heap size in bytes
    pub heap_size_bytes: Option<usize>,
    /// Override debug output setting
    pub debug_output: Option<bool>,
    /// Override WiFi enable setting
    pub wifi_enabled: Option<bool>,
    /// Override MQTT enable setting
    pub mqtt_enabled: Option<bool>,
}

/// Available features that can be enabled/disabled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// Serial console feature
    Console,
    /// Performance monitoring feature
    PerformanceMonitoring,
    /// Status LED feature
    StatusLed,
    /// Runtime configuration feature
    RuntimeConfig,
    /// WiFi connectivity feature
    WiFi,
    /// MQTT messaging feature
    Mqtt,
}

/// Configuration validation errors
#[derive(Debug)]
pub enum ConfigError {
    /// Invalid configuration value error
    InvalidValue(&'static str),
    /// Feature conflict error
    FeatureConflict(&'static str),
    /// Timing configuration error
    TimingError(&'static str),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
            ConfigError::FeatureConflict(msg) => write!(f, "Feature conflict: {}", msg),
            ConfigError::TimingError(msg) => write!(f, "Timing configuration error: {}", msg),
        }
    }
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    architecture: IoTArchitecture,
    overrides: ConfigOverrides,
}

impl ConfigBuilder {
    /// Create new configuration builder
    pub fn new(architecture: IoTArchitecture) -> Self {
        Self {
            architecture,
            overrides: ConfigOverrides::default(),
        }
    }
    
    /// Set custom heap size
    pub fn with_heap_size(mut self, size_bytes: usize) -> Self {
        self.overrides.heap_size_bytes = Some(size_bytes);
        self
    }
    
    /// Enable/disable debug output
    pub fn with_debug_output(mut self, enabled: bool) -> Self {
        self.overrides.debug_output = Some(enabled);
        self
    }
    
    /// Enable/disable WiFi
    pub fn with_wifi(mut self, enabled: bool) -> Self {
        self.overrides.wifi_enabled = Some(enabled);
        self
    }
    
    /// Enable/disable MQTT
    pub fn with_mqtt(mut self, enabled: bool) -> Self {
        self.overrides.mqtt_enabled = Some(enabled);
        self
    }
    
    /// Build final configuration
    pub fn build(self) -> StandardIoTConfig {
        let base_config = match self.architecture {
            IoTArchitecture::Synchronous => StandardIoTConfig::for_main_nodeps(),
            IoTArchitecture::AsyncMinimal => StandardIoTConfig::for_main_min(),
            IoTArchitecture::AsyncFull => StandardIoTConfig::for_main_app(),
        };
        
        base_config.with_overrides(self.overrides)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_creation() {
        let nodeps_config = StandardIoTConfig::for_main_nodeps();
        assert_eq!(nodeps_config.architecture, IoTArchitecture::Synchronous);
        assert!(!nodeps_config.features.console_enabled);
        assert_eq!(nodeps_config.device_id, "esp32-c3-nodeps");
        
        let app_config = StandardIoTConfig::for_main_app();
        assert_eq!(app_config.architecture, IoTArchitecture::AsyncFull);
        assert!(app_config.features.console_enabled);
        assert_eq!(app_config.device_id, "esp32-c3-full");
    }
    
    #[test]
    fn test_config_validation() {
        let valid_config = StandardIoTConfig::for_main_nodeps();
        assert!(valid_config.validate().is_ok());
        
        let mut invalid_config = valid_config.clone();
        invalid_config.mqtt.broker_port = 0;
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_feature_consistency() {
        let min_config = StandardIoTConfig::for_main_min();
        assert!(!min_config.is_feature_enabled(Feature::Console));
        assert!(!min_config.is_feature_enabled(Feature::PerformanceMonitoring));
        
        let app_config = StandardIoTConfig::for_main_app();
        assert!(app_config.is_feature_enabled(Feature::Console));
        assert!(app_config.is_feature_enabled(Feature::PerformanceMonitoring));
    }
    
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new(IoTArchitecture::AsyncMinimal)
            .with_heap_size(48 * 1024)
            .with_debug_output(false)
            .with_wifi(false)
            .build();
        
        assert_eq!(config.system.heap_size_bytes, 48 * 1024);
        assert!(!config.system.debug_output);
        assert!(!config.features.wifi_enabled);
    }
    
    #[test]
    fn test_mqtt_topics() {
        let config = StandardIoTConfig::for_main_nodeps();
        let topics = config.get_mqtt_topics();
        assert_eq!(topics.sensor, "esp32c3/sensor/bme280/sync");
        assert_eq!(topics.heartbeat, "esp32c3/heartbeat/sync");
    }
}