//! Minimal ESP32-C3 IoT Application
//! Clean multi-file approach with minimal main.rs

#![no_std]
#![no_main]

extern crate alloc;

// Import our modules
mod config;
mod sensor;
mod connectivity;
mod monitor;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use esp_hal::{
    timer::timg::TimerGroup,
    i2c::master::{I2c, Config},
};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use static_cell::StaticCell;
use heapless::String;
use core::str::FromStr;

// Use our modules
use config::MinimalConfig;
use sensor::SensorManager;
use connectivity::MqttManager;
use monitor::SystemMonitor;

// Global resources
static I2C_STATIC: StaticCell<I2c<'static, esp_hal::Blocking>> = StaticCell::new();

// WiFi manager resource (using wifi-embassy like main-app)
#[cfg(feature = "wifi")]
static WIFI_MANAGER: StaticCell<wifi_embassy::WiFiManager> = StaticCell::new();

// Sensor data signal for communication between sensor and MQTT tasks
static SENSOR_DATA_SIGNAL: Signal<CriticalSectionRawMutex, sensor::SensorData> = Signal::new();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    rtt_init_print!();
    rprintln!("Minimal ESP32-C3 IoT Starting");

    // Initialize heap allocator (increased for WiFi stack like main-app)
    esp_alloc::heap_allocator!(size: 64 * 1024);

    // Initialize ESP32-C3 peripherals (like working main-app)
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    
    // Initialize WiFi using wifi-embassy module with error handling (like main-app)
    #[cfg(feature = "wifi")]
    let wifi_manager = {
        let wifi_config = wifi_embassy::WiFiConfig {
            ssid: String::from_str(env!("WIFI_SSID", "Set WIFI_SSID")).unwrap(),
            password: String::from_str(env!("WIFI_PASSWORD", "Set WIFI_PASSWORD")).unwrap(),
        };
        
        rprintln!("[MAIN-MIN] Initializing WiFi manager...");
        match wifi_embassy::WiFiManager::new(
            spawner,
            peripherals.TIMG0,
            peripherals.WIFI,
            peripherals.RNG,
            wifi_config,
        ).await {
            Ok(manager) => {
                rprintln!("[MAIN-MIN] WiFi manager initialized successfully");
                Some(WIFI_MANAGER.init(manager))
            }
            Err(e) => {
                rprintln!("[MAIN-MIN] ERROR: Failed to initialize WiFi: {}", e);
                rprintln!("[MAIN-MIN] DEGRADED MODE: Running without WiFi/MQTT");
                rprintln!("[MAIN-MIN] Sensor monitoring will still be available");
                None
            }
        }
    };
    
    #[cfg(not(feature = "wifi"))]
    let wifi_manager: Option<&wifi_embassy::WiFiManager> = None;

    // I2C setup (copy exact pattern from main-app)
    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO8)
        .with_scl(peripherals.GPIO9);
    let i2c_static = I2C_STATIC.init(i2c);

    // Load configuration
    let config = MinimalConfig::load();
    rprintln!("Sensor interval: {}s", config.sensor_interval_secs());
    
    // Start all tasks using modular approach like main-app
    spawner.spawn(sensor_task(i2c_static, config.sensor_interval_secs(), &SENSOR_DATA_SIGNAL)).unwrap();
    
    // Only start MQTT task if WiFi manager was successfully initialized
    #[cfg(feature = "mqtt")]
    if let Some(wifi_manager_ref) = wifi_manager {
        spawner.spawn(mqtt_task(wifi_manager_ref, &SENSOR_DATA_SIGNAL)).unwrap();
    } else {
        rprintln!("[MAIN-MIN] MQTT task skipped - WiFi not available");
    }
    
    spawner.spawn(monitor_task()).unwrap();

    rprintln!("All tasks started");

    // Main loop - keep system alive
    loop {
        Timer::after(Duration::from_secs(60)).await;
        rprintln!("System heartbeat");
    }
}

// Task wrappers using our modules
#[embassy_executor::task]
async fn sensor_task(
    i2c: &'static mut I2c<'static, esp_hal::Blocking>, 
    interval: u32,
    sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, sensor::SensorData>,
) {
    SensorManager::run(i2c, interval, sensor_signal).await;
}


#[cfg(feature = "mqtt")]
#[embassy_executor::task]
async fn mqtt_task(
    wifi_manager: &'static wifi_embassy::WiFiManager,
    sensor_signal: &'static embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, sensor::SensorData>,
) {
    MqttManager::run(wifi_manager, sensor_signal).await;
}

#[embassy_executor::task]
async fn monitor_task() {
    SystemMonitor::run().await;
}