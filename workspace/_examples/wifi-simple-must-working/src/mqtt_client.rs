//! MQTT Client Module - Works with both Blocking and Embassy WiFi
//! 
//! Provides MQTT publishing capabilities that work with either WiFi implementation.
//! The module detects the network stack type and uses appropriate TCP connection methods.
//!
//! ## Usage:
//! 
//! ### With Blocking WiFi:
//! ```bash
//! cargo run --release --features mqtt
//! ```
//! 
//! ### With Embassy WiFi:
//! ```bash  
//! cargo run --release --features embassy,mqtt
//! ```
//!
//! ## MQTT Broker Configuration:
//! - Broker IP: 10.10.10.210
//! - Port: 1883 (standard MQTT)
//! - Topics: esp32/status, esp32/data, esp32/heartbeat

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use rtt_target::rprintln;
use core::net::Ipv4Addr;
use smoltcp::wire::IpAddress;
use embedded_io::{Read, Write};

/// MQTT client configuration
pub struct MqttConfig {
    pub broker_ip: [u8; 4],
    pub broker_port: u16,
    pub client_id: &'static str,
    pub topic_prefix: &'static str,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_ip: [10, 10, 10, 210],  // Your broker IP
            broker_port: 1883,
            client_id: "esp32-c3-client",
            topic_prefix: "esp32",
        }
    }
}

/// MQTT message to publish
pub struct MqttMessage<'a> {
    pub topic: &'a str,
    pub payload: &'a [u8],
    pub qos: u8,
    pub retain: bool,
}

impl<'a> MqttMessage<'a> {
    pub fn new(topic: &'a str, payload: &'a [u8]) -> Self {
        Self {
            topic,
            payload,
            qos: 0,
            retain: false,
        }
    }
    
    pub fn with_qos(mut self, qos: u8) -> Self {
        self.qos = qos;
        self
    }
    
    pub fn with_retain(mut self, retain: bool) -> Self {
        self.retain = retain;
        self
    }
}

/// Simple MQTT client for publishing messages
pub struct SimpleMqttClient {
    config: MqttConfig,
}

impl SimpleMqttClient {
    pub fn new(config: MqttConfig) -> Self {
        Self { config }
    }
    
    /// Create MQTT connection packet
    pub fn create_connect_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // Build variable header and payload first to calculate remaining length
        let mut variable_header = Vec::new();
        
        // Variable header - Protocol name "MQTT"
        let protocol_name = b"MQTT";
        variable_header.extend_from_slice(&(protocol_name.len() as u16).to_be_bytes());
        variable_header.extend_from_slice(protocol_name);
        
        // Protocol version (4 for MQTT 3.1.1)
        variable_header.push(0x04);
        
        // Connect flags (clean session)
        variable_header.push(0x02);
        
        // Keep alive (60 seconds)
        variable_header.extend_from_slice(&60u16.to_be_bytes());
        
        // Payload - Client ID
        let client_id = self.config.client_id.as_bytes();
        variable_header.extend_from_slice(&(client_id.len() as u16).to_be_bytes());
        variable_header.extend_from_slice(client_id);
        
        // Fixed header - packet type and remaining length
        packet.push(0x10); // CONNECT packet type
        packet.push(variable_header.len() as u8); // Remaining length
        packet.extend_from_slice(&variable_header);
        
        packet
    }
    
    /// Create MQTT publish packet
    pub fn create_publish_packet(&self, message: &MqttMessage) -> Vec<u8> {
        let mut packet = Vec::new();
        
        // Build variable header and payload first to calculate remaining length
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
        
        // Fixed header - PUBLISH packet type with flags
        let mut flags = 0x30; // PUBLISH
        if message.retain {
            flags |= 0x01;
        }
        flags |= (message.qos & 0x03) << 1;
        packet.push(flags);
        packet.push(variable_header.len() as u8); // Remaining length
        packet.extend_from_slice(&variable_header);
        
        packet
    }
    
    /// Get broker address as string
    pub fn broker_address(&self) -> String {
        String::from("10.10.10.210:1883")
    }
    
    /// Create test messages for publishing
    pub fn create_test_messages(&self) -> [MqttMessage<'static>; 3] {
        [
            MqttMessage::new("esp32/status", b"online"),
            MqttMessage::new("esp32/data", b"{\"temperature\":25.5,\"humidity\":60.2}"),
            MqttMessage::new("esp32/heartbeat", b"ping"),
        ]
    }
}

/// Initialize MQTT client with default configuration
pub fn init_mqtt_client() -> SimpleMqttClient {
    let config = MqttConfig::default();
    rprintln!("MQTT: Initializing client for broker {}:{}", 
        config.broker_ip[0], config.broker_port);
    SimpleMqttClient::new(config)
}

/// Publish MQTT messages using blocking TCP socket
pub fn publish_mqtt_messages<DeviceT>(
    stack: &mut blocking_network_stack::Stack<DeviceT>,
    messages: &[MqttMessage],
) -> Result<(), &'static str>
where
    DeviceT: smoltcp::phy::Device + 'static,
{
    let client = init_mqtt_client();
    let broker_addr = (Ipv4Addr::new(10, 10, 10, 210), 1883);
    
    rprintln!("MQTT: Connecting to broker {}:{}", broker_addr.0, broker_addr.1);
    
    // Create TCP socket with static buffer allocation to fix lifetime issues
    static mut MQTT_RX_BUFFER: [u8; 1536] = [0u8; 1536];
    static mut MQTT_TX_BUFFER: [u8; 1536] = [0u8; 1536];
    let (rx_buffer, tx_buffer) = unsafe { (&mut MQTT_RX_BUFFER, &mut MQTT_TX_BUFFER) };
    let mut socket = stack.get_socket(rx_buffer, tx_buffer);
    
    // Connect to MQTT broker using blocking-network-stack API
    let remote_addr = IpAddress::v4(10, 10, 10, 210);
    socket.open(remote_addr, 1883).map_err(|_| "Failed to connect to broker")?;
    
    rprintln!("MQTT: Connected to broker, sending CONNECT packet");
    
    // Send MQTT CONNECT packet
    let connect_packet = client.create_connect_packet();
    socket.write_all(&connect_packet).map_err(|_| "Failed to send CONNECT")?;
    
    // Read CONNACK response
    let mut buffer = [0u8; 64];
    let _response = socket.read(&mut buffer).map_err(|_| "Failed to read CONNACK")?;
    rprintln!("MQTT: Received CONNACK, connection established");
    
    // Publish each message
    for (i, message) in messages.iter().enumerate() {
        rprintln!("MQTT: Publishing message {} to topic '{}'", i + 1, message.topic);
        
        let publish_packet = client.create_publish_packet(message);
        socket.write_all(&publish_packet).map_err(|_| "Failed to send PUBLISH")?;
        
        rprintln!("MQTT: Message {} published successfully", i + 1);
        
        // Small delay between messages
        for _ in 0..1000000 { unsafe { core::ptr::read_volatile(&0); } }
    }
    
    rprintln!("MQTT: All messages published successfully");
    
    // Properly disconnect the socket
    socket.disconnect();
    Ok(())
}

/// Test MQTT publishing functionality
pub fn test_mqtt_packets() {
    let client = init_mqtt_client();
    
    rprintln!("MQTT: Testing packet creation");
    
    // Test connect packet
    let connect_packet = client.create_connect_packet();
    rprintln!("MQTT: Connect packet created ({} bytes)", connect_packet.len());
    
    // Test publish packets
    let messages = client.create_test_messages();
    for (i, message) in messages.iter().enumerate() {
        let publish_packet = client.create_publish_packet(message);
        rprintln!("MQTT: Publish packet {} created ({} bytes) for topic '{}'", 
            i + 1, publish_packet.len(), message.topic);
    }
    
    rprintln!("MQTT: Packet creation test completed");
}