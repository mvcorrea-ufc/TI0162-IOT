//! Pure Synchronous ESP32-C3 IoT System - NO EMBASSY
//! All modules are blocking/synchronous for maximum simplicity and minimal dependencies

#![no_std]
#![no_main]

extern crate alloc;

// Import our synchronous modules
mod config;
mod bme280;
mod wifi;
mod mqtt;

use esp_hal::{
    timer::timg::TimerGroup,
    i2c::master::{I2c, Config as I2cConfig},
    rng::Rng,
};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use static_cell::StaticCell;

// Use our zero-dependency modules
use config::NodepsConfig;
use bme280::Bme280;
use mqtt::{MqttConfig, SimpleMqttClient, SensorReading};

// Global static resources for lifetime management
static I2C_STATIC: StaticCell<I2c<'static, esp_hal::Blocking>> = StaticCell::new();
static SOCKET_STORAGE: StaticCell<[smoltcp::iface::SocketStorage; 3]> = StaticCell::new();
static WIFI_CONTROLLER: StaticCell<esp_wifi::EspWifiController> = StaticCell::new();

#[esp_hal::main]  
fn main() -> ! {
    rtt_init_print!();
    
    rprintln!("=== ESP32-C3 Pure Synchronous IoT System ===");
    rprintln!("Target: Blocking/synchronous, zero Embassy dependencies");
    
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: NodepsConfig::heap_size());
    rprintln!("Heap initialized: {} bytes", NodepsConfig::heap_size());
    
    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize I2C for BME280 (GPIO8=SDA, GPIO9=SCL)
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);
    let i2c_static = I2C_STATIC.init(i2c);
    rprintln!("I2C initialized (SDA=GPIO8, SCL=GPIO9)");
    
    // Initialize BME280 sensor synchronously
    let mut bme280 = match Bme280::new_blocking(i2c_static) {
        Ok(sensor) => {
            rprintln!("[SENSOR] BME280 initialized successfully");
            sensor
        }
        Err(e) => {
            rprintln!("[SENSOR] CRITICAL: Failed to initialize BME280: {}", e);
            panic!("BME280 initialization failed");
        }
    };
    
    // CRITICAL ARCHITECTURAL DECISION:
    // Network stack MUST be created directly in main() to avoid lifetime issues.
    // Returning Stack from functions causes complex lifetime management that
    // breaks the network functionality. This pattern matches ALL working examples.
    
    // Initialize WiFi synchronously with proper peripherals and get network stack
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let rng = Rng::new(peripherals.RNG);
    
    // Initialize WiFi hardware (EXACT copy from wifi-synchronous working example) - using static storage
    let esp_wifi_ctrl = WIFI_CONTROLLER.init(esp_wifi::init(timer_group0.timer0, rng).unwrap());
    rprintln!("[WIFI] Hardware: WiFi initialized");

    // Create WiFi controller and interfaces (EXACT copy)
    let (mut controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let mut device = interfaces.sta;
    rprintln!("[WIFI] Hardware: WiFi controller created");

    // Create network interface (EXACT copy)
    let iface = wifi::create_interface(&mut device);
    
    // Set up DHCP socket (EXACT copy) - using static storage for lifetime management
    let socket_set_entries = SOCKET_STORAGE.init(Default::default());
    let mut socket_set = smoltcp::iface::SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = wifi::create_dhcp_socket();
    wifi::set_dhcp_hostname(&mut dhcp_socket, "esp32c3-dev");
    socket_set.add(dhcp_socket);
    
    // Create network stack (EXACT copy)
    let mut stack = wifi::create_stack(iface, device, socket_set);
    rprintln!("[WIFI] Network: Real network stack created");

    // WiFi configuration
    let wifi_config = wifi::WiFiConfig {
        ssid: NodepsConfig::wifi_ssid(),
        password: NodepsConfig::wifi_password(),
    };

    // Configure WiFi (EXACT copy)
    if let Err(e) = wifi::configure_wifi(&mut controller, &wifi_config) {
        rprintln!("[WIFI] ERROR: WiFi configuration failed: {}", e);
        panic!("WiFi configuration failed");
    }
    rprintln!("[WIFI] WiFi: Configuration complete");

    // Wait for connection (EXACT copy)
    rprintln!("[WIFI] Connecting to WiFi network...");
    if let Err(e) = wifi::wait_for_connection(&mut controller) {
        rprintln!("[WIFI] ERROR: WiFi connection failed: {}", e);
        panic!("WiFi connection failed");
    }

    // Wait for IP address (EXACT copy)
    rprintln!("[WIFI] Requesting IP address via DHCP...");
    let _connection = match wifi::wait_for_ip(&mut stack, &wifi_config) {
        Ok(conn) => {
            rprintln!("[WIFI] CONNECTED: IP={} Gateway={} | ping {}", 
                     conn.ip, conn.gateway, conn.ip);
            conn
        }
        Err(e) => {
            rprintln!("[WIFI] ERROR: DHCP failed: {}", e);
            panic!("DHCP failed");
        }
    };
    
    // Initialize MQTT client
    let mqtt_config = MqttConfig::default();
    let mut mqtt_client = SimpleMqttClient::new(mqtt_config);
    rprintln!("[MQTT] MQTT client initialized for broker 10.10.10.210:1883");
    
    rprintln!("System initialization complete - Pure synchronous mode");
    
    // Main sensor and MQTT loop
    run_main_sensor_loop(&mut bme280, i2c_static, &mut stack, &mut mqtt_client);
}


/// Main sensor reading and MQTT publishing loop
/// 
/// CRITICAL NETWORKING DOCUMENTATION:
/// This loop contains the essential stack.work() call that enables ALL network
/// functionality. This single call is responsible for:
/// 1. PING RESPONSES: Device responds to ping requests
/// 2. INCOMING PACKET PROCESSING: Handles network traffic
/// 3. MQTT TRANSMISSION: Required for TCP data delivery
/// 4. DHCP RENEWAL: Maintains IP address lease
/// 5. TCP CONNECTION MANAGEMENT: Handles connection states
/// 
/// WITHOUT the stack.work() call:
/// - Gets DHCP IP but no ping response
/// - MQTT write_all() succeeds but no data transmitted  
/// - Network appears connected but is non-functional
/// 
/// This was the ROOT CAUSE of the original "nothing published!" issue.
/// See TCP_STACK_PROCESSING_CRITICAL_SOLUTION.md for complete technical details.
fn run_main_sensor_loop(
    bme280: &mut Bme280,
    i2c_static: &mut I2c<'static, esp_hal::Blocking>,
    stack: &mut blocking_network_stack::Stack<'static, esp_wifi::wifi::WifiDevice<'static>>,
    mqtt_client: &mut SimpleMqttClient,
) -> ! {
    // Main synchronous loop with MQTT publishing
    let mut reading_count = 0u32;
    let mut loop_count = 0u32;
    let mut heartbeat_count = 0u32;
    
    loop {
        loop_count += 1;
        
        // Read sensor every ~5 seconds (100 * 50ms = 5s) for faster testing
        if loop_count % 100 == 0 {
            reading_count += 1;
            
            match bme280.read_data_blocking(i2c_static) {
                Ok(data) => {
                    rprintln!("[SENSOR] Reading #{}: T={:.2}°C P={:.2}hPa H={:.2}%RH", 
                             reading_count, data.temperature, data.pressure, data.humidity);
                    
                    // Create sensor reading for MQTT
                    let sensor_reading = SensorReading {
                        temperature: data.temperature,
                        pressure: data.pressure,
                        humidity: data.humidity,
                        count: reading_count,
                    };
                    
                    // Publish sensor data to MQTT via REAL TCP socket
                    match mqtt_client.publish_sensor_data_tcp(stack, &sensor_reading) {
                        Ok(_) => {
                            rprintln!("[MQTT] ✓ Sensor data #{} published to broker via TCP!", sensor_reading.count);
                        }
                        Err(e) => {
                            rprintln!("[MQTT] ✗ Failed to publish sensor data via TCP: {}", e);
                        }
                    }
                }
                Err(e) => {
                    rprintln!("[SENSOR] ERROR: Failed to read sensor: {}", e);
                }
            }
        }
        
        // System heartbeat every ~15 seconds (300 * 50ms = 15s) - offset from sensor readings
        // Sensor readings happen at multiples of 100, heartbeat at 150, 450, 750, etc.
        if loop_count % 300 == 150 {
            heartbeat_count += 1;
            rprintln!("=== System Heartbeat #{} ===", heartbeat_count);
            rprintln!("System running normally - heap size: {} bytes", NodepsConfig::heap_size());
            rprintln!("Total sensor readings: {}", reading_count);
            
            // Publish heartbeat to MQTT via TCP (new implementation!)
            match mqtt_client.publish_heartbeat_tcp(stack, heartbeat_count) {
                Ok(_) => {
                    rprintln!("[MQTT] ✓ Heartbeat #{} published via TCP to broker", heartbeat_count);
                }
                Err(e) => {
                    rprintln!("[MQTT] ✗ Failed to publish heartbeat via TCP: {}", e);
                }
            }
        }
        
        // CRITICAL NETWORKING DOCUMENTATION:
        // The stack.work() call below is MANDATORY for ALL network functionality.
        // 
        // This single call enables:
        // 1. PING RESPONSES: Device responds to ping requests
        // 2. INCOMING PACKET PROCESSING: Handles network traffic
        // 3. MQTT TRANSMISSION: Required for TCP data delivery (see TCP_STACK_PROCESSING_CRITICAL_SOLUTION.md)
        // 4. DHCP RENEWAL: Maintains IP address lease
        // 5. TCP CONNECTION MANAGEMENT: Handles connection states
        //
        // WITHOUT this call:
        // - Gets DHCP IP but no ping response
        // - MQTT write_all() succeeds but no data transmitted
        // - Network appears connected but is non-functional
        //
        // TECHNICAL INSIGHT: blocking-network-stack + smoltcp requires continuous
        // stack processing to move data between socket buffers and WiFi radio.
        //
        // NEVER REMOVE THIS CALL - entire network stack depends on it!
        stack.work(); // CRITICAL: Enables ALL network functionality including MQTT
        
        // Blocking delay - 50ms
        blocking_delay_ms(50);
    }
}

/// Simple blocking delay
fn blocking_delay_ms(ms: u32) {
    let cycles = ms * 240_000; // ESP32-C3 at 240MHz
    for _ in 0..cycles {
        unsafe { 
            let dummy: u32 = 0;
            core::ptr::read_volatile(&dummy); 
        }
    }
}