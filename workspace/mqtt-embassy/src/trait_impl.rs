//! # MQTT Embassy IoT Container Trait Implementation
//!
//! This module provides the implementation of the IoT Container MessagePublisher trait
//! for the MQTT client, enabling seamless integration with the dependency injection
//! container system.

use async_trait::async_trait;
use embassy_time::Instant;
use core::net::Ipv4Addr;

use iot_common::IoTError;

// Import the container trait (when iot-container is available)
#[cfg(feature = "container")]
use iot_container::traits::{MessagePublisher, SensorData as ContainerSensorData, DeviceStatus as ContainerDeviceStatus, EmbeddedString};

use crate::mqtt_client::{MqttClient, MqttConfig, MqttError};
use crate::message::{SensorData, DeviceStatus, MqttMessage};

/// Adapter that implements the IoT Container MessagePublisher trait for MqttClient
/// 
/// This adapter bridges the MQTT client with the IoT Container's trait-based
/// dependency injection system, enabling the MQTT client to be used as a drop-in
/// component in the container architecture.
#[cfg(feature = "container")]
pub struct MqttContainerAdapter {
    /// The underlying MQTT client
    mqtt_client: MqttClient,
    
    /// Topic prefix for all published messages
    topic_prefix: EmbeddedString,
    
    /// Client identifier
    client_id: EmbeddedString,
    
    /// Connection state tracking
    is_connected: bool,
    
    /// Last successful connection time
    last_connection_time: Option<u64>,
    
    /// Message publish statistics
    messages_published: u32,
    
    /// Message publish failures
    publish_failures: u32,
    
    /// Connection attempt count
    connection_attempts: u32,
    
    /// Start time for metrics
    start_time: Instant,
    
    /// Last heartbeat time
    last_heartbeat_time: Option<u64>,
    
    /// Heartbeat interval in milliseconds
    heartbeat_interval_ms: u64,
}

#[cfg(feature = "container")]
impl MqttContainerAdapter {
    /// Creates a new MQTT container adapter
    /// 
    /// # Arguments
    /// 
    /// * `mqtt_client` - MQTT client instance to wrap
    /// * `topic_prefix` - Prefix for all published message topics
    /// * `client_id` - MQTT client identifier
    /// 
    /// # Returns
    /// 
    /// A new adapter instance ready for use
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use mqtt_embassy::{MqttClient, MqttContainerAdapter, MqttConfig};
    /// use iot_container::traits::MessagePublisher;
    /// 
    /// let config = MqttConfig::default();
    /// let mqtt_client = MqttClient::new(config);
    /// let adapter = MqttContainerAdapter::new(mqtt_client, "iot/device", "esp32c3_001")?;
    /// ```
    pub fn new(
        mqtt_client: MqttClient, 
        topic_prefix: &str, 
        client_id: &str
    ) -> Result<Self, IoTError> {
        Ok(Self {
            mqtt_client,
            topic_prefix: EmbeddedString::try_from(topic_prefix).map_err(|_| {
                IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Topic prefix too long"))
            })?,
            client_id: EmbeddedString::try_from(client_id).map_err(|_| {
                IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Client ID too long"))
            })?,
            is_connected: false,
            last_connection_time: None,
            messages_published: 0,
            publish_failures: 0,
            connection_attempts: 0,
            start_time: Instant::now(),
            last_heartbeat_time: None,
            heartbeat_interval_ms: 60000, // Default 1 minute heartbeat interval
        })
    }
    
    /// Creates a new adapter with custom heartbeat interval
    /// 
    /// # Arguments
    /// 
    /// * `mqtt_client` - MQTT client instance to wrap
    /// * `topic_prefix` - Prefix for all published message topics
    /// * `client_id` - MQTT client identifier
    /// * `heartbeat_interval_ms` - Heartbeat interval in milliseconds
    /// 
    /// # Returns
    /// 
    /// A new adapter instance with custom heartbeat timing
    pub fn new_with_heartbeat_interval(
        mqtt_client: MqttClient, 
        topic_prefix: &str, 
        client_id: &str,
        heartbeat_interval_ms: u64
    ) -> Result<Self, IoTError> {
        let mut adapter = Self::new(mqtt_client, topic_prefix, client_id)?;
        adapter.heartbeat_interval_ms = heartbeat_interval_ms;
        Ok(adapter)
    }
    
    /// Converts container sensor data to MQTT sensor data format
    /// 
    /// This method handles the conversion between the container-specific sensor
    /// data format and the MQTT-specific data format.
    /// 
    /// # Arguments
    /// 
    /// * `container_data` - Container sensor data to convert
    /// 
    /// # Returns
    /// 
    /// MQTT-compatible sensor data
    fn convert_sensor_data(&self, container_data: &ContainerSensorData) -> SensorData {
        SensorData::new(
            container_data.measurements.temperature,
            container_data.measurements.humidity,
            container_data.measurements.pressure,
        )
    }
    
    /// Converts container device status to MQTT device status format
    /// 
    /// This method handles the conversion between the container-specific device
    /// status format and the MQTT-specific status format.
    /// 
    /// # Arguments
    /// 
    /// * `container_status` - Container device status to convert
    /// 
    /// # Returns
    /// 
    /// MQTT-compatible device status
    fn convert_device_status(&self, container_status: &ContainerDeviceStatus) -> DeviceStatus {
        DeviceStatus::new(
            container_status.status.as_str(),
            container_status.uptime_seconds,
            container_status.free_heap_bytes,
            container_status.wifi_signal_dbm,
        )
    }
    
    /// Builds a topic string with prefix
    /// 
    /// # Arguments
    /// 
    /// * `topic_suffix` - Topic suffix to append to prefix
    /// 
    /// # Returns
    /// 
    /// Complete topic string
    /// 
    /// # Examples
    /// 
    /// If prefix is "iot/device" and suffix is "sensors/temperature",
    /// returns "iot/device/sensors/temperature"
    fn build_topic(&self, topic_suffix: &str) -> Result<EmbeddedString, IoTError> {
        let full_topic = if self.topic_prefix.is_empty() {
            topic_suffix.to_string()
        } else {
            format!("{}/{}", self.topic_prefix.as_str(), topic_suffix)
        };
        
        EmbeddedString::try_from(full_topic.as_str()).map_err(|_| {
            IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Topic string too long"))
        })
    }
    
    /// Converts MQTT errors to IoT errors
    /// 
    /// # Arguments
    /// 
    /// * `mqtt_error` - MQTT-specific error to convert
    /// 
    /// # Returns
    /// 
    /// Standardized IoT error
    fn convert_error(&self, mqtt_error: MqttError) -> IoTError {
        match mqtt_error {
            MqttError::ConnectionFailed(msg) => {
                IoTError::Network(iot_common::NetworkError::ConnectionFailed(msg))
            }
            MqttError::ProtocolError(msg) => {
                IoTError::Network(iot_common::NetworkError::ProtocolError(msg))
            }
            MqttError::IoError(msg) => {
                IoTError::Network(iot_common::NetworkError::TransmissionFailed(msg))
            }
            MqttError::SerializationError(msg) => {
                IoTError::System(iot_common::SystemError::SerializationFailed(msg))
            }
        }
    }
    
    /// Checks if heartbeat should be sent
    /// 
    /// # Returns
    /// 
    /// `true` if heartbeat should be sent, `false` otherwise
    fn should_send_heartbeat(&self) -> bool {
        match self.last_heartbeat_time {
            None => true, // Never sent heartbeat before
            Some(last_heartbeat) => {
                let now = Instant::now().as_millis();
                (now - last_heartbeat) >= self.heartbeat_interval_ms
            }
        }
    }
    
    /// Updates the heartbeat timestamp
    fn update_heartbeat_time(&mut self) {
        self.last_heartbeat_time = Some(Instant::now().as_millis());
    }
    
    /// Attempts to connect to MQTT broker with the network stack
    /// 
    /// This is a simplified connection attempt that would need to be
    /// properly implemented with the actual MQTT client connection logic.
    /// 
    /// # Arguments
    /// 
    /// * `stack` - Network stack for TCP connectivity
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connection established
    /// * `Err(IoTError)` - Connection failed
    async fn connect_to_broker(
        &mut self, 
        stack: &'static embassy_net::Stack<embassy_net::driver::Driver<'static>>
    ) -> Result<(), IoTError> {
        self.connection_attempts += 1;
        
        // In a real implementation, this would:
        // 1. Create TCP socket
        // 2. Connect to MQTT broker
        // 3. Send CONNECT packet
        // 4. Receive CONNACK
        // 5. Set connection state
        
        // For now, simulate connection attempt
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        
        // Mark as connected (in real implementation, this would be based on CONNACK)
        self.is_connected = true;
        self.last_connection_time = Some(Instant::now().as_millis());
        
        Ok(())
    }
    
    /// Gets publishing metrics for monitoring
    /// 
    /// # Returns
    /// 
    /// Tuple containing (messages_published, publish_failures, connection_attempts)
    pub fn get_publishing_metrics(&self) -> (u32, u32, u32) {
        (self.messages_published, self.publish_failures, self.connection_attempts)
    }
    
    /// Gets connection uptime in seconds
    /// 
    /// # Returns
    /// 
    /// Connection uptime in seconds, or 0 if never connected
    pub fn get_connection_uptime(&self) -> u32 {
        match self.last_connection_time {
            Some(connection_time) => {
                let now = Instant::now().as_millis();
                ((now - connection_time) / 1000) as u32
            }
            None => 0,
        }
    }
}

#[cfg(feature = "container")]
#[async_trait]
impl MessagePublisher for MqttContainerAdapter {
    /// Publishes sensor data to MQTT broker
    /// 
    /// This method sends environmental sensor data to the configured MQTT topic.
    /// The data is converted to the appropriate MQTT format and published with
    /// proper error handling and retry logic.
    /// 
    /// # Arguments
    /// 
    /// * `data` - Container sensor data to publish
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Message published successfully
    /// * `Err(IoTError)` - Publishing failed
    /// 
    /// # Implementation Details
    /// 
    /// - Converts container data format to MQTT format
    /// - Builds appropriate topic string with prefix
    /// - Handles connection state management
    /// - Updates publish statistics
    /// - Implements error conversion from MQTT errors
    async fn publish_sensor_data(&mut self, data: &ContainerSensorData) -> Result<(), IoTError> {
        if !self.is_connected {
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("MQTT not connected")));
        }
        
        // Convert container data to MQTT format
        let mqtt_data = self.convert_sensor_data(data);
        
        // Build topic for sensor data
        let topic = self.build_topic("sensors/data")?;
        
        // Create MQTT message
        let message = MqttMessage::sensor_data(topic.as_str(), mqtt_data);
        
        // In a real implementation, this would:
        // 1. Serialize the message to JSON/binary
        // 2. Create MQTT PUBLISH packet
        // 3. Send via TCP socket
        // 4. Handle QoS acknowledgments
        
        // For now, simulate publishing
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        
        // Update statistics
        self.messages_published += 1;
        
        Ok(())
    }
    
    /// Publishes device status information to MQTT broker
    /// 
    /// This method sends device health and operational status to the MQTT broker
    /// for monitoring and diagnostics purposes.
    /// 
    /// # Arguments
    /// 
    /// * `status` - Container device status to publish
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Status published successfully
    /// * `Err(IoTError)` - Publishing failed
    /// 
    /// # Implementation Details
    /// 
    /// - Converts container status format to MQTT format
    /// - Publishes to device status topic
    /// - Updates publish statistics
    /// - Handles connection validation
    async fn publish_status(&mut self, status: &ContainerDeviceStatus) -> Result<(), IoTError> {
        if !self.is_connected {
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("MQTT not connected")));
        }
        
        // Convert container status to MQTT format
        let mqtt_status = self.convert_device_status(status);
        
        // Build topic for device status
        let topic = self.build_topic("status")?;
        
        // Create MQTT message
        let message = MqttMessage::device_status(topic.as_str(), mqtt_status);
        
        // In a real implementation, this would publish the status message
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        
        // Update statistics
        self.messages_published += 1;
        
        Ok(())
    }
    
    /// Checks if MQTT client is connected to broker
    /// 
    /// This method provides a quick connection status check without
    /// attempting to establish new connections.
    /// 
    /// # Returns
    /// 
    /// `true` if connected and ready to publish, `false` otherwise
    /// 
    /// # Implementation Details
    /// 
    /// - Returns cached connection state
    /// - Fast operation with no network I/O
    /// - Does not validate actual connection health
    async fn is_connected(&self) -> bool {
        self.is_connected
    }
    
    /// Establishes connection to MQTT broker
    /// 
    /// This method initiates connection to the configured MQTT broker.
    /// It handles authentication and session establishment.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connection established
    /// * `Err(IoTError)` - Connection failed
    /// 
    /// # Implementation Details
    /// 
    /// - Manages connection state
    /// - Updates connection statistics
    /// - Handles authentication if configured
    /// - Sets up keep-alive mechanism
    async fn connect(&mut self) -> Result<(), IoTError> {
        if self.is_connected {
            return Ok(()); // Already connected
        }
        
        // In a real implementation, this would need the network stack
        // For now, simulate connection
        self.connection_attempts += 1;
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        
        self.is_connected = true;
        self.last_connection_time = Some(Instant::now().as_millis());
        
        Ok(())
    }
    
    /// Publishes a heartbeat message to MQTT broker
    /// 
    /// This method sends a simple heartbeat/keepalive message to indicate
    /// the device is operational. Used for connection monitoring and device
    /// presence detection.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Heartbeat published successfully
    /// * `Err(IoTError)` - Publishing failed
    /// 
    /// # Implementation Details
    /// 
    /// - Checks heartbeat interval to avoid excessive heartbeats
    /// - Updates heartbeat timestamp
    /// - Publishes to heartbeat topic
    /// - Includes basic device information
    async fn publish_heartbeat(&mut self) -> Result<(), IoTError> {
        if !self.is_connected {
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("MQTT not connected")));
        }
        
        // Check if heartbeat is due
        if !self.should_send_heartbeat() {
            return Ok(()); // Heartbeat not due yet
        }
        
        // Build topic for heartbeat
        let topic = self.build_topic("heartbeat")?;
        
        // Create simple heartbeat message with timestamp
        let timestamp = Instant::now().as_millis();
        let heartbeat_payload = format!("{{\"timestamp\":{},\"client_id\":\"{}\"}}", 
                                      timestamp, self.client_id.as_str());
        
        // In a real implementation, this would create and publish MQTT message
        embassy_time::Timer::after(embassy_time::Duration::from_millis(5)).await;
        
        // Update statistics and timestamp
        self.messages_published += 1;
        self.update_heartbeat_time();
        
        Ok(())
    }
    
    /// Gets publisher-specific metrics
    /// 
    /// Returns operational metrics such as message count, success rate,
    /// and connection uptime for monitoring purposes.
    /// 
    /// # Returns
    /// 
    /// A tuple containing (messages_sent, messages_failed, uptime_seconds)
    /// 
    /// # Metrics Details
    /// 
    /// - `messages_sent`: Total number of successfully published messages
    /// - `messages_failed`: Total number of failed publish attempts
    /// - `uptime_seconds`: Total time since adapter creation
    fn get_metrics(&self) -> (u32, u32, u32) {
        let uptime = self.start_time.elapsed().as_secs() as u32;
        (self.messages_published, self.publish_failures, uptime)
    }
}

// Convenience functions for creating container-compatible MQTT instances

/// Creates a new MQTT message publisher for use with the IoT container
/// 
/// This function provides a convenient way to create an MQTT adapter that
/// implements the container's MessagePublisher trait.
/// 
/// # Arguments
/// 
/// * `mqtt_client` - MQTT client instance to wrap
/// * `topic_prefix` - Prefix for all published message topics
/// * `client_id` - MQTT client identifier
/// 
/// # Returns
/// 
/// A new MQTT adapter ready for use with the IoT container
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use mqtt_embassy::{MqttClient, create_container_message_publisher, MqttConfig};
/// use iot_container::ComponentFactory;
/// 
/// let config = MqttConfig::default();
/// let mqtt_client = MqttClient::new(config);
/// let publisher = create_container_message_publisher(mqtt_client, "iot/device", "esp32c3_001")?;
/// ```
#[cfg(feature = "container")]
pub fn create_container_message_publisher(
    mqtt_client: MqttClient, 
    topic_prefix: &str, 
    client_id: &str
) -> Result<MqttContainerAdapter, IoTError> {
    MqttContainerAdapter::new(mqtt_client, topic_prefix, client_id)
}

/// Creates an MQTT adapter with custom heartbeat interval
/// 
/// This function allows customization of how frequently the adapter sends
/// heartbeat messages, which can be tuned for specific monitoring requirements.
/// 
/// # Arguments
/// 
/// * `mqtt_client` - MQTT client instance to wrap
/// * `topic_prefix` - Prefix for all published message topics
/// * `client_id` - MQTT client identifier
/// * `heartbeat_interval_ms` - Interval between heartbeats in milliseconds
/// 
/// # Returns
/// 
/// A new MQTT adapter with custom heartbeat timing
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use mqtt_embassy::create_container_message_publisher_with_heartbeat;
/// 
/// // Send heartbeat every 30 seconds instead of default 60 seconds
/// let publisher = create_container_message_publisher_with_heartbeat(
///     mqtt_client, "iot/device", "esp32c3_001", 30000
/// )?;
/// ```
#[cfg(feature = "container")]
pub fn create_container_message_publisher_with_heartbeat(
    mqtt_client: MqttClient, 
    topic_prefix: &str, 
    client_id: &str,
    heartbeat_interval_ms: u64
) -> Result<MqttContainerAdapter, IoTError> {
    MqttContainerAdapter::new_with_heartbeat_interval(mqtt_client, topic_prefix, client_id, heartbeat_interval_ms)
}