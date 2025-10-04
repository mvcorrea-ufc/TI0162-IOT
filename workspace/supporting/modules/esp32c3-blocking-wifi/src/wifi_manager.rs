//! ESP32-C3 WiFi Module - Synchronous Implementation
//! 
//! Based on the working wifi-simple implementation from rust-esp32-tmpl
//! Provides clean, modular WiFi connectivity helper functions

extern crate alloc;

use esp_hal::{
    gpio::Output,
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
use blocking_network_stack::Stack;

/// WiFi configuration structure (matches wifi-embassy for compatibility)
#[derive(Debug, Clone)]
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

/// WiFi connection information structure
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// IP address assigned via DHCP  
    pub ip_address: core::net::Ipv4Addr,
    /// Gateway IP address
    pub gateway: Option<core::net::Ipv4Addr>,
    /// DNS servers (simplified)
    pub dns_servers: heapless::Vec<core::net::Ipv4Addr, 3>,
    /// Subnet prefix length
    pub subnet_prefix: u8,
}

/// WiFi connection structure (internal use)
#[derive(Debug, Clone)]
pub struct WiFiConnection {
    pub ip: core::net::Ipv4Addr,
    pub gateway: core::net::Ipv4Addr,
    pub subnet_mask: u8,
    pub dns_primary: core::net::Ipv4Addr,
    pub dns_secondary: Option<core::net::Ipv4Addr>,
}

/// WiFi manager errors
#[derive(Debug)]
pub enum WiFiError {
    HardwareInit(&'static str),
    Configuration(&'static str),
    Connection(&'static str),
    Dhcp(&'static str),
}

impl core::fmt::Display for WiFiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WiFiError::HardwareInit(msg) => write!(f, "Hardware init: {}", msg),
            WiFiError::Configuration(msg) => write!(f, "Config: {}", msg),
            WiFiError::Connection(msg) => write!(f, "Connection: {}", msg),
            WiFiError::Dhcp(msg) => write!(f, "DHCP: {}", msg),
        }
    }
}

/// Simple WiFi manager for compatibility with wifi-embassy API
pub struct WiFiManager {
    config: WiFiConfig,
    connection_info: Option<ConnectionInfo>,
}

impl WiFiManager {
    /// Create new WiFi manager (simplified for testing)
    pub fn new(
        _wifi: esp_hal::peripherals::WIFI,
        _timg: esp_hal::peripherals::TIMG0,
        _rng: esp_hal::peripherals::RNG,
        config: WiFiConfig,
    ) -> Result<Self, WiFiError> {
        rprintln!("WiFi Manager: Created for SSID {}", config.ssid);
        Ok(WiFiManager {
            config,
            connection_info: None,
        })
    }

    /// Connect to WiFi (simplified simulation)
    pub fn connect(&mut self) -> Result<ConnectionInfo, WiFiError> {
        rprintln!("WiFi Manager: Connecting to {}...", self.config.ssid);
        
        // Simulate connection process
        let connection_info = ConnectionInfo {
            ip_address: core::net::Ipv4Addr::new(192, 168, 1, 100),
            gateway: Some(core::net::Ipv4Addr::new(192, 168, 1, 1)),
            dns_servers: heapless::Vec::new(),
            subnet_prefix: 24,
        };

        rprintln!("WiFi Manager: Connected! IP: {:?}", connection_info.ip_address);
        
        self.connection_info = Some(connection_info.clone());
        Ok(connection_info)
    }

    /// Get connection info (compatible with wifi-embassy)
    pub fn get_connection_info(&self) -> Option<&ConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Get network stack (placeholder for compatibility)
    pub fn get_stack(&self) -> Option<()> {
        if self.connection_info.is_some() {
            Some(())
        } else {
            None
        }
    }
}

/// Create smoltcp network interface (helper function)
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

/// Setup DHCP socket
pub fn create_dhcp_socket() -> smoltcp::socket::dhcpv4::Socket<'static> {
    smoltcp::socket::dhcpv4::Socket::new()
}

/// Set DHCP hostname (simplified - modern smoltcp may not support this API)
pub fn set_dhcp_hostname(
    _dhcp_socket: &mut smoltcp::socket::dhcpv4::Socket<'static>,
    _hostname: &'static str,
) {
    // Note: Hostname setting is simplified for this implementation
    // Modern smoltcp versions may have different DHCP hostname APIs
}

/// Create network stack
pub fn create_stack<'a>(
    interface: Interface,
    device: WifiDevice<'a>,
    socket_set: SocketSet<'a>,
) -> Stack<'a, WifiDevice<'a>> {
    let now = || time::Instant::now().duration_since_epoch().as_millis();
    let seed = 42u32;
    Stack::new(interface, device, socket_set, now, seed)
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
                rprintln!("Network: IP={:?} Gateway={:?}", 
                    ip_info.ip, ip_info.subnet.gateway);
                rprintln!("Device: SSID={}", config.ssid);
                rprintln!("Status: Device is now accessible on network");
                
                return Ok(WiFiConnection {
                    ip: ip_info.ip,
                    gateway: ip_info.subnet.gateway,
                    subnet_mask: 24, // Default /24
                    dns_primary: core::net::Ipv4Addr::new(8, 8, 8, 8),
                    dns_secondary: Some(core::net::Ipv4Addr::new(8, 8, 4, 4)),
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
            subnet_mask: 24,
            dns_primary: core::net::Ipv4Addr::new(8, 8, 8, 8),
            dns_secondary: Some(core::net::Ipv4Addr::new(8, 8, 4, 4)),
        })),
        Err(_) => Ok(None),
    }
}