//! # WiFi Simple Embassy
//! 
//! A simple, robust WiFi library for ESP32-C3 using Embassy async framework.
//! 
//! This library provides easy WiFi connectivity with DHCP using the latest:
//! - esp-hal 1.0.0-rc.0 
//! - esp-wifi 0.15.0
//! - Embassy async framework
//! 
//! ## Features
//! - Simple WiFi connection management
//! - DHCP IP address acquisition
//! - Embassy async networking foundation
//! - Compatible with vanilla esp-hal (no custom toolchains)
//! - Clean, maintainable API
//! 
//! ## Usage
//! 
//! ```rust,no_run
//! use wifi_simple_embassy::*;
//! use embassy_executor::Spawner;
//! 
//! #[esp_hal_embassy::main]
//! async fn main(spawner: Spawner) {
//!     let peripherals = esp_hal::init(esp_hal::Config::default());
//!     let wifi_manager = WiFiManager::new(
//!         spawner,
//!         peripherals.TIMG0,
//!         peripherals.TIMG1, 
//!         peripherals.WIFI,
//!         peripherals.RNG,
//!         "MyNetwork",
//!         "MyPassword"
//!     ).await?;
//!     
//!     // WiFi is now connected and ready to use!
//!     let stack = wifi_manager.get_stack();
//! }
//! ```

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::{Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::{
    peripherals::{TIMG0, TIMG1, WIFI},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_wifi::{
    init,
    wifi::{Configuration, WifiController, WifiDevice, WifiState, WifiEvent},
    EspWifiController,
};
use log::{info, error, warn};
// StaticCell is used via the mk_static! macro

// Re-exports for convenience
pub use embassy_executor::task;
pub use embassy_net::Config as NetConfig;
pub use esp_hal;
pub use esp_wifi;
pub use log;

/// Utility macro for creating static allocations
#[macro_export]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// WiFi connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// IP address assigned via DHCP
    pub ip_address: embassy_net::Ipv4Address,
    /// Gateway IP address
    pub gateway: Option<embassy_net::Ipv4Address>,
    /// DNS servers
    pub dns_servers: heapless::Vec<embassy_net::Ipv4Address, 3>,
    /// Subnet prefix length
    pub subnet_prefix: u8,
}

/// WiFi manager errors
#[derive(Debug)]
pub enum WiFiError {
    /// Hardware initialization failed
    HardwareInit(&'static str),
    /// WiFi configuration failed
    Configuration(&'static str),
    /// Connection failed
    Connection(&'static str),
    /// DHCP failed
    Dhcp(&'static str),
}

impl core::fmt::Display for WiFiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WiFiError::HardwareInit(msg) => write!(f, "Hardware initialization failed: {}", msg),
            WiFiError::Configuration(msg) => write!(f, "WiFi configuration failed: {}", msg),
            WiFiError::Connection(msg) => write!(f, "WiFi connection failed: {}", msg),
            WiFiError::Dhcp(msg) => write!(f, "DHCP failed: {}", msg),
        }
    }
}

/// Simple WiFi manager using Embassy async framework
pub struct WiFiManager {
    stack: &'static Stack<'static>,
    connection_info: Option<ConnectionInfo>,
}

impl WiFiManager {

    /// Create a new WiFi manager from raw peripherals
    /// 
    /// This follows the exact pattern from wifi-new-01 that works.
    /// 
    /// # Arguments
    /// * `spawner` - Embassy task spawner
    /// * `timg0` - Timer group 0 peripheral (for WiFi)
    /// * `timg1` - Timer group 1 peripheral (for Embassy)  
    /// * `wifi` - WiFi peripheral
    /// * `rng_peripheral` - Random number generator peripheral
    /// * `ssid` - WiFi network name
    /// * `password` - WiFi network password
    pub async fn new(
        spawner: Spawner,
        timg0: TIMG0<'static>,
        timg1: TIMG1<'static>,
        wifi: WIFI<'static>,
        rng_peripheral: esp_hal::peripherals::RNG<'static>,
        ssid: &'static str,
        password: &'static str,
    ) -> Result<Self, WiFiError> {
        info!("WiFi Simple Embassy: Initializing WiFi manager");

        // Initialize timers (like wifi-new-01)
        let timer_group0 = TimerGroup::new(timg0);
        let timer_group1 = TimerGroup::new(timg1);
        
        // Initialize Embassy with TIMG1 (like wifi-new-01)
        esp_hal_embassy::init(timer_group1.timer0);
        info!("WiFi Simple Embassy: Embassy initialized");
        
        // Initialize WiFi with proper RNG (like wifi-new-01)
        let mut rng = Rng::new(rng_peripheral);
        let esp_wifi_ctrl = mk_static!(
            EspWifiController,
            init(timer_group0.timer0, rng.clone())
                .map_err(|_| WiFiError::HardwareInit("Failed to initialize esp-wifi"))?
        );
        info!("WiFi Simple Embassy: WiFi hardware initialized");

        // Create WiFi controller and interfaces (like wifi-new-01)
        let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, wifi)
            .map_err(|_| WiFiError::HardwareInit("Failed to create WiFi controller"))?;
        let device = interfaces.sta;
        info!("WiFi Simple Embassy: WiFi controller created");

        // Initialize Embassy network stack with static allocation
        let stack_resources = mk_static!(StackResources<3>, StackResources::<3>::new());
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;
        let (stack, runner) = embassy_net::new(
            device,
            embassy_net::Config::dhcpv4(Default::default()),
            stack_resources,
            seed,
        );
        let stack = mk_static!(Stack<'static>, stack);
        
        info!("WiFi Simple Embassy: Network stack created");

        // Spawn background tasks
        spawner.spawn(wifi_connection_task(controller, ssid, password))
            .map_err(|_| WiFiError::Configuration("Failed to spawn WiFi task"))?;
        spawner.spawn(network_task(runner))
            .map_err(|_| WiFiError::Configuration("Failed to spawn network task"))?;

        info!("WiFi Simple Embassy: Background tasks started");

        // Wait for link up
        info!("WiFi Simple Embassy: Waiting for WiFi connection...");
        let mut timeout_counter = 0;
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
            timeout_counter += 1;
            
            if timeout_counter > 60 {  // 30 second timeout
                return Err(WiFiError::Connection("WiFi connection timeout"));
            }
            
            if timeout_counter % 10 == 0 {
                info!("WiFi Simple Embassy: Still waiting for connection... ({} seconds)", timeout_counter / 2);
            }
        }
        info!("WiFi Simple Embassy: WiFi link established");
        
        // Wait for IP address
        info!("WiFi Simple Embassy: Waiting for DHCP IP address...");
        timeout_counter = 0;
        loop {
            if let Some(config) = stack.config_v4() {
                let connection_info = ConnectionInfo {
                    ip_address: config.address.address(),
                    gateway: config.gateway,
                    dns_servers: config.dns_servers,
                    subnet_prefix: config.address.prefix_len(),
                };
                
                info!("WiFi Simple Embassy: ✅ Connected successfully!");
                info!("WiFi Simple Embassy: IP address: {}", connection_info.ip_address);
                info!("WiFi Simple Embassy: Gateway: {:?}", connection_info.gateway);
                info!("WiFi Simple Embassy: Device is now pingable!");
                
                return Ok(Self {
                    stack,
                    connection_info: Some(connection_info),
                });
            }
            Timer::after(Duration::from_millis(500)).await;
            timeout_counter += 1;
            
            if timeout_counter > 60 {  // 30 second timeout
                return Err(WiFiError::Dhcp("DHCP timeout"));
            }
            
            if timeout_counter % 10 == 0 {
                info!("WiFi Simple Embassy: Still waiting for DHCP... ({} seconds)", timeout_counter / 2);
            }
        }
    }

    /// Get the network stack for creating sockets
    pub fn get_stack(&self) -> &'static Stack<'static> {
        self.stack
    }

    /// Get current connection information
    pub fn get_connection_info(&self) -> Option<&ConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Check if WiFi is connected and has IP address
    pub fn is_connected(&self) -> bool {
        self.stack.is_link_up() && self.stack.config_v4().is_some()
    }

    /// Get current IP address (if connected)
    pub fn get_ip_address(&self) -> Option<embassy_net::Ipv4Address> {
        self.stack.config_v4().map(|config| config.address.address())
    }

    /// Wait for network to be ready (link up + DHCP)
    pub async fn wait_for_ready(&self) {
        self.stack.wait_config_up().await;
    }

    /// Get updated connection status
    pub async fn get_status(&mut self) -> Result<ConnectionInfo, WiFiError> {
        if !self.is_connected() {
            return Err(WiFiError::Connection("Not connected"));
        }
        
        let config = self.stack.config_v4()
            .ok_or(WiFiError::Dhcp("No IP configuration"))?;
            
        let connection_info = ConnectionInfo {
            ip_address: config.address.address(),
            gateway: config.gateway,
            dns_servers: config.dns_servers,
            subnet_prefix: config.address.prefix_len(),
        };
        
        self.connection_info = Some(connection_info.clone());
        Ok(connection_info)
    }
}

/// WiFi connection management task (internal)
#[embassy_executor::task]
async fn wifi_connection_task(
    mut controller: WifiController<'static>,
    ssid: &'static str,
    password: &'static str,
) {
    info!("WiFi Embassy: Starting connection task for SSID: {}", ssid);
    
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // Wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                warn!("WiFi Embassy: Disconnected from network");
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(esp_wifi::wifi::ClientConfiguration {
                ssid: ssid.try_into().unwrap(),
                password: password.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("WiFi Embassy: Starting WiFi...");
            controller.start_async().await.unwrap();
            info!("WiFi Embassy: WiFi started");
        }
        
        info!("WiFi Embassy: Connecting to '{}'...", ssid);
        match controller.connect_async().await {
            Ok(_) => {
                info!("WiFi Embassy: Connected successfully to '{}'", ssid);
            }
            Err(e) => {
                error!("WiFi Embassy: Connection failed: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

/// Network stack runner task (internal)
#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}