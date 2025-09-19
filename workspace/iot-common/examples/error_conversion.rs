//! # Error Conversion Example
//!
//! This example demonstrates how to convert from existing module-specific errors
//! to the unified IoT error system. It shows integration patterns for migrating
//! from current error types to the new unified system.

#![no_std]
#![no_main]

#[cfg(not(test))]
#[panic_handler] 
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

use iot_common::{
    IoTResult, IoTError, 
    error::{wifi_conversions, mqtt_conversions, console_conversions, utils}
};

/// Example: Converting from existing WiFi errors
/// This simulates the existing WiFiError enum from wifi-embassy
#[derive(Debug)]
pub enum LegacyWiFiError {
    HardwareInit(&'static str),
    Configuration(&'static str), 
    Connection(&'static str),
    Dhcp(&'static str),
}

impl core::fmt::Display for LegacyWiFiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LegacyWiFiError::HardwareInit(msg) => write!(f, "Hardware initialization failed: {}", msg),
            LegacyWiFiError::Configuration(msg) => write!(f, "WiFi configuration failed: {}", msg),
            LegacyWiFiError::Connection(msg) => write!(f, "WiFi connection failed: {}", msg),
            LegacyWiFiError::Dhcp(msg) => write!(f, "DHCP failed: {}", msg),
        }
    }
}

/// Convert legacy WiFi error to unified IoT error
impl From<LegacyWiFiError> for IoTError {
    fn from(error: LegacyWiFiError) -> Self {
        match error {
            LegacyWiFiError::HardwareInit(msg) => {
                wifi_conversions::from_wifi_error("HardwareInit", msg)
            },
            LegacyWiFiError::Configuration(msg) => {
                wifi_conversions::from_wifi_error("Configuration", msg)
            },
            LegacyWiFiError::Connection(msg) => {
                wifi_conversions::from_wifi_error("Connection", msg)
            },
            LegacyWiFiError::Dhcp(msg) => {
                wifi_conversions::from_wifi_error("Dhcp", msg)
            },
        }
    }
}

/// Example: Converting from existing MQTT errors
/// This simulates the existing MqttError enum from mqtt-embassy
#[derive(Debug)]
pub enum LegacyMqttError {
    ConnectionFailed(&'static str),
    ProtocolError(&'static str),
    IoError(&'static str),
    SerializationError(&'static str),
}

impl core::fmt::Display for LegacyMqttError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LegacyMqttError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            LegacyMqttError::ProtocolError(msg) => write!(f, "MQTT protocol error: {}", msg), 
            LegacyMqttError::IoError(msg) => write!(f, "I/O error: {}", msg),
            LegacyMqttError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

/// Convert legacy MQTT error to unified IoT error
impl From<LegacyMqttError> for IoTError {
    fn from(error: LegacyMqttError) -> Self {
        match error {
            LegacyMqttError::ConnectionFailed(msg) => {
                mqtt_conversions::from_mqtt_error("ConnectionFailed", msg)
            },
            LegacyMqttError::ProtocolError(msg) => {
                mqtt_conversions::from_mqtt_error("ProtocolError", msg)
            },
            LegacyMqttError::IoError(msg) => {
                mqtt_conversions::from_mqtt_error("IoError", msg)
            },
            LegacyMqttError::SerializationError(msg) => {
                mqtt_conversions::from_mqtt_error("SerializationError", msg)
            },
        }
    }
}

/// Example: Simulated I2C error (represents embedded-hal or esp-hal I2C errors)
#[derive(Debug)]
pub enum LegacyI2CError {
    BusError,
    NoAcknowledge,
    Timeout,
}

impl core::fmt::Display for LegacyI2CError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LegacyI2CError::BusError => write!(f, "I2C bus error"),
            LegacyI2CError::NoAcknowledge => write!(f, "I2C no acknowledge"),
            LegacyI2CError::Timeout => write!(f, "I2C timeout"),
        }
    }
}

/// Convert legacy I2C error to unified IoT error  
impl From<LegacyI2CError> for IoTError {
    fn from(error: LegacyI2CError) -> Self {
        let msg = match error {
            LegacyI2CError::BusError => "I2C bus error",
            LegacyI2CError::NoAcknowledge => "Device not responding (no ACK)",
            LegacyI2CError::Timeout => "I2C communication timeout",
        };
        utils::sensor_error("i2c_error", msg)
    }
}

/// Example function using legacy WiFi API that returns legacy errors
fn legacy_wifi_connect() -> Result<(), LegacyWiFiError> {
    // Simulate different failure modes
    let status = get_wifi_simulation_status();
    match status {
        0 => Ok(()),
        1 => Err(LegacyWiFiError::HardwareInit("WiFi chip not found")),
        2 => Err(LegacyWiFiError::Configuration("Invalid SSID")),
        3 => Err(LegacyWiFiError::Connection("Authentication failed")),
        _ => Err(LegacyWiFiError::Dhcp("DHCP server timeout")),
    }
}

/// Example function using legacy MQTT API
fn legacy_mqtt_connect() -> Result<(), LegacyMqttError> {
    let status = get_mqtt_simulation_status();
    match status {
        0 => Ok(()),
        1 => Err(LegacyMqttError::ConnectionFailed("TCP connect failed")),
        2 => Err(LegacyMqttError::ProtocolError("Invalid CONNACK")),
        3 => Err(LegacyMqttError::IoError("Socket write failed")),
        _ => Err(LegacyMqttError::SerializationError("JSON encode failed")),
    }
}

/// Example function using legacy I2C API
fn legacy_i2c_read() -> Result<u8, LegacyI2CError> {
    let status = get_i2c_simulation_status();
    match status {
        0 => Ok(0x42), // Success - return some data
        1 => Err(LegacyI2CError::BusError),
        2 => Err(LegacyI2CError::NoAcknowledge),
        _ => Err(LegacyI2CError::Timeout),
    }
}

/// Example: Gradual migration approach - wrapper functions
/// These functions provide the new unified error interface while using legacy APIs internally

/// Unified WiFi connection function
pub fn unified_wifi_connect() -> IoTResult<()> {
    legacy_wifi_connect()
        .map_err(|e| IoTError::from(e).with_context("WiFi connection establishment"))
}

/// Unified MQTT connection function  
pub fn unified_mqtt_connect() -> IoTResult<()> {
    legacy_mqtt_connect()
        .map_err(|e| IoTError::from(e).with_context("MQTT broker connection"))
}

/// Unified sensor reading function
pub fn unified_sensor_read() -> IoTResult<u8> {
    legacy_i2c_read()
        .map_err(|e| IoTError::from(e).with_context("BME280 sensor register read"))
}

/// Example: Complete system initialization with mixed legacy/unified APIs
pub fn mixed_system_init() -> IoTResult<()> {
    // Use unified functions that wrap legacy APIs
    unified_wifi_connect()?;
    unified_mqtt_connect()?;
    
    // Test sensor communication
    let _sensor_data = unified_sensor_read()?;
    
    Ok(())
}

/// Example: Error handling with conversion and context
pub fn advanced_error_handling() -> IoTResult<()> {
    // Try multiple operations and accumulate context
    unified_wifi_connect()
        .map_err(|e| e.with_context("Network layer initialization"))?;
    
    unified_mqtt_connect()
        .map_err(|e| e.with_context("Application layer setup"))?;
    
    // Retry logic with error context preservation
    for attempt in 1..=3 {
        match unified_sensor_read() {
            Ok(_) => break,
            Err(e) if attempt < 3 => {
                // Log retry attempt (in real code, use RTT)
                continue;
            },
            Err(e) => {
                return Err(e.with_context(&format!("Sensor init failed after {} attempts", attempt)));
            }
        }
    }
    
    Ok(())
}

/// Example: Error inspection and recovery strategies
pub fn error_recovery_demo() {
    match mixed_system_init() {
        Ok(()) => {
            // Success path
        },
        Err(e) => {
            // Inspect error details
            let error_code = e.error_code();
            let category = e.category();
            
            // Different recovery strategies based on error type
            if e.is_network_error() {
                // Network errors - maybe try different network
                handle_network_recovery();
            } else if e.is_sensor_error() {
                // Sensor errors - maybe use backup sensor or default values
                handle_sensor_recovery();
            } else {
                // Other errors - enter safe mode
                handle_system_recovery();
            }
            
            // Log error details (in real code, use RTT)
            log_error_details(category, error_code, &e);
        }
    }
}

fn handle_network_recovery() {
    // Implement network recovery logic
}

fn handle_sensor_recovery() {
    // Implement sensor recovery logic  
}

fn handle_system_recovery() {
    // Implement system recovery logic
}

fn log_error_details(category: &str, code: u16, error: &IoTError) {
    // In real embedded code, this would use RTT or UART
    let _ = (category, code, error); // Suppress unused warnings
}

/// Main function demonstrating error conversion patterns
#[no_mangle]
pub fn main() -> ! {
    // Run error recovery demonstration
    error_recovery_demo();
    
    loop {
        // Main application loop
        match advanced_error_handling() {
            Ok(()) => {
                // Normal operation
            },
            Err(_e) => {
                // Handle error and continue
            }
        }
        
        // Delay in real application
        cortex_m::asm::delay(1000);
    }
}

// Simulation functions
fn get_wifi_simulation_status() -> u8 { 0 }
fn get_mqtt_simulation_status() -> u8 { 0 }
fn get_i2c_simulation_status() -> u8 { 0 }