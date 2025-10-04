//! # Mock Hardware Platform Implementation
//!
//! Mock implementation of hardware abstraction traits for testing.
//! Provides simulated hardware behavior without requiring actual hardware.

#[cfg(feature = "mock")]
use crate::{
    HardwarePlatform, I2cInterface, UartTxInterface, UartRxInterface, 
    GpioInterface, TimerInterface, WiFiInterface, WiFiConnectionInfo,
    HardwareConfig, error::*
};
#[cfg(feature = "mock")]
use iot_common::IoTError;
#[cfg(feature = "mock")]
use embassy_time::{Duration, Instant};
#[cfg(feature = "mock")]
use core::net::IpAddr;
#[cfg(feature = "mock")]
use alloc::{vec::Vec, collections::VecDeque, string::String, sync::Arc};
#[cfg(feature = "mock")]
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicI8, Ordering};
#[cfg(feature = "mock")]
use async_trait::async_trait;

#[cfg(feature = "mock")]
/// Mock hardware platform for testing
/// 
/// Provides complete simulation of hardware interfaces without requiring
/// actual hardware. Enables comprehensive unit testing of IoT applications
/// and validation of hardware abstraction layer implementations.
/// 
/// # Testing Capabilities
/// 
/// - **I2C Simulation**: Configurable device responses, error injection
/// - **UART Simulation**: Bidirectional communication with buffering
/// - **GPIO Simulation**: Pin state tracking and validation
/// - **WiFi Simulation**: Network connection simulation with configurable behavior
/// - **Timer Simulation**: Accelerated or real-time delay simulation
/// 
/// # Error Injection
/// 
/// The mock platform supports error injection for testing error handling:
/// 
/// ```rust
/// let mut mock = MockPlatform::initialize().await?;
/// 
/// // Configure I2C device to fail next read
/// mock.i2c_mut().set_next_error(IoTError::Hardware(HardwareError::I2CError("Simulated failure".into())));
/// 
/// // Test error handling
/// let result = mock.get_i2c().read(0x76, &mut buffer).await;
/// assert!(result.is_err());
/// ```
/// 
/// # State Verification
/// 
/// Mock platform tracks all operations for verification:
/// 
/// ```rust
/// let mut mock = MockPlatform::initialize().await?;
/// 
/// // Perform operations
/// mock.get_i2c().write(0x76, &[0x01, 0x02]).await?;
/// mock.get_status_led().set_high().await?;
/// 
/// // Verify operations
/// assert_eq!(mock.i2c_ref().write_history().len(), 1);
/// assert!(mock.gpio_ref().is_led_on());
/// ```
pub struct MockPlatform {
    /// Mock I2C interface
    i2c: MockI2c,
    
    /// Mock UART transmitter
    uart_tx: MockUartTx,
    
    /// Mock UART receiver
    uart_rx: MockUartRx,
    
    /// Mock GPIO interface
    gpio: MockGpio,
    
    /// Mock timer interface
    timer: MockTimer,
    
    /// Mock WiFi interface
    wifi: MockWiFi,
    
    /// Platform configuration
    config: HardwareConfig,
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl HardwarePlatform for MockPlatform {
    type I2cBus = MockI2c;
    type UartTx = MockUartTx;
    type UartRx = MockUartRx;
    type GpioPin = MockGpio;
    type Timer = MockTimer;
    type WiFi = MockWiFi;

    async fn initialize() -> Result<Self, IoTError> {
        Self::initialize_with_config(HardwareConfig::default()).await
    }

    async fn initialize_with_config(config: HardwareConfig) -> Result<Self, IoTError> {
        // Validate configuration
        config.validate().map_err(|e| PlatformError::InvalidConfiguration(e))?;

        Ok(Self {
            i2c: MockI2c::new(),
            uart_tx: MockUartTx::new(),
            uart_rx: MockUartRx::new(),
            gpio: MockGpio::new(config.gpio.status_led_active_high),
            timer: MockTimer::new(),
            wifi: MockWiFi::new(),
            config,
        })
    }

    fn get_i2c(&mut self) -> &mut Self::I2cBus {
        &mut self.i2c
    }

    fn get_console(&mut self) -> (&mut Self::UartTx, &mut Self::UartRx) {
        (&mut self.uart_tx, &mut self.uart_rx)
    }

    fn get_status_led(&mut self) -> &mut Self::GpioPin {
        &mut self.gpio
    }

    fn get_timer(&mut self) -> &mut Self::Timer {
        &mut self.timer
    }

    fn get_wifi(&mut self) -> &mut Self::WiFi {
        &mut self.wifi
    }

    async fn is_healthy(&mut self) -> bool {
        // Mock platform is always healthy unless explicitly configured otherwise
        self.i2c.is_healthy && 
        self.uart_tx.is_healthy && 
        self.uart_rx.is_healthy && 
        self.gpio.is_healthy && 
        self.wifi.is_healthy
    }

    fn platform_info(&self) -> &'static str {
        "Mock Hardware Platform for Testing"
    }
}

#[cfg(feature = "mock")]
impl MockPlatform {
    /// Get immutable reference to I2C mock for verification
    pub fn i2c_ref(&self) -> &MockI2c {
        &self.i2c
    }

    /// Get mutable reference to I2C mock for configuration
    pub fn i2c_mut(&mut self) -> &mut MockI2c {
        &mut self.i2c
    }

    /// Get immutable reference to UART TX mock for verification
    pub fn uart_tx_ref(&self) -> &MockUartTx {
        &self.uart_tx
    }

    /// Get mutable reference to UART TX mock for configuration
    pub fn uart_tx_mut(&mut self) -> &mut MockUartTx {
        &mut self.uart_tx
    }

    /// Get immutable reference to UART RX mock for verification
    pub fn uart_rx_ref(&self) -> &MockUartRx {
        &self.uart_rx
    }

    /// Get mutable reference to UART RX mock for configuration
    pub fn uart_rx_mut(&mut self) -> &mut MockUartRx {
        &mut self.uart_rx
    }

    /// Get immutable reference to GPIO mock for verification
    pub fn gpio_ref(&self) -> &MockGpio {
        &self.gpio
    }

    /// Get mutable reference to GPIO mock for configuration
    pub fn gpio_mut(&mut self) -> &mut MockGpio {
        &mut self.gpio
    }

    /// Get immutable reference to WiFi mock for verification
    pub fn wifi_ref(&self) -> &MockWiFi {
        &self.wifi
    }

    /// Get mutable reference to WiFi mock for configuration
    pub fn wifi_mut(&mut self) -> &mut MockWiFi {
        &mut self.wifi
    }

    /// Reset all mock interfaces to default state
    pub fn reset_all(&mut self) {
        self.i2c.reset();
        self.uart_tx.reset();
        self.uart_rx.reset();
        self.gpio.reset();
        self.timer.reset();
        self.wifi.reset();
    }

    /// Set global health status for all interfaces
    pub fn set_global_health(&mut self, healthy: bool) {
        self.i2c.is_healthy = healthy;
        self.uart_tx.is_healthy = healthy;
        self.uart_rx.is_healthy = healthy;
        self.gpio.is_healthy = healthy;
        self.wifi.is_healthy = healthy;
    }
}

#[cfg(feature = "mock")]
/// Mock I2C interface with configurable behavior
#[derive(Debug)]
pub struct MockI2c {
    /// Device register maps (address -> registers)
    devices: std::collections::HashMap<u8, std::collections::HashMap<u8, u8>>,
    
    /// Read history for verification
    read_history: Vec<(u8, usize)>, // (address, bytes_read)
    
    /// Write history for verification
    write_history: Vec<(u8, Vec<u8>)>, // (address, data)
    
    /// Next error to inject
    next_error: Option<IoTError>,
    
    /// Health status
    is_healthy: bool,
}

#[cfg(feature = "mock")]
impl MockI2c {
    fn new() -> Self {
        Self {
            devices: std::collections::HashMap::new(),
            read_history: Vec::new(),
            write_history: Vec::new(),
            next_error: None,
            is_healthy: true,
        }
    }

    /// Add a mock I2C device with register map
    pub fn add_device(&mut self, address: u8, registers: std::collections::HashMap<u8, u8>) {
        self.devices.insert(address, registers);
    }

    /// Add a simple mock device with initial register values
    pub fn add_simple_device(&mut self, address: u8, register_values: &[(u8, u8)]) {
        let mut registers = std::collections::HashMap::new();
        for &(reg, val) in register_values {
            registers.insert(reg, val);
        }
        self.devices.insert(address, registers);
    }

    /// Set the next operation to fail with specified error
    pub fn set_next_error(&mut self, error: IoTError) {
        self.next_error = Some(error);
    }

    /// Get read operation history
    pub fn read_history(&self) -> &[(u8, usize)] {
        &self.read_history
    }

    /// Get write operation history
    pub fn write_history(&self) -> &[(u8, Vec<u8>)] {
        &self.write_history
    }

    /// Clear operation history
    pub fn clear_history(&mut self) {
        self.read_history.clear();
        self.write_history.clear();
    }

    /// Reset mock to initial state
    pub fn reset(&mut self) {
        self.devices.clear();
        self.clear_history();
        self.next_error = None;
        self.is_healthy = true;
    }

    /// Check if error should be injected
    fn check_error(&mut self) -> Result<(), IoTError> {
        if let Some(error) = self.next_error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl I2cInterface for MockI2c {
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), IoTError> {
        self.check_error()?;

        if let Some(device) = self.devices.get(&address) {
            // Simple simulation: fill buffer with register values starting from 0
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = device.get(&(i as u8)).copied().unwrap_or(0);
            }
            self.read_history.push((address, buffer.len()));
            Ok(())
        } else {
            Err(I2cError::DeviceNotResponding(address).into())
        }
    }

    async fn write(&mut self, address: u8, data: &[u8]) -> Result<(), IoTError> {
        self.check_error()?;

        if self.devices.contains_key(&address) {
            self.write_history.push((address, data.to_vec()));
            
            // Update device registers if write contains register address
            if data.len() >= 2 {
                let device = self.devices.get_mut(&address).unwrap();
                let reg_addr = data[0];
                let reg_value = data[1];
                device.insert(reg_addr, reg_value);
            }
            
            Ok(())
        } else {
            Err(I2cError::DeviceNotResponding(address).into())
        }
    }

    async fn write_read(&mut self, address: u8, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), IoTError> {
        self.check_error()?;

        // First perform write
        self.write(address, write_data).await?;
        
        // Then perform read, potentially from specific register
        if !write_data.is_empty() && self.devices.contains_key(&address) {
            let device = self.devices.get(&address).unwrap();
            let start_reg = write_data[0];
            
            for (i, byte) in read_buffer.iter_mut().enumerate() {
                *byte = device.get(&(start_reg + i as u8)).copied().unwrap_or(0);
            }
            
            // Don't double-count in read_history since write already recorded
            Ok(())
        } else {
            self.read(address, read_buffer).await
        }
    }
}

#[cfg(feature = "mock")]
/// Mock UART transmitter
#[derive(Debug)]
pub struct MockUartTx {
    /// Transmitted data buffer
    transmitted_data: Vec<u8>,
    
    /// Next error to inject
    next_error: Option<IoTError>,
    
    /// Health status
    is_healthy: bool,
}

#[cfg(feature = "mock")]
impl MockUartTx {
    fn new() -> Self {
        Self {
            transmitted_data: Vec::new(),
            next_error: None,
            is_healthy: true,
        }
    }

    /// Get all transmitted data
    pub fn transmitted_data(&self) -> &[u8] {
        &self.transmitted_data
    }

    /// Get transmitted data as string (if valid UTF-8)
    pub fn transmitted_string(&self) -> Result<String, core::str::Utf8Error> {
        core::str::from_utf8(&self.transmitted_data).map(|s| s.to_string())
    }

    /// Clear transmitted data buffer
    pub fn clear_transmitted(&mut self) {
        self.transmitted_data.clear();
    }

    /// Set next operation to fail
    pub fn set_next_error(&mut self, error: IoTError) {
        self.next_error = Some(error);
    }

    /// Reset mock to initial state
    pub fn reset(&mut self) {
        self.transmitted_data.clear();
        self.next_error = None;
        self.is_healthy = true;
    }

    fn check_error(&mut self) -> Result<(), IoTError> {
        if let Some(error) = self.next_error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl UartTxInterface for MockUartTx {
    async fn write(&mut self, data: &[u8]) -> Result<usize, IoTError> {
        self.check_error()?;
        
        self.transmitted_data.extend_from_slice(data);
        Ok(data.len())
    }

    async fn flush(&mut self) -> Result<(), IoTError> {
        self.check_error()?;
        // For mock, flush is always immediate
        Ok(())
    }
}

#[cfg(feature = "mock")]
/// Mock UART receiver
#[derive(Debug)]
pub struct MockUartRx {
    /// Receive data buffer
    receive_buffer: VecDeque<u8>,
    
    /// Next error to inject
    next_error: Option<IoTError>,
    
    /// Health status
    is_healthy: bool,
}

#[cfg(feature = "mock")]
impl MockUartRx {
    fn new() -> Self {
        Self {
            receive_buffer: VecDeque::new(),
            next_error: None,
            is_healthy: true,
        }
    }

    /// Add data to receive buffer (simulates incoming data)
    pub fn add_receive_data(&mut self, data: &[u8]) {
        self.receive_buffer.extend(data.iter());
    }

    /// Add string to receive buffer
    pub fn add_receive_string(&mut self, string: &str) {
        self.add_receive_data(string.as_bytes());
    }

    /// Get remaining data in receive buffer
    pub fn remaining_data(&self) -> usize {
        self.receive_buffer.len()
    }

    /// Clear receive buffer
    pub fn clear_receive_buffer(&mut self) {
        self.receive_buffer.clear();
    }

    /// Set next operation to fail
    pub fn set_next_error(&mut self, error: IoTError) {
        self.next_error = Some(error);
    }

    /// Reset mock to initial state
    pub fn reset(&mut self) {
        self.receive_buffer.clear();
        self.next_error = None;
        self.is_healthy = true;
    }

    fn check_error(&mut self) -> Result<(), IoTError> {
        if let Some(error) = self.next_error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl UartRxInterface for MockUartRx {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoTError> {
        self.check_error()?;
        
        let bytes_to_read = core::cmp::min(buffer.len(), self.receive_buffer.len());
        
        for i in 0..bytes_to_read {
            buffer[i] = self.receive_buffer.pop_front().unwrap();
        }
        
        Ok(bytes_to_read)
    }

    fn available(&self) -> bool {
        !self.receive_buffer.is_empty()
    }
}

#[cfg(feature = "mock")]
/// Mock GPIO interface
#[derive(Debug)]
pub struct MockGpio {
    /// Current pin state (true = high, false = low)
    pin_state: AtomicBool,
    
    /// Active high configuration
    active_high: bool,
    
    /// Operation history
    operations: Arc<std::sync::Mutex<Vec<String>>>,
    
    /// Next error to inject
    next_error: Option<IoTError>,
    
    /// Health status
    is_healthy: bool,
}

#[cfg(feature = "mock")]
impl MockGpio {
    fn new(active_high: bool) -> Self {
        Self {
            pin_state: AtomicBool::new(false),
            active_high,
            operations: Arc::new(std::sync::Mutex::new(Vec::new())),
            next_error: None,
            is_healthy: true,
        }
    }

    /// Get current pin state
    pub fn get_pin_state(&self) -> bool {
        self.pin_state.load(Ordering::Acquire)
    }

    /// Check if LED is logically on (accounting for active high/low)
    pub fn is_led_on(&self) -> bool {
        let pin_high = self.get_pin_state();
        if self.active_high { pin_high } else { !pin_high }
    }

    /// Get operation history
    pub fn operations(&self) -> Vec<String> {
        self.operations.lock().unwrap().clone()
    }

    /// Clear operation history
    pub fn clear_operations(&mut self) {
        self.operations.lock().unwrap().clear();
    }

    /// Set next operation to fail
    pub fn set_next_error(&mut self, error: IoTError) {
        self.next_error = Some(error);
    }

    /// Reset mock to initial state
    pub fn reset(&mut self) {
        self.pin_state.store(false, Ordering::Release);
        self.clear_operations();
        self.next_error = None;
        self.is_healthy = true;
    }

    fn check_error(&mut self) -> Result<(), IoTError> {
        if let Some(error) = self.next_error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn record_operation(&self, operation: &str) {
        self.operations.lock().unwrap().push(operation.to_string());
    }
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl GpioInterface for MockGpio {
    async fn set_high(&mut self) -> Result<(), IoTError> {
        self.check_error()?;
        
        self.pin_state.store(true, Ordering::Release);
        self.record_operation("set_high");
        Ok(())
    }

    async fn set_low(&mut self) -> Result<(), IoTError> {
        self.check_error()?;
        
        self.pin_state.store(false, Ordering::Release);
        self.record_operation("set_low");
        Ok(())
    }

    async fn toggle(&mut self) -> Result<(), IoTError> {
        self.check_error()?;
        
        let current = self.pin_state.load(Ordering::Acquire);
        self.pin_state.store(!current, Ordering::Release);
        self.record_operation("toggle");
        Ok(())
    }

    async fn is_high(&self) -> Result<bool, IoTError> {
        Ok(self.pin_state.load(Ordering::Acquire))
    }
}

#[cfg(feature = "mock")]
/// Mock timer interface
#[derive(Debug)]
pub struct MockTimer {
    /// Start time for timing calculations
    start_time: Instant,
    
    /// Time acceleration factor (1.0 = real time, >1.0 = faster)
    acceleration_factor: f32,
}

#[cfg(feature = "mock")]
impl MockTimer {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            acceleration_factor: 1.0,
        }
    }

    /// Set time acceleration factor for testing
    /// 
    /// Values > 1.0 make time pass faster in tests
    pub fn set_acceleration(&mut self, factor: f32) {
        self.acceleration_factor = factor;
    }

    /// Reset timer to initial state
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.acceleration_factor = 1.0;
    }
}

#[cfg(feature = "mock")]
impl TimerInterface for MockTimer {
    async fn delay(&mut self, duration: Duration) {
        let accelerated_duration = Duration::from_millis(
            (duration.as_millis() as f32 / self.acceleration_factor) as u64
        );
        
        #[cfg(feature = "tokio")]
        tokio::time::sleep(std::time::Duration::from_millis(accelerated_duration.as_millis() as u64)).await;
        
        #[cfg(not(feature = "tokio"))]
        {
            // Fallback for testing without tokio
            let _ = accelerated_duration;
        }
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn deadline(&self, duration: Duration) -> Instant {
        Instant::now() + duration
    }
}

#[cfg(feature = "mock")]
/// Mock WiFi interface
#[derive(Debug)]
pub struct MockWiFi {
    /// Connection state
    connected: AtomicBool,
    
    /// Signal strength
    signal_strength: AtomicI8,
    
    /// Connection info
    connection_info: Arc<std::sync::Mutex<Option<WiFiConnectionInfo>>>,
    
    /// Connection history
    connection_attempts: Arc<std::sync::Mutex<Vec<(String, String)>>>,
    
    /// Next error to inject
    next_error: Option<IoTError>,
    
    /// Health status
    is_healthy: bool,
    
    /// Simulated connection delay
    connection_delay: Duration,
}

#[cfg(feature = "mock")]
impl MockWiFi {
    fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            signal_strength: AtomicI8::new(-50),
            connection_info: Arc::new(std::sync::Mutex::new(None)),
            connection_attempts: Arc::new(std::sync::Mutex::new(Vec::new())),
            next_error: None,
            is_healthy: true,
            connection_delay: Duration::from_millis(100),
        }
    }

    /// Set connection delay for testing
    pub fn set_connection_delay(&mut self, delay: Duration) {
        self.connection_delay = delay;
    }

    /// Set signal strength
    pub fn set_signal_strength(&self, strength: i8) {
        self.signal_strength.store(strength, Ordering::Release);
    }

    /// Get connection attempt history
    pub fn connection_attempts(&self) -> Vec<(String, String)> {
        self.connection_attempts.lock().unwrap().clone()
    }

    /// Set next operation to fail
    pub fn set_next_error(&mut self, error: IoTError) {
        self.next_error = Some(error);
    }

    /// Force connection state (for testing)
    pub fn force_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Release);
    }

    /// Reset mock to initial state
    pub fn reset(&mut self) {
        self.connected.store(false, Ordering::Release);
        self.signal_strength.store(-50, Ordering::Release);
        *self.connection_info.lock().unwrap() = None;
        self.connection_attempts.lock().unwrap().clear();
        self.next_error = None;
        self.is_healthy = true;
        self.connection_delay = Duration::from_millis(100);
    }

    fn check_error(&mut self) -> Result<(), IoTError> {
        if let Some(error) = self.next_error.take() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "mock")]
#[async_trait(?Send)]
impl WiFiInterface for MockWiFi {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), IoTError> {
        self.check_error()?;
        
        // Record connection attempt
        self.connection_attempts.lock().unwrap().push((ssid.to_string(), password.to_string()));
        
        // Simulate connection delay
        #[cfg(feature = "tokio")]
        tokio::time::sleep(std::time::Duration::from_millis(self.connection_delay.as_millis() as u64)).await;
        
        // For mock, always succeed unless error is injected
        self.connected.store(true, Ordering::Release);
        
        // Create mock connection info
        let info = WiFiConnectionInfo {
            ip_address: IpAddr::V4(core::net::Ipv4Addr::new(192, 168, 1, 100)),
            gateway: Some(IpAddr::V4(core::net::Ipv4Addr::new(192, 168, 1, 1))),
            netmask: Some(IpAddr::V4(core::net::Ipv4Addr::new(255, 255, 255, 0))),
            ssid: heapless::String::try_from(ssid).unwrap_or_default(),
            signal_strength: self.signal_strength.load(Ordering::Acquire),
            uptime_seconds: 0,
        };
        
        *self.connection_info.lock().unwrap() = Some(info);
        
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), IoTError> {
        self.check_error()?;
        
        self.connected.store(false, Ordering::Release);
        *self.connection_info.lock().unwrap() = None;
        
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }

    fn get_ip_address(&self) -> Option<IpAddr> {
        self.connection_info.lock().unwrap()
            .as_ref()
            .map(|info| info.ip_address)
    }

    fn get_signal_strength(&self) -> i8 {
        self.signal_strength.load(Ordering::Acquire)
    }

    fn get_connection_info(&self) -> Option<WiFiConnectionInfo> {
        self.connection_info.lock().unwrap().clone()
    }
}