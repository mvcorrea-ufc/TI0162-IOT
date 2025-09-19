//! Testing Utilities and Mock Infrastructure
//!
//! This module provides mock implementations of hardware interfaces for testing
//! ESP32-C3 IoT system components without requiring physical hardware.
//!
//! ## Features
//!
//! - **I2C Mock**: Simulates I2C communication for sensor testing
//! - **UART Mock**: Simulates UART communication for console testing  
//! - **WiFi Mock**: Simulates WiFi network stack for connectivity testing
//! - **GPIO Mock**: Simulates GPIO pin operations
//! - **Timer Mock**: Simulates timer operations for async testing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iot_common::testing::{MockI2c, MockUart};
//!
//! // Create mock I2C for BME280 testing
//! let mut mock_i2c = MockI2c::new();
//! mock_i2c.expect_read_register(0xD0, 0x60); // BME280 chip ID
//!
//! // Use with BME280 driver
//! let mut sensor = BME280::new(&mut mock_i2c);
//! ```

#![cfg(feature = "testing")]

use crate::error::{IoTError, HardwareError};
use crate::error::utils;
use crate::result::IoTResult;
use heapless::{Vec, FnvIndexMap};
use core::fmt;

/// Maximum number of expected calls in mock objects
pub const MAX_MOCK_EXPECTATIONS: usize = 32;

/// Maximum number of stored values in mock memory
pub const MAX_MOCK_MEMORY: usize = 256;

/// Mock I2C interface for testing sensor communication
#[derive(Debug)]
pub struct MockI2c {
    /// Expected read operations (address, register, expected_value)
    read_expectations: Vec<(u8, u8, u8), MAX_MOCK_EXPECTATIONS>,
    /// Expected write operations (address, register, value)
    write_expectations: Vec<(u8, u8, u8), MAX_MOCK_EXPECTATIONS>,
    /// Expected multi-byte read operations (address, start_register, data)
    multi_read_expectations: Vec<(u8, u8, Vec<u8, 32>), MAX_MOCK_EXPECTATIONS>,
    /// Current device address
    device_address: u8,
    /// Simulated register memory
    memory: FnvIndexMap<u8, u8, MAX_MOCK_MEMORY>,
    /// Call counter for verification
    call_count: usize,
    /// Whether to simulate I2C errors
    should_error: bool,
    /// Error to return when should_error is true
    error_message: &'static str,
}

impl MockI2c {
    /// Create a new mock I2C interface
    pub fn new() -> Self {
        Self {
            read_expectations: Vec::new(),
            write_expectations: Vec::new(),
            multi_read_expectations: Vec::new(),
            device_address: 0x76, // Default BME280 address
            memory: FnvIndexMap::new(),
            call_count: 0,
            should_error: false,
            error_message: "Mock I2C error",
        }
    }

    /// Set the current device address
    pub fn set_address(&mut self, address: u8) {
        self.device_address = address;
    }

    /// Expect a register read operation
    pub fn expect_read_register(&mut self, register: u8, value: u8) -> IoTResult<()> {
        self.read_expectations
            .push((self.device_address, register, value))
            .map_err(|_| IoTError::system(crate::SystemError::OutOfMemory(
                utils::error_message("Too many I2C read expectations")
            )))
    }

    /// Expect a register write operation
    pub fn expect_write_register(&mut self, register: u8, value: u8) -> IoTResult<()> {
        self.write_expectations
            .push((self.device_address, register, value))
            .map_err(|_| IoTError::system(crate::SystemError::OutOfMemory(
                utils::error_message("Too many I2C write expectations")
            )))
    }

    /// Expect a multi-byte read operation
    pub fn expect_read_registers(&mut self, start_register: u8, data: &[u8]) -> IoTResult<()> {
        let mut vec_data = Vec::new();
        for &byte in data {
            vec_data.push(byte).map_err(|_| IoTError::system(crate::SystemError::OutOfMemory(
                utils::error_message("Mock data too large")
            )))?;
        }
        
        self.multi_read_expectations
            .push((self.device_address, start_register, vec_data))
            .map_err(|_| IoTError::system(crate::SystemError::OutOfMemory(
                utils::error_message("Too many I2C multi-read expectations")
            )))
    }

    /// Set register value in simulated memory
    pub fn set_register(&mut self, register: u8, value: u8) -> IoTResult<()> {
        self.memory
            .insert(register, value)
            .map_err(|_| IoTError::system(crate::SystemError::OutOfMemory(
                utils::error_message("Mock memory full")
            )))?;
        Ok(())
    }

    /// Configure mock to return errors
    pub fn set_error_mode(&mut self, should_error: bool, message: &'static str) {
        self.should_error = should_error;
        self.error_message = message;
    }

    /// Get number of calls made to this mock
    pub fn call_count(&self) -> usize {
        self.call_count
    }

    /// Verify all expectations were met
    pub fn verify(&self) -> IoTResult<()> {
        if !self.read_expectations.is_empty() || !self.write_expectations.is_empty() 
            || !self.multi_read_expectations.is_empty() {
            return Err(IoTError::system(crate::SystemError::InitializationFailed(
                utils::error_message("Unmet I2C expectations")
            )));
        }
        Ok(())
    }

    /// Simulate reading a register (mock implementation)
    pub async fn read_register(&mut self, register: u8) -> Result<u8, MockI2cError> {
        self.call_count += 1;

        if self.should_error {
            return Err(MockI2cError::Communication(self.error_message));
        }

        // Check if we have a specific expectation
        if let Some(pos) = self.read_expectations.iter().position(|(addr, reg, _)| {
            *addr == self.device_address && *reg == register
        }) {
            let (_, _, value) = self.read_expectations.swap_remove(pos);
            return Ok(value);
        }

        // Fall back to memory simulation
        if let Some(&value) = self.memory.get(&register) {
            Ok(value)
        } else {
            // Default behavior for common registers
            match register {
                0xD0 => Ok(0x60), // BME280 chip ID
                0xF3 => Ok(0x00), // BME280 status register (measurement complete)
                _ => Ok(0x00),    // Default value
            }
        }
    }

    /// Simulate writing a register (mock implementation)
    pub async fn write_register(&mut self, register: u8, value: u8) -> Result<(), MockI2cError> {
        self.call_count += 1;

        if self.should_error {
            return Err(MockI2cError::Communication(self.error_message));
        }

        // Check expectations
        if let Some(pos) = self.write_expectations.iter().position(|(addr, reg, val)| {
            *addr == self.device_address && *reg == register && *val == value
        }) {
            self.write_expectations.swap_remove(pos);
            return Ok(());
        }

        // Store in memory
        let _ = self.memory.insert(register, value);
        Ok(())
    }

    /// Simulate reading multiple registers (mock implementation)
    pub async fn read_registers(&mut self, start_register: u8, buffer: &mut [u8]) -> Result<(), MockI2cError> {
        self.call_count += 1;

        if self.should_error {
            return Err(MockI2cError::Communication(self.error_message));
        }

        // Check if we have a specific expectation
        if let Some(pos) = self.multi_read_expectations.iter().position(|(addr, reg, _)| {
            *addr == self.device_address && *reg == start_register
        }) {
            let (_, _, data) = self.multi_read_expectations.swap_remove(pos);
            let copy_len = buffer.len().min(data.len());
            buffer[..copy_len].copy_from_slice(&data[..copy_len]);
            return Ok(());
        }

        // Fall back to sequential register reads
        for (i, byte) in buffer.iter_mut().enumerate() {
            let register = start_register.wrapping_add(i as u8);
            if let Some(&value) = self.memory.get(&register) {
                *byte = value;
            } else {
                // Default patterns for BME280 calibration data
                *byte = match register {
                    0x88..=0x9F => (register as u16 * 123) as u8, // Temperature/pressure calibration
                    0xA1 => 75,                                    // Humidity H1
                    0xE1..=0xE7 => (register as u16 * 67) as u8,  // Humidity H2-H6
                    0xF7..=0xFE => 0x80,                          // Sensor data registers
                    _ => 0x00,
                };
            }
        }

        Ok(())
    }
}

impl Default for MockI2c {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock I2C error type
#[derive(Debug, Clone)]
pub enum MockI2cError {
    /// Communication error with message
    Communication(&'static str),
    /// Device not found
    DeviceNotFound,
    /// Invalid register address
    InvalidRegister(u8),
}

impl fmt::Display for MockI2cError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockI2cError::Communication(msg) => write!(f, "Mock I2C communication error: {}", msg),
            MockI2cError::DeviceNotFound => write!(f, "Mock I2C device not found"),
            MockI2cError::InvalidRegister(reg) => write!(f, "Mock I2C invalid register: 0x{:02X}", reg),
        }
    }
}

/// Mock UART interface for testing serial console
#[derive(Debug)]
pub struct MockUart {
    /// Input buffer (data to be read)
    input_buffer: Vec<u8, 256>,
    /// Output buffer (data that was written)
    output_buffer: Vec<u8, 256>,
    /// Read position in input buffer
    read_pos: usize,
    /// Whether to simulate UART errors
    should_error: bool,
    /// Error message for simulation
    error_message: &'static str,
}

impl MockUart {
    /// Create a new mock UART interface
    pub fn new() -> Self {
        Self {
            input_buffer: Vec::new(),
            output_buffer: Vec::new(),
            read_pos: 0,
            should_error: false,
            error_message: "Mock UART error",
        }
    }

    /// Add data to input buffer (simulates incoming data)
    pub fn add_input(&mut self, data: &[u8]) -> IoTResult<()> {
        for &byte in data {
            self.input_buffer.push(byte).map_err(|_| IoTError::hardware(HardwareError::UARTError(
                utils::error_message("Input buffer full")
            )))?;
        }
        Ok(())
    }

    /// Add string to input buffer
    pub fn add_input_str(&mut self, s: &str) -> IoTResult<()> {
        self.add_input(s.as_bytes())
    }

    /// Get output buffer contents (data that was written)
    pub fn get_output(&self) -> &[u8] {
        &self.output_buffer
    }

    /// Get output as string (if valid UTF-8)
    pub fn get_output_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(&self.output_buffer)
    }

    /// Clear output buffer
    pub fn clear_output(&mut self) {
        self.output_buffer.clear();
    }

    /// Configure mock to return errors
    pub fn set_error_mode(&mut self, should_error: bool, message: &'static str) {
        self.should_error = should_error;
        self.error_message = message;
    }

    /// Check if output contains specific text
    pub fn output_contains(&self, text: &str) -> bool {
        if let Ok(output_str) = self.get_output_str() {
            output_str.contains(text)
        } else {
            false
        }
    }

    /// Simulate reading from UART
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, MockUartError> {
        if self.should_error {
            return Err(MockUartError::Communication(self.error_message));
        }

        let available = self.input_buffer.len() - self.read_pos;
        if available == 0 {
            return Ok(0);
        }

        let to_read = buffer.len().min(available);
        let end_pos = self.read_pos + to_read;
        
        buffer[..to_read].copy_from_slice(&self.input_buffer[self.read_pos..end_pos]);
        self.read_pos = end_pos;

        Ok(to_read)
    }

    /// Simulate writing to UART
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, MockUartError> {
        if self.should_error {
            return Err(MockUartError::Communication(self.error_message));
        }

        let mut written = 0;
        for &byte in data {
            if self.output_buffer.push(byte).is_ok() {
                written += 1;
            } else {
                break; // Buffer full
            }
        }

        Ok(written)
    }

    /// Simulate writing string to UART
    pub async fn write_str(&mut self, s: &str) -> Result<usize, MockUartError> {
        self.write(s.as_bytes()).await
    }
}

impl Default for MockUart {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock UART error type
#[derive(Debug, Clone)]
pub enum MockUartError {
    /// Communication error
    Communication(&'static str),
    /// Buffer overflow
    BufferOverflow,
    /// No data available
    NoData,
}

impl fmt::Display for MockUartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockUartError::Communication(msg) => write!(f, "Mock UART communication error: {}", msg),
            MockUartError::BufferOverflow => write!(f, "Mock UART buffer overflow"),
            MockUartError::NoData => write!(f, "Mock UART no data available"),
        }
    }
}

/// Mock WiFi network stack for testing connectivity
#[derive(Debug)]
pub struct MockWiFiStack {
    /// Connection state
    is_connected: bool,
    /// IP address when connected
    ip_address: [u8; 4],
    /// Gateway address
    gateway: [u8; 4],
    /// Whether to simulate connection failures
    should_fail_connection: bool,
    /// Whether to simulate DHCP failures
    should_fail_dhcp: bool,
    /// Simulated SSID
    connected_ssid: heapless::String<32>,
    /// Connection attempt counter
    connection_attempts: usize,
}

impl MockWiFiStack {
    /// Create new mock WiFi stack
    pub fn new() -> Self {
        Self {
            is_connected: false,
            ip_address: [192, 168, 1, 100],
            gateway: [192, 168, 1, 1],
            should_fail_connection: false,
            should_fail_dhcp: false,
            connected_ssid: heapless::String::new(),
            connection_attempts: 0,
        }
    }

    /// Configure connection to succeed/fail
    pub fn set_connection_behavior(&mut self, should_fail: bool) {
        self.should_fail_connection = should_fail;
    }

    /// Configure DHCP to succeed/fail
    pub fn set_dhcp_behavior(&mut self, should_fail: bool) {
        self.should_fail_dhcp = should_fail;
    }

    /// Set the IP address to assign
    pub fn set_ip_address(&mut self, ip: [u8; 4]) {
        self.ip_address = ip;
    }

    /// Simulate WiFi connection
    pub async fn connect(&mut self, ssid: &str, _password: &str) -> Result<(), MockWiFiError> {
        self.connection_attempts += 1;

        if self.should_fail_connection {
            return Err(MockWiFiError::ConnectionFailed("Simulated connection failure"));
        }

        // Simulate connection delay
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;

        self.is_connected = true;
        let _ = self.connected_ssid.push_str(ssid);
        Ok(())
    }

    /// Simulate DHCP negotiation
    pub async fn configure_dhcp(&mut self) -> Result<(), MockWiFiError> {
        if !self.is_connected {
            return Err(MockWiFiError::NotConnected);
        }

        if self.should_fail_dhcp {
            return Err(MockWiFiError::DhcpFailed("Simulated DHCP failure"));
        }

        // Simulate DHCP delay
        embassy_time::Timer::after(embassy_time::Duration::from_millis(200)).await;

        Ok(())
    }

    /// Check if connected
    pub fn is_link_up(&self) -> bool {
        self.is_connected
    }

    /// Get IP configuration
    pub fn get_ip_config(&self) -> Option<MockIpConfig> {
        if self.is_connected && !self.should_fail_dhcp {
            Some(MockIpConfig {
                ip: self.ip_address,
                gateway: self.gateway,
                subnet_mask: [255, 255, 255, 0],
            })
        } else {
            None
        }
    }

    /// Get connection attempts count
    pub fn connection_attempts(&self) -> usize {
        self.connection_attempts
    }

    /// Get connected SSID
    pub fn connected_ssid(&self) -> &str {
        &self.connected_ssid
    }

    /// Simulate disconnection
    pub fn disconnect(&mut self) {
        self.is_connected = false;
        self.connected_ssid.clear();
    }
}

impl Default for MockWiFiStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock IP configuration
#[derive(Debug, Clone)]
pub struct MockIpConfig {
    /// IP address
    pub ip: [u8; 4],
    /// Gateway address
    pub gateway: [u8; 4],
    /// Subnet mask
    pub subnet_mask: [u8; 4],
}

/// Mock WiFi error type
#[derive(Debug, Clone)]
pub enum MockWiFiError {
    /// Connection failed
    ConnectionFailed(&'static str),
    /// DHCP failed
    DhcpFailed(&'static str),
    /// Not connected
    NotConnected,
    /// Hardware initialization failed
    HardwareInit(&'static str),
}

impl fmt::Display for MockWiFiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockWiFiError::ConnectionFailed(msg) => write!(f, "Mock WiFi connection failed: {}", msg),
            MockWiFiError::DhcpFailed(msg) => write!(f, "Mock DHCP failed: {}", msg),
            MockWiFiError::NotConnected => write!(f, "Mock WiFi not connected"),
            MockWiFiError::HardwareInit(msg) => write!(f, "Mock WiFi hardware init failed: {}", msg),
        }
    }
}

/// Mock MQTT client for testing message publishing
#[derive(Debug)]
pub struct MockMqttClient {
    /// Connection state
    is_connected: bool,
    /// Published messages (topic, payload)
    published_messages: Vec<(heapless::String<64>, heapless::String<256>), MAX_MOCK_EXPECTATIONS>,
    /// Whether to simulate connection failures
    should_fail_connection: bool,
    /// Whether to simulate publish failures
    should_fail_publish: bool,
    /// Broker host
    broker_host: heapless::String<64>,
    /// Broker port
    broker_port: u16,
}

impl MockMqttClient {
    /// Create new mock MQTT client
    pub fn new() -> Self {
        Self {
            is_connected: false,
            published_messages: Vec::new(),
            should_fail_connection: false,
            should_fail_publish: false,
            broker_host: heapless::String::new(),
            broker_port: 1883,
        }
    }

    /// Configure connection behavior
    pub fn set_connection_behavior(&mut self, should_fail: bool) {
        self.should_fail_connection = should_fail;
    }

    /// Configure publish behavior
    pub fn set_publish_behavior(&mut self, should_fail: bool) {
        self.should_fail_publish = should_fail;
    }

    /// Simulate MQTT connection
    pub async fn connect(&mut self, host: &str, port: u16) -> Result<(), MockMqttError> {
        if self.should_fail_connection {
            return Err(MockMqttError::ConnectionFailed("Simulated MQTT connection failure"));
        }

        // Simulate connection delay
        embassy_time::Timer::after(embassy_time::Duration::from_millis(150)).await;

        let _ = self.broker_host.push_str(host);
        self.broker_port = port;
        self.is_connected = true;
        Ok(())
    }

    /// Simulate message publishing
    pub async fn publish(&mut self, topic: &str, payload: &str) -> Result<(), MockMqttError> {
        if !self.is_connected {
            return Err(MockMqttError::NotConnected);
        }

        if self.should_fail_publish {
            return Err(MockMqttError::PublishFailed("Simulated publish failure"));
        }

        // Store published message
        let mut topic_string = heapless::String::new();
        let mut payload_string = heapless::String::new();
        
        let _ = topic_string.push_str(topic);
        let _ = payload_string.push_str(payload);

        self.published_messages
            .push((topic_string, payload_string))
            .map_err(|_| MockMqttError::BufferFull)?;

        // Simulate publish delay
        embassy_time::Timer::after(embassy_time::Duration::from_millis(50)).await;

        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Get published messages
    pub fn get_published_messages(&self) -> &[(heapless::String<64>, heapless::String<256>)] {
        &self.published_messages
    }

    /// Check if specific message was published
    pub fn was_published(&self, topic: &str, payload_contains: &str) -> bool {
        self.published_messages.iter().any(|(t, p)| {
            t.as_str() == topic && p.as_str().contains(payload_contains)
        })
    }

    /// Get connection info
    pub fn get_connection_info(&self) -> Option<(&str, u16)> {
        if self.is_connected {
            Some((self.broker_host.as_str(), self.broker_port))
        } else {
            None
        }
    }

    /// Simulate disconnection
    pub fn disconnect(&mut self) {
        self.is_connected = false;
        self.broker_host.clear();
        self.published_messages.clear();
    }
}

impl Default for MockMqttClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock MQTT error type
#[derive(Debug, Clone)]
pub enum MockMqttError {
    /// Connection failed
    ConnectionFailed(&'static str),
    /// Publish failed
    PublishFailed(&'static str),
    /// Not connected
    NotConnected,
    /// Buffer full
    BufferFull,
    /// Serialization error
    SerializationError(&'static str),
}

impl fmt::Display for MockMqttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MockMqttError::ConnectionFailed(msg) => write!(f, "Mock MQTT connection failed: {}", msg),
            MockMqttError::PublishFailed(msg) => write!(f, "Mock MQTT publish failed: {}", msg),
            MockMqttError::NotConnected => write!(f, "Mock MQTT not connected"),
            MockMqttError::BufferFull => write!(f, "Mock MQTT buffer full"),
            MockMqttError::SerializationError(msg) => write!(f, "Mock MQTT serialization error: {}", msg),
        }
    }
}

/// Test utilities for creating common mock scenarios
pub mod scenarios {
    use super::*;

    /// Create a properly configured BME280 mock I2C
    pub fn bme280_mock_i2c() -> MockI2c {
        let mut mock = MockI2c::new();
        
        // Set up BME280 chip ID response
        let _ = mock.set_register(0xD0, 0x60);
        
        // Set up status register (measurement complete)
        let _ = mock.set_register(0xF3, 0x00);
        
        // Set up calibration data for realistic responses
        let _ = mock.set_register(0x88, 0x6C); // dig_T1 LSB
        let _ = mock.set_register(0x89, 0x6C); // dig_T1 MSB
        // ... more calibration registers would be set here in a real scenario
        
        mock
    }

    /// Create a mock UART with pre-configured commands
    pub fn console_mock_uart() -> IoTResult<MockUart> {
        let mut mock = MockUart::new();
        
        // Add some common commands to input buffer
        mock.add_input_str("help\r\n")?;
        mock.add_input_str("status\r\n")?;
        mock.add_input_str("wifi config MySSID MyPassword\r\n")?;
        
        Ok(mock)
    }

    /// Create a successful WiFi mock
    pub fn successful_wifi_mock() -> MockWiFiStack {
        let mut mock = MockWiFiStack::new();
        mock.set_connection_behavior(false); // Don't fail
        mock.set_dhcp_behavior(false);       // Don't fail
        mock.set_ip_address([192, 168, 1, 100]);
        mock
    }

    /// Create a failing WiFi mock
    pub fn failing_wifi_mock() -> MockWiFiStack {
        let mut mock = MockWiFiStack::new();
        mock.set_connection_behavior(true); // Fail connection
        mock
    }

    /// Create a successful MQTT mock
    pub fn successful_mqtt_mock() -> MockMqttClient {
        let mut mock = MockMqttClient::new();
        mock.set_connection_behavior(false); // Don't fail
        mock.set_publish_behavior(false);    // Don't fail
        mock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_i2c_basic() {
        let mut mock = MockI2c::new();
        
        // Set a register value
        mock.set_register(0xD0, 0x60).unwrap();
        
        // Test reading it back (this would be async in real usage)
        // For now just verify the mock was set up correctly
        assert_eq!(mock.memory.get(&0xD0), Some(&0x60));
    }

    #[test]
    fn test_mock_uart_basic() {
        let mut mock = MockUart::new();
        
        // Add input data
        mock.add_input_str("test\r\n").unwrap();
        
        // Verify input was added
        assert_eq!(mock.input_buffer.len(), 6);
        assert_eq!(&mock.input_buffer[..], b"test\r\n");
    }

    #[test]
    fn test_scenarios() {
        // Test BME280 scenario setup
        let bme280_mock = scenarios::bme280_mock_i2c();
        assert_eq!(bme280_mock.memory.get(&0xD0), Some(&0x60));
        
        // Test console scenario setup
        let console_mock = scenarios::console_mock_uart().unwrap();
        assert!(!console_mock.input_buffer.is_empty());
        
        // Test WiFi scenarios
        let good_wifi = scenarios::successful_wifi_mock();
        assert!(!good_wifi.should_fail_connection);
        
        let bad_wifi = scenarios::failing_wifi_mock();
        assert!(bad_wifi.should_fail_connection);
    }
}