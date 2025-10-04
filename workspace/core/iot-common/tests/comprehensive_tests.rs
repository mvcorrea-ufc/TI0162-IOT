//! Comprehensive Testing for IoT Common Library
//! 
//! These tests run in std environment but verify no_std compatible functionality.
//! Tests cover error handling, memory bounds, and error conversion functions.

// Allow std for testing while the main library remains no_std
#![cfg(test)]

use std::vec::Vec;

#[test]
fn test_basic_error_creation() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    // Test basic error creation
    let error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("Test error")
    ));
    
    // Test basic properties
    assert!(error.is_sensor_error());
    assert_eq!(error.error_code(), 1002);
    assert_eq!(error.category(), "Sensor");
}

#[test]
fn test_error_context_preservation() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    let error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("I2C timeout")
    )).with_context("BME280 reading");
    
    assert!(!error.context().is_empty());
    assert_eq!(error.context().contexts().len(), 1);
}

#[test]
fn test_error_display_formatting() {
    use iot_common::{IoTError, NetworkError, error::utils};
    
    let error = IoTError::network(NetworkError::WiFiConnectionFailed(
        utils::error_message("Authentication failed")
    ));
    
    let error_string = format!("{}", error);
    assert!(error_string.contains("WiFi connection failed"));
    assert!(error_string.contains("Authentication failed"));
}

#[test] 
fn test_all_sensor_error_variants() {
    use iot_common::{SensorError, error::utils};
    
    let errors = [
        (SensorError::InitializationFailed(utils::error_message("init")), 1001),
        (SensorError::I2CError(utils::error_message("i2c")), 1002),
        (SensorError::InvalidData(utils::error_message("data")), 1003),
        (SensorError::CalibrationError(utils::error_message("calib")), 1004),
        (SensorError::NotResponding(utils::error_message("resp")), 1005),
        (SensorError::InvalidConfiguration(utils::error_message("config")), 1006),
    ];
    
    for (error, expected_code) in errors {
        assert_eq!(error.error_code(), expected_code);
        assert_eq!(error.category(), "Sensor");
    }
}

#[test]
fn test_all_network_error_variants() {
    use iot_common::{NetworkError, error::utils};
    
    let errors = [
        (NetworkError::WiFiConnectionFailed(utils::error_message("wifi")), 2001),
        (NetworkError::WiFiConfigurationError(utils::error_message("config")), 2002),
        (NetworkError::DHCPFailed(utils::error_message("dhcp")), 2003),
        (NetworkError::TCPConnectionFailed(utils::error_message("tcp")), 2004),
        (NetworkError::Timeout(utils::error_message("timeout")), 2005),
        (NetworkError::DNSResolutionFailed(utils::error_message("dns")), 2006),
        (NetworkError::HardwareInitFailed(utils::error_message("hw")), 2007),
    ];
    
    for (error, expected_code) in errors {
        assert_eq!(error.error_code(), expected_code);
        assert_eq!(error.category(), "Network");
    }
}

#[test]
fn test_all_hardware_error_variants() {
    use iot_common::{HardwareError, error::utils};
    
    let errors = [
        (HardwareError::GPIOError(utils::error_message("gpio")), 3001),
        (HardwareError::SPIError(utils::error_message("spi")), 3002),
        (HardwareError::UARTError(utils::error_message("uart")), 3003),
        (HardwareError::TimerError(utils::error_message("timer")), 3004),
        (HardwareError::InterruptError(utils::error_message("int")), 3005),
        (HardwareError::PowerError(utils::error_message("power")), 3006),
        (HardwareError::ClockError(utils::error_message("clock")), 3007),
    ];
    
    for (error, expected_code) in errors {
        assert_eq!(error.error_code(), expected_code);
        assert_eq!(error.category(), "Hardware");
    }
}

#[test]
fn test_all_config_error_variants() {
    use iot_common::{ConfigError, error::utils};
    
    let errors = [
        (ConfigError::InvalidParameter(utils::error_message("param")), 4001),
        (ConfigError::MissingConfiguration(utils::error_message("missing")), 4002),
        (ConfigError::ParsingError(utils::error_message("parse")), 4003),
        (ConfigError::ValidationError(utils::error_message("valid")), 4004),
        (ConfigError::ConfigNotFound(utils::error_message("notfound")), 4005),
    ];
    
    for (error, expected_code) in errors {
        assert_eq!(error.error_code(), expected_code);
        assert_eq!(error.category(), "Configuration");
    }
}

#[test]
fn test_all_system_error_variants() {
    use iot_common::{SystemError, error::utils};
    
    let errors = [
        (SystemError::OutOfMemory(utils::error_message("oom")), 5001),
        (SystemError::TaskCreationFailed(utils::error_message("task")), 5002),
        (SystemError::ResourceUnavailable(utils::error_message("resource")), 5003),
        (SystemError::InitializationFailed(utils::error_message("init")), 5004),
        (SystemError::WatchdogTimeout(utils::error_message("watchdog")), 5005),
        (SystemError::StackOverflow(utils::error_message("stack")), 5006),
        (SystemError::Panic(utils::error_message("panic")), 5007),
    ];
    
    for (error, expected_code) in errors {
        assert_eq!(error.error_code(), expected_code);
        assert_eq!(error.category(), "System");
    }
}

#[test]
fn test_error_context_chaining() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    let error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("Bus timeout")
    ))
    .with_context("BME280 reading")
    .with_context("Temperature sensor")
    .with_context("Main loop");
    
    assert_eq!(error.context().contexts().len(), 3);
    // Contexts are stored in reverse order (most recent first)
    assert_eq!(error.context().contexts()[0].as_str(), "Main loop");
    assert_eq!(error.context().contexts()[1].as_str(), "Temperature sensor");
    assert_eq!(error.context().contexts()[2].as_str(), "BME280 reading");
}

#[test]
fn test_error_context_max_depth() {
    use iot_common::{IoTError, SensorError, error::utils, MAX_ERROR_CONTEXT_DEPTH};
    
    let mut error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("Base error")
    ));
    
    // Add more contexts than the maximum allowed
    for i in 0..MAX_ERROR_CONTEXT_DEPTH + 2 {
        error = error.with_context(&format!("Context {}", i));
    }
    
    // Should not exceed maximum depth
    assert!(error.context().contexts().len() <= MAX_ERROR_CONTEXT_DEPTH);
}

#[test]
fn test_error_message_truncation() {
    use iot_common::{error::utils, MAX_ERROR_MESSAGE_LEN};
    
    let long_message = "This is a very long error message that definitely exceeds the maximum allowed length for error messages in embedded systems and should be truncated properly with ellipsis at the end";
    let truncated = utils::error_message(long_message);
    
    assert!(truncated.len() <= MAX_ERROR_MESSAGE_LEN);
    assert!(truncated.as_str().ends_with("..."));
    assert_eq!(truncated.len(), MAX_ERROR_MESSAGE_LEN);
    
    // Test exact boundary
    let exact_length_msg = "a".repeat(MAX_ERROR_MESSAGE_LEN);
    let exact_truncated = utils::error_message(&exact_length_msg);
    assert_eq!(exact_truncated.len(), MAX_ERROR_MESSAGE_LEN);
    assert!(!exact_truncated.as_str().ends_with("..."));
}

#[test]
fn test_memory_bounds_simulation() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    // Create many errors to test memory bounds (simulating heapless::Vec behavior)
    let mut errors = Vec::new();
    
    for i in 0..10 {
        let error = IoTError::sensor(SensorError::I2CError(
            utils::error_message(&format!("Error {}", i))
        )).with_context(&format!("Context {}", i));
        
        errors.push(error);
    }
    
    assert_eq!(errors.len(), 10);
    
    // Verify each error maintains its properties
    for (i, error) in errors.iter().enumerate() {
        assert!(error.is_sensor_error());
        assert_eq!(error.error_code(), 1002);
        assert!(!error.context().is_empty());
        
        // Verify the context contains the expected index
        let context_str = format!("{}", error.context());
        assert!(context_str.contains(&format!("Context {}", i)));
    }
}

#[test]
fn test_error_conversion_functions() {
    use iot_common::error::{wifi_conversions, mqtt_conversions};
    
    // Test all WiFi error conversion types
    let wifi_errors = [
        ("HardwareInit", "WiFi chip failed", 2007),
        ("Configuration", "Invalid SSID", 2002), 
        ("Connection", "Auth failed", 2001),
        ("Dhcp", "IP timeout", 2003),
        ("Unknown", "Generic error", 2001), // Should default to connection failed
    ];
    
    for (error_type, message, expected_code) in wifi_errors {
        let error = wifi_conversions::from_wifi_error(error_type, message);
        assert!(error.is_network_error());
        assert_eq!(error.error_code(), expected_code);
    }
    
    // Test all MQTT error conversion types
    let mqtt_errors = [
        ("ConnectionFailed", "TCP timeout", 2004),
        ("ProtocolError", "Invalid packet", 2004),
        ("IoError", "Socket error", 2004),
        ("SerializationError", "JSON failed", 5004), // System error
        ("Unknown", "Generic error", 2004),
    ];
    
    for (error_type, message, expected_code) in mqtt_errors {
        let error = mqtt_conversions::from_mqtt_error(error_type, message);
        if error_type == "SerializationError" {
            assert!(error.is_system_error());
        } else {
            assert!(error.is_network_error());
        }
        assert_eq!(error.error_code(), expected_code);
    }
}

#[test]
fn test_nb_error_conversion() {
    use iot_common::IoTError;
    
    // Define a test error type that implements required traits
    #[derive(Debug)]
    struct TestError;
    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }
    
    // Test nb::Error::WouldBlock conversion
    let would_block_error: IoTError = nb::Error::<TestError>::WouldBlock.into();
    assert!(would_block_error.is_system_error());
    assert_eq!(would_block_error.error_code(), 5003); // ResourceUnavailable
    
    // Test nb::Error::Other conversion
    let other_error: IoTError = nb::Error::Other(TestError).into();
    assert!(other_error.is_hardware_error());
    assert_eq!(other_error.error_code(), 3001); // GPIOError
}

#[test]
fn test_utility_functions_comprehensive() {
    use iot_common::error::utils;
    
    // Test sensor error utility function
    let sensor_err = utils::sensor_error("i2c_error", "Bus failure");
    assert!(sensor_err.is_sensor_error());
    assert_eq!(sensor_err.error_code(), 1002);
    
    // Test network error utility function
    let network_err = utils::network_error("wifi_failed", "Connection lost");
    assert!(network_err.is_network_error());
    assert_eq!(network_err.error_code(), 2001);
    
    // Test invalid error types (should default)
    let default_sensor = utils::sensor_error("unknown_type", "Some error");
    assert_eq!(default_sensor.error_code(), 1001); // InitializationFailed
    
    let default_network = utils::network_error("unknown_type", "Some error");
    assert_eq!(default_network.error_code(), 2001); // WiFiConnectionFailed
}

#[test]
fn test_error_macro_functionality() {
    use iot_common::iot_error;
    
    let sensor_err = iot_error!(sensor, i2c_error, "I2C bus failure");
    assert!(sensor_err.is_sensor_error());
    assert_eq!(sensor_err.error_code(), 1002);
    
    let network_err = iot_error!(network, wifi_failed, "WiFi disconnected");
    assert!(network_err.is_network_error());
    assert_eq!(network_err.error_code(), 2001);
    
    let hardware_err = iot_error!(hardware, gpio_error, "GPIO pin failed");
    assert!(hardware_err.is_hardware_error());
    assert_eq!(hardware_err.error_code(), 3001);
}

#[test]
fn test_no_std_compatibility() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    // Verify no heap allocation - all operations should work in no_std
    let error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("Test")
    ));
    
    // Test cloning (should not require heap)
    let cloned_error = error.clone();
    assert_eq!(error.error_code(), cloned_error.error_code());
    
    // Test formatting (should not require heap)
    let error_string = format!("{}", error);
    assert!(error_string.contains("I2C communication error"));
    
    // Test debug formatting
    let debug_string = format!("{:?}", error);
    assert!(!debug_string.is_empty());
}

#[test]
fn test_error_type_checks() {
    use iot_common::{IoTError, SensorError, NetworkError, HardwareError, ConfigError, SystemError, error::utils};
    
    let sensor_err = IoTError::sensor(SensorError::I2CError(utils::error_message("test")));
    let network_err = IoTError::network(NetworkError::WiFiConnectionFailed(utils::error_message("test")));
    let hardware_err = IoTError::hardware(HardwareError::GPIOError(utils::error_message("test")));
    let config_err = IoTError::configuration(ConfigError::InvalidParameter(utils::error_message("test")));
    let system_err = IoTError::system(SystemError::OutOfMemory(utils::error_message("test")));
    
    // Test positive cases
    assert!(sensor_err.is_sensor_error());
    assert!(network_err.is_network_error());
    assert!(hardware_err.is_hardware_error());
    assert!(config_err.is_configuration_error());
    assert!(system_err.is_system_error());
    
    // Test negative cases (each error should not be other types)
    assert!(!sensor_err.is_network_error());
    assert!(!sensor_err.is_hardware_error());
    assert!(!sensor_err.is_configuration_error());
    assert!(!sensor_err.is_system_error());
    
    assert!(!network_err.is_sensor_error());
    assert!(!network_err.is_hardware_error());
    assert!(!network_err.is_configuration_error());
    assert!(!network_err.is_system_error());
}

#[test]
fn test_error_serialization_boundaries() {
    use iot_common::{IoTError, SensorError, error::utils, MAX_ERROR_MESSAGE_LEN};
    
    // Test that errors maintain consistent serialization size in embedded environment
    let errors = vec![
        IoTError::sensor(SensorError::I2CError(utils::error_message("short"))),
        IoTError::sensor(SensorError::I2CError(utils::error_message(&"x".repeat(MAX_ERROR_MESSAGE_LEN * 2)))),
        IoTError::sensor(SensorError::I2CError(utils::error_message(""))),
    ];
    
    for error in errors {
        // All error messages should respect the memory bounds
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
        
        // Error codes should be consistent
        assert_eq!(error.error_code(), 1002);
        assert_eq!(error.category(), "Sensor");
    }
}

#[test]
fn test_console_error_conversion() {
    use iot_common::error::console_conversions;
    
    let uart_error = console_conversions::from_uart_error("Buffer overflow");
    assert!(uart_error.is_hardware_error());
    assert_eq!(uart_error.error_code(), 3003); // UARTError
    
    let error_string = format!("{}", uart_error);
    assert!(error_string.contains("UART error"));
    assert!(error_string.contains("Buffer overflow"));
}

#[test]
fn test_error_chaining_deep() {
    use iot_common::{IoTError, SensorError, error::utils};
    
    // Test deep error context chaining
    let base_error = IoTError::sensor(SensorError::I2CError(
        utils::error_message("Hardware timeout")
    ));
    
    let chained_error = base_error
        .with_context("BME280 sensor read")
        .with_context("Temperature measurement")
        .with_context("Environmental monitoring")
        .with_context("Main application loop");
    
    // Should have all contexts up to the limit
    assert!(!chained_error.context().is_empty());
    let contexts = chained_error.context().contexts();
    assert!(contexts.len() <= iot_common::MAX_ERROR_CONTEXT_DEPTH);
    
    // Most recent context should be first
    if !contexts.is_empty() {
        assert_eq!(contexts[0].as_str(), "Main application loop");
    }
    
    // Error properties should be preserved
    assert!(chained_error.is_sensor_error());
    assert_eq!(chained_error.error_code(), 1002);
}