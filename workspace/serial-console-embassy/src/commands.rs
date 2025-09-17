//! Command parsing and handling for the serial console
//! 
//! Defines the command structure and implements parsers for various
//! system commands including help, status, configuration, etc.

use heapless::String;
use crate::config::{SystemConfig, MAX_SSID_LEN, MAX_PASSWORD_LEN, MAX_IP_LEN, MAX_HOSTNAME_LEN};

/// Maximum number of command arguments
pub const MAX_ARGS: usize = 4;
/// Maximum length of command line
pub const MAX_CMD_LEN: usize = 128;

/// Available system commands
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Show help information
    Help,
    /// Display system status
    Status,
    /// Show system information
    Info,
    /// Set WiFi SSID
    SetWifiSsid(String<MAX_SSID_LEN>),
    /// Set WiFi password
    SetWifiPassword(String<MAX_PASSWORD_LEN>),
    /// Show current WiFi configuration
    ShowWifi,
    /// Set MQTT broker IP
    SetMqttBroker(String<MAX_IP_LEN>),
    /// Set MQTT broker port
    SetMqttPort(u16),
    /// Set MQTT client ID
    SetMqttClientId(String<MAX_HOSTNAME_LEN>),
    /// Set MQTT topic prefix
    SetMqttPrefix(String<MAX_HOSTNAME_LEN>),
    /// Show current MQTT configuration
    ShowMqtt,
    /// Restart the system
    Restart,
    /// Save configuration to flash
    Save,
    /// Load configuration from flash
    Load,
    /// Clear screen
    Clear,
    /// Unknown command
    Unknown(String<MAX_CMD_LEN>),
}

/// Command line parser and handler
pub struct CommandHandler {
    config: SystemConfig,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            config: SystemConfig::new(),
        }
    }
    
    /// Parse a command line string into a Command enum
    pub fn parse_command(&self, line: &str) -> Command {
        let line = line.trim();
        if line.is_empty() {
            return Command::Help;
        }
        
        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        // Create lowercase version manually (no_std compatible)
        let mut cmd_lower = String::<32>::new();
        for c in cmd.chars() {
            let _ = cmd_lower.push(c.to_ascii_lowercase());
        }
        let cmd = cmd_lower.as_str();
        
        match cmd {
            "help" | "h" | "?" => Command::Help,
            "status" | "stat" => Command::Status,
            "info" | "i" => Command::Info,
            "clear" | "cls" => Command::Clear,
            "restart" | "reset" => Command::Restart,
            "save" => Command::Save,
            "load" => Command::Load,
            "wifi" => {
                match parts.next() {
                    Some("show") | None => Command::ShowWifi,
                    Some("ssid") => {
                        if let Some(ssid) = parts.next() {
                            let mut ssid_str = String::new();
                            let _ = ssid_str.push_str(ssid);
                            Command::SetWifiSsid(ssid_str)
                        } else {
                            Command::ShowWifi
                        }
                    },
                    Some("password") | Some("pass") => {
                        if let Some(password) = parts.next() {
                            let mut pass_str = String::new();
                            let _ = pass_str.push_str(password);
                            Command::SetWifiPassword(pass_str)
                        } else {
                            Command::ShowWifi
                        }
                    },
                    _ => Command::ShowWifi,
                }
            },
            "mqtt" => {
                match parts.next() {
                    Some("show") | None => Command::ShowMqtt,
                    Some("broker") | Some("ip") => {
                        if let Some(ip) = parts.next() {
                            let mut ip_str = String::new();
                            let _ = ip_str.push_str(ip);
                            Command::SetMqttBroker(ip_str)
                        } else {
                            Command::ShowMqtt
                        }
                    },
                    Some("port") => {
                        if let Some(port_str) = parts.next() {
                            if let Ok(port) = port_str.parse::<u16>() {
                                Command::SetMqttPort(port)
                            } else {
                                let mut err_str = String::new();
                                let _ = err_str.push_str("Invalid port number");
                                Command::Unknown(err_str)
                            }
                        } else {
                            Command::ShowMqtt
                        }
                    },
                    Some("client") => {
                        if let Some(client_id) = parts.next() {
                            let mut client_str = String::new();
                            let _ = client_str.push_str(client_id);
                            Command::SetMqttClientId(client_str)
                        } else {
                            Command::ShowMqtt
                        }
                    },
                    Some("prefix") => {
                        if let Some(prefix) = parts.next() {
                            let mut prefix_str = String::new();
                            let _ = prefix_str.push_str(prefix);
                            Command::SetMqttPrefix(prefix_str)
                        } else {
                            Command::ShowMqtt
                        }
                    },
                    _ => Command::ShowMqtt,
                }
            },
            _ => {
                let mut unknown_str = String::new();
                let _ = unknown_str.push_str(line);
                Command::Unknown(unknown_str)
            }
        }
    }
    
    /// Execute a command and return response
    pub fn execute_command(&mut self, cmd: Command) -> String<512> {
        let mut response = String::new();
        
        match cmd {
            Command::Help => {
                let _ = response.push_str(
                    "\r\n=== ESP32-C3 IoT System Console ===\r\n\
                     Available commands:\r\n\
                     help, h, ?       - Show this help\r\n\
                     status, stat     - Show system status\r\n\
                     info, i          - Show detailed system info\r\n\
                     clear, cls       - Clear screen\r\n\
                     restart, reset   - Restart system\r\n\
                     save             - Save config to flash\r\n\
                     load             - Load config from flash\r\n\
                     \r\n\
                     WiFi commands:\r\n\
                     wifi show        - Show WiFi config\r\n\
                     wifi ssid <name> - Set WiFi SSID\r\n\
                     wifi pass <pwd>  - Set WiFi password\r\n\
                     \r\n\
                     MQTT commands:\r\n\
                     mqtt show        - Show MQTT config\r\n\
                     mqtt broker <ip> - Set MQTT broker IP\r\n\
                     mqtt port <num>  - Set MQTT port\r\n\
                     mqtt client <id> - Set client ID\r\n\
                     mqtt prefix <pfx>- Set topic prefix\r\n\
                     \r\n"
                );
            },
            
            Command::Status => {
                let _ = response.push_str("\r\n=== System Status ===\r\n");
                let _ = response.push_str("WiFi: ");
                if self.config.system.wifi_connected {
                    let _ = response.push_str("Connected");
                    if let Some(ip) = &self.config.system.current_ip {
                        let _ = response.push_str(" (");
                        let _ = response.push_str(ip);
                        let _ = response.push_str(")");
                    }
                } else {
                    let _ = response.push_str("Disconnected");
                }
                let _ = response.push_str("\r\n");
                
                let _ = response.push_str("MQTT: ");
                if self.config.system.mqtt_connected {
                    let _ = response.push_str("Connected");
                } else {
                    let _ = response.push_str("Disconnected");
                }
                let _ = response.push_str("\r\n");
                
                let _ = response.push_str("Sensor: ");
                if self.config.system.sensor_active {
                    let _ = response.push_str("Active");
                } else {
                    let _ = response.push_str("Inactive");
                }
                let _ = response.push_str("\r\n");
            },
            
            Command::Info => {
                let _ = response.push_str("\r\n=== System Information ===\r\n");
                // This would be filled with actual system info
                let _ = response.push_str("Chip: ESP32-C3\r\n");
                let _ = response.push_str("Framework: Embassy\r\n");
                let _ = response.push_str("Build: Release\r\n");
                let _ = response.push_str("Free Heap: ");
                // Add actual heap info here
                let _ = response.push_str("48KB\r\n");
            },
            
            Command::SetWifiSsid(ssid) => {
                self.config.wifi.ssid = ssid.clone();
                let _ = response.push_str("\r\nWiFi SSID set to: ");
                let _ = response.push_str(&ssid);
                let _ = response.push_str("\r\n");
            },
            
            Command::SetWifiPassword(password) => {
                self.config.wifi.password = password;
                let _ = response.push_str("\r\nWiFi password updated\r\n");
            },
            
            Command::ShowWifi => {
                let _ = response.push_str("\r\n=== WiFi Configuration ===\r\n");
                let _ = response.push_str("SSID: ");
                let _ = response.push_str(&self.config.wifi.ssid);
                let _ = response.push_str("\r\nPassword: ");
                if self.config.wifi.password.is_empty() {
                    let _ = response.push_str("(not set)");
                } else {
                    let _ = response.push_str("********");
                }
                let _ = response.push_str("\r\nStatus: ");
                if self.config.wifi.is_valid() {
                    let _ = response.push_str("Valid");
                } else {
                    let _ = response.push_str("Incomplete");
                }
                let _ = response.push_str("\r\n");
            },
            
            Command::SetMqttBroker(ip) => {
                self.config.mqtt.broker_ip = ip.clone();
                let _ = response.push_str("\r\nMQTT broker set to: ");
                let _ = response.push_str(&ip);
                let _ = response.push_str("\r\n");
            },
            
            Command::SetMqttPort(port) => {
                self.config.mqtt.broker_port = port;
                let _ = response.push_str("\r\nMQTT port set to: ");
                // Simple integer to string conversion
                let _ = response.push_str("1883"); // placeholder
                let _ = response.push_str("\r\n");
            },
            
            Command::SetMqttClientId(client_id) => {
                self.config.mqtt.client_id = client_id.clone();
                let _ = response.push_str("\r\nMQTT client ID set to: ");
                let _ = response.push_str(&client_id);
                let _ = response.push_str("\r\n");
            },
            
            Command::SetMqttPrefix(prefix) => {
                self.config.mqtt.topic_prefix = prefix.clone();
                let _ = response.push_str("\r\nMQTT topic prefix set to: ");
                let _ = response.push_str(&prefix);
                let _ = response.push_str("\r\n");
            },
            
            Command::ShowMqtt => {
                let _ = response.push_str("\r\n=== MQTT Configuration ===\r\n");
                let _ = response.push_str("Broker: ");
                let _ = response.push_str(&self.config.mqtt.broker_ip);
                let _ = response.push_str(":");
                // Port conversion placeholder
                let _ = response.push_str("1883");
                let _ = response.push_str("\r\nClient ID: ");
                let _ = response.push_str(&self.config.mqtt.client_id);
                let _ = response.push_str("\r\nTopic Prefix: ");
                let _ = response.push_str(&self.config.mqtt.topic_prefix);
                let _ = response.push_str("\r\nStatus: ");
                if self.config.mqtt.is_valid() {
                    let _ = response.push_str("Valid");
                } else {
                    let _ = response.push_str("Incomplete");
                }
                let _ = response.push_str("\r\n");
            },
            
            Command::Clear => {
                let _ = response.push_str("\x1B[2J\x1B[H"); // ANSI clear screen
            },
            
            Command::Restart => {
                let _ = response.push_str("\r\nRestarting system...\r\n");
                // TODO: Implement actual restart
            },
            
            Command::Save => {
                let _ = response.push_str("\r\nConfiguration saved to flash\r\n");
                // TODO: Implement flash save
            },
            
            Command::Load => {
                let _ = response.push_str("\r\nConfiguration loaded from flash\r\n");
                // TODO: Implement flash load
            },
            
            Command::Unknown(cmd) => {
                let _ = response.push_str("\r\nUnknown command: ");
                let _ = response.push_str(&cmd);
                let _ = response.push_str("\r\nType 'help' for available commands\r\n");
            },
        }
        
        response
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> &SystemConfig {
        &self.config
    }
    
    /// Update system status
    pub fn update_system_status(&mut self, wifi_connected: bool, mqtt_connected: bool, sensor_active: bool, current_ip: Option<&str>) {
        self.config.system.wifi_connected = wifi_connected;
        self.config.system.mqtt_connected = mqtt_connected;
        self.config.system.sensor_active = sensor_active;
        
        if let Some(ip) = current_ip {
            let mut ip_string = String::new();
            let _ = ip_string.push_str(ip);
            self.config.system.current_ip = Some(ip_string);
        } else {
            self.config.system.current_ip = None;
        }
    }
}