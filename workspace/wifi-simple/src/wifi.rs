//! ESP32-C3 WiFi Module - Simple and Clean
//! 
//! Provides WiFi connectivity helper functions
//! Keeps it simple - no complex lifetimes, just helper functions

extern crate alloc;

use blocking_network_stack::Stack;
use esp_hal::{
    gpio::Output,
    rng::Rng,
    time,
};
use rtt_target::rprintln;
use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice},
};
use smoltcp::{
    iface::{SocketSet, Interface, Config as IfaceConfig},
    wire::{HardwareAddress, EthernetAddress},
};

/// WiFi configuration
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
    pub hostname: &'static str,
}

/// WiFi connection information
#[derive(Debug, Clone)]
pub struct WiFiConnection {
    pub ip: core::net::Ipv4Addr,
    pub gateway: core::net::Ipv4Addr,
    pub subnet_mask: u8,
    pub dns_primary: Option<core::net::Ipv4Addr>,
    pub dns_secondary: Option<core::net::Ipv4Addr>,
}

/// Create smoltcp network interface
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

/// Setup DHCP socket - returns the socket for adding to socket set
pub fn create_dhcp_socket() -> smoltcp::socket::dhcpv4::Socket<'static> {
    smoltcp::socket::dhcpv4::Socket::new()
}

/// Set DHCP hostname option (simplified - hostname set via other means)
pub fn set_dhcp_hostname(_dhcp_socket: &mut smoltcp::socket::dhcpv4::Socket<'static>, _hostname: &'static str) {
    // Hostname configuration simplified for clean modular approach
    // DHCP hostname can be set via router configuration if needed
}

/// Create network stack
pub fn create_stack<'a>(
    iface: Interface,
    device: WifiDevice<'a>,
    socket_set: SocketSet<'a>,
) -> Stack<'a, WifiDevice<'a>> {
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let rng = Rng::new();
    Stack::new(iface, device, socket_set, now, rng.random())
}

/// Configure WiFi connection
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

/// Scan and display networks
pub fn scan_networks(controller: &mut WifiController) {
    rprintln!("WiFi: Scanning for networks...");
    if let Ok(networks) = controller.scan_n(5) {
        rprintln!("Found {} networks:", networks.len());
        for (i, ap) in networks.iter().enumerate() {
            rprintln!("  {}: {} ({})", i, ap.ssid, ap.signal_strength);
        }
    }
}

/// Wait for WiFi connection
pub fn wait_for_connection(
    controller: &mut WifiController,
    led: &mut Output,
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
                led.toggle();
            }
            Err(_) => {
                if attempts > 100 {
                    return Err("Connection timeout");
                }
            }
        }
        
        // Simple delay
        for _ in 0..100000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}

/// Wait for DHCP IP address
pub fn wait_for_ip<'a>(
    stack: &mut Stack<'a, WifiDevice<'a>>,
    led: &mut Output,
    config: &WiFiConfig,
) -> Result<WiFiConnection, &'static str> {
    rprintln!("DHCP: Waiting for IP address...");
    
    loop {
        stack.work();
        
        if stack.is_iface_up() {
            if let Ok(ip_info) = stack.get_ip_info() {
                rprintln!("DHCP: IP address acquired successfully");
                rprintln!("Network: IP={:?} Gateway={:?} Subnet=/{} DNS={:?}", 
                    ip_info.ip, ip_info.subnet.gateway, ip_info.subnet.mask.0, ip_info.dns);
                rprintln!("Device: Hostname={} SSID={}", config.hostname, config.ssid);
                rprintln!("Status: Device is now accessible on network");
                
                return Ok(WiFiConnection {
                    ip: ip_info.ip,
                    gateway: ip_info.subnet.gateway,
                    subnet_mask: ip_info.subnet.mask.0,
                    dns_primary: ip_info.dns,
                    dns_secondary: ip_info.secondary_dns,
                });
            }
        }
        
        // Fast blink while getting IP
        led.toggle();
        for _ in 0..50000 {
            unsafe { core::ptr::read_volatile(&0); }
        }
    }
}

/// Get current connection status
pub fn get_status<'a>(stack: &mut Stack<'a, WifiDevice<'a>>) -> Result<Option<WiFiConnection>, &'static str> {
    stack.work();
    
    if !stack.is_iface_up() {
        return Ok(None);
    }
    
    match stack.get_ip_info() {
        Ok(ip_info) => Ok(Some(WiFiConnection {
            ip: ip_info.ip,
            gateway: ip_info.subnet.gateway,
            subnet_mask: ip_info.subnet.mask.0,
            dns_primary: ip_info.dns,
            dns_secondary: ip_info.secondary_dns,
        })),
        Err(_) => Ok(None),
    }
}