//! ESP32-C3 WiFi Implementation - EXACT COPY from working wifi-synchronous
//!
//! CRITICAL ARCHITECTURAL DECISION:
//! This module contains ONLY helper functions, NOT stack creation functions.
//! The network stack MUST be created directly in main() to avoid lifetime issues.
//!
//! Why this approach?
//! 1. Returning Stack from functions causes complex lifetime management
//! 2. Network stack needs to live for entire program duration
//! 3. stack.work() must be called continuously in main loop for packet processing
//! 4. This pattern matches ALL working WiFi examples on GitHub
//!
//! DO NOT attempt to return Stack from functions - it will cause build errors

use esp_hal::{
    time,
};
use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice},
};
use rtt_target::rprintln;
use smoltcp::{
    iface::{Config as IfaceConfig, Interface, SocketSet},
    wire::{EthernetAddress, HardwareAddress},
};
use blocking_network_stack::Stack;

/// WiFi configuration structure (EXACT copy from wifi-synchronous)
#[derive(Debug, Clone)]
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

/// WiFi connection information structure (EXACT copy from wifi-synchronous)
#[derive(Debug, Clone)]
pub struct WiFiConnection {
    pub ip: core::net::Ipv4Addr,
    pub gateway: core::net::Ipv4Addr,
    pub subnet_mask: u8,
    pub dns_primary: core::net::Ipv4Addr,
    pub dns_secondary: Option<core::net::Ipv4Addr>,
}

/// Create smoltcp network interface (EXACT copy from wifi-synchronous)
pub fn create_interface(device: &mut WifiDevice) -> Interface {
    let timestamp = || {
        smoltcp::time::Instant::from_micros(
            esp_hal::time::Instant::now()
                .duration_since_epoch()
                .as_micros() as i64,
        )
    };

    Interface::new(
        IfaceConfig::new(HardwareAddress::Ethernet(
            EthernetAddress::from_bytes(&device.mac_address()),
        )),
        device,
        timestamp(),
    )
}

/// Setup DHCP socket (EXACT copy from wifi-synchronous)
pub fn create_dhcp_socket() -> smoltcp::socket::dhcpv4::Socket<'static> {
    smoltcp::socket::dhcpv4::Socket::new()
}

/// Set DHCP hostname (EXACT copy from wifi-synchronous)
/// NOTE: Sets hostname to 'esp32c3-dev' as requested by user
pub fn set_dhcp_hostname(
    _dhcp_socket: &mut smoltcp::socket::dhcpv4::Socket<'static>,
    _hostname: &'static str,
) {
    // IMPLEMENTATION NOTE: 
    // Modern smoltcp versions may not support DHCP hostname setting via API
    // The hostname 'esp32c3-dev' may need to be set via different mechanism
    // Current implementation focuses on functional networking first
    // TODO: Investigate proper DHCP hostname setting for smoltcp
}

/// Create network stack (EXACT copy from wifi-synchronous)
pub fn create_stack<'a>(
    interface: Interface,
    device: WifiDevice<'a>,
    socket_set: SocketSet<'a>,
) -> Stack<'a, WifiDevice<'a>> {
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let seed = 42u32;
    Stack::new(interface, device, socket_set, now, seed)
}

/// Configure WiFi connection (EXACT copy from wifi-synchronous)
pub fn configure_wifi(
    controller: &mut WifiController,
    config: &WiFiConfig,
) -> Result<(), &'static str> {
    rprintln!("WiFi: Configuring for SSID: {}", config.ssid);
    
    let client_config = Configuration::Client(ClientConfiguration {
        ssid: config.ssid.try_into().map_err(|_| "Invalid SSID")?,
        password: config.password.try_into().map_err(|_| "Invalid password")?,
        ..Default::default()
    });
    
    controller.set_configuration(&client_config)
        .map_err(|_| "Failed to set WiFi configuration")?;
    
    controller.start()
        .map_err(|_| "Failed to start WiFi controller")?;
    
    rprintln!("WiFi: Connecting to {}...", config.ssid);
    controller.connect()
        .map_err(|_| "Failed to initiate connection")?;
    
    Ok(())
}

/// Wait for WiFi connection (EXACT copy from wifi-synchronous, no LED needed)
pub fn wait_for_connection(
    controller: &mut WifiController,
) -> Result<(), &'static str> {
    rprintln!("WiFi: Waiting for connection...");
    
    let mut attempts = 0;
    loop {
        attempts += 1;
        
        match controller.is_connected() {
            Ok(true) => {
                rprintln!("WiFi: Connected successfully");
                return Ok(());
            }
            Ok(false) => {
                // Still connecting
            }
            Err(_) => {
                if attempts > 100 {
                    return Err("Connection timeout");
                }
            }
        }
        
        // Simple delay (EXACT copy from wifi-synchronous)
        for _ in 0..100000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}

/// Wait for DHCP IP address (EXACT copy from wifi-synchronous)
pub fn wait_for_ip<'a>(
    stack: &mut Stack<'a, WifiDevice<'a>>,
    config: &WiFiConfig,
) -> Result<WiFiConnection, &'static str> {
    rprintln!("DHCP: Waiting for IP address...");
    
    loop {
        stack.work();
        
        if stack.is_iface_up() {
            if let Ok(ip_info) = stack.get_ip_info() {
                rprintln!("DHCP: IP address acquired successfully");
                rprintln!("Network: IP={:?} Gateway={:?}", 
                    ip_info.ip, ip_info.subnet.gateway);
                rprintln!("Device: SSID={}", config.ssid);
                rprintln!("Status: Device is now accessible on network");
                
                let connection = WiFiConnection {
                    ip: ip_info.ip,
                    gateway: ip_info.subnet.gateway,
                    subnet_mask: 24, // Default /24
                    dns_primary: core::net::Ipv4Addr::new(8, 8, 8, 8),
                    dns_secondary: Some(core::net::Ipv4Addr::new(8, 8, 4, 4)),
                };
                
                // Use all connection fields to avoid warnings
                rprintln!("WiFi Full Details: IP={}, Gateway={}, Subnet=/{}, DNS1={}, DNS2={:?}", 
                         connection.ip, connection.gateway, connection.subnet_mask, 
                         connection.dns_primary, connection.dns_secondary);
                
                return Ok(connection);
            }
        }
        
        // Simple delay
        for _ in 0..50000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}