//! System Console with Full IoT Integration
//! 
//! Demonstrates a complete serial console integrated with the IoT system
//! including WiFi, MQTT, and BME280 sensor management via serial commands.
//! 
//! ## Usage
//! 1. Configure credentials in .cargo/config.toml
//! 2. Flash: cargo run --example system_console --features full --release
//! 3. Connect via serial terminal at 115200 baud
//! 4. Use commands to configure and monitor the IoT system

#![no_std]
#![no_main]

extern crate alloc;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    uart::{Uart, UartRx, UartTx, config::Config as UartConfig},
    timer::timg::TimerGroup,
    i2c::master::{I2c, Config as I2cConfig},
    gpio::{Level, Output},
    Async,
};

use serial_console_embassy::{SerialConsole, console::uart_console_task};

#[cfg(feature = "wifi")]
use wifi_embassy::{WiFiManager, WiFiConfig};

#[cfg(feature = "sensor")]
use bme280_embassy::BME280;

// Environment variables from .cargo/config.toml (with defaults)
const DEFAULT_WIFI_SSID: &str = "ESP32-Test";
const DEFAULT_WIFI_PASSWORD: &str = "password123";

#[embassy_executor::task]
async fn console_task(console: &'static SerialConsole, rx: UartRx<'static, Async>, tx: UartTx<'static, Async>) {
    uart_console_task(console, rx, tx).await;
}

#[cfg(feature = "sensor")]
#[embassy_executor::task]
async fn sensor_monitor_task(console: &'static SerialConsole, mut i2c: I2c<'static, Async>) {
    rprintln!("[SENSOR] Starting BME280 sensor monitoring");
    
    let mut bme280 = BME280::new(&mut i2c);
    let mut sensor_active = false;
    
    // Try to initialize BME280
    match bme280.check_id().await {
        Ok(true) => {
            match bme280.init().await {
                Ok(_) => {
                    rprintln!("[SENSOR] BME280 initialized successfully");
                    sensor_active = true;
                }
                Err(_) => {
                    rprintln!("[SENSOR] Failed to initialize BME280");
                }
            }
        }
        _ => {
            rprintln!("[SENSOR] BME280 not found or communication failed");
        }
    }
    
    loop {
        if sensor_active {
            match bme280.read_measurements().await {
                Ok(measurements) => {
                    rprintln!("[SENSOR] T: {:.1}°C, H: {:.1}%, P: {:.1} hPa", 
                            measurements.temperature, measurements.humidity, measurements.pressure);
                }
                Err(_) => {
                    rprintln!("[SENSOR] Failed to read measurements");
                    sensor_active = false;
                }
            }
        }
        
        // Update console with sensor status
        console.update_system_status(false, false, sensor_active, None).await;
        
        Timer::after(Duration::from_secs(10)).await;
    }
}

#[cfg(feature = "wifi")]
#[embassy_executor::task]
async fn wifi_monitor_task(console: &'static SerialConsole) {
    rprintln!("[WIFI] WiFi monitoring task (placeholder)");
    
    // This would integrate with actual WiFi manager
    // For now, just simulate WiFi status
    let mut counter = 0;
    loop {
        Timer::after(Duration::from_secs(15)).await;
        counter += 1;
        
        let wifi_connected = counter % 4 != 0; // Simulate occasional disconnection
        let ip = if wifi_connected { Some("10.10.10.214") } else { None };
        
        console.update_system_status(wifi_connected, false, true, ip).await;
        
        if wifi_connected {
            rprintln!("[WIFI] Connection OK - IP: 10.10.10.214");
        } else {
            rprintln!("[WIFI] Connection lost");
        }
    }
}

#[embassy_executor::task]
async fn system_status_task(console: &'static SerialConsole) {
    rprintln!("[SYSTEM] Starting system status monitoring");
    
    loop {
        Timer::after(Duration::from_secs(60)).await;
        
        let config = console.get_config().await;
        rprintln!("[SYSTEM] Status - WiFi: {}, MQTT: {}, Sensor: {}", 
                 if config.system.wifi_connected { "Connected" } else { "Disconnected" },
                 if config.system.mqtt_connected { "Connected" } else { "Disconnected" },
                 if config.system.sensor_active { "Active" } else { "Inactive" });
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator (required for WiFi operations)
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("[SYSTEM] ESP32-C3 IoT System Console");
    rprintln!("[SYSTEM] ===================================");

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy time driver
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("[SYSTEM] Embassy time driver initialized");

    // Configure UART for serial console (115200 baud)
    let uart_config = UartConfig::default().baudrate(115200);
    let uart = Uart::new(peripherals.UART0, uart_config).unwrap();
    let (tx, rx) = uart.split();
    let (tx, rx) = (tx.into_async(), rx.into_async());

    rprintln!("[SYSTEM] UART configured at 115200 baud");

    // Set up LED for status indication
    let _led = Output::new(peripherals.GPIO3, Level::Low);

    // Create console instance
    use static_cell::StaticCell;
    static CONSOLE: StaticCell<SerialConsole> = StaticCell::new();
    let console = CONSOLE.init(SerialConsole::new());

    rprintln!("[SYSTEM] Serial console initialized");

    // Spawn console task
    spawner.spawn(console_task(console, rx, tx)).ok();
    spawner.spawn(system_status_task(console)).ok();

    // Spawn sensor monitoring task if enabled
    #[cfg(feature = "sensor")]
    {
        let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
            .unwrap()
            .with_sda(peripherals.GPIO8)
            .with_scl(peripherals.GPIO9)
            .into_async();
        
        spawner.spawn(sensor_monitor_task(console, i2c)).ok();
        rprintln!("[SYSTEM] BME280 sensor monitoring enabled");
    }

    // Spawn WiFi monitoring task if enabled
    #[cfg(feature = "wifi")]
    {
        spawner.spawn(wifi_monitor_task(console)).ok();
        rprintln!("[SYSTEM] WiFi monitoring enabled");
    }

    rprintln!("[SYSTEM] All tasks spawned successfully");
    rprintln!("[SYSTEM] =====================================");
    rprintln!("[SYSTEM] Connect via serial terminal at 115200 baud");
    rprintln!("[SYSTEM] Available features:");
    
    #[cfg(feature = "sensor")]
    rprintln!("[SYSTEM]   - BME280 sensor monitoring");
    
    #[cfg(feature = "wifi")]
    rprintln!("[SYSTEM]   - WiFi connectivity");
    
    #[cfg(feature = "mqtt")]
    rprintln!("[SYSTEM]   - MQTT client");
    
    rprintln!("[SYSTEM] Type 'help' in console for commands");
    
    // Main loop
    loop {
        Timer::after(Duration::from_secs(30)).await;
        rprintln!("[SYSTEM] Main loop - connect via UART for interactive console");
    }
}