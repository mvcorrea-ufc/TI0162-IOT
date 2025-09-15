//! MQTT Embassy Working Test Example
//! 
//! This example demonstrates MQTT publishing using the mqtt-embassy module
//! with a working WiFi connection.
//! 
//! ## Usage
//! 1. Ensure your MQTT broker is running and accessible
//! 2. Update .cargo/config.toml with your credentials:
//!    ```toml
//!    [env]
//!    WIFI_SSID = "YourNetworkName"
//!    WIFI_PASSWORD = "YourPassword"
//!    MQTT_BROKER_IP = "10.10.10.210"
//!    MQTT_BROKER_PORT = "1883"
//!    ```
//! 3. Run: `cargo run --example mqtt_test_working --features examples --release`

#![no_std]
#![no_main]

extern crate alloc;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use wifi_embassy::{WiFiManager, WiFiConfig};
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};

// Environment variables from .cargo/config.toml
const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");

#[embassy_executor::task]
async fn mqtt_publishing_task(wifi_manager: &'static mut WiFiManager) {
    rprintln!("MQTT Task: Starting MQTT publishing task");
    
    // Wait a bit for WiFi to be fully established
    Timer::after(Duration::from_secs(5)).await;
    
    // Verify WiFi connection
    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("MQTT Task: WiFi connected, IP: {}", connection_info.ip_address);
    } else {
        rprintln!("MQTT Task: No WiFi connection available!");
        return;
    }
    
    // Get network stack from WiFi manager
    let stack = wifi_manager.get_stack();
    
    // Initialize MQTT client with environment variables
    let mqtt_config = MqttConfig::default();
    let broker_ip = mqtt_config.broker_ip;
    let broker_port = mqtt_config.broker_port;
    let client = MqttClient::new(mqtt_config);
    
    rprintln!("MQTT Task: Connecting to MQTT broker at {}:{}...", 
             broker_ip, broker_port);
    
    // Main MQTT publishing loop
    let mut reading_counter = 0;
    loop {
        reading_counter += 1;
        
        // Create mock sensor data (simulating BME280 readings with simple variation)
        let temperature = 22.0 + (reading_counter as f32 * 0.1) % 5.0;
        let humidity = 68.0 - (reading_counter as f32 * 0.2) % 10.0;
        let pressure = 1013.25 + (reading_counter as f32 * 0.05) % 3.0;
        
        let sensor_data = SensorData::new(temperature, humidity, pressure);
        
        rprintln!("MQTT Task: Reading #{} - T: {:.1}°C, H: {:.1}%, P: {:.1} hPa", 
                 reading_counter, temperature, humidity, pressure);
        
        // Create socket buffers for MQTT connection
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];
        
        // Attempt MQTT connection and publishing
        match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
            Ok(mut socket) => {
                rprintln!("MQTT Task: ✅ Connected to MQTT broker successfully!");
                
                // Publish sensor data
                match client.publish_sensor_data(&mut socket, &sensor_data).await {
                    Ok(_) => rprintln!("MQTT Task: ✅ Sensor data published to topic 'esp32/sensor/bme280'"),
                    Err(e) => rprintln!("MQTT Task: ❌ Failed to publish sensor data: {}", e),
                }
                
                // Publish device status every 5 readings (5 minutes)
                if reading_counter % 5 == 0 {
                    let uptime_seconds = reading_counter * 60; // Approximate uptime
                    let device_status = DeviceStatus::new("online", uptime_seconds, 48000, -38);
                    
                    match client.publish_device_status(&mut socket, &device_status).await {
                        Ok(_) => rprintln!("MQTT Task: ✅ Device status published to topic 'esp32/status'"),
                        Err(e) => rprintln!("MQTT Task: ❌ Failed to publish device status: {}", e),
                    }
                }
                
                // Publish heartbeat every 10 readings (10 minutes)
                if reading_counter % 10 == 0 {
                    match client.publish_heartbeat(&mut socket).await {
                        Ok(_) => rprintln!("MQTT Task: ✅ Heartbeat published to topic 'esp32/heartbeat'"),
                        Err(e) => rprintln!("MQTT Task: ❌ Failed to publish heartbeat: {}", e),
                    }
                }
                
                rprintln!("MQTT Task: 🔄 MQTT session completed successfully");
            }
            Err(e) => {
                rprintln!("MQTT Task: ❌ Failed to connect to MQTT broker: {}", e);
                rprintln!("MQTT Task: Will retry on next reading cycle...");
            }
        }
        
        // Wait 60 seconds between readings (like real sensor intervals)
        rprintln!("MQTT Task: ⏳ Waiting 60 seconds for next reading...");
        Timer::after(Duration::from_secs(60)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator (required for WiFi and MQTT)
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("================================================");
    rprintln!("🚀 ESP32-C3 MQTT Embassy Test");
    rprintln!("📡 WiFi + MQTT Integration Test");
    rprintln!("================================================");
    rprintln!("Target SSID: {}", WIFI_SSID);
    rprintln!("MQTT Broker: {}:{}", 
             env!("MQTT_BROKER_IP", "10.10.10.210"), 
             env!("MQTT_BROKER_PORT", "1883"));

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver BEFORE creating WiFi manager
    let timer_group1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("✅ Embassy time driver initialized successfully");
    
    // Check for placeholder credentials
    if WIFI_SSID == "YourWiFiNetwork" || WIFI_PASSWORD == "YourWiFiPassword" {
        rprintln!("⚠️  WARNING: Using placeholder WiFi credentials!");
        rprintln!("Please update .cargo/config.toml with your actual WiFi credentials");
        loop {
            Timer::after(Duration::from_secs(5)).await;
            rprintln!("Waiting for proper WiFi credentials...");
        }
    }

    // Create WiFi configuration
    let wifi_config = WiFiConfig {
        ssid: WIFI_SSID,
        password: WIFI_PASSWORD,
    };

    rprintln!("🔧 Hardware initialized, starting WiFi connection...");

    // Initialize WiFi manager
    let wifi_manager = match WiFiManager::new(
        spawner,
        peripherals.TIMG0,  // Use TIMG0 for WiFi (TIMG1 used for Embassy)
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
        rprintln!("  🏷️  DNS Servers: {:?}", connection_info.dns_servers);
        rprintln!("");
        rprintln!("💡 Test MQTT: mosquitto_sub -h {} -t 'esp32/#' -v", 
                 env!("MQTT_BROKER_IP", "10.10.10.210"));
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
    spawner.spawn(mqtt_publishing_task(wifi_manager_static)).ok();

    rprintln!("🚀 MQTT publishing task started!");
    rprintln!("📊 Publishing sensor data every 60 seconds...");
    rprintln!("🔄 System operational - monitor your MQTT broker!");
    rprintln!("💡 Subscribe to messages: mosquitto_sub -h {} -t 'esp32/#' -v", 
             env!("MQTT_BROKER_IP", "10.10.10.210"));

    // Main system heartbeat loop - simplified without wifi_manager access
    loop {
        Timer::after(Duration::from_secs(300)).await; // Every 5 minutes
        rprintln!("💓 System heartbeat - WiFi + MQTT operational");
    }
}