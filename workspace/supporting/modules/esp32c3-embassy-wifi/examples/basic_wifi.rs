//! Basic WiFi example using wifi-simple-embassy
//! 
//! This example demonstrates how to:
//! 1. Initialize the WiFi manager
//! 2. Connect to a WiFi network
//! 3. Get IP address via DHCP
//! 4. Keep the connection alive
//!
//! To run this example:
//! 1. Set your WiFi credentials in .cargo/config.toml:
//!    ```toml
//!    [env]
//!    SSID = "YourWiFiNetwork"
//!    PASSWORD = "YourWiFiPassword"
//!    ```
//! 2. Run: `cargo run --example basic_wifi --release`

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use rtt_target::{rprintln, rtt_init_print};
use wifi_simple_embassy::WiFiManager;

// WiFi credentials from environment
const SSID: &str = env!("SSID", "Set SSID environment variable");
const PASSWORD: &str = env!("PASSWORD", "Set PASSWORD environment variable");

use panic_rtt_target as _;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize heap allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize RTT logging
    rtt_init_print!();
    
    rprintln!("WiFi Simple Embassy - Basic Example");
    rprintln!("Connecting to SSID: {}", SSID);

    // Setup hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Initialize WiFi manager
    let mut wifi_manager = match WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.TIMG1,
        peripherals.WIFI,
        peripherals.RNG,
        SSID,
        PASSWORD,
    ).await {
        Ok(manager) => {
            rprintln!("✅ WiFi manager initialized successfully!");
            manager
        }
        Err(e) => {
            rprintln!("❌ Failed to initialize WiFi: {:?}", e);
            
            // Check for common credential issues
            if SSID == "YourWiFiNetwork" || PASSWORD == "YourWiFiPassword" {
                rprintln!("WARNING: You are using placeholder credentials!");
                rprintln!("Please set your actual WiFi credentials in .cargo/config.toml:");
                rprintln!("  SSID = \"YourActualNetworkName\"");
                rprintln!("  PASSWORD = \"YourActualPassword\"");
            }
            
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
                    rprintln!("⚠️ Status check failed: {:?}", e);
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
                rprintln!("❌ Connection lost - attempting to reconnect...");
            }
        }
        
        Timer::after(Duration::from_millis(1)).await;
    }
}