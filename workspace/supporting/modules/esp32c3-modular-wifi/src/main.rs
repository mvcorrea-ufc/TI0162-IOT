//! ESP32-C3 WiFi Simple - Swappable WiFi Implementation
//! 
//! Supports multiple WiFi implementations that can be swapped at compile time:
//! - Blocking implementation (wifi_blocking.rs) - Default, simple sequential approach
//! - Embassy implementation (wifi_embassy.rs) - Async non-blocking with Embassy executor
//!
//! ## Usage:
//! 
//! ### Blocking Implementation (Default):
//! ```bash
//! cargo run --release --features blocking,mqtt
//! ```
//! 
//! ### Embassy Async Implementation:
//! ```bash
//! cargo run --release --features embassy,mqtt
//! ```
//!
//! ### HTTP Client Testing:
//! ```bash
//! cargo run --release --features blocking,http
//! ```
//! 
//! ## Implementation Selection:
//! - Add `blocking = []` feature for blocking implementation (default)
//! - Add `embassy = []` feature for Embassy async implementation  
//! - Add `mqtt = []` feature to enable MQTT client functionality
//! - Add `http = []` feature to enable HTTP client testing
//! - Conditional compilation automatically selects the appropriate module
//! - Same WiFiConfig structure across implementations for consistency

#![no_std]
#![no_main]

extern crate alloc;

// Conditional compilation for WiFi implementation selection
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
mod wifi_blocking;
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
use wifi_blocking as wifi;

#[cfg(feature = "embassy")]
mod wifi_embassy;
#[cfg(feature = "embassy")]
use wifi_embassy as wifi;

#[cfg(all(feature = "mqtt", feature = "blocking", not(feature = "embassy")))]
mod mqtt_client;

#[cfg(all(feature = "http", feature = "blocking", not(feature = "embassy")))]
mod http_client;

use esp_alloc as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, Output, OutputConfig},
    main,
    timer::timg::TimerGroup,
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use esp_wifi::init;
use smoltcp::{
    iface::{SocketSet, SocketStorage},
};
use embedded_io::{Read, Write};

// Define mk_static macro for blocking implementation
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

// Conditional imports based on WiFi implementation
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
use wifi::{
    WiFiConfig, 
    create_interface, 
    create_dhcp_socket, 
    set_dhcp_hostname,
    create_stack, 
    configure_wifi, 
    scan_networks,
    wait_for_connection, 
    wait_for_ip, 
    get_status
};

#[cfg(feature = "embassy")]
use wifi::{init_wifi_embassy, wifi_connection_task, network_task, status_monitor_task};

#[cfg(feature = "embassy")]
use embassy_executor::Spawner;

#[cfg(feature = "embassy")]
use embassy_net::StackResources;

// Import StaticCell for blocking implementation 
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
use static_cell;

// Main function - different implementations based on feature
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
#[main]
fn main() -> ! {
    blocking_main()
}

#[cfg(feature = "embassy")]
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    embassy_main(spawner).await
}

// Blocking implementation main function
#[cfg(all(feature = "blocking", not(feature = "embassy")))]
fn blocking_main() -> ! {
    // Initialize heap allocator for WiFi
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    rtt_init_print!();
    rprintln!("ESP32-C3 WiFi Simple - Blocking Implementation");

    // WiFi configuration from environment variables
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
        hostname: "ESP32-C3-WiFi-Test",
    };

    rprintln!("Config: SSID={} Hostname={}", wifi_config.ssid, wifi_config.hostname);

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Status LED
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());
    led.set_high();

    // Initialize WiFi hardware
    let timer_group = TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    
    // Initialize WiFi controller with static allocation to fix lifetime issues
    let esp_wifi_ctrl = mk_static!(
        esp_wifi::EspWifiController,
        init(timer_group.timer0, rng.clone()).unwrap()
    );
    rprintln!("Hardware: WiFi initialized");

    // Create WiFi controller and interfaces
    let (mut controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let mut device = interfaces.sta;
    rprintln!("Hardware: WiFi controller created");

    // Create network interface using module
    let iface = create_interface(&mut device);
    
    // Set up DHCP socket using module functions - use static storage for lifetimes
    static mut SOCKET_SET_ENTRIES: [SocketStorage; 3] = [SocketStorage::EMPTY; 3];
    let socket_set_entries = unsafe { &mut SOCKET_SET_ENTRIES };
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = create_dhcp_socket();
    set_dhcp_hostname(&mut dhcp_socket, wifi_config.hostname);
    socket_set.add(dhcp_socket);
    
    // Create network stack using module
    let mut stack = create_stack(iface, device, socket_set, &mut rng);
    rprintln!("Network: Stack created");

    // Configure WiFi using module
    if let Err(e) = configure_wifi(&mut controller, &wifi_config) {
        rprintln!("ERROR: WiFi configuration failed: {}", e);
        loop {
            led.toggle();
            for _ in 0..250000 {
                unsafe { core::ptr::read_volatile(&0); }
            }
        }
    }
    rprintln!("WiFi: Configuration complete");

    // Scan networks using module
    scan_networks(&mut controller);

    // Wait for connection using module
    if let Err(e) = wait_for_connection(&mut controller, &mut led) {
        rprintln!("ERROR: WiFi connection failed: {}", e);
        
        // Check if user is using placeholder credentials
        if wifi_config.ssid == "YourWiFiNetwork" || wifi_config.password == "YourWiFiPassword" {
            rprintln!("WARNING: You are using placeholder WiFi credentials!");
            rprintln!("Please update .cargo/config.toml with your actual WiFi network details:");
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

    // Wait for IP address using module
    let connection = match wait_for_ip(&mut stack, &mut led, &wifi_config) {
        Ok(conn) => conn,
        Err(e) => {
            rprintln!("ERROR: DHCP failed: {}", e);
            loop {
                led.toggle();
                for _ in 0..50000 {
                    unsafe { core::ptr::read_volatile(&0); }
                }
            }
        }
    };

    // Success! Network ready
    led.set_low();
    rprintln!("NETWORK READY");
    rprintln!("Connection: {:?}", connection);
    rprintln!("Test: ping {}", connection.ip);

    // Test HTTP client for robust networking validation
    #[cfg(all(feature = "http", feature = "blocking", not(feature = "embassy")))]
    {
        use http_client::test_http_client;
        
        rprintln!("HTTP: Starting robust network connectivity tests");
        
        match test_http_client(&mut stack) {
            Ok(()) => {
                rprintln!("HTTP: ✅ All network tests completed successfully");
            }
            Err(e) => {
                rprintln!("HTTP: ❌ Network test failed: {}", e);
            }
        }
    }
    
    // Test MQTT functionality if both features enabled
    #[cfg(all(feature = "mqtt", feature = "blocking", not(feature = "embassy")))]
    {
        use mqtt_client::{init_mqtt_client, publish_mqtt_messages};
        
        rprintln!("MQTT: Starting MQTT client tests");
        
        let client = init_mqtt_client();
        let test_messages = client.create_test_messages();
        
        match publish_mqtt_messages(&mut stack, &test_messages) {
            Ok(()) => {
                rprintln!("MQTT: ✅ All messages published successfully");
            }
            Err(e) => {
                rprintln!("MQTT: ❌ Publishing failed: {}", e);
            }
        }
    }

    // Main loop with periodic status monitoring using module
    let mut counter = 0;
    rprintln!("Status: Starting network monitoring loop");

    loop {
        counter += 1;
        
        // Show status periodically using module function
        if counter % 20000 == 0 {
            match get_status(&mut stack) {
                Ok(Some(status)) => {
                    rprintln!("Status: CONNECTED {:?}", status);
                }
                Ok(None) => {
                    rprintln!("Status: Connection lost");
                }
                Err(e) => {
                    rprintln!("Status: Check failed: {}", e);
                }
            }
        }

        // Slow blink when connected
        if counter % 10000 == 0 {
            led.toggle();
        }
        
        // Simple delay
        for _ in 0..1000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}

// Embassy async implementation main function
#[cfg(feature = "embassy")]
async fn embassy_main(spawner: Spawner) -> ! {
    // Initialize heap allocator for WiFi
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    rtt_init_print!();
    rprintln!("ESP32-C3 WiFi Simple - Embassy Async Implementation");

    // WiFi configuration from environment variables
    let ssid = env!("WIFI_SSID");
    let password = env!("WIFI_PASSWORD");
    
    rprintln!("Config: SSID={} Password=***", ssid);

    // Initialize hardware
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    
    // Status LED
    let mut led = Output::new(peripherals.GPIO8, Level::Low, OutputConfig::default());

    // Initialize timers (esp-hal 1.0.0-rc.0 API)
    let timer_group0 = TimerGroup::new(peripherals.TIMG0);
    let timer_group1 = TimerGroup::new(peripherals.TIMG1);
    
    // Initialize Embassy with TIMG1 
    esp_hal_embassy::init(timer_group1.timer0);
    
    // Initialize WiFi with proper RNG (esp-hal 1.0.0-rc.0 API)
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let esp_wifi_ctrl = mk_static!(
        esp_wifi::EspWifiController,
        esp_wifi::init(timer_group0.timer0, rng.clone()).unwrap()
    );
    rprintln!("Hardware: WiFi initialized");

    // Create WiFi controller and interfaces
    let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let device = interfaces.sta;
    rprintln!("Hardware: WiFi controller created");

    // Initialize Embassy network stack with static allocation  
    let stack_resources = mk_static!(StackResources<3>, StackResources::<3>::new());
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let (stack, runner) = init_wifi_embassy(device, stack_resources, seed);
    let stack = mk_static!(embassy_net::Stack<'static>, stack);
    
    rprintln!("Embassy: Starting async tasks");

    // Spawn tasks like the working example
    spawner.spawn(wifi_connection_task(controller, ssid, password)).unwrap();
    spawner.spawn(network_task(runner)).unwrap();

    // Wait for link up (like working example)
    led.set_high();
    loop {
        if stack.is_link_up() {
            break;
        }
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
    rprintln!("Embassy: Link is UP, waiting for IP address...");
    
    // Wait for IP address (like working example)  
    loop {
        if let Some(config) = stack.config_v4() {
            rprintln!("NETWORK READY - Embassy Async");
            rprintln!("Got IP: {}", config.address);
            rprintln!("Test: ping {}", config.address.address());
            led.set_low();
            break;
        }
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }

    // Test MQTT functionality if enabled
    #[cfg(feature = "mqtt")]
    {
        rprintln!("MQTT: Starting Embassy MQTT client");
        
        // Spawn MQTT client task
        spawner.spawn(embassy_mqtt_task(stack)).unwrap();
        rprintln!("MQTT: Embassy MQTT task spawned");
    }

    // Main embassy loop with status monitoring
    let mut counter = 0;
    loop {
        counter += 1;
        
        // Show status periodically
        if counter % 4000 == 0 {
            if let Some(config) = stack.config_v4() {
                rprintln!("Status: CONNECTED IP={}", config.address.address());
            } else {
                rprintln!("Status: Connection lost");
            }
        }
        
        // Slow blink when connected
        if counter % 2000 == 0 {
            led.toggle();
        }
        
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
    }
}

// Embassy MQTT client task - async implementation
#[cfg(feature = "embassy")]
#[cfg(feature = "mqtt")]
#[embassy_executor::task]
async fn embassy_mqtt_task(stack: &'static embassy_net::Stack<'static>) {
    use embassy_net::tcp::TcpSocket;
    use embassy_time::{Duration, Timer};
    use embedded_io_async::{Read, Write};
    use alloc::format;
    
    rprintln!("MQTT Embassy: Task started, waiting for network...");
    
    // Wait for network to be ready
    stack.wait_config_up().await;
    
    if let Some(config) = stack.config_v4() {
        rprintln!("MQTT Embassy: Network ready, IP: {}", config.address.address());
    }
    
    // MQTT broker configuration
    let broker_addr = (core::net::Ipv4Addr::new(10, 10, 10, 210), 1883);
    rprintln!("MQTT Embassy: Connecting to broker {}:{}", broker_addr.0, broker_addr.1);
    
    // Create TCP socket with buffers
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    
    // Connect to MQTT broker
    match socket.connect(broker_addr).await {
        Ok(()) => {
            rprintln!("MQTT Embassy: Connected to broker");
        }
        Err(e) => {
            rprintln!("MQTT Embassy: Failed to connect: {:?}", e);
            return;
        }
    }
    
    // Create and send MQTT CONNECT packet
    let client_id = "esp32-embassy-client";
    let connect_packet = create_mqtt_connect_packet(client_id);
    
    match socket.write_all(&connect_packet).await {
        Ok(()) => {
            rprintln!("MQTT Embassy: CONNECT packet sent");
        }
        Err(e) => {
            rprintln!("MQTT Embassy: Failed to send CONNECT: {:?}", e);
            return;
        }
    }
    
    // Read CONNACK response
    let mut buffer = [0u8; 64];
    match socket.read(&mut buffer).await {
        Ok(n) => {
            if n >= 4 && buffer[0] == 0x20 && buffer[3] == 0x00 {
                rprintln!("MQTT Embassy: CONNACK received - connection accepted");
            } else {
                rprintln!("MQTT Embassy: Unexpected CONNACK response");
                return;
            }
        }
        Err(e) => {
            rprintln!("MQTT Embassy: Failed to read CONNACK: {:?}", e);
            return;
        }
    }
    
    // Publish test messages
    let test_messages = [
        ("esp32/status", "online"),
        ("esp32/data", "{\"temperature\":25.5,\"humidity\":60.2}"),
        ("esp32/heartbeat", "ping"),
    ];
    
    for (i, (topic, payload)) in test_messages.iter().enumerate() {
        rprintln!("MQTT Embassy: Publishing message {} to topic '{}'", i + 1, topic);
        
        let publish_packet = create_mqtt_publish_packet(topic, payload.as_bytes());
        
        match socket.write_all(&publish_packet).await {
            Ok(()) => {
                rprintln!("MQTT Embassy: Message {} published successfully", i + 1);
            }
            Err(e) => {
                rprintln!("MQTT Embassy: Failed to publish message {}: {:?}", i + 1, e);
            }
        }
        
        // Wait between messages
        Timer::after(Duration::from_millis(1000)).await;
    }
    
    rprintln!("MQTT Embassy: All messages published, keeping connection alive");
    
    // Keep connection alive with periodic heartbeats
    let mut counter = 0;
    loop {
        Timer::after(Duration::from_millis(10000)).await; // 10 second intervals
        
        counter += 1;
        let heartbeat_msg = format!("heartbeat-{}", counter);
        let publish_packet = create_mqtt_publish_packet("esp32/heartbeat", heartbeat_msg.as_bytes());
        
        match socket.write_all(&publish_packet).await {
            Ok(()) => {
                rprintln!("MQTT Embassy: Heartbeat {} sent", counter);
            }
            Err(e) => {
                rprintln!("MQTT Embassy: Heartbeat failed: {:?}", e);
                break;
            }
        }
        
        // Publish every 30 seconds
        if counter % 3 == 0 {
            let data_msg = format!("{{\"uptime\":{},\"counter\":{}}}", counter * 10, counter);
            let publish_packet = create_mqtt_publish_packet("esp32/data", data_msg.as_bytes());
            
            match socket.write_all(&publish_packet).await {
                Ok(()) => {
                    rprintln!("MQTT Embassy: Data message {} sent", counter);
                }
                Err(e) => {
                    rprintln!("MQTT Embassy: Data publish failed: {:?}", e);
                }
            }
        }
    }
}

// Helper function to create MQTT CONNECT packet
#[cfg(feature = "embassy")]
#[cfg(feature = "mqtt")]
fn create_mqtt_connect_packet(client_id: &str) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    
    let mut packet = Vec::new();
    
    // Fixed header
    packet.push(0x10); // CONNECT packet type
    
    // Variable header
    let mut variable_header = Vec::new();
    
    // Protocol name "MQTT"
    let protocol_name = b"MQTT";
    variable_header.extend_from_slice(&(protocol_name.len() as u16).to_be_bytes());
    variable_header.extend_from_slice(protocol_name);
    
    // Protocol version (4 for MQTT 3.1.1)
    variable_header.push(0x04);
    
    // Connect flags (clean session)
    variable_header.push(0x02);
    
    // Keep alive (60 seconds)
    variable_header.extend_from_slice(&60u16.to_be_bytes());
    
    // Payload - Client ID
    let client_id_bytes = client_id.as_bytes();
    variable_header.extend_from_slice(&(client_id_bytes.len() as u16).to_be_bytes());
    variable_header.extend_from_slice(client_id_bytes);
    
    // Remaining length
    packet.push(variable_header.len() as u8);
    packet.extend_from_slice(&variable_header);
    
    packet
}

// Helper function to create MQTT PUBLISH packet
#[cfg(feature = "embassy")]
#[cfg(feature = "mqtt")]
fn create_mqtt_publish_packet(topic: &str, payload: &[u8]) -> alloc::vec::Vec<u8> {
    use alloc::vec::Vec;
    
    let mut packet = Vec::new();
    
    // Fixed header - PUBLISH packet type (0x30)
    packet.push(0x30);
    
    // Variable header and payload
    let mut variable_header = Vec::new();
    
    // Topic name
    variable_header.extend_from_slice(&(topic.len() as u16).to_be_bytes());
    variable_header.extend_from_slice(topic.as_bytes());
    
    // Payload
    variable_header.extend_from_slice(payload);
    
    // Remaining length
    packet.push(variable_header.len() as u8);
    packet.extend_from_slice(&variable_header);
    
    packet
}