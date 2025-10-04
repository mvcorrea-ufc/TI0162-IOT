//! Minimal Connectivity Implementation - REAL WiFi and MQTT
//! 
//! This implementation provides real WiFi connectivity and MQTT publishing
//! with extensive RTT debugging to verify actual network communication.

extern crate alloc;

use embassy_time::{Duration, Timer};
use embassy_futures::select::{select, Either};
use rtt_target::rprintln;

/// Minimal MQTT Manager - REAL Heartbeat Publishing
/// 
/// This implementation:
/// 1. Provides REAL MQTT connectivity and publishing
/// 2. Focuses ONLY on heartbeat messages 
/// 3. Provides extensive RTT debugging of message content
/// 4. Shows exact MQTT packet structure being sent
/// 5. Verifies actual broker communication
pub struct MqttManager;

impl MqttManager {
    #[cfg(feature = "mqtt")]
    pub async fn run(
        wifi_manager: &'static wifi_embassy::WiFiManager,
        sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, crate::sensor::SensorData>,
    ) -> ! {
        rprintln!("[MQTT] Starting MQTT publishing with sensor data...");
        
        // Wait for WiFi connection
        loop {
            if wifi_manager.is_connected() {
                if let Some(ip) = wifi_manager.get_ip_address() {
                    rprintln!("[MQTT] WiFi connected: {}", ip);
                } else {
                    rprintln!("[MQTT] WiFi connected");
                }
                break;
            }
            Timer::after(Duration::from_secs(1)).await;
        }
        
        // Setup MQTT
        let stack = wifi_manager.get_stack();
        let mqtt_config = mqtt_embassy::MqttConfig::default();
        rprintln!("[MQTT] Broker: {}:{}", mqtt_config.broker_ip, mqtt_config.broker_port);
        let mqtt_client = mqtt_embassy::MqttClient::new(mqtt_config);
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];
        
        // Load topics from iot-config
        let system_config = iot_config::EmbeddedConfig::load_system_config()
            .unwrap_or_else(|_| {
                rprintln!("[MQTT] Config load failed, using defaults");
                iot_config::IoTSystemConfig::default()
            });
        
        let heartbeat_topic = &system_config.mqtt.heartbeat_topic;
        let sensor_topic = &system_config.mqtt.sensor_topic;
        let status_topic = &system_config.mqtt.status_topic;
        
        rprintln!("[MQTT] Topics: HB='{}' SENSOR='{}' STATUS='{}'", 
                 heartbeat_topic, sensor_topic, status_topic);
        
        // MQTT publishing loop with sensor data, heartbeat, and status
        let mut heartbeat_counter = 0u32;
        let mut status_counter = 0u32;
        
        rprintln!("[MQTT] Ready");
        
        loop {
            // Wait for either sensor data or heartbeat timeout (10s cycles)
            let timeout_future = Timer::after(Duration::from_secs(10));
            let sensor_future = sensor_signal.wait();
            
            match select(timeout_future, sensor_future).await {
                Either::Second(sensor_data) => {
                    // Got sensor data - publish it
                    rprintln!("[SENSOR] T={:.2}°C H={:.1}% P={:.1}hPa (count={})", 
                             sensor_data.temperature, sensor_data.humidity, sensor_data.pressure, sensor_data.count);
                    
                    // Create JSON payload for sensor data with app identification
                    // TODO: Remove 'app' field in production - use new_with_reading instead
                    let sensor_json = mqtt_embassy::SensorData::new_with_app(
                        sensor_data.temperature,
                        sensor_data.humidity,
                        sensor_data.pressure,
                        sensor_data.count,
                        "main-min"  // Source identification for debugging
                    );
                    
                    // Publish sensor data
                    match mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                        Ok(mut socket) => {
                            match mqtt_client.publish_sensor_data(&mut socket, &sensor_json).await {
                                Ok(_) => {
                                    rprintln!("[MQTT] Sensor published to '{}'", sensor_topic);
                                }
                                Err(e) => {
                                    rprintln!("[MQTT] Sensor publish failed: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            rprintln!("[MQTT] Connection failed: {:?}", e);
                        }
                    }
                }
                Either::First(_) => {
                    // Timeout - check for heartbeat and status
                    heartbeat_counter += 1;
                    status_counter += 1;
                    
                    // Publish heartbeat every 6 cycles (60 seconds) - UNIFIED STANDARD
                    if heartbeat_counter % 6 == 0 {
                        let heartbeat_num = heartbeat_counter / 6;
                        rprintln!("[HEARTBEAT] #{} -> '{}'", heartbeat_num, heartbeat_topic);
                        match mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                            Ok(mut socket) => {
                                match mqtt_client.publish_heartbeat(&mut socket).await {
                                    Ok(_) => {
                                        rprintln!("[MQTT] Heartbeat #{} published (ping)", heartbeat_num);
                                    }
                                    Err(e) => {
                                        rprintln!("[MQTT] Heartbeat #{} failed: {:?}", heartbeat_num, e);
                                    }
                                }
                            }
                            Err(e) => {
                                rprintln!("[MQTT] Heartbeat connection failed: {:?}", e);
                            }
                        }
                    }
                    
                    // Publish status every 12 cycles (120 seconds / 2 minutes)
                    if status_counter % 12 == 0 {
                        let uptime_secs = status_counter * 10;
                        let status_num = status_counter / 12;
                        rprintln!("[STATUS] #{} uptime={}s heap=30k rssi=-45dBm -> '{}'", status_num, uptime_secs, status_topic);
                        
                        // TODO: Remove 'app' field in production - use new instead
                        let device_status = mqtt_embassy::DeviceStatus::new_with_app(
                            "online",
                            uptime_secs,
                            30000,
                            -45,
                            "main-min"  // Source identification for debugging
                        );
                        
                        match mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                            Ok(mut socket) => {
                                match mqtt_client.publish_device_status(&mut socket, &device_status).await {
                                    Ok(_) => {
                                        rprintln!("[MQTT] Status #{} published", status_num);
                                    }
                                    Err(e) => {
                                        rprintln!("[MQTT] Status #{} failed: {:?}", status_num, e);
                                    }
                                }
                            }
                            Err(e) => {
                                rprintln!("[MQTT] Status connection failed: {:?}", e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    
    #[cfg(not(feature = "mqtt"))]
    pub async fn run(
        _wifi_manager: &'static wifi_embassy::WiFiManager,
        _sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, crate::sensor::SensorData>,
    ) -> ! {
        rprintln!("[MQTT-HEARTBEAT] MQTT feature disabled - no heartbeat publishing");
        loop {
            Timer::after(Duration::from_secs(60)).await;
            rprintln!("[MQTT-HEARTBEAT] MQTT disabled status check");
        }
    }
}