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

use wifi_embassy::{WiFiManager, WiFiConfig};

#[embassy_executor::task]
async fn connection_monitor_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[MONITOR] Starting WiFi connection monitor...");
    
    loop {
        // Wait 10 seconds between checks
        Timer::after(Duration::from_secs(10)).await;
        
        if let Some(connection_info) = wifi_manager.get_connection_info() {
            rprintln!("[MONITOR] ✅ WiFi Status: Connected");
            rprintln!("[MONITOR]   📍 IP Address: {}", connection_info.ip_address);
            rprintln!("[MONITOR]   🌐 Gateway: {:?}", connection_info.gateway);
            rprintln!("[MONITOR]   🔧 Subnet: /{}", connection_info.subnet_prefix);
            rprintln!("[MONITOR]   ⏱️  Uptime: {}s", embassy_time::Instant::now().as_secs());
        } else {
            rprintln!("[MONITOR] ❌ WiFi Status: Disconnected or connecting...");
        }
        
        rprintln!("[MONITOR] ----------------------------------------");
    }
}

#[embassy_executor::task]
async fn network_test_task(wifi_manager: &'static WiFiManager) {
    rprintln!("[TEST] Starting network connectivity test...");
    
    // Wait for initial connection
    Timer::after(Duration::from_secs(15)).await;
    
    loop {
        if let Some(_connection_info) = wifi_manager.get_connection_info() {
            rprintln!("[TEST] 🌐 Network stack available for TCP/UDP operations");
            rprintln!("[TEST] 📡 Embassy network stack ready for applications");
            rprintln!("[TEST] 🔗 Can be used with MQTT, HTTP clients, etc.");
            
            // Demonstrate network stack access
            let stack = wifi_manager.get_stack();
            rprintln!("[TEST] ✅ Network stack reference obtained");
            rprintln!("[TEST] 📊 Stack configuration: {:?}", stack.config_v4());
        } else {
            rprintln!("[TEST] ⚠️  No network connection available");
        }
        
        // Test every 30 seconds
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[main]
async fn main(spawner: Spawner) -> ! {
    // Initialize RTT for console output
    rtt_init_print!();
    
    rprintln!("🚀 ESP32-C3 Simple WiFi Connection Test");
    rprintln!("=======================================");
    rprintln!("📡 Demonstrates basic WiFi connectivity with Embassy");
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

    rprintln!("📡 Target Network: {}", wifi_config.ssid);
    rprintln!("🔧 Hardware initialized, starting WiFi connection...");
    rprintln!("");

    // Initialize WiFi manager
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG1,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await.unwrap();

    rprintln!("✅ WiFi manager initialized successfully!");
    rprintln!("⏳ Establishing connection to network...");
    rprintln!("");

    // Wait a moment for initial connection
    Timer::after(Duration::from_secs(3)).await;

    if let Some(connection_info) = wifi_manager.get_connection_info() {
        rprintln!("🎉 WiFi Connected Successfully!");
        rprintln!("📡 Network Details:");
        rprintln!("  📍 IP Address: {}", connection_info.ip_address);
        rprintln!("  🌐 Gateway: {:?}", connection_info.gateway);
        rprintln!("  🔧 Subnet: /{}", connection_info.subnet_prefix);
        rprintln!("  📶 WiFi Radio: Active");
        rprintln!("  🔗 DHCP: Configured");
        rprintln!("");
        rprintln!("✅ Network stack ready for applications!");
    } else {
        rprintln!("⚠️  Initial connection not yet established...");
        rprintln!("   Connection may still be in progress");
    }

    rprintln!("🚀 Starting monitoring and test tasks...");
    rprintln!("");

    // Spawn monitoring tasks
    spawner.spawn(connection_monitor_task(&wifi_manager)).ok();
    spawner.spawn(network_test_task(&wifi_manager)).ok();

    rprintln!("📊 Background tasks started:");
    rprintln!("   - Connection monitoring every 10s");
    rprintln!("   - Network test every 30s");
    rprintln!("");
    rprintln!("🔄 System running - check output for status updates");
    
    // Main loop - keep system alive
    loop {
        Timer::after(Duration::from_secs(60)).await;
        rprintln!("[MAIN] System heartbeat - {}s uptime", 
                  embassy_time::Instant::now().as_secs());
    }
}