#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use esp_hal::{
    timer::timg::TimerGroup,
    main,
};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_net::tcp::TcpSocket;

use wifi_embassy::{WiFiManager, WiFiConfig};
use mqtt_embassy::{MqttClient, MqttConfig, SensorData};

#[embassy_executor::task]
async fn mqtt_publish_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[MQTT] Starting MQTT publish task...");
    
    // Wait for WiFi to be ready
    Timer::after(Duration::from_secs(5)).await;
    
    // Configure MQTT client
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    
    // Get network stack from WiFi manager
    let stack = wifi_manager.get_stack();
    
    let mut reading_count = 1;
    
    loop {
        // Create sample sensor data (simulated BME280 readings)
        let sensor_data = SensorData::new(
            22.5 + (reading_count as f32 * 0.1), // Temperature
            65.0 + (reading_count as f32 * 0.5), // Humidity  
            1013.2 + (reading_count as f32 * 0.1), // Pressure
        );
        
        rprintln!("[MQTT] Attempting to publish reading #{}", reading_count);
        rprintln!("[MQTT] Data: T={:.1}°C, H={:.1}%, P={:.1}hPa", 
                  sensor_data.temperature, sensor_data.humidity, sensor_data.pressure);
        
        // Buffers for TCP connection
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];
        
        // Connect to MQTT broker and publish
        match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
            Ok(mut socket) => {
                rprintln!("[MQTT] ✅ Connected to MQTT broker successfully!");
                
                // Publish sensor data
                if let Err(_) = client.publish_sensor_data(&mut socket, &sensor_data).await {
                    rprintln!("[MQTT] ❌ Failed to publish sensor data");
                } else {
                    rprintln!("[MQTT] ✅ Sensor data published to topic 'esp32/sensor/bme280'");
                }
                
                // Publish heartbeat every 5 readings
                if reading_count % 5 == 0 {
                    if let Err(_) = client.publish_heartbeat(&mut socket).await {
                        rprintln!("[MQTT] ❌ Failed to publish heartbeat");
                    } else {
                        rprintln!("[MQTT] ✅ Heartbeat published");
                    }
                }
                
                // Close connection
                socket.close();
            }
            Err(_) => {
                rprintln!("[MQTT] ❌ Failed to connect to MQTT broker");
            }
        }
        
        reading_count += 1;
        
        // Wait 30 seconds before next publication
        rprintln!("[MQTT] Waiting 30 seconds for next reading...");
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("🚀 ESP32-C3 Basic MQTT Client Test");
    rprintln!("==================================");
    rprintln!("📨 Publishing simulated sensor data to MQTT broker");
    rprintln!("🌐 Broker: {}:{}", env!("MQTT_BROKER_IP"), env!("MQTT_BROKER_PORT"));
    rprintln!("");

    // Initialize ESP32-C3 hardware
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    rprintln!("✅ Embassy time driver initialized");

    // Configure WiFi credentials from environment
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };

    rprintln!("📡 Target SSID: {}", wifi_config.ssid);
    rprintln!("🔧 Hardware initialized, starting WiFi connection...");

    // Initialize WiFi manager
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG1,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await.unwrap();

    rprintln!("✅ WiFi manager initialized successfully!");

    // Wait for WiFi connection
    Timer::after(Duration::from_secs(2)).await;

    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("");
        rprintln!("🎉 WiFi Connected Successfully!");
        rprintln!("📡 Network Details:");
        rprintln!("  📍 IP Address: {}", connection_info.ip_address);
        rprintln!("  🌐 Gateway: {:?}", connection_info.gateway);
        rprintln!("  🔧 Subnet: /{}", connection_info.subnet_prefix);
        rprintln!("");
    }

    // Spawn MQTT task
    spawner.spawn(mqtt_publish_task(&wifi_manager)).ok();

    rprintln!("🚀 MQTT task spawned - starting sensor data publication cycle");
    
    // Main loop - system monitoring
    loop {
        Timer::after(Duration::from_secs(60)).await;
        
        if let Some(info) = wifi_manager.get_connection_info() {
            rprintln!("[SYS] System running - IP: {}, Uptime: {}s", 
                      info.ip_address, embassy_time::Instant::now().as_secs());
        } else {
            rprintln!("[SYS] ⚠️  WiFi disconnected - system monitoring");
        }
    }
}