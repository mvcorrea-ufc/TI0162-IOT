//! WiFi + MQTT Integration Test
//! 
//! This example demonstrates WiFi connectivity + MQTT publishing
//! using the working wifi-embassy module with direct MQTT implementation.
//! 
//! ## Usage
//! 1. Ensure your MQTT broker is running at 10.10.10.210:1883
//! 2. Start monitoring: mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v
//! 3. Run: cargo run --example wifi_mqtt_test --release

#![no_std]
#![no_main]

extern crate alloc;
use alloc::{vec::Vec, format};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_net::tcp::TcpSocket;
use embedded_io_async::Write;
use core::net::Ipv4Addr;

use wifi_embassy::{WiFiManager, WiFiConfig};

// Environment variables from .cargo/config.toml
const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");

// MQTT Configuration
const MQTT_BROKER_IP: &str = "10.10.10.210";
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_CLIENT_ID: &str = "esp32-c3-wifi-mqtt";

// Simple MQTT CONNECT packet
fn create_mqtt_connect_packet() -> Vec<u8> {
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
    
    // Keep alive (60 seconds)
    variable_header.extend_from_slice(&60u16.to_be_bytes());
    
    // Client ID
    let client_id_bytes = MQTT_CLIENT_ID.as_bytes();
    variable_header.extend_from_slice(&(client_id_bytes.len() as u16).to_be_bytes());
    variable_header.extend_from_slice(client_id_bytes);
    
    // Remaining length
    packet.push(variable_header.len() as u8);
    packet.extend_from_slice(&variable_header);
    
    packet
}

// Simple MQTT PUBLISH packet - Fixed format
fn create_mqtt_publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();
    
    // Calculate remaining length
    let topic_len = topic.len();
    let remaining_length = 2 + topic_len + payload.len(); // 2 bytes for topic length + topic + payload
    
    // Fixed header - PUBLISH packet type (QoS 0, no retain, no dup)
    packet.push(0x30); // PUBLISH packet type
    packet.push(remaining_length as u8); // Remaining length
    
    // Variable header - Topic name
    packet.extend_from_slice(&(topic_len as u16).to_be_bytes()); // Topic length (2 bytes)
    packet.extend_from_slice(topic.as_bytes()); // Topic string
    
    // Payload
    packet.extend_from_slice(payload);
    
    packet
}

#[embassy_executor::task]
async fn mqtt_publisher_task(wifi_manager: &'static WiFiManager) {
    rprintln!("MQTT Publisher: Starting MQTT publishing task");
    
    // Wait for WiFi to be established
    Timer::after(Duration::from_secs(10)).await;
    
    // Main publishing loop
    let mut counter = 0;
    loop {
        counter += 1;
        
        // Check WiFi status
        if let Some(connection_info) = wifi_manager.get_connection_info() {
            rprintln!("MQTT Publisher: WiFi OK - IP: {}", connection_info.ip_address);
            
            // Get network stack
            let stack = wifi_manager.get_stack();
            
            // Create socket buffers
            let mut rx_buffer = [0; 1024];
            let mut tx_buffer = [0; 1024];
            let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
            
            // Try to connect to MQTT broker
            rprintln!("MQTT Publisher: Connecting to broker {}:{}...", MQTT_BROKER_IP, MQTT_BROKER_PORT);
            
            let broker_addr: (Ipv4Addr, u16) = (MQTT_BROKER_IP.parse().unwrap(), MQTT_BROKER_PORT);
            
            match socket.connect(broker_addr).await {
                Ok(()) => {
                    rprintln!("MQTT Publisher: ✅ TCP connected to broker!");
                    
                    // Send MQTT CONNECT packet
                    let connect_packet = create_mqtt_connect_packet();
                    match socket.write_all(&connect_packet).await {
                        Ok(()) => {
                            rprintln!("MQTT Publisher: ✅ MQTT CONNECT sent");
                            
                            // Read CONNACK
                            let mut buffer = [0u8; 64];
                            match socket.read(&mut buffer).await {
                                Ok(n) if n >= 4 && buffer[0] == 0x20 && buffer[3] == 0x00 => {
                                    rprintln!("MQTT Publisher: ✅ CONNACK received - connected to broker!");
                                    
                                    // Create mock sensor data
                                    let temperature = 22.0 + (counter as f32 * 0.1) % 5.0;
                                    let humidity = 68.0 + (counter as f32 * 0.2) % 15.0;
                                    let pressure = 1013.25 + (counter as f32 * 0.05) % 3.0;
                                    
                                    // Create JSON payload (simple format)
                                    let json_payload = format!(
                                        "{{\"temperature\":{:.1},\"humidity\":{:.1},\"pressure\":{:.1},\"reading\":{}}}",
                                        temperature, humidity, pressure, counter
                                    );
                                    
                                    rprintln!("MQTT Publisher: Publishing - T:{:.1}°C H:{:.1}% P:{:.1}hPa",
                                             temperature, humidity, pressure);
                                    
                                    // Publish sensor data
                                    let publish_packet = create_mqtt_publish_packet(
                                        "esp32/sensor/bme280", 
                                        json_payload.as_bytes()
                                    );
                                    
                                    rprintln!("MQTT Publisher: Publishing packet size: {} bytes", publish_packet.len());
                                    rprintln!("MQTT Publisher: Packet content: {:02X?}", &publish_packet[..publish_packet.len().min(20)]);
                                    
                                    match socket.write_all(&publish_packet).await {
                                        Ok(()) => {
                                            rprintln!("MQTT Publisher: ✅ Sensor data published to esp32/sensor/bme280");
                                            // Add small delay to ensure packet is processed
                                            Timer::after(Duration::from_millis(100)).await;
                                        }
                                        Err(e) => {
                                            rprintln!("MQTT Publisher: ❌ Failed to publish: {:?}", e);
                                        }
                                    }
                                    
                                    // Publish heartbeat every 5 readings
                                    if counter % 5 == 0 {
                                        let heartbeat_packet = create_mqtt_publish_packet(
                                            "esp32/heartbeat",
                                            b"ping"
                                        );
                                        
                                        match socket.write_all(&heartbeat_packet).await {
                                            Ok(()) => {
                                                rprintln!("MQTT Publisher: ✅ Heartbeat published to esp32/heartbeat");
                                            }
                                            Err(e) => {
                                                rprintln!("MQTT Publisher: ❌ Failed to publish heartbeat: {:?}", e);
                                            }
                                        }
                                    }
                                    
                                    // Publish device status every 10 readings
                                    if counter % 10 == 0 {
                                        let status_payload = format!(
                                            "{{\"status\":\"online\",\"uptime\":{},\"free_heap\":45000,\"wifi_rssi\":-42}}",
                                            counter * 30
                                        );
                                        
                                        let status_packet = create_mqtt_publish_packet(
                                            "esp32/status",
                                            status_payload.as_bytes()
                                        );
                                        
                                        match socket.write_all(&status_packet).await {
                                            Ok(()) => {
                                                rprintln!("MQTT Publisher: ✅ Status published to esp32/status");
                                            }
                                            Err(e) => {
                                                rprintln!("MQTT Publisher: ❌ Failed to publish status: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                Ok(_) => {
                                    rprintln!("MQTT Publisher: ❌ Invalid CONNACK received");
                                }
                                Err(e) => {
                                    rprintln!("MQTT Publisher: ❌ Failed to read CONNACK: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            rprintln!("MQTT Publisher: ❌ Failed to send CONNECT: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    rprintln!("MQTT Publisher: ❌ TCP connection failed: {:?}", e);
                }
            }
            
            // Wait a bit before closing socket to ensure messages are processed
            Timer::after(Duration::from_millis(500)).await;
            
            // Close socket
            socket.close();
            
        } else {
            rprintln!("MQTT Publisher: ❌ No WiFi connection");
        }
        
        // Wait 30 seconds between publications
        rprintln!("MQTT Publisher: ⏳ Waiting 30 seconds for next publication...");
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator (required for WiFi and MQTT)
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("================================================");
    rprintln!("🚀 ESP32-C3 WiFi + MQTT Integration Test");
    rprintln!("📡 WiFi: {} -> MQTT: {}:{}", WIFI_SSID, MQTT_BROKER_IP, MQTT_BROKER_PORT);
    rprintln!("================================================");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("✅ Embassy time driver initialized");
    
    // Create WiFi configuration
    let wifi_config = WiFiConfig {
        ssid: WIFI_SSID,
        password: WIFI_PASSWORD,
    };

    rprintln!("🔧 Starting WiFi connection...");

    // Initialize WiFi manager
    let wifi_manager = match WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await {
        Ok(manager) => {
            rprintln!("✅ WiFi manager initialized successfully!");
            manager
        }
        Err(e) => {
            rprintln!("❌ Failed to initialize WiFi: {}", e);
            panic!("WiFi initialization failed");
        }
    };

    // Show WiFi connection information
    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("");
        rprintln!("🎉 WiFi Connected Successfully!");
        rprintln!("📡 Network Details:");
        rprintln!("  📍 IP Address: {}", connection_info.ip_address);
        rprintln!("  🌐 Gateway: {:?}", connection_info.gateway);
        rprintln!("  🔧 Subnet: /{}", connection_info.subnet_prefix);
        rprintln!("");
        rprintln!("🎯 MQTT Broker: {}:{}", MQTT_BROKER_IP, MQTT_BROKER_PORT);
        rprintln!("💡 Monitor: mosquitto_sub -h {} -t 'esp32/#' -v", MQTT_BROKER_IP);
        rprintln!("");
    } else {
        rprintln!("❌ WiFi connection failed!");
        panic!("Cannot continue without WiFi");
    }

    // Create static reference for the MQTT task
    use static_cell::StaticCell;
    static WIFI_MANAGER: StaticCell<WiFiManager> = StaticCell::new();
    let wifi_manager_static = WIFI_MANAGER.init(wifi_manager);

    // Spawn MQTT publishing task
    spawner.spawn(mqtt_publisher_task(wifi_manager_static)).ok();

    rprintln!("🚀 MQTT publishing task started!");
    rprintln!("📊 Publishing sensor data every 30 seconds...");

    // Main loop
    loop {
        Timer::after(Duration::from_secs(60)).await;
        rprintln!("💓 System heartbeat - WiFi + MQTT operational");
    }
}