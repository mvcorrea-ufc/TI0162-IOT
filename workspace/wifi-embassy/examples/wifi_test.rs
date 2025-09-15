//! WiFi Embassy Test Example
//! 
//! This example demonstrates WiFi connectivity using the wifi-embassy module.
//! 
//! ## Usage
//! 1. Set your WiFi credentials in .cargo/config.toml:
//!    ```toml
//!    [env]
//!    WIFI_SSID = "YourNetworkName"
//!    WIFI_PASSWORD = "YourPassword"
//!    ```
//! 2. Run: `cargo run --example wifi_test --release`

#![no_std]
#![no_main]

extern crate alloc;

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use wifi_embassy::{WiFiManager, WiFiConfig};

// WiFi credentials from environment variables in .cargo/config.toml
const WIFI_SSID: &str = env!("WIFI_SSID", "Set WIFI_SSID in .cargo/config.toml");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD", "Set WIFI_PASSWORD in .cargo/config.toml");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize heap allocator (required for WiFi)
    esp_alloc::heap_allocator!(72 * 1024);
    
    // Initialize RTT for console output (following bme280-embassy pattern)
    rtt_init_print!();
    
    rprintln!("ESP32-C3 WiFi Embassy Test");
    rprintln!("==========================");
    rprintln!("Target SSID: {}", WIFI_SSID);

    // Initialize ESP32-C3 peripherals
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize Embassy BEFORE creating WiFi manager (to avoid schedule_wake panic)
    let timer_group1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer_group1.timer0);
    rprintln!("Embassy time driver initialized successfully");
    
    // Check for placeholder credentials
    if WIFI_SSID == "YourWiFiNetwork" || WIFI_PASSWORD == "YourWiFiPassword" {
        rprintln!("⚠️  WARNING: Using placeholder WiFi credentials!");
        rprintln!("Please update .cargo/config.toml with your actual WiFi credentials:");
        rprintln!("  WIFI_SSID = \"YourActualNetworkName\"");
        rprintln!("  WIFI_PASSWORD = \"YourActualPassword\"");
        rprintln!("Current SSID: {}", WIFI_SSID);
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

    rprintln!("Hardware initialized, starting WiFi...");

    // Initialize WiFi manager (TIMG1 already used for Embassy above)
    let mut wifi_manager = match WiFiManager::new(
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

    // Show connection information
    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("📡 WiFi Connection Details:");
        rprintln!("  IP Address: {}", connection_info.ip_address);
        rprintln!("  Gateway: {:?}", connection_info.gateway);
        rprintln!("  Subnet: /{}", connection_info.subnet_prefix);
        rprintln!("  DNS Servers: {:?}", connection_info.dns_servers);
        rprintln!("🌐 Device is now accessible on the network!");
        rprintln!("💡 Try: ping {}", connection_info.ip_address);
    }

    // Get the network stack for further use
    let _stack = wifi_manager.get_stack();
    rprintln!("📶 Network stack is ready for TCP/UDP operations");
    rprintln!("🔄 Starting continuous monitoring...");

    // Main loop - keep connection alive and show periodic status
    let mut counter = 0;
    loop {
        counter += 1;
        
        // Show status every 30 seconds
        if counter % 30000 == 0 {
            match wifi_manager.get_status().await {
                Ok(status) => {
                    rprintln!("📊 Status: CONNECTED - IP: {}", status.ip_address);
                }
                Err(e) => {
                    rprintln!("⚠️ Status check failed: {}", e);
                }
            }
        }
        
        // Quick connectivity check every 5 seconds
        if counter % 5000 == 0 {
            if wifi_manager.is_connected() {
                if let Some(ip) = wifi_manager.get_ip_address() {
                    rprintln!("✅ Connection healthy - IP: {}", ip);
                }
            } else {
                rprintln!("❌ Connection lost - automatic reconnection in progress...");
            }
        }
        
        Timer::after(Duration::from_millis(1)).await;
    }
}