//! # Mock Implementations for IoT Container Testing
//!
//! This module provides comprehensive mock implementations of all IoT system components.
//! These mocks enable thorough testing of the dependency injection container and
//! application logic without requiring real hardware.

use async_trait::async_trait;
use alloc::{boxed::Box, vec::Vec, string::String};
use heapless::{Deque, Vec as HeaplessVec};
use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};

use iot_common::{IoTError, IoTResult};
use iot_hal::{HardwarePlatform, I2cInterface, UartTxInterface, UartRxInterface, GpioInterface, TimerInterface, WiFiInterface};

use crate::traits::{
    SensorReader, NetworkManager, MessagePublisher, ConsoleInterface,
    Measurements, ConnectionInfo, SensorData, DeviceStatus, EmbeddedString
};
use crate::config::{SensorConfig, WiFiConfig, MqttConfig, ConsoleConfig};

/// Mock sensor reader for testing
/// 
/// Provides controllable sensor behavior for testing various scenarios including
/// normal operation, sensor failures, and edge cases.
pub struct MockSensorReader {
    /// Predefined measurements to return
    measurements: Deque<Measurements, 16>,
    
    /// Whether sensor should report as available
    available: AtomicBool,
    
    /// Whether next operation should fail
    should_fail: AtomicBool,
    
    /// Number of measurements read
    read_count: AtomicU32,
    
    /// Sensor type identifier
    sensor_type: &'static str,
    
    /// Last measurement timestamp
    last_measurement_time: Option<u64>,
    
    /// Whether initialization was called
    initialized: AtomicBool,
}

impl MockSensorReader {
    /// Creates a new mock sensor with default test data
    pub fn new() -> Self {
        let mut measurements = Deque::new();
        
        // Add some default test measurements
        let _ = measurements.push_back(Measurements::new(23.5, 1013.25, 65.0));
        let _ = measurements.push_back(Measurements::new(24.0, 1012.80, 66.5));
        let _ = measurements.push_back(Measurements::new(23.8, 1013.10, 64.8));
        
        Self {
            measurements,
            available: AtomicBool::new(true),
            should_fail: AtomicBool::new(false),
            read_count: AtomicU32::new(0),
            sensor_type: "MOCK_BME280",
            last_measurement_time: None,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Creates a mock sensor with configuration
    pub fn new_with_config(config: &SensorConfig) -> Self {
        let mut sensor = Self::new();
        sensor.sensor_type = match config.sensor_type.as_str() {
            "BME280" => "MOCK_BME280",
            "SHT30" => "MOCK_SHT30",
            _ => "MOCK_UNKNOWN",
        };
        sensor
    }
    
    /// Adds a measurement to the mock data queue
    pub fn add_measurement(&mut self, measurement: Measurements) {
        if self.measurements.len() >= 16 {
            self.measurements.pop_front();
        }
        let _ = self.measurements.push_back(measurement);
    }
    
    /// Sets whether the sensor should report as available
    pub fn set_available(&self, available: bool) {
        self.available.store(available, Ordering::Relaxed);
    }
    
    /// Sets whether the next operation should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        self.should_fail.store(should_fail, Ordering::Relaxed);
    }
    
    /// Gets the number of measurements read
    pub fn get_read_count(&self) -> u32 {
        self.read_count.load(Ordering::Relaxed)
    }
    
    /// Checks if sensor was initialized
    pub fn was_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl SensorReader for MockSensorReader {
    async fn read_measurements(&mut self) -> Result<Measurements, IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Sensor(iot_common::SensorError::CommunicationFailed("Mock failure")));
        }
        
        if !self.available.load(Ordering::Relaxed) {
            return Err(IoTError::Sensor(iot_common::SensorError::NotAvailable("Mock sensor unavailable")));
        }
        
        let measurement = self.measurements.pop_front()
            .unwrap_or_else(|| Measurements::new(25.0, 1013.0, 60.0));
        
        self.read_count.fetch_add(1, Ordering::Relaxed);
        self.last_measurement_time = Some(embassy_time::Instant::now().as_millis());
        
        Ok(measurement)
    }
    
    async fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
    
    async fn initialize(&mut self) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Sensor(iot_common::SensorError::InitializationFailed("Mock initialization failure")));
        }
        
        self.initialized.store(true, Ordering::Relaxed);
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await; // Simulate init time
        Ok(())
    }
    
    fn get_sensor_type(&self) -> &'static str {
        self.sensor_type
    }
    
    fn get_last_measurement_time(&self) -> Option<u64> {
        self.last_measurement_time
    }
    
    async fn self_test(&mut self) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Sensor(iot_common::SensorError::SelfTestFailed("Mock self-test failure")));
        }
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(5)).await; // Simulate test time
        Ok(())
    }
}

/// Mock network manager for testing
/// 
/// Provides controllable network behavior for testing connectivity scenarios.
pub struct MockNetworkManager {
    /// Whether network should report as connected
    connected: AtomicBool,
    
    /// Whether next operation should fail
    should_fail: AtomicBool,
    
    /// Connection attempt count
    connection_attempts: AtomicU32,
    
    /// Mock connection info
    connection_info: Option<ConnectionInfo>,
    
    /// Mock signal strength
    signal_strength: i8,
}

impl MockNetworkManager {
    /// Creates a new mock network manager
    pub fn new() -> Self {
        let connection_info = ConnectionInfo::new("192.168.1.100").ok();
        
        Self {
            connected: AtomicBool::new(false),
            should_fail: AtomicBool::new(false),
            connection_attempts: AtomicU32::new(0),
            connection_info,
            signal_strength: -45,
        }
    }
    
    /// Creates a mock network manager with configuration
    pub fn new_with_config(config: &WiFiConfig) -> Self {
        let mut manager = Self::new();
        
        // Configure based on WiFi config
        manager.signal_strength = config.min_signal_strength_dbm;
        
        manager
    }
    
    /// Sets whether the network should report as connected
    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
    }
    
    /// Sets whether the next operation should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        self.should_fail.store(should_fail, Ordering::Relaxed);
    }
    
    /// Gets the number of connection attempts
    pub fn get_connection_attempts(&self) -> u32 {
        self.connection_attempts.load(Ordering::Relaxed)
    }
    
    /// Sets the mock signal strength
    pub fn set_signal_strength(&mut self, strength: i8) {
        self.signal_strength = strength;
    }
}

#[async_trait]
impl NetworkManager for MockNetworkManager {
    async fn connect(&mut self) -> Result<(), IoTError> {
        self.connection_attempts.fetch_add(1, Ordering::Relaxed);
        
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Network(iot_common::NetworkError::ConnectionFailed("Mock connection failure")));
        }
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await; // Simulate connection time
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), IoTError> {
        self.connected.store(false, Ordering::Relaxed);
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
    
    async fn get_connection_info(&self) -> Option<ConnectionInfo> {
        if self.connected.load(Ordering::Relaxed) {
            self.connection_info.clone()
        } else {
            None
        }
    }
    
    async fn get_signal_strength(&self) -> Option<i8> {
        if self.connected.load(Ordering::Relaxed) {
            Some(self.signal_strength)
        } else {
            None
        }
    }
    
    async fn test_connectivity(&self) -> Result<(), IoTError> {
        if self.connected.load(Ordering::Relaxed) {
            Ok(())
        } else {
            Err(IoTError::Network(iot_common::NetworkError::ConnectionLost("Mock connectivity test failed")))
        }
    }
    
    fn get_stack(&self) -> &'static embassy_net::Stack<embassy_net::driver::Driver<'static>> {
        // This is a placeholder - in real testing, you'd need a mock stack
        // For now, we'll panic if this is called in mock mode
        panic!("Mock network stack not implemented - should not be called in tests")
    }
}

/// Mock message publisher for testing
/// 
/// Provides controllable message publishing behavior for testing communication scenarios.
pub struct MockMessagePublisher {
    /// Whether publisher should report as connected
    connected: AtomicBool,
    
    /// Whether next operation should fail
    should_fail: AtomicBool,
    
    /// Published sensor data messages
    published_sensor_data: Vec<SensorData>,
    
    /// Published status messages
    published_status: Vec<DeviceStatus>,
    
    /// Published heartbeat count
    heartbeat_count: AtomicU32,
    
    /// Message send count
    message_count: AtomicU32,
    
    /// Error count
    error_count: AtomicU32,
    
    /// Start time for metrics
    start_time: embassy_time::Instant,
}

impl MockMessagePublisher {
    /// Creates a new mock message publisher
    pub fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            should_fail: AtomicBool::new(false),
            published_sensor_data: Vec::new(),
            published_status: Vec::new(),
            heartbeat_count: AtomicU32::new(0),
            message_count: AtomicU32::new(0),
            error_count: AtomicU32::new(0),
            start_time: embassy_time::Instant::now(),
        }
    }
    
    /// Creates a mock message publisher with configuration
    pub fn new_with_config(config: &MqttConfig) -> Self {
        let mut publisher = Self::new();
        // Configure based on MQTT config if needed
        publisher
    }
    
    /// Sets whether the publisher should report as connected
    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
    }
    
    /// Sets whether the next operation should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        self.should_fail.store(should_fail, Ordering::Relaxed);
    }
    
    /// Gets the published sensor data messages
    pub fn get_published_sensor_data(&self) -> &Vec<SensorData> {
        &self.published_sensor_data
    }
    
    /// Gets the published status messages
    pub fn get_published_status(&self) -> &Vec<DeviceStatus> {
        &self.published_status
    }
    
    /// Gets the heartbeat count
    pub fn get_heartbeat_count(&self) -> u32 {
        self.heartbeat_count.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl MessagePublisher for MockMessagePublisher {
    async fn publish_sensor_data(&mut self, data: &SensorData) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::PublishFailed("Mock publish failure")));
        }
        
        if !self.connected.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("Mock publisher not connected")));
        }
        
        self.published_sensor_data.push(data.clone());
        self.message_count.fetch_add(1, Ordering::Relaxed);
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await; // Simulate publish time
        Ok(())
    }
    
    async fn publish_status(&mut self, status: &DeviceStatus) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::PublishFailed("Mock status publish failure")));
        }
        
        if !self.connected.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("Mock publisher not connected")));
        }
        
        self.published_status.push(status.clone());
        self.message_count.fetch_add(1, Ordering::Relaxed);
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await; // Simulate publish time
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
    
    async fn connect(&mut self) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Network(iot_common::NetworkError::ConnectionFailed("Mock connection failure")));
        }
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(50)).await; // Simulate connection time
        self.connected.store(true, Ordering::Relaxed);
        Ok(())
    }
    
    async fn publish_heartbeat(&mut self) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::PublishFailed("Mock heartbeat failure")));
        }
        
        if !self.connected.load(Ordering::Relaxed) {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("Mock publisher not connected")));
        }
        
        self.heartbeat_count.fetch_add(1, Ordering::Relaxed);
        self.message_count.fetch_add(1, Ordering::Relaxed);
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(5)).await; // Simulate heartbeat time
        Ok(())
    }
    
    fn get_metrics(&self) -> (u32, u32, u32) {
        let uptime = self.start_time.elapsed().as_secs() as u32;
        (
            self.message_count.load(Ordering::Relaxed),
            self.error_count.load(Ordering::Relaxed),
            uptime
        )
    }
}

/// Mock console interface for testing
/// 
/// Provides controllable console behavior for testing user interaction scenarios.
pub struct MockConsoleInterface {
    /// Whether console should report as ready
    ready: AtomicBool,
    
    /// Whether next operation should fail
    should_fail: AtomicBool,
    
    /// Queue of commands to return from read_command
    command_queue: Deque<EmbeddedString, 8>,
    
    /// Written output lines
    output_lines: Vec<String>,
    
    /// Session start time
    session_start: embassy_time::Instant,
}

impl MockConsoleInterface {
    /// Creates a new mock console interface
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(true),
            should_fail: AtomicBool::new(false),
            command_queue: Deque::new(),
            output_lines: Vec::new(),
            session_start: embassy_time::Instant::now(),
        }
    }
    
    /// Creates a mock console interface with configuration
    pub fn new_with_config(config: &ConsoleConfig) -> Self {
        let mut console = Self::new();
        // Configure based on console config if needed
        console
    }
    
    /// Sets whether the console should report as ready
    pub fn set_ready(&self, ready: bool) {
        self.ready.store(ready, Ordering::Relaxed);
    }
    
    /// Sets whether the next operation should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        self.should_fail.store(should_fail, Ordering::Relaxed);
    }
    
    /// Adds a command to the command queue
    pub fn add_command(&mut self, command: &str) -> Result<(), IoTError> {
        let cmd = EmbeddedString::try_from(command).map_err(|_| {
            IoTError::configuration(iot_common::ConfigError::ValidationError("Command too long".try_into().unwrap_or_default()))
        })?;
        
        if self.command_queue.len() >= 8 {
            self.command_queue.pop_front();
        }
        let _ = self.command_queue.push_back(cmd);
        Ok(())
    }
    
    /// Gets the written output lines
    pub fn get_output_lines(&self) -> &Vec<String> {
        &self.output_lines
    }
    
    /// Clears the output lines
    pub fn clear_output(&mut self) {
        self.output_lines.clear();
    }
}

#[async_trait]
impl ConsoleInterface for MockConsoleInterface {
    async fn write_line(&mut self, message: &str) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Hardware(iot_common::HardwareError::InterfaceError("Mock write failure")));
        }
        
        self.output_lines.push(message.to_string());
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await; // Simulate write time
        Ok(())
    }
    
    async fn read_command(&mut self) -> Result<Option<EmbeddedString>, IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Hardware(iot_common::HardwareError::InterfaceError("Mock read failure")));
        }
        
        Ok(self.command_queue.pop_front())
    }
    
    async fn handle_command(&mut self, command: &str) -> Result<EmbeddedString, IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::System(iot_common::SystemError::InvalidOperation("Mock command handling failure")));
        }
        
        let response = match command.trim() {
            "help" => "Mock help: Available commands: help, status, info, test",
            "status" => "Mock status: System operational",
            "info" => "Mock info: ESP32-C3 IoT Test System v1.0",
            "test" => "Mock test: All subsystems OK",
            _ => "Mock: Unknown command",
        };
        
        EmbeddedString::try_from(response).map_err(|_| {
            IoTError::configuration(iot_common::ConfigError::ValidationError("Response too long".try_into().unwrap_or_default()))
        })
    }
    
    async fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }
    
    fn get_session_info(&self) -> EmbeddedString {
        let uptime = self.session_start.elapsed().as_secs();
        EmbeddedString::try_from(&format!("Mock session uptime: {}s", uptime)[..])
            .unwrap_or_else(|_| EmbeddedString::try_from("Mock session active").unwrap())
    }
    
    async fn show_prompt(&mut self) -> Result<(), IoTError> {
        if self.should_fail.load(Ordering::Relaxed) {
            return Err(IoTError::Hardware(iot_common::HardwareError::InterfaceError("Mock prompt failure")));
        }
        
        self.output_lines.push("mock> ".to_string());
        Ok(())
    }
}

/// Mock hardware platform for testing
/// 
/// Provides a complete mock hardware platform implementation for testing the
/// dependency injection container without real hardware dependencies.
pub struct MockPlatform {
    /// Mock I2C interface
    i2c: MockI2c,
    
    /// Mock UART TX interface
    uart_tx: MockUartTx,
    
    /// Mock UART RX interface
    uart_rx: MockUartRx,
    
    /// Mock GPIO interface
    gpio: MockGpio,
    
    /// Mock timer interface
    timer: MockTimer,
    
    /// Mock WiFi interface
    wifi: MockWifi,
}

impl MockPlatform {
    /// Creates a new mock platform
    pub fn new() -> Self {
        Self {
            i2c: MockI2c::new(),
            uart_tx: MockUartTx::new(),
            uart_rx: MockUartRx::new(),
            gpio: MockGpio::new(),
            timer: MockTimer::new(),
            wifi: MockWifi::new(),
        }
    }
}

#[async_trait]
impl HardwarePlatform for MockPlatform {
    type I2c = MockI2c;
    type UartTx = MockUartTx;
    type UartRx = MockUartRx;
    type Gpio = MockGpio;
    type Timer = MockTimer;
    type Wifi = MockWifi;
    
    async fn initialize() -> Result<Self, IoTError> {
        Ok(Self::new())
    }
    
    async fn initialize_with_config(config: iot_hal::HardwareConfig) -> Result<Self, IoTError> {
        Ok(Self::new())
    }
    
    fn get_i2c(&mut self) -> &mut Self::I2c {
        &mut self.i2c
    }
    
    fn get_console(&mut self) -> (&mut Self::UartTx, &mut Self::UartRx) {
        (&mut self.uart_tx, &mut self.uart_rx)
    }
    
    fn get_uart(&mut self) -> (&mut Self::UartTx, &mut Self::UartRx) {
        (&mut self.uart_tx, &mut self.uart_rx)
    }
    
    fn get_status_led(&mut self) -> &mut Self::Gpio {
        &mut self.gpio
    }
    
    fn get_timer(&mut self) -> &mut Self::Timer {
        &mut self.timer
    }
    
    fn get_wifi(&mut self) -> &mut Self::Wifi {
        &mut self.wifi
    }
    
    fn get_spawner(&self) -> embassy_executor::Spawner {
        // This is a placeholder - real implementation would need proper spawner
        // For testing, we'll use a dummy implementation
        todo!("Mock spawner not implemented")
    }
    
    fn get_rng(&mut self) -> &mut dyn iot_hal::RngInterface {
        todo!("Mock RNG not implemented")
    }
}

// Mock hardware interface implementations
// These are simplified implementations for testing purposes

pub struct MockI2c;
impl MockI2c {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl I2cInterface for MockI2c {
    async fn write(&mut self, address: u8, data: &[u8]) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
        Ok(())
    }
    
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
        // Fill with mock data
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (address + i as u8) % 256;
        }
        Ok(())
    }
    
    async fn write_read(&mut self, address: u8, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), IoTError> {
        self.write(address, write_data).await?;
        self.read(address, read_buffer).await?;
        Ok(())
    }
}

pub struct MockUartTx;
impl MockUartTx {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl UartTxInterface for MockUartTx {
    async fn write(&mut self, data: &[u8]) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
        Ok(())
    }
    
    async fn flush(&mut self) -> Result<(), IoTError> {
        Ok(())
    }
}

pub struct MockUartRx;
impl MockUartRx {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl UartRxInterface for MockUartRx {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
        Ok(0) // No data available
    }
}

pub struct MockGpio;
impl MockGpio {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl GpioInterface for MockGpio {
    async fn set_high(&mut self) -> Result<(), IoTError> {
        Ok(())
    }
    
    async fn set_low(&mut self) -> Result<(), IoTError> {
        Ok(())
    }
    
    async fn is_high(&self) -> Result<bool, IoTError> {
        Ok(false)
    }
}

pub struct MockTimer;
impl MockTimer {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl TimerInterface for MockTimer {
    async fn delay_ms(&mut self, duration_ms: u32) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(duration_ms as u64)).await;
        Ok(())
    }
    
    async fn delay_us(&mut self, duration_us: u32) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_micros(duration_us as u64)).await;
        Ok(())
    }
}

pub struct MockWifi;
impl MockWifi {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl WiFiInterface for MockWifi {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), IoTError> {
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), IoTError> {
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        true
    }
    
    async fn get_connection_info(&self) -> Option<iot_hal::WiFiConnectionInfo> {
        Some(iot_hal::WiFiConnectionInfo {
            ip_address: heapless::String::try_from("192.168.1.100").unwrap(),
            gateway: None,
            subnet_mask: None,
            dns_servers: heapless::Vec::new(),
            signal_strength_dbm: Some(-45),
            ssid: Some(heapless::String::try_from("MockNetwork").unwrap()),
        })
    }
}