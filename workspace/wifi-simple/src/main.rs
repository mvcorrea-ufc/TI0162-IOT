//! ESP32-C3 WiFi Simple - Clean Modular Implementation
//! 
//! Uses clean wifi module with helper functions
//! Clean separation of concerns and easy to understand

#![no_std]
#![no_main]

extern crate alloc;

mod wifi;

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

#[main]
fn main() -> ! {
    // Initialize heap allocator for WiFi
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    rtt_init_print!();
    rprintln!("ESP32-C3 WiFi Simple - Clean Modular Implementation");

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
    let esp_wifi_ctrl = init(timer_group.timer0).unwrap();
    rprintln!("Hardware: WiFi initialized");

    // Create WiFi controller and interfaces
    let (mut controller, interfaces) = esp_wifi::wifi::new(&esp_wifi_ctrl, peripherals.WIFI).unwrap();
    let mut device = interfaces.sta;
    rprintln!("Hardware: WiFi controller created");

    // Create network interface using module
    let iface = create_interface(&mut device);
    
    // Set up DHCP socket using module functions
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut socket_set = SocketSet::new(&mut socket_set_entries[..]);
    let mut dhcp_socket = create_dhcp_socket();
    set_dhcp_hostname(&mut dhcp_socket, wifi_config.hostname);
    socket_set.add(dhcp_socket);
    
    // Create network stack using module
    let mut stack = create_stack(iface, device, socket_set);
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

    // Main loop with periodic status monitoring using module
    let mut counter = 0;
    rprintln!("Status: Starting network monitoring loop");

    loop {
        counter += 1;
        
        // Show status periodically using module function
        if counter % 20000 == 0 {
            match get_status(&mut stack) {
                Ok(Some(status)) => {
                    rprintln!("Status: CONNECTED IP={} GW={}", status.ip, status.gateway);
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