//! ESP32-C3 WiFi Embassy Module - Async Non-Blocking Implementation
//! 
//! Provides async WiFi connectivity using Embassy executor
//! Non-blocking approach with cooperative multitasking
//!
//! ## MQTT Client Foundation
//!
//! This module serves as the network foundation for MQTT client applications.
//! The async network stack and WiFi management enable efficient MQTT operations
//! through a separate MQTT module.
//!
//! ### Integration with MQTT Module:
//! ```rust,ignore
//! // WiFi Embassy provides the network stack
//! let stack = init_wifi_embassy(wifi_device, &mut stack_resources, rng_seed);
//! 
//! // MQTT module uses the same stack for TCP connections
//! // (See separate mqtt_embassy.rs module)
//! spawner.spawn(wifi_connection_task(manager, led)).unwrap();
//! spawner.spawn(network_task(&stack)).unwrap();
//! ```
//!
//! ### Embassy Network Stack Benefits:
//! - **Shared TCP Stack**: Both WiFi and MQTT use same embassy-net stack
//! - **Concurrent Operations**: WiFi monitoring + MQTT operations without blocking
//! - **Efficient Resource Usage**: Single network stack, cooperative multitasking
//! - **Robust Connection Handling**: Async retry logic for network operations

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, Stack, Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::Output, 
    rng::Rng,
};
use esp_wifi::{
    wifi::{ClientConfiguration, Configuration, WifiController, WifiDevice},
    init,
};
use rtt_target::rprintln;
use core::net::Ipv4Addr;

// Macro for creating static allocations
#[macro_export]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// WiFi configuration structure
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
    pub hostname: &'static str,
}

/// WiFi connection information
#[derive(Debug, Clone)]
pub struct WiFiConnection {
    pub ip: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub subnet_mask: u8,
    pub dns_primary: Option<Ipv4Addr>,
    pub dns_secondary: Option<Ipv4Addr>,
}

/// WiFi Embassy Manager - manages async WiFi operations
pub struct WiFiEmbassyManager<'a> {
    controller: WifiController<'a>,
    stack: &'a Stack<'a>,
    config: WiFiConfig,
}

impl<'a> WiFiEmbassyManager<'a> {
    /// Create new WiFi Embassy Manager
    pub fn new(
        controller: WifiController<'a>,
        stack: &'a Stack<'a>,
        config: WiFiConfig,
    ) -> Self {
        Self {
            controller,
            stack,
            config,
        }
    }

    /// Configure WiFi connection (async)
    pub async fn configure_wifi(&mut self) -> Result<(), &'static str> {
        rprintln!("WiFi Embassy: Configuring for SSID: {}", self.config.ssid);
        
        let client_config = Configuration::Client(ClientConfiguration {
            ssid: self.config.ssid.try_into().map_err(|_| "Invalid SSID")?,
            password: self.config.password.try_into().map_err(|_| "Invalid password")?,
            ..Default::default()
        });
        
        self.controller.set_configuration(&client_config)
            .map_err(|_| "Failed to set WiFi configuration")?;
        
        self.controller.start()
            .map_err(|_| "Failed to start WiFi controller")?;
        
        Ok(())
    }

    /// Scan networks async
    pub async fn scan_networks(&mut self) -> Result<(), &'static str> {
        rprintln!("WiFi Embassy: Scanning for networks...");
        
        match self.controller.scan_n(5) {
            Ok(networks) => {
                rprintln!("Found {} networks:", networks.len());
                for (i, ap) in networks.iter().enumerate() {
                    rprintln!("  {}: {} ({})", i, ap.ssid, ap.signal_strength);
                }
                Ok(())
            }
            Err(_) => Err("Failed to scan networks")
        }
    }

    /// Connect WiFi using async API
    pub async fn connect_wifi(&mut self) -> Result<(), &'static str> {
        rprintln!("WiFi Embassy: Starting connection to {}...", self.config.ssid);
        
        self.controller.start_async()
            .await
            .map_err(|_| "Failed to start WiFi controller")?;
            
        rprintln!("WiFi Embassy: Connecting...");
        
        match self.controller.connect_async().await {
            Ok(_) => {
                rprintln!("WiFi Embassy: Connected successfully");
                Ok(())
            }
            Err(_) => Err("Failed to connect to WiFi")
        }
    }

    /// Wait for IP address via DHCP (async)
    pub async fn wait_for_ip(&self) -> Result<WiFiConnection, &'static str> {
        rprintln!("WiFi Embassy: Waiting for DHCP IP address...");
        
        // Wait for network stack to be ready
        self.stack.wait_config_up().await;
        
        // Get IP configuration
        let config = self.stack.config_v4().ok_or("Failed to get IPv4 config")?;
        
        let ip = config.address.address();
        let gw = config.gateway.unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
        let subnet_bits = config.address.prefix_len();
        let dns_primary = config.dns_servers.first().copied();
        let dns_secondary = config.dns_servers.get(1).copied();
        
        rprintln!("WiFi Embassy: IP address acquired successfully");
        rprintln!("Network: IP={:?} Gateway={:?} Subnet=/{} DNS={:?}", 
            ip, gw, subnet_bits, dns_primary);
        rprintln!("Device: Hostname={} SSID={}", self.config.hostname, self.config.ssid);
        rprintln!("Status: Device is now accessible on network");
        
        Ok(WiFiConnection {
            ip,
            gateway: gw,
            subnet_mask: subnet_bits,
            dns_primary,
            dns_secondary,
        })
    }

    /// Get current connection status (async)
    pub fn get_status(&self) -> Result<Option<WiFiConnection>, &'static str> {
        if !self.stack.is_link_up() {
            return Ok(None);
        }
        
        match self.stack.config_v4() {
            Some(config) => {
                let ip = config.address.address();
                let gw = config.gateway.unwrap_or(Ipv4Addr::new(0, 0, 0, 0));
                let subnet_bits = config.address.prefix_len();
                let dns_primary = config.dns_servers.first().copied();
                let dns_secondary = config.dns_servers.get(1).copied();
                
                Ok(Some(WiFiConnection {
                    ip,
                    gateway: gw,
                    subnet_mask: subnet_bits,
                    dns_primary,
                    dns_secondary,
                }))
            }
            None => Ok(None),
        }
    }
}

/// Initialize WiFi Embassy infrastructure
pub fn init_wifi_embassy(
    wifi_device: WifiDevice<'static>,
    stack_resources: &'static mut StackResources<3>,
    rng: u64,
) -> (Stack<'static>, Runner<'static, WifiDevice<'static>>) {
    // Configure network with DHCP (like working example)
    let config = NetConfig::dhcpv4(Default::default());
    
    rprintln!("WiFi Embassy: Creating network stack with DHCP (based on working example)");
    
    // Create embassy network stack
    embassy_net::new(
        wifi_device,
        config,
        stack_resources,
        rng,
    )
}

/// Embassy task for WiFi connection management (based on working example)
#[embassy_executor::task]
pub async fn wifi_connection_task(
    mut controller: WifiController<'static>,
    ssid: &'static str,
    password: &'static str,
) {
    use esp_wifi::wifi::{WifiState, WifiEvent};
    
    rprintln!("WiFi Embassy: Starting connection task");
    rprintln!("Device capabilities: {:?}", controller.capabilities());
    
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: ssid.try_into().unwrap(),
                password: password.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            rprintln!("WiFi Embassy: Starting WiFi...");
            controller.start_async().await.unwrap();
            rprintln!("WiFi Embassy: WiFi started!");
        }
        
        rprintln!("WiFi Embassy: About to connect to {}...", ssid);
        match controller.connect_async().await {
            Ok(_) => {
                rprintln!("WiFi Embassy: Connected successfully!");
            }
            Err(e) => {
                rprintln!("WiFi Embassy: Connection failed: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

/// Embassy task for status monitoring with LED
#[embassy_executor::task]
pub async fn status_monitor_task(
    stack: &'static Stack<'static>,
    mut led: Output<'static>,
) {
    rprintln!("Status Monitor: Starting DHCP wait (can take up to 30 seconds)...");
    
    // Wait for link up first
    while !stack.is_link_up() {
        rprintln!("Status Monitor: Waiting for WiFi link...");
        Timer::after(Duration::from_millis(1000)).await;
    }
    rprintln!("Status Monitor: WiFi link is UP, waiting for DHCP IP...");
    
    // Wait for network to be ready with timeout monitoring
    let mut dhcp_wait_counter = 0;
    loop {
        if let Some(config) = stack.config_v4() {
            let ip = config.address.address();
            rprintln!("NETWORK READY - Embassy Async");
            rprintln!("IP Address: {}", ip);
            rprintln!("Test: ping {}", ip);
            break;
        }
        
        // Show DHCP wait progress every 2 seconds
        if dhcp_wait_counter % 200 == 0 {
            rprintln!("Status Monitor: Still waiting for DHCP IP... ({} seconds)", dhcp_wait_counter / 100);
        }
        
        dhcp_wait_counter += 1;
        Timer::after(Duration::from_millis(10)).await;
        
        // Timeout after 60 seconds
        if dhcp_wait_counter > 6000 {
            rprintln!("ERROR: DHCP timeout after 60 seconds");
            break;
        }
    }
    
    led.set_low();
    
    // Status monitoring loop
    let mut counter = 0;
    rprintln!("Status: Starting async network monitoring");
    
    loop {
        counter += 1;
        
        // Check status periodically
        if counter % 200 == 0 {
            if let Some(config) = stack.config_v4() {
                let ip = config.address.address();
                rprintln!("Status: CONNECTED IP={}", ip);
            } else {
                rprintln!("Status: Connection lost");
            }
        }
        
        // Slow blink when connected
        if counter % 100 == 0 {
            led.toggle();
        }
        
        // Async delay - yields to other tasks
        Timer::after(Duration::from_millis(10)).await;
    }
}

/// Embassy task for network stack processing
#[embassy_executor::task]
pub async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}

/// Check if network stack is ready for MQTT connections
pub fn is_network_ready(stack: &Stack<'_>) -> bool {
    stack.is_link_up() && stack.config_v4().is_some()
}

/// Embassy task that signals when network is ready for MQTT
/// 
/// This task can be used by the MQTT module to wait for network readiness
/// before attempting MQTT broker connections
#[embassy_executor::task]
pub async fn network_ready_task(
    stack: &'static Stack<'static>,
    spawner: Spawner,
) {
    // Wait for network configuration
    stack.wait_config_up().await;
    
    rprintln!("Network Ready: Embassy stack configured and ready for MQTT");
    
    // Here the MQTT module can be spawned
    // Example: spawner.spawn(mqtt_client_task(stack)).unwrap();
}