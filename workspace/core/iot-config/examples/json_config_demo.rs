//! JSON Configuration Demo
//! 
//! Demonstrates loading configuration from embedded JSON files

use iot_config::{EmbeddedConfig, RuntimeConfigManager, ConfigValidator};

fn main() {
    println!("=== IoT Configuration JSON Demo ===\n");

    // Show current profile
    println!("Current configuration profile: {}", EmbeddedConfig::get_profile_name());
    println!();

    // Load embedded configuration
    match EmbeddedConfig::load_system_config() {
        Ok(config) => {
            println!("✅ Successfully loaded configuration from embedded JSON");
            
            // Display configuration sections
            println!("\n📡 WiFi Configuration:");
            println!("  SSID: {}", config.wifi.ssid);
            println!("  Timeout: {}s", config.wifi.timeout_seconds);
            println!("  Auto-reconnect: {}", config.wifi.auto_reconnect);
            
            println!("\n📨 MQTT Configuration:");
            println!("  Broker: {}:{}", config.mqtt.broker_ip, config.mqtt.broker_port);
            println!("  Client ID: {}", config.mqtt.client_id);
            println!("  Sensor Topic: {}", config.mqtt.sensor_topic);
            println!("  Sensor Interval: {}s", config.mqtt.sensor_interval_secs);
            
            println!("\n🌡️ Sensor Configuration:");
            println!("  I2C Address: 0x{:02X}", config.sensor.i2c_address);
            println!("  Reading Interval: {}s", config.sensor.reading_interval_secs);
            println!("  Calibration: {}", config.sensor.calibration_enabled);
            
            println!("\n💻 Console Configuration:");
            println!("  Enabled: {}", config.console.enabled);
            println!("  Baud Rate: {}", config.console.baud_rate);
            println!("  Prompt: {}", config.console.prompt);
            
            println!("\n⚙️ System Configuration:");
            println!("  Performance Monitoring: {}", config.system.performance_monitoring);
            println!("  Debug Output: {}", config.system.debug_output);
            println!("  Heap Size: {}KB", config.system.heap_size / 1024);
            
            println!("\n💾 Storage Configuration:");
            println!("  Flash Offset: 0x{:06X}", config.storage.flash_offset);
            println!("  Backup Enabled: {}", config.storage.backup_enabled);
            println!("  Wear Leveling: {}", config.storage.wear_leveling);
            
            println!("\n🔧 Hardware Configuration:");
            println!("  I2C SDA Pin: GPIO{}", config.hardware.i2c_sda_pin);
            println!("  I2C SCL Pin: GPIO{}", config.hardware.i2c_scl_pin);
            println!("  Status LED Pin: GPIO{}", config.hardware.status_led_pin);
            println!("  I2C Frequency: {}Hz", config.hardware.i2c_frequency_hz);
            
            println!("\n🚀 Feature Flags:");
            println!("  WiFi: {}", config.features.wifi_enabled);
            println!("  MQTT: {}", config.features.mqtt_enabled);
            println!("  Console: {}", config.features.console_enabled);
            println!("  Performance: {}", config.features.performance_enabled);
            println!("  Storage: {}", config.features.storage_enabled);
            
            // Validate configuration
            println!("\n✅ Configuration Validation:");
            let validation_report = config.validate();
            
            if validation_report.is_valid {
                println!("  ✅ Configuration is valid!");
            } else {
                println!("  ❌ Configuration has issues:");
            }
            
            println!("  Critical: {}", validation_report.critical_count());
            println!("  Warnings: {}", validation_report.warning_count());
            println!("  Total Issues: {}", validation_report.issues.len());
            
            // Show validation details
            if !validation_report.issues.is_empty() {
                println!("\n📋 Validation Issues:");
                for issue in &validation_report.issues {
                    let severity_icon = match issue.severity {
                        iot_config::ValidationSeverity::Critical => "🔥",
                        iot_config::ValidationSeverity::Warning => "⚠️",
                        iot_config::ValidationSeverity::Info => "ℹ️",
                    };
                    println!("  {} {}: {}", severity_icon, issue.field, issue.message);
                    if let Some(suggestion) = &issue.suggestion {
                        println!("     💡 {}", suggestion);
                    }
                }
            }
            
            // Show raw JSON
            println!("\n📄 Raw Embedded JSON:");
            println!("{}", EmbeddedConfig::get_config_json());
            
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {:?}", e);
        }
    }

    println!("\n=== Demo Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    use iot_config::*;

    #[test]
    fn test_json_config_loading() {
        let config = EmbeddedConfig::load_system_config();
        assert!(config.is_ok(), "Should load embedded configuration");
        
        let config = config.unwrap();
        assert!(!config.wifi.ssid.is_empty(), "WiFi SSID should be configured");
        assert_ne!(config.mqtt.broker_port, 0, "MQTT port should be set");
        assert_ne!(config.hardware.i2c_frequency_hz, 0, "I2C frequency should be set");
    }
    
    #[test]
    fn test_configuration_validation() {
        let config = EmbeddedConfig::load_system_config().unwrap();
        let report = config.validate();
        
        // Configuration should be valid or have only warnings/info
        if !report.is_valid {
            assert_eq!(report.critical_count(), 0, "Should not have critical validation errors");
        }
    }
    
    #[test]
    fn test_profile_detection() {
        let profile = EmbeddedConfig::get_profile_name();
        assert!(
            ["default", "development", "production"].contains(&profile),
            "Should detect valid profile"
        );
    }
}