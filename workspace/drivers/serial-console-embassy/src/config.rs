//! Configuration structures for the IoT system
//! 
//! Defines data structures for storing and managing system configuration
//! including WiFi credentials, MQTT settings, and system parameters.

use heapless::String;

/// Maximum length for SSID strings
pub const MAX_SSID_LEN: usize = 32;
/// Maximum length for password strings  
pub const MAX_PASSWORD_LEN: usize = 64;
/// Maximum length for IP address strings
pub const MAX_IP_LEN: usize = 15;
/// Maximum length for hostname/URL strings
pub const MAX_HOSTNAME_LEN: usize = 64;

/// WiFi network credentials
#[derive(Debug, Clone)]
pub struct WiFiCredentials {
    pub ssid: String<MAX_SSID_LEN>,
    pub password: String<MAX_PASSWORD_LEN>,
}

impl WiFiCredentials {
    pub fn new() -> Self {
        Self {
            ssid: String::new(),
            password: String::new(),
        }
    }
    
    pub fn is_valid(&self) -> bool {
        !self.ssid.is_empty() && !self.password.is_empty()
    }
}

impl Default for WiFiCredentials {
    fn default() -> Self {
        Self::new()
    }
}

/// MQTT broker configuration
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub broker_ip: String<MAX_IP_LEN>,
    pub broker_port: u16,
    pub client_id: String<MAX_HOSTNAME_LEN>,
    pub topic_prefix: String<MAX_HOSTNAME_LEN>,
}

impl MqttConfig {
    pub fn new() -> Self {
        Self {
            broker_ip: String::new(),
            broker_port: 1883,
            client_id: {
                let mut s = String::new();
                let _ = s.push_str("esp32-c3");
                s
            },
            topic_prefix: {
                let mut s = String::new();
                let _ = s.push_str("esp32");
                s
            },
        }
    }
    
    pub fn is_valid(&self) -> bool {
        !self.broker_ip.is_empty() && self.broker_port > 0
    }
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// System information and status
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub uptime_seconds: u64,
    pub free_heap: u32,
    pub wifi_connected: bool,
    pub mqtt_connected: bool,
    pub sensor_active: bool,
    pub current_ip: Option<String<MAX_IP_LEN>>,
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            uptime_seconds: 0,
            free_heap: 0,
            wifi_connected: false,
            mqtt_connected: false,
            sensor_active: false,
            current_ip: None,
        }
    }
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete system configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    pub wifi: WiFiCredentials,
    pub mqtt: MqttConfig,
    pub system: SystemInfo,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self {
            wifi: WiFiCredentials::new(),
            mqtt: MqttConfig::new(),
            system: SystemInfo::new(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self::new()
    }
}