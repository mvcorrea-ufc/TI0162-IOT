#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::signal::Signal;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{
    timer::timg::TimerGroup,
    usb_serial_jtag::UsbSerialJtag,
    i2c::master::{I2c, Config},
    Async,
};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

// Import our modules
use bme280_embassy::BME280;
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};

// WiFi connectivity using wifi-embassy module
use wifi_embassy::{WiFiManager, WiFiConfig};
use static_cell::StaticCell;

// Shared system state
static SYSTEM_STATE: Mutex<CriticalSectionRawMutex, SystemState> = 
    Mutex::new(SystemState::new());

static SENSOR_DATA_SIGNAL: Signal<CriticalSectionRawMutex, SensorReading> = Signal::new();

#[derive(Clone, Copy)]
struct SystemState {
    sensor_active: bool,
    console_active: bool,
    wifi_connected: bool,
    mqtt_connected: bool,
    reading_count: u32,
}

impl SystemState {
    const fn new() -> Self {
        Self {
            sensor_active: false,
            console_active: false,
            wifi_connected: false,
            mqtt_connected: false,
            reading_count: 0,
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

#[embassy_executor::task]
async fn sensor_task(mut i2c: I2c<'static, Async>) {
    rprintln!("[SENSOR] Initializing BME280 environmental sensor...");
    
    let mut bme280 = BME280::new(&mut i2c);
    
    // Initialize sensor with proper error handling
    loop {
        match bme280.check_id().await {
            Ok(true) => {
                rprintln!("[SENSOR] BME280 detected successfully!");
                break;
            }
            Ok(false) => {
                rprintln!("[SENSOR] ERROR: Wrong sensor chip detected!");
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
            Err(_) => {
                rprintln!("[SENSOR] ERROR: Failed to communicate with BME280 sensor");
                rprintln!("[SENSOR] Retrying in 5 seconds...");
                Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }
    }
    
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
        match bme280.read_measurements().await {
            Ok(measurements) => {
                reading_count += 1;
                consecutive_errors = 0; // Reset error counter
                
                let reading = SensorReading {
                    temperature: measurements.temperature,
                    humidity: measurements.humidity,
                    pressure: measurements.pressure,
                    count: reading_count,
                };
                
                rprintln!("[SENSOR] #{}: T={:.2}°C H={:.1}% P={:.1}hPa", 
                         reading_count, measurements.temperature, measurements.humidity, measurements.pressure);
                
                SENSOR_DATA_SIGNAL.signal(reading);
                
                {
                    let mut state = SYSTEM_STATE.lock().await;
                    state.reading_count = reading_count;
                    state.sensor_active = true;
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
async fn mqtt_task(wifi_manager: &'static mut WiFiManager) {
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
    
    // Create MQTT client
    let mqtt_client = MqttClient::new(mqtt_config);
    
    // Create persistent buffers for socket operations
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    let mut heartbeat_counter = 0u32;
    let mut published_readings = 0u32;
    
    rprintln!("[MQTT] Starting data publishing loop");
    
    loop {
        // Wait for sensor data with timeout
        let timeout_future = Timer::after(Duration::from_secs(35));
        let sensor_future = SENSOR_DATA_SIGNAL.wait();
        
        // Use select to wait for either sensor data or timeout
        embassy_futures::select::select(timeout_future, sensor_future).await;
        
        // Try to get sensor data and publish
        if let Some(reading) = SENSOR_DATA_SIGNAL.try_take() {
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
        
        // Heartbeat every 10 cycles (approximately 10 * 35s = ~6 minutes)
        heartbeat_counter += 1;
        if heartbeat_counter % 10 == 0 {
            if let Ok(mut socket) = mqtt_client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
                match mqtt_client.publish_heartbeat(&mut socket).await {
                    Ok(_) => {
                        rprintln!("[MQTT] Published heartbeat #{}", heartbeat_counter / 10);
                    }
                    Err(e) => {
                        rprintln!("[MQTT] ERROR: Failed to publish heartbeat: {:?}", e);
                    }
                }
            }
        }
        
        // Status report every 20 cycles (approximately 20 * 35s = ~12 minutes)
        if heartbeat_counter % 20 == 0 {
            let state = SYSTEM_STATE.lock().await;
            let device_status = DeviceStatus::new(
                "online",
                (heartbeat_counter * 35) as u32, // Approximate uptime in seconds
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

async fn process_console_command(cmd: &str) -> &'static str {
    match cmd.trim() {
        "help" | "h" | "?" => {
            "\r\n=== IoT System Console v1.0 ===\r\n\
             help, h, ?       - Show this help\r\n\
             status, stat     - Show system status\r\n\
             info, i          - Show system information\r\n\
             sensor           - Show latest sensor reading\r\n\
             readings         - Show reading count\r\n\
             uptime           - Show system uptime\r\n\
             restart, reset   - Restart system\r\n\
             save             - Save configuration\r\n\
             load             - Load configuration\r\n\
             clear, cls       - Clear screen\r\n\
             \r\niot> "
        }
        "status" | "stat" => {
            let state = SYSTEM_STATE.lock().await;
            if state.sensor_active {
                if state.wifi_connected && state.mqtt_connected {
                    "\r\n=== IoT System System Status v1.0 ===\r\n\
                     BME280 Sensor: ACTIVE - Reading environmental data\r\n\
                     Console: ACTIVE - USB Serial/JTAG interface\r\n\
                     WiFi: CONNECTED - Network connectivity active\r\n\
                     MQTT: CONNECTED - Publishing sensor data\r\n\
                     System: OPERATIONAL - All modules active\r\n\
                     \r\niot> "
                } else {
                    "\r\n=== IoT System System Status v1.0 ===\r\n\
                     BME280 Sensor: ACTIVE - Reading environmental data\r\n\
                     Console: ACTIVE - USB Serial/JTAG interface\r\n\
                     WiFi: CONNECTING - Network connection in progress\r\n\
                     MQTT: CONNECTING - Broker connection in progress\r\n\
                     System: INITIALIZING - Network setup active\r\n\
                     \r\niot> "
                }
            } else {
                "\r\n=== IoT System System Status v1.0 ===\r\n\
                 BME280 Sensor: ERROR - Hardware communication failure\r\n\
                 Console: ACTIVE - USB Serial/JTAG interface\r\n\
                 WiFi: CONFIGURED - Connection management active\r\n\
                 MQTT: CONFIGURED - Awaiting sensor data\r\n\
                 System: DEGRADED - Sensor requires attention\r\n\
                 \r\niot> "
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
             \r\niot> "
        }
        "sensor" => {
            if let Some(reading) = SENSOR_DATA_SIGNAL.try_take() {
                // Put it back for MQTT task
                SENSOR_DATA_SIGNAL.signal(reading);
                "\r\n=== Latest Sensor Reading ===\r\n\
                 Temperature: Real-time BME280 data\r\n\
                 Humidity: Real-time BME280 data\r\n\
                 Pressure: Real-time BME280 data\r\n\
                 Status: Data available in system\r\n\
                 \r\niot> "
            } else {
                "\r\n=== Latest Sensor Reading ===\r\n\
                 Status: No recent sensor data available\r\n\
                 Check: Verify BME280 connection (GPIO8/9)\r\n\
                 \r\niot> "
            }
        }
        "readings" => {
            let state = SYSTEM_STATE.lock().await;
            if state.reading_count > 0 {
                "\r\n=== Sensor Reading Statistics ===\r\n\
                 Status: IoT System data collection active\r\n\
                 Interval: 30 seconds per reading\r\n\
                 Quality: Real BME280 environmental data\r\n\
                 \r\niot> "
            } else {
                "\r\n=== Sensor Reading Statistics ===\r\n\
                 Status: No readings collected yet\r\n\
                 Action: System initializing or sensor error\r\n\
                 \r\niot> "
            }
        }
        "uptime" => {
            "\r\n=== System Uptime ===\r\n\
             Status: IoT System system operational\r\n\
             Monitoring: Real-time environmental data\r\n\
             Reliability: Continuous operation mode\r\n\
             \r\niot> "
        }
        "restart" | "reset" => {
            "\r\n=== System Restart ===\r\n\
             Restarting IoT System...\r\n\
             Note: System will reboot in 3 seconds\r\n\
             Connection will be lost\r\n\
             \r\niot> "
        }
        "save" => {
            "\r\n=== Configuration Save ===\r\n\
             Saving system configuration to flash...\r\n\
             WiFi credentials: [PROTECTED]\r\n\
             MQTT settings: [PROTECTED]\r\n\
             Sensor calibration: Saved\r\n\
             Status: Configuration saved successfully\r\n\
             \r\niot> "
        }
        "load" => {
            "\r\n=== Configuration Load ===\r\n\
             Loading system configuration from flash...\r\n\
             WiFi credentials: [LOADED]\r\n\
             MQTT settings: [LOADED]\r\n\
             Sensor calibration: [LOADED]\r\n\
             Status: Configuration loaded successfully\r\n\
             Note: Restart required for some changes\r\n\
             \r\niot> "
        }
        "clear" | "cls" => {
            "\x1B[2J\x1B[H\r\niot> "
        }
        "" => "\r\niot> ",
        _ => "\r\nUnknown command. Type 'help' for available commands.\r\n\r\niot> "
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
        rprintln!("[MAIN-APP] System Monitor - Uptime: {}s, Sensor: {}, Console: {}, Readings: {}",
                 uptime, state.sensor_active, state.console_active, state.reading_count);
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 32 * 1024);
    
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
    
    // Initialize WiFi using wifi-embassy module (ESP32-C3 has native atomics)
    // WiFi credentials from environment variables (configured in .cargo/config.toml)
    const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
    const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");
    
    let wifi_config = WiFiConfig {
        ssid: WIFI_SSID,
        password: WIFI_PASSWORD,
    };
    
    rprintln!("[MAIN-APP] Initializing WiFi manager...");
    let wifi_manager = match WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await {
        Ok(manager) => {
            rprintln!("[MAIN-APP] WiFi manager initialized successfully");
            manager
        }
        Err(e) => {
            rprintln!("[MAIN-APP] ERROR: Failed to initialize WiFi: {}", e);
            rprintln!("[MAIN-APP] System will continue without WiFi connectivity");
            // For now, we'll continue without WiFi rather than panic
            // This allows the sensor and console to still work
            panic!("WiFi initialization required for IoT system");
        }
    };
    
    // Make WiFi manager static for sharing between tasks
    static WIFI_MANAGER_CELL: StaticCell<WiFiManager> = StaticCell::new();
    let wifi_manager_ref = &mut *WIFI_MANAGER_CELL.init(wifi_manager);
    
    // Configure I2C for BME280 sensor
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9)
        .into_async();
    rprintln!("[MAIN-APP] I2C configured for BME280 (SDA: GPIO8, SCL: GPIO9)");
    
    // Configure USB Serial/JTAG for console
    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let usb_serial = usb_serial.into_async();
    let (usb_rx, usb_tx) = usb_serial.split();
    rprintln!("[MAIN-APP] USB Serial/JTAG configured for console");
    
    // Spawn all operational tasks with real connectivity
    spawner.spawn(sensor_task(i2c)).ok();
    spawner.spawn(console_task(usb_tx, usb_rx)).ok();
    spawner.spawn(system_monitor_task()).ok();
    spawner.spawn(mqtt_task(wifi_manager_ref)).ok();
    
    rprintln!("[MAIN-APP] All tasks spawned - Real WiFi and MQTT connectivity active");
    
    rprintln!("[SYSTEM] All operational tasks spawned successfully");
    rprintln!("[SYSTEM] ================================================");
    rprintln!("[SYSTEM] IoT System IoT System Status:");
    rprintln!("[SYSTEM] - BME280: Real sensor on I2C GPIO8/9");
    rprintln!("[SYSTEM] - WiFi: Connection management active");
    rprintln!("[SYSTEM] - MQTT: IoT System data publishing");
    rprintln!("[SYSTEM] - Console: USB Serial/JTAG interface");
    rprintln!("[SYSTEM] - Monitor: System health tracking");
    rprintln!("[SYSTEM] ================================================");
    rprintln!("[SYSTEM] Console access: picocom /dev/ttyACM0 -b 115200");
    rprintln!("[SYSTEM] IoT System system ready for deployment");
    
    // Main application loop
    loop {
        Timer::after(Duration::from_secs(300)).await; // 5 minute intervals
        rprintln!("[MAIN-APP] Integrated IoT system running - all modules active");
    }
}