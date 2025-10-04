//! ESP32-C3 WiFi Synchronous - WiFiManager API Example
//! 
//! Demonstrates the new WiFiManager API that matches wifi-embassy
//! for easy migration between async and synchronous approaches.

#![no_std]
#![no_main]

extern crate alloc;

use esp_alloc as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    main,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use wifi_synchronous::{WiFiManager, WiFiConfig};

#[main]
fn main() -> ! {
    // Initialize heap allocator for WiFi
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    rtt_init_print!();
    rprintln!("🚀 ESP32-C3 WiFi Synchronous - WiFiManager API Example");
    rprintln!("=======================================================");
    rprintln!("📡 Demonstrates WiFiManager API compatible with wifi-embassy");
    rprintln!("");

    // WiFi configuration (same as wifi-embassy)
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };

    rprintln!("📡 Target Network: {}", wifi_config.ssid);
    rprintln!("");

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Status LED
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    led.set_high();

    rprintln!("✅ Hardware initialized");

    // Create WiFiManager (blocking API - no async/await needed)
    rprintln!("🔧 Creating WiFi manager...");
    let mut wifi_manager = match WiFiManager::new(
        peripherals.WIFI,
        peripherals.TIMG0,
        peripherals.RNG,
        wifi_config.clone(),
    ) {
        Ok(manager) => {
            rprintln!("✅ WiFi manager created successfully");
            manager
        }
        Err(e) => {
            rprintln!("❌ Failed to create WiFi manager: {}", e);
            loop {
                led.toggle();
                for _ in 0..500000 {
                    unsafe { core::ptr::read_volatile(&0); }
                }
            }
        }
    };

    // Connect to WiFi (blocking call)
    rprintln!("🔗 Connecting to WiFi network...");
    let connection_info = match wifi_manager.connect() {
        Ok(info) => {
            rprintln!("✅ WiFi connected successfully!");
            info
        }
        Err(e) => {
            rprintln!("❌ WiFi connection failed: {}", e);
            
            // Check if using placeholder credentials
            if wifi_config.ssid == "YourWiFiNetwork" {
                rprintln!("");
                rprintln!("⚠️  WARNING: Using placeholder credentials!");
                rprintln!("Please update .cargo/config.toml:");
                rprintln!("  WIFI_SSID = \"YourActualNetworkName\"");
                rprintln!("  WIFI_PASSWORD = \"YourActualPassword\"");
            }
            
            loop {
                led.toggle();
                for _ in 0..100000 {
                    unsafe { core::ptr::read_volatile(&0); }
                }
            }
        }
    };

    // Success! Display connection info (same as wifi-embassy)
    led.set_low();
    rprintln!("");
    rprintln!("🎉 WIFI CONNECTED SUCCESSFULLY!");
    rprintln!("📡 Network Details:");
    rprintln!("  📍 IP Address: {}", connection_info.ip_address);
    rprintln!("  🌐 Gateway: {:?}", connection_info.gateway);
    rprintln!("  🔧 Subnet: /{}", connection_info.subnet_prefix);
    rprintln!("  📡 DNS Servers: {} entries", connection_info.dns_servers.len());
    rprintln!("");
    
    // Get network stack (same API as wifi-embassy)
    let _stack = wifi_manager.get_stack();
    rprintln!("🔗 Network stack ready for TCP/UDP operations");
    rprintln!("📊 Stack type: blocking_network_stack (not embassy-net)");
    rprintln!("");

    // Main monitoring loop
    rprintln!("📊 Starting connection monitoring...");
    let mut counter = 0;

    loop {
        counter += 1;
        
        // Check connection status periodically (same API as wifi-embassy)
        if counter % 20000 == 0 {
            if let Some(current_info) = wifi_manager.get_connection_info() {
                rprintln!("[MONITOR] ✅ Connected - IP: {}, Uptime: {}s", 
                          current_info.ip_address, counter / 1000);
            } else {
                rprintln!("[MONITOR] ⚠️  Connection lost");
            }
        }

        // Heartbeat LED
        if counter % 10000 == 0 {
            led.toggle();
        }
        
        // Simple delay
        for _ in 0..1000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}