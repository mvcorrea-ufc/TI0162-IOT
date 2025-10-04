//! # Trait Interfaces for IoT Container Components
//!
//! This module defines the core trait interfaces that enable dependency injection
//! and clean architecture in the ESP32-C3 IoT system. All components implement
//! these traits, allowing for easy testing, mocking, and configuration-driven
//! component selection.

use alloc::boxed::Box;
use async_trait::async_trait;
use iot_common::IoTError;
use heapless::String;

/// Maximum length for string fields in embedded environment
pub const MAX_STRING_LEN: usize = 64;

/// Type alias for embedded-friendly strings
pub type EmbeddedString = String<MAX_STRING_LEN>;

/// Environmental sensor measurements
/// 
/// Contains calibrated measurements from environmental sensors like BME280.
/// All values are compensated using factory calibration data.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurements {
    /// Temperature in degrees Celsius
    /// Range: -40°C to +85°C, Accuracy: ±1°C
    pub temperature: f32,
    
    /// Atmospheric pressure in hectopascals (hPa)
    /// Range: 300-1100 hPa, Accuracy: ±1 hPa
    pub pressure: f32,
    
    /// Relative humidity as percentage
    /// Range: 0-100% RH, Accuracy: ±3% RH
    pub humidity: f32,
    
    /// Timestamp when measurement was taken (milliseconds since system start)
    pub timestamp_ms: u64,
}

impl Measurements {
    /// Creates a new Measurements instance with current timestamp
    pub fn new(temperature: f32, pressure: f32, humidity: f32) -> Self {
        Self {
            temperature,
            pressure, 
            humidity,
            timestamp_ms: embassy_time::Instant::now().as_millis(),
        }
    }
    
    /// Checks if all measurements are within expected ranges
    pub fn is_valid(&self) -> bool {
        self.temperature >= -40.0 && self.temperature <= 85.0 &&
        self.pressure >= 300.0 && self.pressure <= 1100.0 &&
        self.humidity >= 0.0 && self.humidity <= 100.0
    }
}

/// Network connection information
/// 
/// Contains details about the current network connection state and parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionInfo {
    /// IP address assigned to the device
    pub ip_address: EmbeddedString,
    
    /// Gateway IP address
    pub gateway: Option<EmbeddedString>,
    
    /// Subnet mask
    pub subnet_mask: Option<EmbeddedString>,
    
    /// DNS server addresses
    pub dns_servers: heapless::Vec<EmbeddedString, 2>,
    
    /// WiFi signal strength in dBm (negative values, closer to 0 is stronger)
    pub signal_strength_dbm: Option<i8>,
    
    /// Network SSID (for WiFi connections)
    pub ssid: Option<EmbeddedString>,
}

impl ConnectionInfo {
    /// Creates a basic connection info with just IP address
    pub fn new(ip_address: &str) -> Result<Self, IoTError> {
        Ok(Self {
            ip_address: EmbeddedString::try_from(ip_address).map_err(|_| {
                IoTError::configuration(iot_common::ConfigError::ValidationError("IP address too long".try_into().unwrap_or_default()))
            })?,
            gateway: None,
            subnet_mask: None,
            dns_servers: heapless::Vec::new(),
            signal_strength_dbm: None,
            ssid: None,
        })
    }
}

/// Sensor data for publishing to remote systems
/// 
/// Structured data format for transmitting sensor measurements via MQTT or other protocols.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
    /// Device identifier
    pub device_id: EmbeddedString,
    
    /// Environmental measurements
    pub measurements: Measurements,
    
    /// Data quality indicator (0.0 = poor, 1.0 = excellent)
    pub quality_score: f32,
    
    /// Number of measurements averaged (for noise reduction)
    pub sample_count: u16,
}

impl SensorData {
    /// Creates new sensor data from measurements
    pub fn new(device_id: &str, measurements: Measurements) -> Result<Self, IoTError> {
        let quality_score = if measurements.is_valid() { 1.0 } else { 0.5 };
        Ok(Self {
            device_id: EmbeddedString::try_from(device_id).map_err(|_| {
                IoTError::configuration(iot_common::ConfigError::ValidationError("Device ID too long".try_into().unwrap_or_default()))
            })?,
            measurements,
            quality_score,
            sample_count: 1,
        })
    }
    
    /// Creates sensor data with quality assessment
    pub fn with_quality(device_id: &str, measurements: Measurements, quality_score: f32) -> Result<Self, IoTError> {
        Ok(Self {
            device_id: EmbeddedString::try_from(device_id).map_err(|_| {
                IoTError::configuration(iot_common::ConfigError::ValidationError("Device ID too long".try_into().unwrap_or_default()))
            })?,
            measurements,
            quality_score: quality_score.max(0.0).min(1.0),
            sample_count: 1,
        })
    }
}

/// Device status information
/// 
/// System health and operational status data for monitoring and diagnostics.
#[derive(Debug, Clone, PartialEq)]
pub struct DeviceStatus {
    /// Device identifier
    pub device_id: EmbeddedString,
    
    /// Operational status
    pub status: EmbeddedString,
    
    /// System uptime in seconds
    pub uptime_seconds: u32,
    
    /// Free heap memory in bytes
    pub free_heap_bytes: u32,
    
    /// WiFi signal strength in dBm
    pub wifi_signal_dbm: i8,
    
    /// Number of sensor readings taken
    pub sensor_readings_count: u32,
    
    /// Number of MQTT messages published
    pub mqtt_messages_count: u32,
    
    /// Last error code (0 = no error)
    pub last_error_code: u32,
}

impl DeviceStatus {
    /// Creates a new device status
    pub fn new(
        device_id: &str, 
        status: &str, 
        uptime_seconds: u32, 
        free_heap_bytes: u32, 
        wifi_signal_dbm: i8
    ) -> Result<Self, IoTError> {
        Ok(Self {
            device_id: EmbeddedString::try_from(device_id).map_err(|_| {
                IoTError::configuration(iot_common::ConfigError::ValidationError("Device ID too long".try_into().unwrap_or_default()))
            })?,
            status: EmbeddedString::try_from(status).map_err(|_| {
                IoTError::configuration(iot_common::ConfigError::ValidationError("Status string too long".try_into().unwrap_or_default()))
            })?,
            uptime_seconds,
            free_heap_bytes,
            wifi_signal_dbm,
            sensor_readings_count: 0,
            mqtt_messages_count: 0,
            last_error_code: 0,
        })
    }
}

/// Trait for reading environmental sensor data
/// 
/// Provides a unified interface for all environmental sensors in the IoT system.
/// Implementations should handle sensor initialization, measurement acquisition,
/// and error recovery automatically.
#[async_trait]
pub trait SensorReader {
    /// Reads current environmental measurements from the sensor
    /// 
    /// This method performs a complete measurement cycle including:
    /// - Triggering sensor measurement
    /// - Waiting for completion
    /// - Reading and compensating raw data
    /// - Returning calibrated measurements
    /// 
    /// # Returns
    /// 
    /// * `Ok(Measurements)` - Successfully read and compensated measurements
    /// * `Err(IoTError)` - Sensor communication failure or invalid data
    /// 
    /// # Error Conditions
    /// 
    /// - I2C communication failure
    /// - Sensor not responding
    /// - Invalid measurement data
    /// - Calibration data corruption
    async fn read_measurements(&mut self) -> Result<Measurements, IoTError>;
    
    /// Checks if the sensor is available and responding
    /// 
    /// This method performs a quick health check without taking measurements.
    /// It should verify sensor presence and basic communication.
    /// 
    /// # Returns
    /// 
    /// `true` if sensor is detected and responding, `false` otherwise
    async fn is_available(&self) -> bool;
    
    /// Initializes the sensor for measurements
    /// 
    /// Performs complete sensor initialization including:
    /// - Reading calibration coefficients
    /// - Configuring sensor registers
    /// - Performing initial measurement cycle
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Sensor successfully initialized
    /// * `Err(IoTError)` - Initialization failed
    async fn initialize(&mut self) -> Result<(), IoTError>;
    
    /// Returns the sensor type identifier
    /// 
    /// Provides a string identifier for the sensor type (e.g., "BME280", "SHT30").
    /// Used for logging, configuration, and diagnostic purposes.
    fn get_sensor_type(&self) -> &'static str;
    
    /// Returns the last measurement timestamp
    /// 
    /// Provides the timestamp of the most recent successful measurement.
    /// Used for data aging and synchronization purposes.
    fn get_last_measurement_time(&self) -> Option<u64>;
    
    /// Performs sensor self-test
    /// 
    /// Executes sensor-specific self-test procedures to verify operation.
    /// This may include reading sensor ID, verifying calibration data,
    /// or performing measurement consistency checks.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Self-test passed
    /// * `Err(IoTError)` - Self-test failed
    async fn self_test(&mut self) -> Result<(), IoTError>;
}

/// Trait for managing network connectivity
/// 
/// Provides a unified interface for all network connection types (WiFi, Ethernet, cellular).
/// Implementations should handle connection establishment, monitoring, and recovery.
#[async_trait]
pub trait NetworkManager {
    /// Establishes network connection
    /// 
    /// Initiates connection to the configured network. This method should:
    /// - Configure network interface
    /// - Authenticate with network
    /// - Acquire IP address (DHCP/static)
    /// - Verify connectivity
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connection established successfully
    /// * `Err(IoTError)` - Connection failed
    async fn connect(&mut self) -> Result<(), IoTError>;
    
    /// Disconnects from network
    /// 
    /// Gracefully disconnects from the current network and releases resources.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Disconnection completed
    /// * `Err(IoTError)` - Disconnection failed
    async fn disconnect(&mut self) -> Result<(), IoTError>;
    
    /// Checks if network is currently connected
    /// 
    /// Performs a quick connectivity check without establishing new connections.
    /// 
    /// # Returns
    /// 
    /// `true` if connected and network is reachable, `false` otherwise
    async fn is_connected(&self) -> bool;
    
    /// Gets current connection information
    /// 
    /// Returns detailed information about the current network connection
    /// including IP address, gateway, DNS servers, and signal strength.
    /// 
    /// # Returns
    /// 
    /// `Some(ConnectionInfo)` if connected, `None` if disconnected
    async fn get_connection_info(&self) -> Option<ConnectionInfo>;
    
    /// Gets current signal strength
    /// 
    /// Returns the signal strength of the current connection in dBm.
    /// For WiFi, typical values range from -30 dBm (excellent) to -90 dBm (poor).
    /// 
    /// # Returns
    /// 
    /// `Some(signal_dbm)` if connected, `None` if disconnected
    async fn get_signal_strength(&self) -> Option<i8>;
    
    /// Performs network connectivity test
    /// 
    /// Tests actual internet connectivity by attempting to reach a known endpoint.
    /// This goes beyond local network connectivity to verify internet access.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Internet connectivity verified
    /// * `Err(IoTError)` - No internet connectivity
    async fn test_connectivity(&self) -> Result<(), IoTError>;
    
    /// Gets the network stack for protocol operations
    /// 
    /// Returns a reference to the underlying network stack for TCP/UDP operations.
    /// This enables other components to perform network I/O operations.
    fn get_stack(&self) -> &'static embassy_net::Stack<'static>;
}

/// Trait for publishing messages to remote systems
/// 
/// Provides a unified interface for all message publishing mechanisms (MQTT, HTTP, CoAP).
/// Implementations should handle connection management, message queuing, and delivery confirmation.
#[async_trait]
pub trait MessagePublisher {
    /// Publishes sensor data to remote system
    /// 
    /// Sends environmental sensor data to the configured remote endpoint.
    /// This method should handle:
    /// - Message serialization
    /// - Connection establishment
    /// - Reliable delivery
    /// - Error recovery
    /// 
    /// # Arguments
    /// 
    /// * `data` - Sensor data to publish
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message published successfully
    /// * `Err(IoTError)` - Publishing failed
    async fn publish_sensor_data(&mut self, data: &SensorData) -> Result<(), IoTError>;
    
    /// Publishes device status information
    /// 
    /// Sends device health and operational status to the remote system
    /// for monitoring and diagnostics.
    /// 
    /// # Arguments
    /// 
    /// * `status` - Device status to publish
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Status published successfully
    /// * `Err(IoTError)` - Publishing failed
    async fn publish_status(&mut self, status: &DeviceStatus) -> Result<(), IoTError>;
    
    /// Checks if publisher is connected to remote system
    /// 
    /// Verifies connection to the remote messaging system without
    /// attempting to establish new connections.
    /// 
    /// # Returns
    /// 
    /// `true` if connected and ready to publish, `false` otherwise
    async fn is_connected(&self) -> bool;
    
    /// Establishes connection to remote system
    /// 
    /// Initiates connection to the configured remote messaging system.
    /// This method should handle authentication and session establishment.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connection established
    /// * `Err(IoTError)` - Connection failed
    async fn connect(&mut self) -> Result<(), IoTError>;
    
    /// Publishes a heartbeat message
    /// 
    /// Sends a simple heartbeat/keepalive message to indicate device is operational.
    /// Used for connection monitoring and device presence detection.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Heartbeat published successfully
    /// * `Err(IoTError)` - Publishing failed
    async fn publish_heartbeat(&mut self) -> Result<(), IoTError>;
    
    /// Gets publisher-specific metrics
    /// 
    /// Returns operational metrics such as message count, success rate,
    /// and connection uptime for monitoring purposes.
    /// 
    /// # Returns
    /// 
    /// A tuple containing (messages_sent, messages_failed, uptime_seconds)
    fn get_metrics(&self) -> (u32, u32, u32);
}

/// Trait for interactive console interfaces
/// 
/// Provides a unified interface for all console/terminal interfaces (UART, USB, network).
/// Implementations should handle command parsing, response formatting, and session management.
#[async_trait]
pub trait ConsoleInterface {
    /// Writes a line of text to the console
    /// 
    /// Sends a text message to the console with automatic line termination.
    /// This method should handle formatting and transmission.
    /// 
    /// # Arguments
    /// 
    /// * `message` - Text message to write
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message written successfully
    /// * `Err(IoTError)` - Write operation failed
    async fn write_line(&mut self, message: &str) -> Result<(), IoTError>;
    
    /// Reads a command from the console
    /// 
    /// Attempts to read a complete command from the console input.
    /// This method should handle line buffering and command parsing.
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some(command))` - Command received
    /// * `Ok(None)` - No command available (non-blocking)
    /// * `Err(IoTError)` - Read operation failed
    async fn read_command(&mut self) -> Result<Option<EmbeddedString>, IoTError>;
    
    /// Handles a console command and returns response
    /// 
    /// Processes a received command and generates an appropriate response.
    /// This method should handle command parsing, execution, and response formatting.
    /// 
    /// # Arguments
    /// 
    /// * `command` - Command string to process
    /// 
    /// # Returns
    /// 
    /// * `Ok(response)` - Command processed, response generated
    /// * `Err(IoTError)` - Command processing failed
    async fn handle_command(&mut self, command: &str) -> Result<EmbeddedString, IoTError>;
    
    /// Checks if console is ready for input/output
    /// 
    /// Verifies that the console interface is operational and ready
    /// for command processing.
    /// 
    /// # Returns
    /// 
    /// `true` if console is ready, `false` otherwise
    async fn is_ready(&self) -> bool;
    
    /// Gets console session information
    /// 
    /// Returns information about the current console session including
    /// connection status and session duration.
    /// 
    /// # Returns
    /// 
    /// Session information as a formatted string
    fn get_session_info(&self) -> EmbeddedString;
    
    /// Sends a formatted prompt to the console
    /// 
    /// Displays a command prompt to indicate the console is ready for input.
    /// This method should handle prompt formatting and display.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Prompt displayed successfully
    /// * `Err(IoTError)` - Prompt display failed
    async fn show_prompt(&mut self) -> Result<(), IoTError>;
}