//! Ultra-minimal Configuration Module
//! Sensor-only configuration for maximum performance and minimal binary size

/// BME280 Sensor Configuration - ONLY ESSENTIAL CONSTANTS
pub const BME280_I2C_ADDRESS: u8 = 0x76;

/// System Configuration - MINIMAL SET
pub const SENSOR_INTERVAL_SECS: u32 = 30;

/// Memory Configuration - OPTIMIZED (increased for WiFi controller)
pub const HEAP_SIZE: usize = 72 * 1024; // 72KB heap REQUIRED for WiFi operations (same as working wifi-synchronous)

/// WiFi Configuration - Environment Variables
pub const WIFI_SSID: &'static str = "FamiliaFeliz-2Ghz";  // Default for development
pub const WIFI_PASSWORD: &'static str = "ines#sara";  // Default for development

/// MQTT Configuration - Production Settings
pub const MQTT_BROKER_IP: [u8; 4] = [10, 10, 10, 210];
pub const MQTT_BROKER_PORT: u16 = 1883;
pub const MQTT_CLIENT_ID: &'static str = "esp32-c3-nodeps";
pub const MQTT_TOPIC_PREFIX: &'static str = "esp32";
pub const MQTT_QOS: u8 = 0; // QoS 0 = At most once delivery

/// Minimal configuration struct for sensor-only operation
pub struct NodepsConfig;

impl NodepsConfig {
    /// Get BME280 I2C address
    pub const fn bme280_address() -> u8 {
        BME280_I2C_ADDRESS
    }
    
    /// Get sensor reading interval
    pub const fn sensor_interval_secs() -> u32 {
        SENSOR_INTERVAL_SECS
    }
    
    /// Get heap size for memory allocation
    pub const fn heap_size() -> usize {
        HEAP_SIZE
    }
    
    /// Get WiFi SSID
    pub const fn wifi_ssid() -> &'static str {
        WIFI_SSID
    }
    
    /// Get WiFi password
    pub const fn wifi_password() -> &'static str {
        WIFI_PASSWORD
    }
    
    /// Get MQTT broker IP
    pub const fn mqtt_broker_ip() -> [u8; 4] {
        MQTT_BROKER_IP
    }
    
    /// Get MQTT broker port
    pub const fn mqtt_broker_port() -> u16 {
        MQTT_BROKER_PORT
    }
    
    /// Get MQTT client ID
    pub const fn mqtt_client_id() -> &'static str {
        MQTT_CLIENT_ID
    }
    
    /// Get MQTT topic prefix
    pub const fn mqtt_topic_prefix() -> &'static str {
        MQTT_TOPIC_PREFIX
    }
    
    /// Get MQTT QoS level
    pub const fn mqtt_qos() -> u8 {
        MQTT_QOS
    }
}