//! Configuration Validation
//! 
//! Comprehensive validation system for IoT configuration with detailed error reporting.

extern crate alloc;
use alloc::{string::String, vec::Vec, format, string::ToString};

use crate::{IoTSystemConfig, WiFiConfig, MqttConfig, SensorConfig, ConsoleConfig, StorageConfig, HardwareConfig};
#[allow(unused_imports)]
use crate::{ConfigResult, ConfigError, SystemConfig};

/// Validation severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationSeverity {
    Critical,  // Configuration will not work
    Warning,   // Configuration may cause issues
    Info,      // Configuration suggestions
}

/// Validation result with detailed information
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Complete validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub is_valid: bool,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            is_valid: true,
        }
    }

    pub fn add_issue(&mut self, severity: ValidationSeverity, field: &str, message: &str, suggestion: Option<&str>) {
        if severity == ValidationSeverity::Critical {
            self.is_valid = false;
        }
        
        self.issues.push(ValidationIssue {
            severity,
            field: field.to_string(),
            message: message.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        });
    }

    pub fn critical_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == ValidationSeverity::Critical).count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count()
    }

    pub fn summary(&self) -> String {
        format!("Validation: {} critical, {} warnings, {} total issues", 
                self.critical_count(), self.warning_count(), self.issues.len())
    }
}

/// Configuration validator trait
pub trait ConfigValidator {
    fn validate(&self) -> ValidationReport;
}

impl ConfigValidator for IoTSystemConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Validate WiFi configuration
        let wifi_report = self.wifi.validate();
        report.issues.extend(wifi_report.issues);
        if !wifi_report.is_valid {
            report.is_valid = false;
        }

        // Validate MQTT configuration
        let mqtt_report = self.mqtt.validate();
        report.issues.extend(mqtt_report.issues);
        if !mqtt_report.is_valid {
            report.is_valid = false;
        }

        // Validate sensor configuration
        let sensor_report = self.sensor.validate();
        report.issues.extend(sensor_report.issues);
        if !sensor_report.is_valid {
            report.is_valid = false;
        }

        // Validate console configuration
        let console_report = self.console.validate();
        report.issues.extend(console_report.issues);
        if !console_report.is_valid {
            report.is_valid = false;
        }

        // Validate storage configuration
        let storage_report = self.storage.validate();
        report.issues.extend(storage_report.issues);
        if !storage_report.is_valid {
            report.is_valid = false;
        }

        // Validate hardware configuration
        let hardware_report = self.hardware.validate();
        report.issues.extend(hardware_report.issues);
        if !hardware_report.is_valid {
            report.is_valid = false;
        }

        // Cross-component validation
        if self.features.mqtt_enabled && !self.features.wifi_enabled {
            report.add_issue(
                ValidationSeverity::Critical,
                "features.mqtt_enabled",
                "MQTT requires WiFi to be enabled",
                Some("Enable WiFi feature or disable MQTT")
            );
        }

        // System resource validation
        if self.system.heap_size < 32 * 1024 {
            report.add_issue(
                ValidationSeverity::Warning,
                "system.heap_size",
                "Heap size may be too small for WiFi operations",
                Some("Consider increasing heap size to 64KB")
            );
        }

        if self.system.task_stack_size < 2048 {
            report.add_issue(
                ValidationSeverity::Warning,
                "system.task_stack_size",
                "Stack size may be insufficient for complex operations",
                Some("Consider increasing stack size to 4KB")
            );
        }

        report
    }
}

impl ConfigValidator for WiFiConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // SSID validation
        if self.ssid.is_empty() {
            report.add_issue(
                ValidationSeverity::Critical,
                "wifi.ssid",
                "WiFi SSID cannot be empty",
                Some("Set a valid WiFi network name")
            );
        } else if self.ssid.len() > 32 {
            report.add_issue(
                ValidationSeverity::Critical,
                "wifi.ssid",
                "WiFi SSID too long (max 32 characters)",
                Some("Shorten the SSID")
            );
        }

        // Password validation
        if self.password.is_empty() {
            report.add_issue(
                ValidationSeverity::Warning,
                "wifi.password",
                "WiFi password is empty (open network)",
                Some("Consider using a secured network")
            );
        } else if self.password.len() < 8 {
            report.add_issue(
                ValidationSeverity::Warning,
                "wifi.password",
                "WiFi password is short (less than 8 characters)",
                Some("Use a stronger password")
            );
        } else if self.password.len() > 64 {
            report.add_issue(
                ValidationSeverity::Critical,
                "wifi.password",
                "WiFi password too long (max 64 characters)",
                Some("Shorten the password")
            );
        }

        // Timeout validation
        if self.timeout_seconds == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "wifi.timeout_seconds",
                "WiFi timeout cannot be zero",
                Some("Set timeout to at least 5 seconds")
            );
        } else if self.timeout_seconds > 60 {
            report.add_issue(
                ValidationSeverity::Warning,
                "wifi.timeout_seconds",
                "WiFi timeout is very long",
                Some("Consider reducing timeout to 10-30 seconds")
            );
        }

        // Retry attempts validation
        if self.retry_attempts == 0 {
            report.add_issue(
                ValidationSeverity::Warning,
                "wifi.retry_attempts",
                "No retry attempts configured",
                Some("Consider allowing 1-3 retry attempts")
            );
        } else if self.retry_attempts > 10 {
            report.add_issue(
                ValidationSeverity::Warning,
                "wifi.retry_attempts",
                "Too many retry attempts may cause delays",
                Some("Limit retry attempts to 3-5")
            );
        }

        report
    }
}

impl ConfigValidator for MqttConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Broker IP validation
        if self.broker_ip.is_empty() {
            report.add_issue(
                ValidationSeverity::Critical,
                "mqtt.broker_ip",
                "MQTT broker IP cannot be empty",
                Some("Set a valid broker IP address")
            );
        } else {
            // Basic IP format validation
            let ip_str = self.broker_ip.as_str();
            if !is_valid_ip_format(ip_str) {
                report.add_issue(
                    ValidationSeverity::Critical,
                    "mqtt.broker_ip",
                    "MQTT broker IP format is invalid",
                    Some("Use format like 192.168.1.100")
                );
            }
        }

        // Port validation
        if self.broker_port == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "mqtt.broker_port",
                "MQTT broker port cannot be zero",
                Some("Use standard port 1883 or 8883 for TLS")
            );
        } else if self.broker_port != 1883 && self.broker_port != 8883 {
            report.add_issue(
                ValidationSeverity::Info,
                "mqtt.broker_port",
                "Non-standard MQTT port",
                Some("Standard ports are 1883 (plain) or 8883 (TLS)")
            );
        }

        // Client ID validation
        if self.client_id.is_empty() {
            report.add_issue(
                ValidationSeverity::Critical,
                "mqtt.client_id",
                "MQTT client ID cannot be empty",
                Some("Set a unique client identifier")
            );
        } else if self.client_id.len() > 23 {
            report.add_issue(
                ValidationSeverity::Warning,
                "mqtt.client_id",
                "MQTT client ID may be too long for some brokers",
                Some("Keep client ID under 23 characters")
            );
        }

        // Topic validation
        validate_mqtt_topic(&mut report, "mqtt.sensor_topic", self.sensor_topic.as_str());
        validate_mqtt_topic(&mut report, "mqtt.status_topic", self.status_topic.as_str());
        validate_mqtt_topic(&mut report, "mqtt.heartbeat_topic", self.heartbeat_topic.as_str());

        // Interval validation
        if self.sensor_interval_secs < 1 {
            report.add_issue(
                ValidationSeverity::Critical,
                "mqtt.sensor_interval_secs",
                "Sensor interval too short",
                Some("Set interval to at least 1 second")
            );
        } else if self.sensor_interval_secs > 3600 {
            report.add_issue(
                ValidationSeverity::Warning,
                "mqtt.sensor_interval_secs",
                "Sensor interval very long (over 1 hour)",
                Some("Consider shorter interval for better monitoring")
            );
        }

        if self.heartbeat_interval_secs < 10 {
            report.add_issue(
                ValidationSeverity::Warning,
                "mqtt.heartbeat_interval_secs",
                "Heartbeat interval may be too short",
                Some("Consider 30-60 second intervals")
            );
        }

        report
    }
}

impl ConfigValidator for SensorConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // I2C address validation
        if self.i2c_address == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "sensor.i2c_address",
                "I2C address cannot be zero",
                Some("Use 0x76 or 0x77 for BME280")
            );
        } else if self.i2c_address != 0x76 && self.i2c_address != 0x77 {
            report.add_issue(
                ValidationSeverity::Warning,
                "sensor.i2c_address",
                "Non-standard BME280 I2C address",
                Some("BME280 typically uses 0x76 or 0x77")
            );
        }

        // Reading interval validation
        if self.reading_interval_secs == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "sensor.reading_interval_secs",
                "Reading interval cannot be zero",
                Some("Set interval to at least 1 second")
            );
        } else if self.reading_interval_secs > 3600 {
            report.add_issue(
                ValidationSeverity::Warning,
                "sensor.reading_interval_secs",
                "Reading interval very long (over 1 hour)",
                Some("Consider shorter interval for responsive monitoring")
            );
        }

        // Offset validation
        if self.temperature_offset.abs() > 10.0 {
            report.add_issue(
                ValidationSeverity::Warning,
                "sensor.temperature_offset",
                "Large temperature offset may indicate calibration issue",
                Some("Verify sensor calibration")
            );
        }

        if self.humidity_offset.abs() > 20.0 {
            report.add_issue(
                ValidationSeverity::Warning,
                "sensor.humidity_offset",
                "Large humidity offset may indicate calibration issue",
                Some("Verify sensor calibration")
            );
        }

        if self.pressure_offset.abs() > 100.0 {
            report.add_issue(
                ValidationSeverity::Warning,
                "sensor.pressure_offset",
                "Large pressure offset may indicate calibration issue",
                Some("Verify sensor calibration")
            );
        }

        report
    }
}

impl ConfigValidator for ConsoleConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Baud rate validation
        if self.baud_rate == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "console.baud_rate",
                "Console baud rate cannot be zero",
                Some("Use standard baud rate like 115200")
            );
        } else if ![9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600].contains(&self.baud_rate) {
            report.add_issue(
                ValidationSeverity::Warning,
                "console.baud_rate",
                "Non-standard baud rate",
                Some("Consider using standard baud rate for compatibility")
            );
        }

        // Command timeout validation
        if self.command_timeout_ms < 1000 {
            report.add_issue(
                ValidationSeverity::Warning,
                "console.command_timeout_ms",
                "Very short command timeout",
                Some("Consider timeout of at least 1000ms")
            );
        } else if self.command_timeout_ms > 30000 {
            report.add_issue(
                ValidationSeverity::Warning,
                "console.command_timeout_ms",
                "Very long command timeout",
                Some("Consider timeout under 30 seconds")
            );
        }

        // History size validation
        if self.history_size == 0 {
            report.add_issue(
                ValidationSeverity::Info,
                "console.history_size",
                "No command history configured",
                Some("Consider enabling command history")
            );
        } else if self.history_size > 50 {
            report.add_issue(
                ValidationSeverity::Warning,
                "console.history_size",
                "Large history size may use too much memory",
                Some("Consider reducing history size to save RAM")
            );
        }

        report
    }
}

impl ConfigValidator for StorageConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Flash offset validation
        if self.flash_offset < 0x100000 {
            report.add_issue(
                ValidationSeverity::Warning,
                "storage.flash_offset",
                "Flash offset may conflict with application code",
                Some("Use offset above 1MB (0x100000)")
            );
        }

        // ESP32-C3 typically has 4MB flash, warn if offset is too high
        if self.flash_offset > 0x380000 {
            report.add_issue(
                ValidationSeverity::Warning,
                "storage.flash_offset",
                "Flash offset may exceed available flash memory",
                Some("Ensure offset is within your flash size")
            );
        }

        // Flash offset should be sector-aligned (4KB)
        if self.flash_offset % 4096 != 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "storage.flash_offset",
                "Flash offset must be sector-aligned (4KB)",
                Some("Use offset that is multiple of 4096")
            );
        }

        report
    }
}

impl ConfigValidator for HardwareConfig {
    fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Pin validation for ESP32-C3
        let valid_gpio_pins = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 18, 19, 20, 21];

        if !valid_gpio_pins.contains(&self.i2c_sda_pin) {
            report.add_issue(
                ValidationSeverity::Critical,
                "hardware.i2c_sda_pin",
                "Invalid GPIO pin for ESP32-C3",
                Some("Use valid GPIO pin (0-10, 18-21)")
            );
        }

        if !valid_gpio_pins.contains(&self.i2c_scl_pin) {
            report.add_issue(
                ValidationSeverity::Critical,
                "hardware.i2c_scl_pin",
                "Invalid GPIO pin for ESP32-C3",
                Some("Use valid GPIO pin (0-10, 18-21)")
            );
        }

        if !valid_gpio_pins.contains(&self.status_led_pin) {
            report.add_issue(
                ValidationSeverity::Critical,
                "hardware.status_led_pin",
                "Invalid GPIO pin for ESP32-C3",
                Some("Use valid GPIO pin (0-10, 18-21)")
            );
        }

        // Check for pin conflicts
        if self.i2c_sda_pin == self.i2c_scl_pin {
            report.add_issue(
                ValidationSeverity::Critical,
                "hardware.i2c_sda_pin",
                "SDA and SCL pins cannot be the same",
                Some("Use different pins for SDA and SCL")
            );
        }

        if self.i2c_sda_pin == self.status_led_pin || self.i2c_scl_pin == self.status_led_pin {
            report.add_issue(
                ValidationSeverity::Warning,
                "hardware.status_led_pin",
                "LED pin conflicts with I2C pins",
                Some("Consider using different pin for LED")
            );
        }

        // I2C frequency validation
        if self.i2c_frequency_hz == 0 {
            report.add_issue(
                ValidationSeverity::Critical,
                "hardware.i2c_frequency_hz",
                "I2C frequency cannot be zero",
                Some("Use standard frequency like 100000 Hz")
            );
        } else if self.i2c_frequency_hz > 1_000_000 {
            report.add_issue(
                ValidationSeverity::Warning,
                "hardware.i2c_frequency_hz",
                "Very high I2C frequency may cause communication issues",
                Some("Consider frequency under 1MHz")
            );
        } else if ![100_000, 400_000, 1_000_000].contains(&self.i2c_frequency_hz) {
            report.add_issue(
                ValidationSeverity::Info,
                "hardware.i2c_frequency_hz",
                "Non-standard I2C frequency",
                Some("Standard frequencies are 100kHz, 400kHz, 1MHz")
            );
        }

        report
    }
}

/// Validate IP address format (basic validation)
fn is_valid_ip_format(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    
    for part in parts {
        if let Ok(num) = part.parse::<u8>() {
            if part != &num.to_string() {
                return false; // Leading zeros not allowed
            }
        } else {
            return false;
        }
    }
    
    true
}

/// Validate MQTT topic format
fn validate_mqtt_topic(report: &mut ValidationReport, field: &str, topic: &str) {
    if topic.is_empty() {
        report.add_issue(
            ValidationSeverity::Critical,
            field,
            "MQTT topic cannot be empty",
            Some("Set a valid topic name")
        );
    } else if topic.contains('#') && !topic.ends_with('#') {
        report.add_issue(
            ValidationSeverity::Critical,
            field,
            "MQTT wildcard '#' must be at the end",
            Some("Move '#' to end or remove it")
        );
    } else if topic.contains("//") {
        report.add_issue(
            ValidationSeverity::Warning,
            field,
            "MQTT topic contains empty levels",
            Some("Remove double slashes")
        );
    } else if topic.starts_with('/') {
        report.add_issue(
            ValidationSeverity::Info,
            field,
            "MQTT topic starts with '/' (may be unnecessary)",
            Some("Consider removing leading slash")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_wifi_validation() {
        let mut config = WiFiConfig::default();
        config.ssid = create_bounded_string("test_network", "ssid").unwrap();
        config.password = create_bounded_string("password123", "password").unwrap();
        
        let report = config.validate();
        assert!(report.is_valid);
    }

    #[test]
    fn test_wifi_validation_empty_ssid() {
        let config = WiFiConfig::default(); // Empty SSID
        let report = config.validate();
        assert!(!report.is_valid);
        assert!(report.critical_count() > 0);
    }

    #[test]
    fn test_mqtt_validation() {
        let config = MqttConfig::default();
        let report = config.validate();
        assert!(report.is_valid);
    }

    #[test]
    fn test_ip_validation() {
        assert!(is_valid_ip_format("192.168.1.1"));
        assert!(is_valid_ip_format("10.0.0.1"));
        assert!(!is_valid_ip_format("192.168.1"));
        assert!(!is_valid_ip_format("192.168.1.256"));
        assert!(!is_valid_ip_format("192.168.01.1")); // Leading zero
    }

    #[test]
    fn test_system_validation() {
        let config = IoTSystemConfig::default();
        let report = config.validate();
        // Should pass with warnings only
        assert!(report.is_valid);
    }
}