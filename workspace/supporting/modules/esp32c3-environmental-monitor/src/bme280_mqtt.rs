//! Simple ESP32-C3 IoT System with BME280 Sensor
//! Optimized version with working external sensor detection

#![no_std]
#![no_main]

extern crate alloc;
use alloc::{vec::Vec, format};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_net::tcp::TcpSocket;
use embedded_io_async::{Write, Read};
use core::net::Ipv4Addr;

use simple_iot::{WiFiManager, WiFiConfig};
use esp_hal::{
    i2c::master::{I2c, Config as I2cConfig},
    delay::Delay,
    Blocking,
};
// Use our working custom BME280 implementation from the library
use simple_iot::{SimpleBME280, Measurements};

// EXACT copy of working external sensor finder detection logic
fn scan_i2c_for_bme280(i2c: &mut I2c<Blocking>, delay: &mut Delay) -> Option<u8> {
    rprintln!("[I2C] Looking for EXTERNAL BME280/BMP280 on GPIO8/GPIO9...");
    
    // NO additional delay here - already done in caller
    
    // Test the exact addresses that worked before
    let bme_addresses = [0x76, 0x77];
    
    for &addr in &bme_addresses {
        rprintln!("[I2C] 📍 Detailed test for address 0x{:02X}:", addr);
        
        // Test address ping
        match i2c.transaction(addr, &mut []) {
            Ok(_) => rprintln!("[I2C]   ✅ Address responds"),
            Err(e) => {
                rprintln!("[I2C]   ❌ No response: {:?}", e);
                continue;
            }
        }
        
        delay.delay_millis(100);
        
        // Try to read BME280/BMP280 chip ID register (0xD0) - EXACT same method as finder
        rprintln!("[I2C]   🔍 Reading chip ID register (0xD0)...");
        
        // Method 1: Direct write_read (this worked in finder!)
        let mut chip_id = [0u8; 1];
        match i2c.write_read(addr, &[0xD0], &mut chip_id) {
            Ok(_) => {
                rprintln!("[I2C]     Method 1 (write_read): SUCCESS - Chip ID: 0x{:02X}", chip_id[0]);
                
                match chip_id[0] {
                    0x60 => {
                        rprintln!("[I2C]     🎯 FOUND EXTERNAL BME280 at address 0x{:02X}!", addr);
                        return Some(addr);
                    }
                    0x58 => {
                        rprintln!("[I2C]     🎯 FOUND EXTERNAL BMP280 at address 0x{:02X}!", addr);
                        return Some(addr);
                    }
                    0x61 => {
                        rprintln!("[I2C]     🎯 FOUND EXTERNAL BME680 at address 0x{:02X}!", addr);
                        return Some(addr);
                    }
                    _ => {
                        rprintln!("[I2C]     ⚠️  Unknown chip ID: 0x{:02X} (not BME280/BMP280)", chip_id[0]);
                    }
                }
            }
            Err(e) => {
                rprintln!("[I2C]     Method 1 (write_read): FAILED - {:?}", e);
                
                // Method 2: Separate write then read (fallback from finder)
                delay.delay_millis(50);
                
                match i2c.write(addr, &[0xD0]) {
                    Ok(_) => {
                        delay.delay_millis(20); // Give sensor time to prepare data
                        
                        let mut chip_id = [0u8; 1];
                        match i2c.read(addr, &mut chip_id) {
                            Ok(_) => {
                                rprintln!("[I2C]     Method 2 (separate ops): SUCCESS - Chip ID: 0x{:02X}", chip_id[0]);
                                
                                match chip_id[0] {
                                    0x60 => {
                                        rprintln!("[I2C]     🎯 FOUND EXTERNAL BME280 at address 0x{:02X}!", addr);
                                        return Some(addr);
                                    }
                                    0x58 => {
                                        rprintln!("[I2C]     🎯 FOUND EXTERNAL BMP280 at address 0x{:02X}!", addr);
                                        return Some(addr);
                                    }
                                    _ => {
                                        rprintln!("[I2C]     ⚠️  Unknown chip ID: 0x{:02X}", chip_id[0]);
                                    }
                                }
                            }
                            Err(e) => rprintln!("[I2C]     Method 2 (read): FAILED - {:?}", e),
                        }
                    }
                    Err(e) => rprintln!("[I2C]     Method 2 (write): FAILED - {:?}", e),
                }
            }
        }
        
        delay.delay_millis(200);
    }
    
    rprintln!("[I2C] ❌ EXTERNAL BME280/BMP280 NOT FOUND");
    rprintln!("[I2C] Check if finder tool still works: cargo run -p simple-iot --bin find_external_bme280 --release");
    
    None
}

// Environment variables from .cargo/config.toml
const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");
const MQTT_BROKER_IP: &str = env!("MQTT_BROKER_IP", "Set MQTT_BROKER_IP in .cargo/config.toml");
const MQTT_TOPIC_PREFIX: &str = env!("MQTT_TOPIC_PREFIX", "Set MQTT_TOPIC_PREFIX in .cargo/config.toml");

// MQTT Configuration
const MQTT_BROKER_PORT: u16 = 1883;
const MQTT_CLIENT_ID: &str = "esp32-c3-simple-iot";

// Sensor data structure (compatible with BME280 measurements)
#[derive(Debug, Clone, serde::Serialize)]
pub struct SensorData {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub timestamp: u64,
}

impl SensorData {
    fn new(temperature: f32, humidity: f32, pressure: f32) -> Self {
        use embassy_time::Instant;
        Self {
            temperature,
            humidity,
            pressure,
            timestamp: Instant::now().as_millis(),
        }
    }
}

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

// Simple MQTT PUBLISH packet
fn create_mqtt_publish_packet(topic: &str, payload: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();
    
    // Calculate remaining length
    let topic_len = topic.len();
    let remaining_length = 2 + topic_len + payload.len();
    
    // Fixed header
    packet.push(0x30); // PUBLISH packet type (QoS 0)
    packet.push(remaining_length as u8);
    
    // Variable header (topic)
    packet.extend_from_slice(&(topic_len as u16).to_be_bytes());
    packet.extend_from_slice(topic.as_bytes());
    
    // Payload
    packet.extend_from_slice(payload);
    
    packet
}

async fn mqtt_publish(stack: &embassy_net::Stack<'_>, topic: &str, payload: &str) -> bool {
    // Create TCP socket
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    
    // Parse broker IP
    let broker_ip: Ipv4Addr = MQTT_BROKER_IP.parse().unwrap();
    let remote_endpoint = (broker_ip, MQTT_BROKER_PORT);
    
    // Connect to MQTT broker
    if socket.connect(remote_endpoint).await.is_ok() {
        // Send MQTT CONNECT
        let connect_packet = create_mqtt_connect_packet();
        if socket.write_all(&connect_packet).await.is_ok() {
            // Read CONNACK
            let mut connack_buf = [0u8; 4];
            if socket.read_exact(&mut connack_buf).await.is_ok() {
                // Send PUBLISH packet
                let publish_packet = create_mqtt_publish_packet(topic, payload.as_bytes());
                if socket.write_all(&publish_packet).await.is_ok() {
                    socket.flush().await.ok();
                    return true;
                }
            }
        }
    }
    false
}

#[embassy_executor::task]
async fn sensor_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[SENSOR] Starting REAL BME280 sensor task");
    
    // Initialize I2C for BME280 using EXACT same approach as working finder
    let peripherals = unsafe { esp_hal::peripherals::Peripherals::steal() };
    let mut delay = Delay::new();
    
    // Use exact same I2C setup as the working finder tool
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);

    rprintln!("[BME280] I2C initialized - SDA: GPIO8, SCL: GPIO9");
    
    // Use exact same stabilization timing as working finder
    delay.delay_millis(1000);
    rprintln!("[BME280] Extended stabilization complete (1000ms delay)");
    
    // Scan for BME280 sensor - MUST find real sensor
    let sensor_addr = match scan_i2c_for_bme280(&mut i2c, &mut delay) {
        Some(addr) => {
            rprintln!("[SENSOR] ✅ External sensor found at address 0x{:02X}", addr);
            addr
        }
        None => {
            rprintln!("[SENSOR] ❌ No external BME280/BMP280 sensor detected!");
            rprintln!("[SENSOR] MQTT will continue with status/heartbeat only");
            return;
        }
    };
    
    // Initialize real BME280/BMP280 with our working custom implementation
    rprintln!("[SENSOR] Initializing sensor at address 0x{:02X}...", sensor_addr);
    
    // Use our working custom BME280 implementation
    let mut sensor = SimpleBME280::new(&mut i2c, sensor_addr);
    
    // Initialize the sensor (important step!)
    match sensor.init() {
        Ok(_) => {
            rprintln!("[SENSOR] ✅ Custom BME280 initialized and configured successfully!");
            rprintln!("[SENSOR] Note: Using our working custom BME280 implementation");
        }
        Err(e) => {
            rprintln!("[SENSOR] ❌ Failed to initialize custom sensor: {}", e);
            rprintln!("[SENSOR] Sensor task terminating");
            return;
        }
    }
    
    // Wait for WiFi to be ready
    Timer::after(Duration::from_secs(3)).await;
    
    loop {
        if wifi_manager.is_connected() {
            // Read REAL sensor data using our working custom implementation
            rprintln!("[BME280] Reading sensor data with custom implementation...");
            
            // Use our working custom read_measurements method
            match sensor.read_measurements() {
                Ok(measurements) => {
                    rprintln!("[BME280] ✅ Custom measurements read successfully");
                    
                    rprintln!("[BME280] Temperature: {:.2}°C", measurements.temperature);
                    rprintln!("[BME280] Pressure: {:.2} hPa", measurements.pressure);
                    
                    let (sensor_type, topic_suffix) = if measurements.humidity > 0.0 {
                        rprintln!("[BME280] Humidity: {:.2}%", measurements.humidity);
                        rprintln!("[BME280] ✅ REAL BME280 DATA: T={:.1}°C, H={:.1}%, P={:.1}hPa", 
                                 measurements.temperature, measurements.humidity, measurements.pressure);
                        ("BME280", "bme280")
                    } else {
                        rprintln!("[BMP280] Humidity: N/A (BMP280 or disabled)");
                        rprintln!("[BMP280] ✅ REAL BMP280 DATA: T={:.1}°C, P={:.1}hPa", 
                                 measurements.temperature, measurements.pressure);
                        ("BMP280", "bmp280")
                    };
                    
                    // Create sensor data
                    let sensor_data = SensorData::new(measurements.temperature, measurements.humidity, measurements.pressure);
                    
                    // Create JSON payload
                    let json_payload = if sensor_type == "BMP280" {
                        format!(
                            r#"{{"temperature":{:.3},"pressure":{:.3},"sensor":"BMP280","timestamp":{}}}"#,
                            sensor_data.temperature, sensor_data.pressure, sensor_data.timestamp
                        )
                    } else {
                        format!(
                            r#"{{"temperature":{:.3},"humidity":{:.3},"pressure":{:.3},"sensor":"BME280","timestamp":{}}}"#,
                            sensor_data.temperature, sensor_data.humidity, sensor_data.pressure, sensor_data.timestamp
                        )
                    };
                    
                    // Publish to MQTT
                    let topic = format!("{}/sensor/{}", MQTT_TOPIC_PREFIX, topic_suffix);
                    let stack = wifi_manager.get_stack();
                    
                    if mqtt_publish(stack, &topic, &json_payload).await {
                        rprintln!("[MQTT] ✅ Published REAL {} data", sensor_type);
                        rprintln!("[MQTT] Topic: {} | Data: {}", topic, json_payload);
                    } else {
                        rprintln!("[MQTT] ❌ Failed to publish {} data", sensor_type);
                    }
                }
                Err(e) => {
                    rprintln!("[BME280] ❌ Failed to read custom measurements: {}", e);
                    
                    // Send error status to MQTT
                    let error_payload = r#"{"error":"CUSTOM_BME280_READ_FAILED","status":"check_sensor"}"#;
                    let topic = format!("{}/sensor/error", MQTT_TOPIC_PREFIX);
                    let stack = wifi_manager.get_stack();
                    mqtt_publish(stack, &topic, error_payload).await;
                }
            }
        }
        
        Timer::after(Duration::from_secs(10)).await;
    }
}

#[embassy_executor::task]
async fn status_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[STATUS] Starting status task");
    
    Timer::after(Duration::from_secs(10)).await;
    
    loop {
        if wifi_manager.is_connected() {
            let status_json = r#"{"status":"online","uptime_mins":5,"wifi":"connected","mqtt":"connected","sensor":"external_bme280"}"#;
            let topic = format!("{}/status", MQTT_TOPIC_PREFIX);
            let stack = wifi_manager.get_stack();
            
            if mqtt_publish(stack, &topic, status_json).await {
                rprintln!("[STATUS] ✅ System status published");
            }
        }
        
        Timer::after(Duration::from_secs(300)).await; // Every 5 minutes
    }
}

#[embassy_executor::task]
async fn heartbeat_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[HEARTBEAT] Starting heartbeat task");
    
    Timer::after(Duration::from_secs(15)).await;
    
    loop {
        if wifi_manager.is_connected() {
            let topic = format!("{}/heartbeat", MQTT_TOPIC_PREFIX);
            let stack = wifi_manager.get_stack();
            
            if mqtt_publish(stack, &topic, "ping").await {
                rprintln!("[HEARTBEAT] ✅ Heartbeat sent");
            }
        }
        
        Timer::after(Duration::from_secs(120)).await; // Every 2 minutes
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("🚀 ESP32-C3 Simple IoT System - OPTIMIZED");
    rprintln!("========================================");
    rprintln!("📡 WiFi SSID: {}", WIFI_SSID);
    rprintln!("📡 MQTT Broker: {}:{}", MQTT_BROKER_IP, MQTT_BROKER_PORT);
    rprintln!("🌡️  Sensor: External BME280/BMP280 on GPIO8/GPIO9");
    
    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("✅ Embassy time driver initialized");
    
    // Configure WiFi
    let wifi_config = WiFiConfig {
        ssid: WIFI_SSID,
        password: WIFI_PASSWORD,
    };
    
    // Initialize WiFi manager
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config
    ).await.unwrap();
    
    let wifi_manager: &'static WiFiManager = {
        use static_cell::StaticCell;
        static CELL: StaticCell<WiFiManager> = StaticCell::new();
        CELL.init(wifi_manager)
    };
    
    rprintln!("✅ WiFi manager initialized");
    
    // Wait for WiFi connection
    rprintln!("[WIFI] Connecting...");
    loop {
        if wifi_manager.is_connected() {
            rprintln!("[WIFI] ✅ Connected to WiFi!");
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    
    // Test MQTT connectivity
    rprintln!("🧪 Testing MQTT connectivity...");
    let stack = wifi_manager.get_stack();
    let test_topic = format!("{}/test", MQTT_TOPIC_PREFIX);
    if mqtt_publish(stack, &test_topic, "Hello from optimized ESP32-C3!").await {
        rprintln!("✅ MQTT test successful!");
    } else {
        rprintln!("❌ MQTT test failed!");
    }
    
    // Spawn all tasks
    spawner.spawn(sensor_task(wifi_manager)).ok();
    spawner.spawn(status_task(wifi_manager)).ok();
    spawner.spawn(heartbeat_task(wifi_manager)).ok();
    
    rprintln!("🎯 All tasks started - REAL BME280 data only");
    rprintln!("==========================================");
    
    // Main loop
    loop {
        Timer::after(Duration::from_secs(30)).await;
        rprintln!("💓 System operational - WiFi: {}", wifi_manager.is_connected());
    }
}