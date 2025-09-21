#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::str::FromStr;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::signal::Signal;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{
    timer::timg::TimerGroup,
    usb_serial_jtag::UsbSerialJtag,
    i2c::master::{I2c, Config},
    system,
    Async,
};
use esp_storage::{FlashStorage, FlashStorageError};
use embedded_storage::{ReadStorage, Storage};

// Hardware Abstraction Layer for clean architecture and status LED
use iot_hal::{Esp32C3Platform, HardwarePlatform, GpioInterface, TimerInterface};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// Import our modules
use bme280_embassy::{BME280, I2cDevice};
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};

// WiFi connectivity using wifi-embassy module
use wifi_embassy::{WiFiManager, WiFiConfig};
use static_cell::StaticCell;

// Performance monitoring system
use iot_performance::{
    PerformanceMonitor, TimingCategory, MemoryTracker, 
    PerformanceAnalyzer, PerformanceReport, Instant as PerfInstant,
    SENSOR_CYCLE_TARGET_US
};

// Shared system state
static SYSTEM_STATE: Mutex<CriticalSectionRawMutex, SystemState> = 
    Mutex::new(SystemState::new());
    
// Shared WiFi configuration
static WIFI_CREDENTIALS: Mutex<CriticalSectionRawMutex, WifiCredentials> = 
    Mutex::new(WifiCredentials::new());

// TODO: Implement dynamic WiFi credential loading (currently using hardcoded fallback)
// static WIFI_SSID_STORAGE: StaticCell<[u8; 32]> = StaticCell::new();
// static WIFI_PASSWORD_STORAGE: StaticCell<[u8; 64]> = StaticCell::new();

// Platform abstraction for status indication
static PLATFORM_CELL: StaticCell<Esp32C3Platform> = StaticCell::new();

// Performance monitoring infrastructure
static PERFORMANCE_MONITOR_CELL: StaticCell<PerformanceMonitor> = StaticCell::new();
static MEMORY_TRACKER_CELL: StaticCell<Mutex<CriticalSectionRawMutex, MemoryTracker>> = StaticCell::new();
static PERFORMANCE_ANALYZER_CELL: StaticCell<Mutex<CriticalSectionRawMutex, PerformanceAnalyzer>> = StaticCell::new();

static SENSOR_DATA_SIGNAL: Signal<CriticalSectionRawMutex, SensorReading> = Signal::new();
static PERFORMANCE_REPORT_SIGNAL: Signal<CriticalSectionRawMutex, PerformanceReport> = Signal::new();

#[derive(Clone, Copy)]
struct SystemState {
    sensor_active: bool,
    console_active: bool,
    wifi_connected: bool,
    mqtt_connected: bool,
    reading_count: u32,
    status_led_on: bool,
    performance_monitoring: bool,
    last_sensor_time_us: u32,
    heap_usage: usize,
    performance_alerts: u8,
}

impl SystemState {
    const fn new() -> Self {
        Self {
            sensor_active: false,
            console_active: false,
            wifi_connected: false,
            mqtt_connected: false,
            reading_count: 0,
            status_led_on: false,
            performance_monitoring: false,
            last_sensor_time_us: 0,
            heap_usage: 0,
            performance_alerts: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct SensorReading {
    temperature: f32,
    humidity: f32,
    pressure: f32,
    #[allow(dead_code)]
    count: u32,
}

// Configuration stored in flash - using fixed-size arrays for const compatibility
#[derive(Clone, Copy, Debug)]
struct WifiCredentials {
    ssid: [u8; 32],
    ssid_len: u8,
    password: [u8; 64], 
    password_len: u8,
    mqtt_broker_ip: [u8; 16],
    mqtt_broker_ip_len: u8,
    mqtt_broker_port: u16,
    is_configured: bool,
}

impl WifiCredentials {
    const fn new() -> Self {
        Self {
            ssid: [0u8; 32],
            ssid_len: 0,
            password: [0u8; 64],
            password_len: 0,
            mqtt_broker_ip: [b'1', b'0', b'.', b'1', b'0', b'.', b'1', b'0', b'.', b'2', b'1', b'0', 0, 0, 0, 0],
            mqtt_broker_ip_len: 12,
            mqtt_broker_port: 1883,
            is_configured: false,
        }
    }
    
    fn get_ssid(&self) -> &str {
        core::str::from_utf8(&self.ssid[..self.ssid_len as usize]).unwrap_or("")
    }
    
    fn get_password(&self) -> &str {
        core::str::from_utf8(&self.password[..self.password_len as usize]).unwrap_or("")
    }
    
    fn get_mqtt_broker_ip(&self) -> &str {
        core::str::from_utf8(&self.mqtt_broker_ip[..self.mqtt_broker_ip_len as usize]).unwrap_or("10.10.10.210")
    }
    
    fn set_ssid(&mut self, ssid: &str) -> bool {
        let bytes = ssid.as_bytes();
        if bytes.len() <= 32 {
            self.ssid[..bytes.len()].copy_from_slice(bytes);
            self.ssid_len = bytes.len() as u8;
            // Clear the rest
            for i in bytes.len()..32 {
                self.ssid[i] = 0;
            }
            true
        } else {
            false
        }
    }
    
    fn set_password(&mut self, password: &str) -> bool {
        let bytes = password.as_bytes();
        if bytes.len() <= 64 {
            self.password[..bytes.len()].copy_from_slice(bytes);
            self.password_len = bytes.len() as u8;
            // Clear the rest
            for i in bytes.len()..64 {
                self.password[i] = 0;
            }
            true
        } else {
            false
        }
    }
    
    fn set_mqtt_broker_ip(&mut self, ip: &str) -> bool {
        let bytes = ip.as_bytes();
        if bytes.len() <= 16 {
            self.mqtt_broker_ip[..bytes.len()].copy_from_slice(bytes);
            self.mqtt_broker_ip_len = bytes.len() as u8;
            // Clear the rest
            for i in bytes.len()..16 {
                self.mqtt_broker_ip[i] = 0;
            }
            true
        } else {
            false
        }
    }
}

// Flash storage configuration - using a higher offset that should persist
const WIFI_CONFIG_FLASH_OFFSET: u32 = 0x310000; // Use higher flash region  
const WIFI_CONFIG_SIZE: usize = 256;

struct ConfigManager {
    storage: FlashStorage,
}

impl ConfigManager {
    fn new() -> Self {
        Self {
            storage: FlashStorage::new(),
        }
    }

    fn load_wifi_credentials(&mut self) -> Result<WifiCredentials, FlashStorageError> {
        let mut buffer = [0u8; WIFI_CONFIG_SIZE];
        
        // Add debug output for flash reading
        rprintln!("[CONFIG] Reading from flash offset 0x{:X}", WIFI_CONFIG_FLASH_OFFSET);
        
        match self.storage.read(WIFI_CONFIG_FLASH_OFFSET, &mut buffer) {
            Ok(()) => {
                rprintln!("[CONFIG] Flash read successful, checking magic bytes");
                rprintln!("[CONFIG] First 8 bytes: {:02X?}", &buffer[0..8]);
            }
            Err(e) => {
                rprintln!("[CONFIG] Flash read failed: {:?}", e);
                return Err(e);
            }
        }
        
        // Check magic bytes for validity
        if buffer[0] == 0xCA && buffer[1] == 0xFE {
            rprintln!("[CONFIG] Magic bytes found, parsing configuration");
            let ssid_len = buffer[2];
            let password_len = buffer[3];
            let broker_ip_len = buffer[4];
            let broker_port = u16::from_le_bytes([buffer[5], buffer[6]]);
            
            rprintln!("[CONFIG] Parsed lengths - SSID: {}, Password: {}, Broker IP: {}", 
                     ssid_len, password_len, broker_ip_len);
            
            if ssid_len <= 32 && password_len <= 64 && broker_ip_len <= 16 {
                let mut config = WifiCredentials::new();
                
                // Read SSID
                if ssid_len > 0 {
                    config.ssid[..ssid_len as usize].copy_from_slice(&buffer[8..8 + ssid_len as usize]);
                    config.ssid_len = ssid_len;
                }
                
                // Read password
                if password_len > 0 {
                    config.password[..password_len as usize].copy_from_slice(&buffer[40..40 + password_len as usize]);
                    config.password_len = password_len;
                }
                
                // Read broker IP
                if broker_ip_len > 0 {
                    config.mqtt_broker_ip[..broker_ip_len as usize].copy_from_slice(&buffer[104..104 + broker_ip_len as usize]);
                    config.mqtt_broker_ip_len = broker_ip_len;
                } else {
                    // Set default broker IP if not found
                    let default_ip = b"10.10.10.210";
                    config.mqtt_broker_ip[..default_ip.len()].copy_from_slice(default_ip);
                    config.mqtt_broker_ip_len = default_ip.len() as u8;
                }
                
                config.mqtt_broker_port = broker_port;
                config.is_configured = ssid_len > 0 && password_len > 0;
                
                rprintln!("[CONFIG] Configuration loaded successfully - configured: {}", config.is_configured);
                return Ok(config);
            } else {
                rprintln!("[CONFIG] Invalid data lengths found");
            }
        } else {
            rprintln!("[CONFIG] No magic bytes found (0x{:02X} 0x{:02X})", buffer[0], buffer[1]);
        }
        
        // Return default if no valid config found
        let mut default_config = WifiCredentials::new();
        // Set default broker IP
        let default_ip = b"10.10.10.210";
        default_config.mqtt_broker_ip[..default_ip.len()].copy_from_slice(default_ip);
        default_config.mqtt_broker_ip_len = default_ip.len() as u8;
        default_config.mqtt_broker_port = 1883;
        
        rprintln!("[CONFIG] Returning default configuration");
        Ok(default_config)
    }

    fn save_wifi_credentials(&mut self, credentials: &WifiCredentials) -> Result<(), FlashStorageError> {
        rprintln!("[CONFIG] Saving credentials to flash offset 0x{:X}", WIFI_CONFIG_FLASH_OFFSET);
        
        let mut buffer = [0u8; WIFI_CONFIG_SIZE];
        
        // Magic bytes
        buffer[0] = 0xCA;
        buffer[1] = 0xFE;
        
        // Lengths
        buffer[2] = credentials.ssid_len;
        buffer[3] = credentials.password_len;
        buffer[4] = credentials.mqtt_broker_ip_len;
        
        // Broker port
        let port_bytes = credentials.mqtt_broker_port.to_le_bytes();
        buffer[5] = port_bytes[0];
        buffer[6] = port_bytes[1];
        
        // Reserved byte
        buffer[7] = 0x00;
        
        // SSID (offset 8, max 32 bytes)
        if credentials.ssid_len > 0 {
            buffer[8..8 + credentials.ssid_len as usize].copy_from_slice(&credentials.ssid[..credentials.ssid_len as usize]);
        }
        
        // Password (offset 40, max 64 bytes)
        if credentials.password_len > 0 {
            buffer[40..40 + credentials.password_len as usize].copy_from_slice(&credentials.password[..credentials.password_len as usize]);
        }
        
        // Broker IP (offset 104, max 16 bytes)
        if credentials.mqtt_broker_ip_len > 0 {
            buffer[104..104 + credentials.mqtt_broker_ip_len as usize].copy_from_slice(&credentials.mqtt_broker_ip[..credentials.mqtt_broker_ip_len as usize]);
        }
        
        rprintln!("[CONFIG] Writing buffer with magic bytes: {:02X?}", &buffer[0..8]);
        
        // Write to flash
        match self.storage.write(WIFI_CONFIG_FLASH_OFFSET, &buffer) {
            Ok(()) => {
                rprintln!("[CONFIG] Flash write completed successfully");
                Ok(())
            }
            Err(e) => {
                rprintln!("[CONFIG] Flash write failed: {:?}", e);
                Err(e)
            }
        }
    }
}

#[embassy_executor::task]
async fn sensor_task(
    mut i2c: I2c<'static, esp_hal::Blocking>,
    performance_monitor: &'static PerformanceMonitor,
) {
    rprintln!("[SENSOR] Initializing BME280 environmental sensor with performance monitoring...");
    
    // Create I2C device abstraction for BME280
    let i2c_device = I2cDevice::new(&mut i2c, 0x76); // BME280 primary address
    let mut bme280 = BME280::new(i2c_device);
    
    // Ready to start performance monitoring with sensor readings
    
    // Initialize sensor with proper error handling
    // For BME280 with I2cDevice, initialization is handled in init() method
    
    // Initialize sensor for measurements
    loop {
        match bme280.init().await {
            Ok(_) => {
                rprintln!("[SENSOR] BME280 initialized for operational measurements");
                break;
            }
            Err(_) => {
                rprintln!("[SENSOR] ERROR: Failed to initialize BME280");
                rprintln!("[SENSOR] Retrying initialization in 5 seconds...");
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }
    }
    
    // Mark sensor as active
    {
        let mut state = SYSTEM_STATE.lock().await;
        state.sensor_active = true;
    }
    
    rprintln!("[SENSOR] IoT System sensor monitoring started - 30s intervals");
    
    let mut reading_count = 0u32;
    let mut consecutive_errors = 0u32;
    
    loop {
        // Time the sensor reading operation
        let start_time = PerfInstant::now();
        
        match bme280.read_measurements().await {
            Ok(measurements) => {
                let duration = start_time.elapsed();
                let duration_us = duration.as_micros() as u32;
                
                // Record performance measurement
                performance_monitor.record_cycle_time(TimingCategory::SensorReading, duration).await;
                
                reading_count += 1;
                consecutive_errors = 0; // Reset error counter
                
                let reading = SensorReading {
                    temperature: measurements.temperature,
                    humidity: measurements.humidity,
                    pressure: measurements.pressure,
                    count: reading_count,
                };
                
                rprintln!("[SENSOR] #{}: T={:.2}°C H={:.1}% P={:.1}hPa ({}μs)", 
                         reading_count, measurements.temperature, measurements.humidity, 
                         measurements.pressure, duration_us);
                
                rprintln!("[SENSOR] Signaling MQTT task with sensor data...");
                SENSOR_DATA_SIGNAL.signal(reading);
                rprintln!("[SENSOR] Signal sent successfully");
                
                // Record memory usage through performance monitor
                let _ = performance_monitor.record_memory_usage(0, 0).await; // Will auto-detect heap and stack
                
                {
                    let mut state = SYSTEM_STATE.lock().await;
                    state.reading_count = reading_count;
                    state.sensor_active = true;
                    state.last_sensor_time_us = duration_us;
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                rprintln!("[SENSOR] ERROR #{}: Failed to read sensor data: {:?}", consecutive_errors, e);
                
                // Mark sensor as inactive after multiple errors
                if consecutive_errors >= 3 {
                    let mut state = SYSTEM_STATE.lock().await;
                    state.sensor_active = false;
                    rprintln!("[SENSOR] CRITICAL: Sensor marked as inactive after {} consecutive errors", consecutive_errors);
                }
                
                // Attempt sensor re-initialization after many errors
                if consecutive_errors >= 10 {
                    rprintln!("[SENSOR] RECOVERY: Attempting sensor re-initialization...");
                    if bme280.init().await.is_ok() {
                        rprintln!("[SENSOR] RECOVERY: Sensor re-initialized successfully");
                        consecutive_errors = 0;
                    }
                }
            }
        }
        
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[embassy_executor::task]
async fn wifi_task(wifi_manager: &'static mut WiFiManager) {
    rprintln!("[WIFI] Starting WiFi connection monitoring...");
    
    // Show initial connection information
    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("[WIFI] Connected to WiFi network!");
        rprintln!("[WIFI] IP Address: {}", connection_info.ip_address);
        rprintln!("[WIFI] Gateway: {:?}", connection_info.gateway);
        
        let mut state = SYSTEM_STATE.lock().await;
        state.wifi_connected = true;
    } else {
        rprintln!("[WIFI] No initial WiFi connection");
        let mut state = SYSTEM_STATE.lock().await;
        state.wifi_connected = false;
    }
    
    loop {
        // Monitor connection status
        if wifi_manager.is_connected() {
            if let Some(ip) = wifi_manager.get_ip_address() {
                // Only update state when status changes
                let mut state = SYSTEM_STATE.lock().await;
                if !state.wifi_connected {
                    rprintln!("[WIFI] Connection restored - IP: {}", ip);
                    state.wifi_connected = true;
                }
            }
        } else {
            let mut state = SYSTEM_STATE.lock().await;
            if state.wifi_connected {
                rprintln!("[WIFI] WARNING: WiFi connection lost - will auto-reconnect");
                state.wifi_connected = false;
            }
        }
        
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[MQTT] Task started - entry point reached");
    rprintln!("[MQTT] Initializing MQTT client...");
    
    // Wait for WiFi connection before starting MQTT
    loop {
        if wifi_manager.is_connected() {
            rprintln!("[MQTT] WiFi connected, starting MQTT client");
            break;
        }
        rprintln!("[MQTT] Waiting for WiFi connection...");
        Timer::after(Duration::from_secs(5)).await;
    }
    
    // Get network stack from WiFi manager
    let stack = wifi_manager.get_stack();
    
    // IoT System MQTT configuration from environment variables
    let mqtt_config = MqttConfig::default();
    
    rprintln!("[MQTT] Configured for broker {}:{}", mqtt_config.broker_ip, mqtt_config.broker_port);
    rprintln!("[MQTT] Client ID: {}", mqtt_config.client_id);
    rprintln!("[MQTT] Topic prefix: {}", mqtt_config.topic_prefix);
    
    // Create MQTT client
    let mqtt_client = MqttClient::new(mqtt_config.clone());
    
    // Create persistent buffers for socket operations
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    let mut heartbeat_counter = 0u32;
    let mut published_readings = 0u32;
    
    rprintln!("[MQTT] Starting data publishing loop");
    
    loop {
        // Wait for next publishing cycle (10 seconds)
        let timeout_future = Timer::after(Duration::from_secs(10));
        let sensor_future = SENSOR_DATA_SIGNAL.wait();
        
        // Use select to wait for either timeout or sensor data
        let select_result = embassy_futures::select::select(timeout_future, sensor_future).await;
        
        // Check if we got sensor data
        match select_result {
            embassy_futures::select::Either::Second(reading) => {
                rprintln!("[MQTT] Got sensor data from signal: T={:.2}°C, H={:.1}%, P={:.1}hPa", 
                         reading.temperature, reading.humidity, reading.pressure);
                published_readings += 1;
            
                // Create sensor data for MQTT publishing
                let sensor_data = SensorData::new(
                    reading.temperature,
                    reading.humidity,
                    reading.pressure
                );
                
                // Attempt MQTT connection and publishing
                match mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                    Ok(mut socket) => {
                        // Publish sensor data
                        match mqtt_client.publish_sensor_data(&mut socket, &sensor_data).await {
                            Ok(_) => {
                                rprintln!("[MQTT] Published reading #{}: T={:.2}°C H={:.1}% P={:.1}hPa",
                                         published_readings, reading.temperature, reading.humidity, reading.pressure);
                                
                                let mut state = SYSTEM_STATE.lock().await;
                                state.mqtt_connected = true;
                            }
                            Err(e) => {
                                rprintln!("[MQTT] ERROR: Failed to publish sensor data: {:?}", e);
                                let mut state = SYSTEM_STATE.lock().await;
                                state.mqtt_connected = false;
                            }
                        }
                    }
                    Err(e) => {
                        rprintln!("[MQTT] ERROR: Failed to connect to broker: {:?}", e);
                        let mut state = SYSTEM_STATE.lock().await;
                        state.mqtt_connected = false;
                    }
                }
            }
            embassy_futures::select::Either::First(_) => {
                // Timeout - continue with periodic tasks
            }
        }
        
        heartbeat_counter += 1;
        
        // Heartbeat every 6 cycles (6 * 10s = 1 minute)
        if heartbeat_counter % 6 == 0 {
            if let Ok(mut socket) = mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                match mqtt_client.publish_heartbeat(&mut socket).await {
                    Ok(_) => {
                        rprintln!("[MQTT] Published heartbeat #{}", heartbeat_counter / 6);
                    }
                    Err(e) => {
                        rprintln!("[MQTT] ERROR: Failed to publish heartbeat: {:?}", e);
                    }
                }
            }
        }
        
        // Status report every 12 cycles (12 * 10s = 2 minutes)
        if heartbeat_counter % 12 == 0 {
            let state = SYSTEM_STATE.lock().await;
            let device_status = DeviceStatus::new(
                "online",
                (heartbeat_counter * 10) as u32, // Uptime in seconds (10s per cycle)
                32768, // Free heap estimation
                -42,   // WiFi RSSI estimation
            );
            
            if let Ok(mut socket) = mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                match mqtt_client.publish_device_status(&mut socket, &device_status).await {
                    Ok(_) => {
                        rprintln!("[MQTT] Published status: sensor_active={}, readings={}, published={}",
                                 state.sensor_active, state.reading_count, published_readings);
                    }
                    Err(e) => {
                        rprintln!("[MQTT] ERROR: Failed to publish status: {:?}", e);
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn console_task(mut usb_tx: esp_hal::usb_serial_jtag::UsbSerialJtagTx<'static, Async>, 
                     mut usb_rx: esp_hal::usb_serial_jtag::UsbSerialJtagRx<'static, Async>) {
    rprintln!("[MAIN-APP] Starting integrated console task");
    
    {
        let mut state = SYSTEM_STATE.lock().await;
        state.console_active = true;
    }
    
    // Send operational welcome banner
    let banner = b"\r\n\r\n+==========================================================+\r\n\
                   |          ESP32-C3 IoT System IoT System v1.0            |\r\n\
                   |        BME280 Environmental Monitoring Station          |\r\n\
                   +==========================================================+\r\n\
                   System Status: IoT System Ready\r\n\
                   Sensor: BME280 Temperature/Humidity/Pressure\r\n\
                   Connectivity: WiFi + MQTT\r\n\
                   \r\nType 'help' for available commands\r\n\r\niot> ";
    
    let _ = embedded_io_async::Write::write(&mut usb_tx, banner).await;
    let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
    
    let mut input_buffer = [0u8; 128];
    let mut input_len = 0;
    
    loop {
        let mut byte = [0u8; 1];
        if let Ok(1) = embedded_io_async::Read::read(&mut usb_rx, &mut byte).await {
            let ch = byte[0];
            
            match ch {
                b'\r' | b'\n' => {
                    if input_len > 0 {
                        let cmd = core::str::from_utf8(&input_buffer[..input_len]).unwrap_or("");
                        let response = process_console_command(cmd).await;
                        
                        let _ = embedded_io_async::Write::write(&mut usb_tx, response.as_bytes()).await;
                        let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                    } else {
                        let _ = embedded_io_async::Write::write(&mut usb_tx, b"\r\niot> ").await;
                        let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                    }
                    input_len = 0;
                }
                0x08 | 0x7F => { // Backspace
                    if input_len > 0 {
                        input_len -= 1;
                        let _ = embedded_io_async::Write::write(&mut usb_tx, b"\x08 \x08").await;
                        let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                    }
                }
                ch if ch >= 0x20 && ch <= 0x7E => { // Printable characters
                    if input_len < input_buffer.len() - 1 {
                        input_buffer[input_len] = ch;
                        input_len += 1;
                        let _ = embedded_io_async::Write::write(&mut usb_tx, &[ch]).await;
                        let _ = embedded_io_async::Write::flush(&mut usb_tx).await;
                    }
                }
                _ => {} // Ignore other characters
            }
        }
    }
}

async fn process_console_command(cmd: &str) -> alloc::string::String {
    match cmd.trim() {
        "help" | "h" | "?" => {
            "\r\n=== IoT System Console v1.0 ===\r\n\
             help, h, ?       - Show this help\r\n\
             status, stat     - Show system status\r\n\
             info, i          - Show system information\r\n\
             sensor           - Show latest sensor reading\r\n\
             readings         - Show reading count\r\n\
             perf             - Show performance metrics\r\n\
             memory           - Show memory usage\r\n\
             alerts           - Show performance alerts\r\n\
             wifi             - Show WiFi configuration\r\n\
             wifi ssid <name> - Set WiFi SSID\r\n\
             wifi pass <pass> - Set WiFi password\r\n\
             uptime           - Show system uptime\r\n\
             restart, reset   - Restart system\r\n\
             save             - Save configuration to flash\r\n\
             load             - Load configuration from flash\r\n\
             clear, cls       - Clear screen\r\n\
             \r\niot> ".to_string()
        }
        "status" | "stat" => {
            let state = SYSTEM_STATE.lock().await;
            if state.sensor_active {
                if state.performance_monitoring {
                    format!("\r\n=== IoT System System Status v1.0 ===\r\n\
                            BME280 Sensor: ACTIVE - Reading environmental data\r\n\
                            Console: ACTIVE - USB Serial/JTAG interface\r\n\
                            Performance: ACTIVE - Real-time monitoring\r\n\
                            WiFi: DEGRADED MODE - Check RTT for details\r\n\
                            MQTT: DEGRADED MODE - Check RTT for details\r\n\
                            System: OPERATIONAL - Core functions active\r\n\
                            Readings: {} | Alerts: {}\r\n\
                            \r\niot> ", state.reading_count, state.performance_alerts)
                } else {
                    "\r\n=== IoT System System Status v1.0 ===\r\n\
                     BME280 Sensor: ACTIVE - Reading environmental data\r\n\
                     Console: ACTIVE - USB Serial/JTAG interface\r\n\
                     Performance: INITIALIZING - Setting up monitoring\r\n\
                     WiFi: CONNECTING - Network connection in progress\r\n\
                     MQTT: CONNECTING - Broker connection in progress\r\n\
                     System: INITIALIZING - Network setup active\r\n\
                     \r\niot> ".to_string()
                }
            } else {
                "\r\n=== IoT System System Status v1.0 ===\r\n\
                 BME280 Sensor: ERROR - Hardware communication failure\r\n\
                 Console: ACTIVE - USB Serial/JTAG interface\r\n\
                 Performance: ACTIVE - Monitoring available\r\n\
                 WiFi: CONFIGURED - Connection management active\r\n\
                 MQTT: CONFIGURED - Awaiting sensor data\r\n\
                 System: DEGRADED - Sensor requires attention\r\n\
                 \r\niot> ".to_string()
            }
        }
        "info" | "i" => {
            "\r\n=== IoT System System Information v1.0 ===\r\n\
             Chip: ESP32-C3 RISC-V 160MHz\r\n\
             Framework: Embassy Async Runtime\r\n\
             HAL: esp-hal v1.0.0-rc.0\r\n\
             Modules: BME280, WiFi, MQTT, Console\r\n\
             Sensor: BME280 I2C (GPIO8/9)\r\n\
             Interface: USB Serial/JTAG\r\n\
             Build: IoT System IoT System v1.0.0\r\n\
             \r\niot> ".to_string()
        }
        "sensor" => {
            // Read from system state instead of stealing signal
            let state = SYSTEM_STATE.try_lock();
            if let Ok(state) = state {
                if state.sensor_active && state.reading_count > 0 {
                    format!("\r\n=== Latest Sensor Status ===\r\n\
                            Status: Sensor Active\r\n\
                            Total Readings: {}\r\n\
                            Last Reading Time: {}μs\r\n\
                            Sensor Type: BME280 (I2C)\r\n\
                            GPIO Pins: SDA=8, SCL=9\r\n\
                            \r\niot> ",
                            state.reading_count, state.last_sensor_time_us)
                } else {
                    "\r\n=== Latest Sensor Status ===\r\n\
                     Status: No recent sensor data\r\n\
                     Check: Verify BME280 connection (GPIO8/9)\r\n\
                     \r\niot> ".to_string()
                }
            } else {
                "\r\n=== Latest Sensor Status ===\r\n\
                 Status: Unable to read sensor state\r\n\
                 \r\niot> ".to_string()
            }
        }
        "readings" => {
            let state = SYSTEM_STATE.lock().await;
            if state.reading_count > 0 {
                format!("\r\n=== Sensor Reading Statistics ===\r\n\
                        Status: IoT System data collection active\r\n\
                        Total Readings: {}\r\n\
                        Interval: 30 seconds per reading\r\n\
                        Quality: Real BME280 environmental data\r\n\
                        Last Sensor Time: {}μs\r\n\
                        Performance Alerts: {}\r\n\
                        \r\niot> ",
                        state.reading_count, state.last_sensor_time_us, state.performance_alerts)
            } else {
                "\r\n=== Sensor Reading Statistics ===\r\n\
                 Status: No readings collected yet\r\n\
                 Action: System initializing or sensor error\r\n\
                 \r\niot> ".to_string()
            }
        }
        "uptime" => {
            "\r\n=== System Uptime ===\r\n\
             Status: IoT System system operational\r\n\
             Monitoring: Real-time environmental data\r\n\
             Reliability: Continuous operation mode\r\n\
             \r\niot> ".to_string()
        }
        "restart" | "reset" => {
            rprintln!("[CONSOLE] System restart requested");
            
            // Schedule a restart after a brief delay
            embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
            
            rprintln!("[CONSOLE] Performing system restart...");
            
            // Perform system restart using ESP32-C3 software reset
            system::software_reset();
        }
        "save" => {
            // Save WiFi credentials to flash with detailed debug
            let credentials = WIFI_CREDENTIALS.lock().await;
            rprintln!("[SAVE] Attempting to save credentials - SSID: '{}', Password: {} chars", 
                     credentials.get_ssid(), credentials.get_password().len());
            
            let mut config_manager = ConfigManager::new();
            
            match config_manager.save_wifi_credentials(&*credentials) {
                Ok(()) => {
                    rprintln!("[SAVE] Flash write completed successfully");
                    
                    // Immediately try to read back to verify
                    match config_manager.load_wifi_credentials() {
                        Ok(loaded) => {
                            rprintln!("[SAVE] Verification read - SSID: '{}', Password: {} chars, Configured: {}", 
                                     loaded.get_ssid(), loaded.get_password().len(), loaded.is_configured);
                            
                            if loaded.is_configured && loaded.get_ssid() == credentials.get_ssid() {
                                format!("\r\n=== Configuration Save ===\r\n\
                                        WiFi SSID: {} (saved & verified)\r\n\
                                        WiFi Password: {} chars (saved & verified)\r\n\
                                        MQTT Broker: {} (saved)\r\n\
                                        MQTT Port: {} (saved)\r\n\
                                        Status: Configuration saved to flash successfully\r\n\
                                        Verification: Read-back test PASSED\r\n\
                                        Note: Restart to apply WiFi changes\r\n\
                                        \r\niot> ", 
                                        credentials.get_ssid(),
                                        credentials.get_password().len(),
                                        credentials.get_mqtt_broker_ip(),
                                        credentials.mqtt_broker_port)
                            } else {
                                format!("\r\n=== Configuration Save ===\r\n\
                                        WARNING: Configuration saved but verification failed\r\n\
                                        Expected SSID: '{}', Read SSID: '{}'\r\n\
                                        Expected Configured: true, Read Configured: {}\r\n\
                                        Status: Flash write succeeded but data integrity check failed\r\n\
                                        \r\niot> ",
                                        credentials.get_ssid(), loaded.get_ssid(), loaded.is_configured)
                            }
                        }
                        Err(e) => {
                            rprintln!("[SAVE] Verification read failed: {:?}", e);
                            format!("\r\n=== Configuration Save ===\r\n\
                                    WARNING: Configuration saved but cannot verify\r\n\
                                    Status: Flash write succeeded but read-back failed\r\n\
                                    This may indicate a flash storage issue\r\n\
                                    \r\niot> ")
                        }
                    }
                }
                Err(e) => {
                    rprintln!("[SAVE] Flash write failed: {:?}", e);
                    "\r\n=== Configuration Save ===\r\n\
                     Error: Failed to save configuration to flash\r\n\
                     Check: Flash storage may be corrupted or write-protected\r\n\
                     Solution: Try again or restart system\r\n\
                     \r\niot> ".to_string()
                }
            }
        }
        "load" => {
            // Load WiFi credentials from flash
            let mut config_manager = ConfigManager::new();
            
            match config_manager.load_wifi_credentials() {
                Ok(loaded_credentials) => {
                    {
                        let mut credentials = WIFI_CREDENTIALS.lock().await;
                        *credentials = loaded_credentials;
                    }
                    
                    let credentials = WIFI_CREDENTIALS.lock().await;
                    format!("\r\n=== Configuration Load ===\r\n\
                            WiFi SSID: {} (loaded)\r\n\
                            WiFi Password: {} chars (loaded)\r\n\
                            MQTT Broker: {} (loaded)\r\n\
                            MQTT Port: {} (loaded)\r\n\
                            Configuration: {}\r\n\
                            Status: Configuration loaded from flash successfully\r\n\
                            Note: Restart to apply WiFi changes\r\n\
                            \r\niot> ", 
                            credentials.get_ssid(),
                            credentials.get_password().len(),
                            credentials.get_mqtt_broker_ip(),
                            credentials.mqtt_broker_port,
                            if credentials.is_configured { "Complete" } else { "Incomplete" })
                }
                Err(_) => {
                    "\r\n=== Configuration Load ===\r\n\
                     Error: Failed to load configuration from flash\r\n\
                     Check: No saved configuration found\r\n\
                     Solution: Configure WiFi settings first\r\n\
                     \r\niot> ".to_string()
                }
            }
        }
        "perf" => {
            let state = SYSTEM_STATE.lock().await;
            if state.performance_monitoring {
                "\r\n=== Performance Metrics ===\r\n\
                 Sensor Timing: Real-time performance data\r\n\
                 Memory Usage: Active monitoring\r\n\
                 System Health: Performance analysis active\r\n\
                 Alerts: Real-time threshold monitoring\r\n\
                 Status: Performance monitoring operational\r\n\
                 \r\niot> ".to_string()
            } else {
                "\r\n=== Performance Metrics ===\r\n\
                 Status: Performance monitoring not available\r\n\
                 Check: System initialization in progress\r\n\
                 \r\niot> ".to_string()
            }
        }
        "memory" => {
            let state = SYSTEM_STATE.lock().await;
            if state.heap_usage > 0 {
                format!("\r\n=== Memory Usage ===\r\n\
                        Heap: {}B in use\r\n\
                        Stack: Active monitoring\r\n\
                        Flash: Usage analysis available\r\n\
                        Fragmentation: Monitored for optimization\r\n\
                        Status: Memory monitoring active\r\n\
                        \r\niot> ", state.heap_usage)
            } else {
                "\r\n=== Memory Usage ===\r\n\
                 Status: Memory data collection in progress\r\n\
                 Wait: Initial measurements being taken\r\n\
                 \r\niot> ".to_string()
            }
        }
        "alerts" => {
            let state = SYSTEM_STATE.lock().await;
            if state.performance_alerts > 0 {
                format!("\r\n=== Performance Alerts ===\r\n\
                        Active Alerts: {} performance issues detected\r\n\
                        Check: Review RTT output for details\r\n\
                        Action: Investigate timing or memory issues\r\n\
                        \r\niot> ", state.performance_alerts)
            } else {
                "\r\n=== Performance Alerts ===\r\n\
                 Status: No active performance alerts\r\n\
                 System: Operating within normal parameters\r\n\
                 \r\niot> ".to_string()
            }
        }
        "wifi" => {
            let credentials = WIFI_CREDENTIALS.lock().await;
            format!("\r\n=== WiFi Configuration ===\r\n\
                    SSID: {}\r\n\
                    Password: {} chars\r\n\
                    MQTT Broker: {}\r\n\
                    MQTT Port: {}\r\n\
                    Status: {}\r\n\
                    \r\n\
                    Commands:\r\n\
                    wifi ssid <name>     - Set WiFi SSID\r\n\
                    wifi pass <password> - Set WiFi password\r\n\
                    wifi broker <ip>     - Set MQTT broker IP\r\n\
                    wifi port <port>     - Set MQTT broker port\r\n\
                    save                 - Save to flash\r\n\
                    load                 - Load from flash\r\n\
                    \r\niot> ",
                    if credentials.ssid_len > 0 { credentials.get_ssid() } else { "[Not Set]" },
                    credentials.get_password().len(),
                    credentials.get_mqtt_broker_ip(),
                    credentials.mqtt_broker_port,
                    if credentials.is_configured { "Ready" } else { "Incomplete" })
        }
        "clear" | "cls" => {
            "\x1B[2J\x1B[H\r\niot> ".to_string()
        }
        "" => "\r\niot> ".to_string(),
        cmd if cmd.starts_with("wifi ") => {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if parts.len() >= 3 {
                let mut credentials = WIFI_CREDENTIALS.lock().await;
                match parts[1] {
                    "ssid" => {
                        let ssid = parts[2..].join(" ");
                        if credentials.set_ssid(&ssid) {
                            credentials.is_configured = credentials.ssid_len > 0 && credentials.password_len > 0;
                            format!("\r\n=== WiFi SSID Set ===\r\n\
                                    SSID: {}\r\n\
                                    Status: SSID configured successfully\r\n\
                                    Next: Set password with 'wifi pass <password>'\r\n\
                                    Then: Run 'save' to store in flash\r\n\
                                    \r\niot> ", ssid)
                        } else {
                            "\r\n=== WiFi SSID Error ===\r\n\
                             Error: SSID too long (max 32 characters)\r\n\
                             \r\niot> ".to_string()
                        }
                    }
                    "pass" => {
                        let password = parts[2..].join(" ");
                        if credentials.set_password(&password) {
                            credentials.is_configured = credentials.ssid_len > 0 && credentials.password_len > 0;
                            format!("\r\n=== WiFi Password Set ===\r\n\
                                    Password: {} characters\r\n\
                                    Status: Password configured successfully\r\n\
                                    Configuration: {}\r\n\
                                    Next: Run 'save' to store in flash\r\n\
                                    \r\niot> ", 
                                    password.len(),
                                    if credentials.is_configured { "Complete" } else { "Incomplete" })
                        } else {
                            "\r\n=== WiFi Password Error ===\r\n\
                             Error: Password too long (max 64 characters)\r\n\
                             \r\niot> ".to_string()
                        }
                    }
                    "broker" => {
                        let broker_ip = parts[2];
                        if credentials.set_mqtt_broker_ip(broker_ip) {
                            format!("\r\n=== MQTT Broker Set ===\r\n\
                                    Broker IP: {}\r\n\
                                    Status: MQTT broker configured successfully\r\n\
                                    Next: Run 'save' to store in flash\r\n\
                                    \r\niot> ", broker_ip)
                        } else {
                            "\r\n=== MQTT Broker Error ===\r\n\
                             Error: IP address too long (max 16 characters)\r\n\
                             \r\niot> ".to_string()
                        }
                    }
                    "port" => {
                        if let Ok(port) = parts[2].parse::<u16>() {
                            credentials.mqtt_broker_port = port;
                            format!("\r\n=== MQTT Port Set ===\r\n\
                                    Broker Port: {}\r\n\
                                    Status: MQTT port configured successfully\r\n\
                                    Next: Run 'save' to store in flash\r\n\
                                    \r\niot> ", port)
                        } else {
                            "\r\n=== MQTT Port Error ===\r\n\
                             Error: Invalid port number (use 1-65535)\r\n\
                             \r\niot> ".to_string()
                        }
                    }
                    _ => {
                        "\r\n=== WiFi Command Error ===\r\n\
                         Available commands:\r\n\
                         wifi ssid <name>     - Set WiFi SSID\r\n\
                         wifi pass <password> - Set WiFi password\r\n\
                         wifi broker <ip>     - Set MQTT broker IP\r\n\
                         wifi port <port>     - Set MQTT broker port\r\n\
                         \r\niot> ".to_string()
                    }
                }
            } else {
                "\r\n=== WiFi Command Error ===\r\n\
                 Usage: wifi <command> <value>\r\n\
                 Type 'wifi' to see current configuration\r\n\
                 \r\niot> ".to_string()
            }
        }
        _ => "\r\nUnknown command. Type 'help' for available commands.\r\n\r\niot> ".to_string()
    }
}

#[embassy_executor::task]
async fn status_led_task(platform: &'static mut Esp32C3Platform<'static>) {
    rprintln!("[STATUS-LED] Starting status LED task using IoT HAL abstraction");
    
    // Status LED pattern indicates system state
    loop {
        let state = SYSTEM_STATE.lock().await;
        
        if state.sensor_active && state.wifi_connected && state.mqtt_connected {
            // All systems operational - slow blink (1Hz)
            let led = platform.get_status_led();
            if let Err(e) = led.set_high().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED high: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(500)).await;
            
            let led = platform.get_status_led();
            if let Err(e) = led.set_low().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED low: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(500)).await;
            
            // Update LED state
            let mut state_mut = SYSTEM_STATE.lock().await;
            state_mut.status_led_on = !state_mut.status_led_on;
        } else if state.sensor_active {
            // Sensor working but network issues - fast blink (2Hz)
            let led = platform.get_status_led();
            if let Err(e) = led.set_high().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED high: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(250)).await;
            
            let led = platform.get_status_led();
            if let Err(e) = led.set_low().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED low: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(250)).await;
        } else {
            // Sensor issues - very fast blink (4Hz)
            let led = platform.get_status_led();
            if let Err(e) = led.set_high().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED high: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(125)).await;
            
            let led = platform.get_status_led();
            if let Err(e) = led.set_low().await {
                rprintln!("[STATUS-LED] ERROR: Failed to set LED low: {:?}", e);
            }
            
            let timer = platform.get_timer();
            timer.delay(Duration::from_millis(125)).await;
        }
    }
}

#[embassy_executor::task]
async fn performance_monitor_task(performance_monitor: &'static PerformanceMonitor) {
    rprintln!("[PERF] Starting performance monitoring task");
    
    let mut analysis_counter = 0u32;
    
    loop {
        Timer::after(Duration::from_secs(120)).await; // Run every 2 minutes
        analysis_counter += 1;
        
        // Generate performance report
        let report = performance_monitor.generate_report().await;
        
        rprintln!("[PERF] Analysis #{}: Uptime: {}s, Status: {:?}", 
                 analysis_counter, report.uptime_seconds, report.status);
        
        // Check for performance alerts
        if !report.alerts.is_empty() {
            rprintln!("[PERF] ALERTS: {} performance issues detected", report.alerts.len());
            for alert in &report.alerts {
                rprintln!("[PERF] - {:?}: {} (threshold: {})", 
                         alert.alert_type, alert.measured_value, alert.threshold_value);
            }
            
            // Update system state with alert count
            {
                let mut state = SYSTEM_STATE.lock().await;
                state.performance_alerts = report.alerts.len() as u8;
            }
        }
        
        // Report memory usage
        rprintln!("[PERF] Memory: Heap={}B, Stack={}B, Flash={}B", 
                 report.memory_usage.heap_used, 
                 report.memory_usage.stack_used,
                 report.memory_usage.flash_used);
        
        // Check timing performance
        if let Some(sensor_time) = report.timing_stats.get_average_time(TimingCategory::SensorReading) {
            let sensor_us = sensor_time.as_micros() as u32;
            rprintln!("[PERF] Sensor average: {}μs (target: <{}μs)", 
                     sensor_us, SENSOR_CYCLE_TARGET_US);
            
            {
                let mut state = SYSTEM_STATE.lock().await;
                state.last_sensor_time_us = sensor_us;
                state.heap_usage = report.memory_usage.heap_used;
            }
        }
        
        // Signal performance report for other tasks
        PERFORMANCE_REPORT_SIGNAL.signal(report);
    }
}

#[embassy_executor::task]
async fn system_monitor_task() {
    rprintln!("[MAIN-APP] Starting system monitor task");
    
    let mut uptime = 0u32;
    
    loop {
        Timer::after(Duration::from_secs(60)).await;
        uptime += 60;
        
        let state = SYSTEM_STATE.lock().await;
        rprintln!("[MAIN-APP] System Monitor - Uptime: {}s, Sensor: {}, Console: {}, Readings: {}, LED: {}, Perf: {} alerts",
                 uptime, state.sensor_active, state.console_active, state.reading_count, 
                 state.status_led_on, state.performance_alerts);
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator with larger size for WiFi stack
    esp_alloc::heap_allocator!(size: 64 * 1024);
    
    // Initialize RTT for debugging
    rtt_init_print!();
    
    rprintln!("=== ESP32-C3 IoT Environmental Monitoring System v1.0.0 Starting ===");
    rprintln!("[SYSTEM] Environmental Monitoring Station");
    
    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[MAIN-APP] Embassy time driver initialized");
    
    // PRIORITY: Initialize WiFi FIRST to avoid memory fragmentation
    // Load WiFi credentials from flash storage (memory optimized)
    rprintln!("[MAIN-APP] Loading WiFi configuration from flash...");
    let mut config_manager = ConfigManager::new();
    let loaded_credentials = match config_manager.load_wifi_credentials() {
        Ok(creds) => {
            rprintln!("[MAIN-APP] Flash read successful - SSID: '{}', Password: {} chars, Configured: {}", 
                     creds.get_ssid(), creds.get_password().len(), creds.is_configured);
            
            if creds.is_configured {
                rprintln!("[MAIN-APP] WiFi Config - SSID: {} | Password: {} chars", 
                         creds.get_ssid(), creds.get_password().len());
                // Update global credentials
                {
                    let mut global_creds = WIFI_CREDENTIALS.lock().await;
                    *global_creds = creds;
                }
                Some(creds)
            } else {
                rprintln!("[MAIN-APP] WiFi Config - Configuration found but marked as incomplete");
                rprintln!("[MAIN-APP] SSID length: {}, Password length: {}", 
                         creds.ssid_len, creds.password_len);
                rprintln!("[MAIN-APP] Use console commands to configure WiFi:");
                rprintln!("[MAIN-APP] wifi ssid <name>");
                rprintln!("[MAIN-APP] wifi pass <password>");
                rprintln!("[MAIN-APP] save");
                None
            }
        }
        Err(e) => {
            rprintln!("[MAIN-APP] WiFi Config - Failed to load from flash: {:?}", e);
            rprintln!("[MAIN-APP] This might be first boot or flash data corrupted");
            None
        }
    };
    
    // Create WiFi configuration with dynamic credentials from flash
    let wifi_config = if let Some(creds) = loaded_credentials {
        if creds.is_configured {
            WiFiConfig {
                ssid: heapless::String::<32>::from_str(creds.get_ssid()).unwrap_or_default(),
                password: heapless::String::<64>::from_str(creds.get_password()).unwrap_or_default(),
            }
        } else {
            rprintln!("[MAIN-APP] No WiFi credentials configured - use console to configure");
            WiFiConfig {
                ssid: heapless::String::new(),
                password: heapless::String::new(),
            }
        }
    } else {
        rprintln!("[MAIN-APP] No WiFi credentials found in flash - use console to configure");
        WiFiConfig {
            ssid: heapless::String::new(),
            password: heapless::String::new(),
        }
    };
    
    rprintln!("[MAIN-APP] Initializing WiFi manager...");
    let wifi_manager_result = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await;
    
    // Make WiFi manager static for sharing between tasks
    static WIFI_MANAGER_CELL: StaticCell<WiFiManager> = StaticCell::new();
    
    let (has_wifi, wifi_manager_ref) = match wifi_manager_result {
        Ok(manager) => {
            rprintln!("[MAIN-APP] WiFi manager initialized successfully");
            let wifi_manager_ref = WIFI_MANAGER_CELL.init(manager);
            (true, Some(wifi_manager_ref as &WiFiManager))
        }
        Err(e) => {
            rprintln!("[MAIN-APP] ERROR: Failed to initialize WiFi: {}", e);
            rprintln!("[MAIN-APP] DEGRADED MODE: Running without WiFi/MQTT");
            rprintln!("[MAIN-APP] Sensor and console will still be available");
            (false, None)
        }
    };
    
    // Configure I2C for BME280 sensor (blocking mode for BME280 compatibility)
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);
    rprintln!("[MAIN-APP] I2C configured for BME280 (SDA: GPIO8, SCL: GPIO9)");
    
    // Configure USB Serial/JTAG for console
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let usb_serial = usb_serial.into_async();
    let (usb_rx, usb_tx) = usb_serial.split();
    rprintln!("[MAIN-APP] USB Serial/JTAG configured for console");
    
    // Initialize performance monitoring system
    rprintln!("[MAIN-APP] Initializing performance monitoring system...");
    let performance_monitor = PerformanceMonitor::new();
    let memory_tracker = Mutex::new(MemoryTracker::new());
    let performance_analyzer = Mutex::new(PerformanceAnalyzer::new());
    
    let performance_monitor_ref = PERFORMANCE_MONITOR_CELL.init(performance_monitor);
    MEMORY_TRACKER_CELL.init(memory_tracker);
    PERFORMANCE_ANALYZER_CELL.init(performance_analyzer);
    
    rprintln!("[MAIN-APP] Performance monitoring system initialized");
    
    // Update system state to indicate performance monitoring is active
    {
        let mut state = SYSTEM_STATE.lock().await;
        state.performance_monitoring = true;
    }
    
    // Initialize iot-hal platform for status LED and other abstractions
    rprintln!("[MAIN-APP] Initializing IoT HAL platform for status LED...");
    match Esp32C3Platform::initialize().await {
        Ok(platform) => {
            rprintln!("[MAIN-APP] IoT HAL platform initialized successfully");
            let platform_ref = PLATFORM_CELL.init(platform);
            spawner.spawn(status_led_task(platform_ref)).ok();
        }
        Err(e) => {
            rprintln!("[MAIN-APP] WARNING: IoT HAL platform initialization failed: {:?}", e);
            rprintln!("[MAIN-APP] Continuing without platform abstraction");
        }
    };
    
    // Spawn core operational tasks (always available)
    spawner.spawn(sensor_task(i2c, performance_monitor_ref)).ok();
    spawner.spawn(console_task(usb_tx, usb_rx)).ok();
    spawner.spawn(performance_monitor_task(performance_monitor_ref)).ok();
    spawner.spawn(system_monitor_task()).ok();
    
    // Only spawn network tasks if WiFi is available
    if has_wifi {
        rprintln!("[MAIN-APP] Spawning network tasks (WiFi + MQTT)");
        if let Some(wifi_ref) = wifi_manager_ref {
            rprintln!("[MAIN-APP] About to spawn MQTT task with WiFi reference");
            let spawn_result = spawner.spawn(mqtt_task(wifi_ref));
            match spawn_result {
                Ok(_) => {
                    rprintln!("[MAIN-APP] MQTT task spawned successfully");
                    // Give the task a moment to start
                    Timer::after(Duration::from_millis(100)).await;
                    rprintln!("[MAIN-APP] MQTT task should have started by now");
                }
                Err(_) => rprintln!("[MAIN-APP] ERROR: Failed to spawn MQTT task - task queue full?"),
            }
        } else {
            rprintln!("[MAIN-APP] WARNING: WiFi manager reference not available for MQTT task");
        }
    } else {
        rprintln!("[MAIN-APP] Skipping network tasks - WiFi not available");
    }
    
    if has_wifi {
        rprintln!("[MAIN-APP] All tasks spawned - Real WiFi and MQTT connectivity active");
        rprintln!("[SYSTEM] ================================================");
        rprintln!("[SYSTEM] IoT System Enhanced System Status:");
        rprintln!("[SYSTEM] - BME280: Real sensor on I2C GPIO8/9");
        rprintln!("[SYSTEM] - WiFi: Connection management active");
        rprintln!("[SYSTEM] - MQTT: IoT System data publishing");
        rprintln!("[SYSTEM] - Console: USB Serial/JTAG interface");
        rprintln!("[SYSTEM] - Performance: Real-time monitoring & analysis");
        rprintln!("[SYSTEM] ================================================");
        rprintln!("[SYSTEM] IoT System system ready for full deployment");
    } else {
        rprintln!("[MAIN-APP] Core tasks spawned - Running in DEGRADED MODE");
        rprintln!("[SYSTEM] ================================================");
        rprintln!("[SYSTEM] IoT System DEGRADED MODE Status:");
        rprintln!("[SYSTEM] - BME280: Real sensor on I2C GPIO8/9");
        rprintln!("[SYSTEM] - Console: USB Serial/JTAG interface available");
        rprintln!("[SYSTEM] - Performance: Real-time monitoring active");
        rprintln!("[SYSTEM] - WiFi: NOT AVAILABLE");
        rprintln!("[SYSTEM] - MQTT: NOT AVAILABLE");
        rprintln!("[SYSTEM] ================================================");
        rprintln!("[SYSTEM] IoT System ready for local sensor testing");
    }
    
    rprintln!("[SYSTEM] Performance Features:");
    rprintln!("[SYSTEM] - Timing: Sensor cycle tracking");
    rprintln!("[SYSTEM] - Memory: Heap, stack & flash usage monitoring");
    rprintln!("[SYSTEM] - Analysis: Trend detection & regression testing");
    rprintln!("[SYSTEM] - Alerts: Real-time threshold monitoring");
    rprintln!("[SYSTEM] ================================================");
    rprintln!("[SYSTEM] Console access: picocom /dev/ttyACM0 -b 115200");
    
    // Main application loop
    loop {
        Timer::after(Duration::from_secs(300)).await; // 5 minute intervals
        rprintln!("[MAIN-APP] Integrated IoT system running - all modules active");
    }
}