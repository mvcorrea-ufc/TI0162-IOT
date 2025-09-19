//! # System Configuration for IoT Container
//!
//! This module provides configuration management for the ESP32-C3 IoT system.
//! Configuration is loaded from environment variables and provides sensible defaults
//! for embedded operation.

use serde::{Deserialize, Serialize};
use heapless::String;
use iot_common::{IoTError, ConfigError};

/// Maximum length for configuration strings
pub const MAX_CONFIG_STRING_LEN: usize = 64;

/// Maximum length for device identifiers
pub const MAX_DEVICE_ID_LEN: usize = 32;

/// Type alias for configuration strings
pub type ConfigString = String<MAX_CONFIG_STRING_LEN>;

/// Type alias for device identifiers
pub type DeviceId = String<MAX_DEVICE_ID_LEN>;

/// System operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperatingMode {
    /// Development mode - relaxed error handling, verbose logging
    Development,
    
    /// Testing mode - enhanced validation, deterministic behavior
    Testing,
    
    /// Production mode - strict error handling, optimized performance
    Production,
}

impl Default for OperatingMode {
    fn default() -> Self {
        Self::Development
    }
}

/// System logging level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum LogLevel {
    /// No logging output
    None = 0,
    
    /// Only critical errors
    Error = 1,
    
    /// Warnings and errors
    Warning = 2,
    
    /// General information, warnings, and errors
    Info = 3,
    
    /// Detailed debug information
    Debug = 4,
}

impl LogLevel {
    /// Returns the log level as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::None => "NONE",
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Sensor configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    /// Type of sensor (e.g., "BME280", "SHT30")
    pub sensor_type: ConfigString,
    
    /// I2C address (0x76 or 0x77 for BME280)
    pub i2c_address: u8,
    
    /// Measurement oversampling factor (1x, 2x, 4x, 8x, 16x)
    pub oversampling: u8,
    
    /// IIR filter coefficient (0=off, 1-4=enabled)
    pub filter_coefficient: u8,
    
    /// Measurement timeout in milliseconds
    pub measurement_timeout_ms: u32,
    
    /// Enable sensor self-test on initialization
    pub enable_self_test: bool,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            sensor_type: ConfigString::try_from("BME280").unwrap(),
            i2c_address: 0x76,
            oversampling: 1,
            filter_coefficient: 0,
            measurement_timeout_ms: 1000,
            enable_self_test: true,
        }
    }
}

/// WiFi configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WiFiConfig {
    /// Network SSID
    pub ssid: ConfigString,
    
    /// Network password
    pub password: ConfigString,
    
    /// Connection timeout in seconds
    pub connection_timeout_secs: u32,
    
    /// Maximum connection retry attempts
    pub max_retry_attempts: u32,
    
    /// Retry delay between attempts in seconds
    pub retry_delay_secs: u32,
    
    /// Enable WiFi power saving mode
    pub power_save_mode: bool,
    
    /// Minimum acceptable signal strength in dBm
    pub min_signal_strength_dbm: i8,
}

impl Default for WiFiConfig {
    fn default() -> Self {
        Self {
            ssid: ConfigString::try_from("IoT_Network").unwrap(),
            password: ConfigString::try_from("password123").unwrap(),
            connection_timeout_secs: 30,
            max_retry_attempts: 5,
            retry_delay_secs: 5,
            power_save_mode: false,
            min_signal_strength_dbm: -80,
        }
    }
}

/// MQTT configuration  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MqttConfig {
    /// MQTT broker hostname or IP address
    pub broker_host: ConfigString,
    
    /// MQTT broker port (typically 1883 for non-TLS, 8883 for TLS)
    pub broker_port: u16,
    
    /// Client identifier
    pub client_id: ConfigString,
    
    /// Username for authentication (optional)
    pub username: Option<ConfigString>,
    
    /// Password for authentication (optional)
    pub password: Option<ConfigString>,
    
    /// Topic prefix for all published messages
    pub topic_prefix: ConfigString,
    
    /// Quality of Service level (0, 1, or 2)
    pub qos_level: u8,
    
    /// Retain messages flag
    pub retain_messages: bool,
    
    /// Keep-alive interval in seconds
    pub keep_alive_secs: u16,
    
    /// Connection timeout in seconds
    pub connection_timeout_secs: u32,
    
    /// Maximum retry attempts for failed operations
    pub max_retry_attempts: u32,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_host: ConfigString::try_from("mqtt.broker.local").unwrap(),
            broker_port: 1883,
            client_id: ConfigString::try_from("esp32c3_iot_device").unwrap(),
            username: None,
            password: None,
            topic_prefix: ConfigString::try_from("iot/esp32c3").unwrap(),
            qos_level: 1,
            retain_messages: false,
            keep_alive_secs: 60,
            connection_timeout_secs: 30,
            max_retry_attempts: 3,
        }
    }
}

/// Console configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsoleConfig {
    /// Console interface type ("UART", "USB", "Network")
    pub interface_type: ConfigString,
    
    /// Baud rate for UART interfaces
    pub baud_rate: u32,
    
    /// Input buffer size in bytes
    pub input_buffer_size: usize,
    
    /// Output buffer size in bytes
    pub output_buffer_size: usize,
    
    /// Command timeout in seconds
    pub command_timeout_secs: u32,
    
    /// Enable command history
    pub enable_history: bool,
    
    /// Maximum command history entries
    pub max_history_entries: usize,
    
    /// Enable command echo
    pub enable_echo: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            interface_type: ConfigString::try_from("USB").unwrap(),
            baud_rate: 115200,
            input_buffer_size: 128,
            output_buffer_size: 512,
            command_timeout_secs: 30,
            enable_history: true,
            max_history_entries: 10,
            enable_echo: true,
        }
    }
}

/// Complete system configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemConfiguration {
    /// Unique device identifier
    pub device_id: DeviceId,
    
    /// System operating mode
    pub operation_mode: OperatingMode,
    
    /// System logging level
    pub log_level: LogLevel,
    
    /// Sensor reading interval in seconds
    pub sensor_read_interval_secs: u64,
    
    /// Status reporting interval in seconds
    pub status_report_interval_secs: u64,
    
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    
    /// Maximum system memory usage in bytes
    pub max_memory_usage_bytes: u32,
    
    /// Enable system watchdog
    pub enable_watchdog: bool,
    
    /// Watchdog timeout in seconds
    pub watchdog_timeout_secs: u32,
    
    /// Sensor configuration
    pub sensor: SensorConfig,
    
    /// WiFi configuration
    pub wifi: WiFiConfig,
    
    /// MQTT configuration
    pub mqtt: MqttConfig,
    
    /// Console configuration
    pub console: ConsoleConfig,
}

impl Default for SystemConfiguration {
    fn default() -> Self {
        Self {
            device_id: DeviceId::try_from("esp32c3_iot_001").unwrap(),
            operation_mode: OperatingMode::Development,
            log_level: LogLevel::Info,
            sensor_read_interval_secs: 30,
            status_report_interval_secs: 300, // 5 minutes
            heartbeat_interval_secs: 60,
            max_memory_usage_bytes: 64 * 1024, // 64KB
            enable_watchdog: true,
            watchdog_timeout_secs: 60,
            sensor: SensorConfig::default(),
            wifi: WiFiConfig::default(),
            mqtt: MqttConfig::default(),
            console: ConsoleConfig::default(),
        }
    }
}

impl SystemConfiguration {
    /// Creates configuration from environment variables
    /// 
    /// Loads configuration values from environment variables with fallback to defaults.
    /// Environment variables follow the pattern: IOT_<SECTION>_<PARAMETER>
    /// 
    /// # Examples
    /// 
    /// Environment variables:
    /// - `IOT_DEVICE_ID=esp32c3_sensor_01`
    /// - `IOT_WIFI_SSID=MyNetwork`
    /// - `IOT_WIFI_PASSWORD=SecretPassword`
    /// - `IOT_MQTT_BROKER_HOST=192.168.1.100`
    /// - `IOT_MQTT_BROKER_PORT=1883`
    /// 
    /// # Returns
    /// 
    /// * `Ok(SystemConfiguration)` - Configuration loaded successfully
    /// * `Err(IoTError)` - Configuration loading failed
    pub fn from_env() -> Result<Self, IoTError> {
        let mut config = Self::default();
        
        // Load device configuration
        if let Ok(device_id) = core::env!("IOT_DEVICE_ID", "esp32c3_iot_001") {
            config.device_id = DeviceId::try_from(device_id).map_err(|_| {
                IoTError::Configuration(ConfigError::InvalidFormat("Device ID too long"))
            })?;
        }
        
        // Load WiFi configuration from environment
        if let Ok(ssid) = core::env!("WIFI_SSID", "IoT_Network") {
            config.wifi.ssid = ConfigString::try_from(ssid).map_err(|_| {
                IoTError::Configuration(ConfigError::InvalidFormat("WiFi SSID too long"))
            })?;
        }
        
        if let Ok(password) = core::env!("WIFI_PASSWORD", "password123") {
            config.wifi.password = ConfigString::try_from(password).map_err(|_| {
                IoTError::Configuration(ConfigError::InvalidFormat("WiFi password too long"))
            })?;
        }
        
        // Load MQTT configuration from environment
        if let Ok(broker) = core::env!("MQTT_BROKER_HOST", "mqtt.broker.local") {
            config.mqtt.broker_host = ConfigString::try_from(broker).map_err(|_| {
                IoTError::Configuration(ConfigError::InvalidFormat("MQTT broker host too long"))
            })?;
        }
        
        if let Ok(port_str) = core::env!("MQTT_BROKER_PORT", "1883") {
            config.mqtt.broker_port = port_str.parse().map_err(|_| {
                IoTError::Configuration(ConfigError::InvalidFormat("Invalid MQTT broker port"))
            })?;
        }
        
        // Load operation mode
        if let Ok(mode_str) = core::env!("IOT_OPERATION_MODE", "development") {
            config.operation_mode = match mode_str.to_lowercase().as_str() {
                "production" => OperatingMode::Production,
                "testing" => OperatingMode::Testing,
                _ => OperatingMode::Development,
            };
        }
        
        // Load log level
        if let Ok(log_str) = core::env!("IOT_LOG_LEVEL", "info") {
            config.log_level = match log_str.to_lowercase().as_str() {
                "none" => LogLevel::None,
                "error" => LogLevel::Error,
                "warning" | "warn" => LogLevel::Warning,
                "debug" => LogLevel::Debug,
                _ => LogLevel::Info,
            };
        }
        
        Ok(config)
    }
    
    /// Creates a test configuration with mock-friendly settings
    /// 
    /// This configuration is optimized for testing with faster intervals
    /// and relaxed error handling.
    /// 
    /// # Returns
    /// 
    /// Test-optimized system configuration
    pub fn test_config() -> Self {
        Self {
            device_id: DeviceId::try_from("test_device_001").unwrap(),
            operation_mode: OperatingMode::Testing,
            log_level: LogLevel::Debug,
            sensor_read_interval_secs: 1, // Fast for testing
            status_report_interval_secs: 5, // Fast for testing
            heartbeat_interval_secs: 5, // Fast for testing
            max_memory_usage_bytes: 32 * 1024, // Smaller for testing
            enable_watchdog: false, // Disabled for testing
            watchdog_timeout_secs: 10,
            sensor: SensorConfig {
                measurement_timeout_ms: 100, // Fast for testing
                enable_self_test: false, // Disabled for testing
                ..SensorConfig::default()
            },
            wifi: WiFiConfig {
                connection_timeout_secs: 5, // Fast for testing
                max_retry_attempts: 2, // Fewer retries for testing
                retry_delay_secs: 1, // Fast retry for testing
                ..WiFiConfig::default()
            },
            mqtt: MqttConfig {
                connection_timeout_secs: 5, // Fast for testing
                max_retry_attempts: 2, // Fewer retries for testing
                ..MqttConfig::default()
            },
            console: ConsoleConfig {
                command_timeout_secs: 5, // Fast for testing
                input_buffer_size: 64, // Smaller for testing
                output_buffer_size: 256, // Smaller for testing
                ..ConsoleConfig::default()
            },
        }
    }
    
    /// Validates the configuration for consistency and constraints
    /// 
    /// Checks all configuration parameters for valid ranges and logical consistency.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Configuration is valid
    /// * `Err(IoTError)` - Configuration contains errors
    pub fn validate(&self) -> Result<(), IoTError> {
        // Validate device ID
        if self.device_id.is_empty() {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Device ID cannot be empty")));
        }
        
        // Validate timing intervals
        if self.sensor_read_interval_secs == 0 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Sensor read interval must be > 0")));
        }
        
        if self.status_report_interval_secs == 0 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Status report interval must be > 0")));
        }
        
        // Validate sensor configuration
        if self.sensor.i2c_address == 0 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Invalid I2C address")));
        }
        
        if self.sensor.oversampling == 0 || self.sensor.oversampling > 16 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Oversampling must be 1-16")));
        }
        
        // Validate WiFi configuration
        if self.wifi.ssid.is_empty() {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("WiFi SSID cannot be empty")));
        }
        
        // Validate MQTT configuration
        if self.mqtt.broker_host.is_empty() {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("MQTT broker host cannot be empty")));
        }
        
        if self.mqtt.broker_port == 0 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("MQTT broker port must be > 0")));
        }
        
        if self.mqtt.qos_level > 2 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("MQTT QoS level must be 0-2")));
        }
        
        // Validate console configuration
        if self.console.input_buffer_size == 0 || self.console.output_buffer_size == 0 {
            return Err(IoTError::Configuration(ConfigError::InvalidFormat("Console buffer sizes must be > 0")));
        }
        
        Ok(())
    }
    
    /// Updates configuration from a partial configuration
    /// 
    /// Allows selective updating of configuration parameters while maintaining
    /// existing values for unspecified parameters.
    /// 
    /// # Arguments
    /// 
    /// * `partial` - Partial configuration with only changed values
    /// 
    /// # Example
    /// 
    /// ```rust
    /// let mut config = SystemConfiguration::default();
    /// let partial = PartialSystemConfiguration {
    ///     log_level: Some(LogLevel::Debug),
    ///     wifi_ssid: Some("NewNetwork".into()),
    ///     ..Default::default()
    /// };
    /// config.update_from_partial(partial);
    /// ```
    pub fn update_from_partial(&mut self, partial: PartialSystemConfiguration) {
        if let Some(device_id) = partial.device_id {
            self.device_id = device_id;
        }
        if let Some(operation_mode) = partial.operation_mode {
            self.operation_mode = operation_mode;
        }
        if let Some(log_level) = partial.log_level {
            self.log_level = log_level;
        }
        if let Some(interval) = partial.sensor_read_interval_secs {
            self.sensor_read_interval_secs = interval;
        }
        // Add other fields as needed...
    }
}

/// Partial system configuration for selective updates
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PartialSystemConfiguration {
    /// Device identifier update
    pub device_id: Option<DeviceId>,
    
    /// Operation mode update
    pub operation_mode: Option<OperatingMode>,
    
    /// Log level update
    pub log_level: Option<LogLevel>,
    
    /// Sensor read interval update
    pub sensor_read_interval_secs: Option<u64>,
    
    /// Status report interval update
    pub status_report_interval_secs: Option<u64>,
}