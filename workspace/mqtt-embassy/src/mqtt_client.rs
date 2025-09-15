//! MQTT client implementation using Embassy async framework
//!
//! Provides async MQTT publishing capabilities based on the working example
//! from wifi-simple-must-working, adapted for Embassy integration.

extern crate alloc;

use alloc::vec::Vec;
use core::net::Ipv4Addr;
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use rtt_target::rprintln;

use crate::message::{MqttMessage, SensorData, DeviceStatus, MqttPayload};

/// MQTT client configuration
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub broker_ip: Ipv4Addr,
    pub broker_port: u16,
    pub client_id: &'static str,
    pub topic_prefix: &'static str,
    pub keep_alive: u16,
}

impl Default for MqttConfig {
    fn default() -> Self {
        // Parse broker IP from environment variable
        let broker_ip_str = env!("MQTT_BROKER_IP", "Set MQTT_BROKER_IP in .cargo/config.toml");
        let ip_parts: Vec<&str> = broker_ip_str.split('.').collect();
        let broker_ip = if ip_parts.len() == 4 {
            if let (Ok(a), Ok(b), Ok(c), Ok(d)) = (
                ip_parts[0].parse::<u8>(),
                ip_parts[1].parse::<u8>(),
                ip_parts[2].parse::<u8>(),
                ip_parts[3].parse::<u8>(),
            ) {
                Ipv4Addr::new(a, b, c, d)
            } else {
                Ipv4Addr::new(192, 168, 1, 100) // Fallback
            }
        } else {
            Ipv4Addr::new(192, 168, 1, 100) // Fallback
        };
        
        // Parse broker port from environment variable
        let broker_port_str = env!("MQTT_BROKER_PORT", "Set MQTT_BROKER_PORT in .cargo/config.toml");
        let broker_port = broker_port_str.parse::<u16>().unwrap_or(1883);
        
        Self {
            broker_ip,
            broker_port,
            client_id: env!("MQTT_CLIENT_ID", "Set MQTT_CLIENT_ID in .cargo/config.toml"),
            topic_prefix: env!("MQTT_TOPIC_PREFIX", "Set MQTT_TOPIC_PREFIX in .cargo/config.toml"),
            keep_alive: 60,
        }
    }
}

/// MQTT client errors
#[derive(Debug)]
pub enum MqttError {
    /// Network connection failed
    ConnectionFailed(&'static str),
    /// MQTT protocol error
    ProtocolError(&'static str),
    /// Send/receive error
    IoError(&'static str),
    /// JSON serialization error
    SerializationError(&'static str),
}

impl core::fmt::Display for MqttError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MqttError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            MqttError::ProtocolError(msg) => write!(f, "MQTT protocol error: {}", msg),
            MqttError::IoError(msg) => write!(f, "I/O error: {}", msg),
            MqttError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

/// MQTT client using Embassy async framework
pub struct MqttClient {
    config: MqttConfig,
}

impl MqttClient {
    /// Create new MQTT client
    pub fn new(config: MqttConfig) -> Self {
        Self { config }
    }
    
    /// Create MQTT CONNECT packet (based on working example)
    fn create_connect_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // Fixed header
        packet.push(0x10); // CONNECT packet type
        
        // Variable header and payload
        let mut variable_header = Vec::new();
        
        // Protocol name "MQTT"
        let protocol_name = b"MQTT";
        variable_header.extend_from_slice(&(protocol_name.len() as u16).to_be_bytes());
        variable_header.extend_from_slice(protocol_name);
        
        // Protocol version (4 for MQTT 3.1.1)
        variable_header.push(0x04);
        
        // Connect flags (clean session)
        variable_header.push(0x02);
        
        // Keep alive
        variable_header.extend_from_slice(&self.config.keep_alive.to_be_bytes());
        
        // Payload - Client ID
        let client_id_bytes = self.config.client_id.as_bytes();
        variable_header.extend_from_slice(&(client_id_bytes.len() as u16).to_be_bytes());
        variable_header.extend_from_slice(client_id_bytes);
        
        // Remaining length
        packet.push(variable_header.len() as u8);
        packet.extend_from_slice(&variable_header);
        
        packet
    }
    
    /// Create MQTT PUBLISH packet (based on working example)
    fn create_publish_packet(&self, message: &MqttMessage) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // Fixed header - PUBLISH packet type with flags
        let mut flags = 0x30; // PUBLISH
        if message.retain {
            flags |= 0x01;
        }
        flags |= (message.qos & 0x03) << 1;
        packet.push(flags);
        
        // Variable header and payload
        let mut variable_header = Vec::new();
        
        // Topic name
        variable_header.extend_from_slice(&(message.topic.len() as u16).to_be_bytes());
        variable_header.extend_from_slice(message.topic.as_bytes());
        
        // Packet identifier (for QoS > 0)
        if message.qos > 0 {
            variable_header.extend_from_slice(&1u16.to_be_bytes()); // Packet ID = 1
        }
        
        // Payload
        variable_header.extend_from_slice(message.payload);
        
        // Remaining length
        packet.push(variable_header.len() as u8);
        packet.extend_from_slice(&variable_header);
        
        packet
    }
    
    /// Connect to MQTT broker using Embassy TCP socket
    pub async fn connect<'a>(&self, stack: &Stack<'static>, rx_buffer: &'a mut [u8], tx_buffer: &'a mut [u8]) -> Result<TcpSocket<'a>, MqttError> {
        rprintln!("[MQTT] Connecting to broker {}:{}", 
                 self.config.broker_ip, self.config.broker_port);
        
        // Create TCP socket with provided buffers
        let mut socket = TcpSocket::new(*stack, rx_buffer, tx_buffer);
        
        // Connect to MQTT broker
        let broker_addr = (self.config.broker_ip, self.config.broker_port);
        socket.connect(broker_addr).await
            .map_err(|_| MqttError::ConnectionFailed("TCP connection failed"))?;
        
        rprintln!("[MQTT] TCP connection established");
        
        // Send MQTT CONNECT packet
        let connect_packet = self.create_connect_packet();
        socket.write_all(&connect_packet).await
            .map_err(|_| MqttError::IoError("Failed to send CONNECT packet"))?;
        
        rprintln!("[MQTT] CONNECT packet sent");
        
        // Read CONNACK response
        let mut buffer = [0u8; 64];
        let n = socket.read(&mut buffer).await
            .map_err(|_| MqttError::IoError("Failed to read CONNACK"))?;
        
        if n >= 4 && buffer[0] == 0x20 && buffer[3] == 0x00 {
            rprintln!("[MQTT] CONNACK received - connection accepted");
            Ok(socket)
        } else {
            Err(MqttError::ProtocolError("Invalid CONNACK response"))
        }
    }
    
    /// Publish a single MQTT message
    pub async fn publish<'a>(&self, socket: &mut TcpSocket<'a>, message: &MqttMessage<'_>) -> Result<(), MqttError> {
        rprintln!("[MQTT] Publishing to topic '{}'", message.topic);
        
        let publish_packet = self.create_publish_packet(message);
        socket.write_all(&publish_packet).await
            .map_err(|_| MqttError::IoError("Failed to send PUBLISH packet"))?;
        
        rprintln!("[MQTT] Message published successfully");
        Ok(())
    }
    
    /// Publish sensor data as JSON (following CLAUDE.md specification)
    pub async fn publish_sensor_data<'a>(
        &self, 
        socket: &mut TcpSocket<'a>, 
        sensor_data: &SensorData
    ) -> Result<(), MqttError> {
        let payload = MqttPayload::new(sensor_data.clone());
        let json_str = payload.to_json()
            .map_err(|e| MqttError::SerializationError(e))?;
        
        let topic = "esp32/sensor/bme280";
        let message = MqttMessage::new(topic, json_str.as_bytes());
        
        self.publish(socket, &message).await
    }
    
    /// Publish device status
    pub async fn publish_device_status<'a>(
        &self, 
        socket: &mut TcpSocket<'a>, 
        status: &DeviceStatus
    ) -> Result<(), MqttError> {
        let json_str = status.to_json()
            .map_err(|e| MqttError::SerializationError(e))?;
        
        let topic = "esp32/status";
        let message = MqttMessage::new(topic, json_str.as_bytes());
        
        self.publish(socket, &message).await
    }
    
    /// Publish simple heartbeat message
    pub async fn publish_heartbeat<'a>(&self, socket: &mut TcpSocket<'a>) -> Result<(), MqttError> {
        let topic = "esp32/heartbeat"; 
        let payload = b"ping";
        let message = MqttMessage::new(topic, payload);
        
        self.publish(socket, &message).await
    }
    
    /// Get topic name with prefix
    pub fn get_topic(&self, suffix: &str) -> heapless::String<64> {
        let mut topic = heapless::String::new();
        topic.push_str(self.config.topic_prefix).ok();
        topic.push('/').ok();
        topic.push_str(suffix).ok();
        topic
    }
}

/// Embassy task for continuous MQTT publishing
#[embassy_executor::task]
pub async fn mqtt_publish_task(
    stack: &'static Stack<'static>,
    config: MqttConfig,
) {
    rprintln!("[MQTT] Task started, waiting for network...");
    
    // Wait for network to be ready
    stack.wait_config_up().await;
    
    if let Some(network_config) = stack.config_v4() {
        rprintln!("[MQTT] Network ready, IP: {}", network_config.address.address());
    }
    
    let client = MqttClient::new(config);
    
    loop {
        // Create fresh buffers for each connection attempt
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];
        
        // Connect and use socket within the same scope as buffers
        let connection_result = client.connect(stack, &mut rx_buffer, &mut tx_buffer).await;
        match connection_result {
            Ok(mut socket) => {
                rprintln!("[MQTT] Connected successfully");
                
                // Publish test messages
                let test_sensor_data = SensorData::new(25.5, 60.2, 1013.25);
                let test_status = DeviceStatus::new("online", 300, 32000, -45);
                
                // Initial status message
                if let Err(e) = client.publish_device_status(&mut socket, &test_status).await {
                    rprintln!("[MQTT] ERROR: Failed to publish status: {}", e);
                    continue;
                }
                
                // Periodic sensor data publishing
                let mut counter = 0;
                loop {
                    counter += 1;
                    
                    // Publish sensor data every 10 seconds
                    if let Err(e) = client.publish_sensor_data(&mut socket, &test_sensor_data).await {
                        rprintln!("[MQTT] ERROR: Failed to publish sensor data: {}", e);
                        break;
                    }
                    
                    // Publish heartbeat every 30 seconds
                    if counter % 3 == 0 {
                        if let Err(e) = client.publish_heartbeat(&mut socket).await {
                            rprintln!("[MQTT] ERROR: Failed to publish heartbeat: {}", e);
                            break;
                        }
                    }
                    
                    Timer::after(Duration::from_secs(10)).await;
                }
            }
            Err(e) => {
                rprintln!("[MQTT] ERROR: Connection failed: {}", e);
                Timer::after(Duration::from_secs(5)).await; // Retry after 5 seconds
            }
        }
    }
}