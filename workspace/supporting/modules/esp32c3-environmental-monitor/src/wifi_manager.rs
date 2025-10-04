//! Simple WiFi Manager implementation for standalone simple-iot module
//! 
//! This is a simplified WiFi manager based on the wifi-embassy module but
//! designed to be self-contained without workspace dependencies.

extern crate alloc;

use embassy_executor::Spawner;
use embassy_net::{Config as NetConfig, Stack, StackResources, Runner};
use embassy_time::{Duration, Timer};
use esp_hal::{
    peripherals::{TIMG0, WIFI, RNG},
    timer::timg::TimerGroup,
    rng::Rng,
};
use esp_wifi::{
    init,
    wifi::{WifiController, WifiDevice, WifiState},
    EspWifiController,
};
use heapless::String;
use rtt_target::rprintln;

/// WiFi network configuration
#[derive(Debug, Clone)]
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str,
}

/// WiFi connection information
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub ip_address: embassy_net::Ipv4Address,
    pub gateway: Option<embassy_net::Ipv4Address>,
    pub subnet_prefix: u8,
}

/// WiFi error types
#[derive(Debug)]
pub enum WiFiError {
    InitializationFailed(&'static str),
    ConnectionFailed(&'static str),
    NetworkError(&'static str),
}

/// Utility macro for creating static allocations
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// Simple WiFi Manager
pub struct WiFiManager {
    stack: &'static Stack<'static>,
    config: WiFiConfig,
    connection_info: Option<ConnectionInfo>,
}

impl WiFiManager {
    /// Create a new WiFi manager and establish connection
    pub async fn new(
        spawner: Spawner,
        timg0: TIMG0<'static>,
        wifi: WIFI<'static>,
        rng_peripheral: RNG<'static>,
        config: WiFiConfig,
    ) -> Result<Self, WiFiError> {
        rprintln!("[WiFi] Initializing WiFi manager");
        rprintln!("[WiFi] Target SSID: {}", config.ssid);

        // Initialize timers
        let timer_group0 = TimerGroup::new(timg0);
        
        // Initialize WiFi with proper RNG
        let mut rng = Rng::new(rng_peripheral);
        let esp_wifi_ctrl = mk_static!(
            EspWifiController,
            init(timer_group0.timer0, rng.clone())
                .map_err(|_| WiFiError::InitializationFailed("Failed to initialize esp-wifi"))?
        );
        rprintln!("[WiFi] WiFi hardware initialized");

        // Create WiFi controller and interfaces
        rprintln!("[WiFi] Creating WiFi controller...");
        let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, wifi)
            .map_err(|_| WiFiError::InitializationFailed("Failed to create WiFi controller"))?;
        let device = interfaces.sta;
        rprintln!("[WiFi] WiFi controller created");

        // Initialize Embassy network stack with static allocation
        let stack_resources = mk_static!(StackResources<3>, StackResources::<3>::new());
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;
        let (stack, runner) = embassy_net::new(
            device,
            NetConfig::dhcpv4(Default::default()),
            stack_resources,
            seed,
        );
        let stack = mk_static!(Stack<'static>, stack);
        
        rprintln!("[WiFi] Network stack created with DHCP");

        // Convert WiFi config to proper format
        let ssid_string: String<32> = String::try_from(config.ssid)
            .map_err(|_| WiFiError::InitializationFailed("Invalid SSID"))?;
        let password_string: String<64> = String::try_from(config.password)
            .map_err(|_| WiFiError::InitializationFailed("Invalid password"))?;

        // Spawn background tasks
        spawner.spawn(wifi_connection_task(controller, ssid_string.clone(), password_string.clone()))
            .map_err(|_| WiFiError::InitializationFailed("Failed to spawn WiFi task"))?;
        spawner.spawn(network_task(runner))
            .map_err(|_| WiFiError::InitializationFailed("Failed to spawn network task"))?;

        rprintln!("[WiFi] Background tasks started");

        // Wait for link up
        rprintln!("[WiFi] Waiting for WiFi connection...");
        let mut timeout_counter = 0;
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
            timeout_counter += 1;
            
            if timeout_counter > 60 {  // 30 second timeout
                return Err(WiFiError::ConnectionFailed("WiFi connection timeout"));
            }
            
            if timeout_counter % 10 == 0 {
                rprintln!("[WiFi] Still waiting for connection... ({} seconds)", timeout_counter / 2);
            }
        }
        rprintln!("[WiFi] WiFi link established");
        
        // Wait for IP address (DHCP)
        rprintln!("[WiFi] Waiting for DHCP IP address...");
        timeout_counter = 0;
        loop {
            if let Some(config_v4) = stack.config_v4() {
                let connection_info = ConnectionInfo {
                    ip_address: config_v4.address.address(),
                    gateway: config_v4.gateway,
                    subnet_prefix: config_v4.address.prefix_len(),
                };
                
                rprintln!("[WiFi] SUCCESS: Connected successfully!");
                rprintln!("[WiFi] IP address: {}", connection_info.ip_address);
                rprintln!("[WiFi] Gateway: {:?}", connection_info.gateway);
                
                return Ok(Self {
                    stack,
                    connection_info: Some(connection_info),
                    config,
                });
            }
            Timer::after(Duration::from_millis(500)).await;
            timeout_counter += 1;
            
            if timeout_counter > 60 {  // 30 second timeout
                return Err(WiFiError::ConnectionFailed("DHCP timeout"));
            }
            
            if timeout_counter % 10 == 0 {
                rprintln!("[WiFi] Still waiting for DHCP... ({} seconds)", timeout_counter / 2);
            }
        }
    }

    /// Get the network stack
    pub fn get_stack(&self) -> &'static Stack<'static> {
        self.stack
    }

    /// Check if WiFi is connected
    pub fn is_connected(&self) -> bool {
        self.stack.is_link_up() && self.stack.config_v4().is_some()
    }

    /// Get connection information
    pub fn get_connection_info(&self) -> Option<&ConnectionInfo> {
        self.connection_info.as_ref()
    }
}

#[embassy_executor::task]
async fn wifi_connection_task(
    mut controller: WifiController<'static>,
    ssid: String<32>,
    password: String<64>,
) {
    rprintln!("[WiFi] Starting connection task for SSID: {}", ssid);
    
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // Wait for disconnection
                rprintln!("[WiFi] Connected, monitoring for disconnection");
                let wait_result = controller.wait_for_event(esp_wifi::wifi::WifiEvent::StaDisconnected).await;
                rprintln!("[WiFi] Disconnected: {:?}", wait_result);
                Timer::after(Duration::from_millis(5000)).await;
            }
            _ => {
                if !matches!(controller.is_started(), Ok(true)) {
                    let client_config = esp_wifi::wifi::Configuration::Client(esp_wifi::wifi::ClientConfiguration {
                        ssid: alloc::string::String::from(ssid.as_str()),
                        password: alloc::string::String::from(password.as_str()),
                        ..Default::default()
                    });
                    controller.set_configuration(&client_config).unwrap();
                    rprintln!("[WiFi] Starting WiFi...");
                    controller.start_async().await.unwrap();
                    rprintln!("[WiFi] WiFi started");
                }
                
                rprintln!("[WiFi] Attempting to connect to {}...", ssid);
                match controller.connect_async().await {
                    Ok(_) => rprintln!("[WiFi] ✅ Connected successfully!"),
                    Err(e) => {
                        rprintln!("[WiFi] ❌ Connection attempt failed: {:?}", e);
                        Timer::after(Duration::from_millis(5000)).await;
                    }
                }
            }
        }
        Timer::after(Duration::from_millis(2000)).await;
    }
}

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}