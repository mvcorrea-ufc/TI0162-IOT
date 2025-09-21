//! WiFi Manager implementation using Embassy async framework
//! 
//! Provides robust WiFi connectivity with automatic reconnection and DHCP support.
//! Based on proven examples from the workspace and following the bme280-embassy pattern.

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
    wifi::{Configuration, ClientConfiguration, WifiController, WifiDevice, WifiState, WifiEvent},
    EspWifiController,
};
use rtt_target::rprintln;

/// Utility macro for creating static allocations (from working examples)
#[macro_export]
macro_rules! mk_static {
    ($t:ty, $val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

/// WiFi network configuration
/// 
/// Contains the necessary parameters for connecting to a WiFi network.
/// The strings must be static to ensure they remain valid for the lifetime
/// of the WiFi connection.
/// 
/// # Examples
/// 
/// ```rust
/// use wifi_embassy::WiFiConfig;
/// 
/// // Using environment variables (recommended)
/// let config = WiFiConfig {
///     ssid: env!("WIFI_SSID"),
///     password: env!("WIFI_PASSWORD"),
/// };
/// 
/// // Using static strings
/// let config = WiFiConfig {
///     ssid: "MyNetwork",
///     password: "MyPassword",
/// };
/// ```
/// 
/// # Security Notes
/// 
/// - Passwords should be loaded from environment variables, not hardcoded
/// - Supports WPA2-Personal and WPA3-Personal security
/// - SSID and password are case-sensitive
#[derive(Debug, Clone)]
pub struct WiFiConfig {
    /// Network SSID (Service Set Identifier)
    /// 
    /// Must be the exact network name as broadcast by the access point.
    /// Maximum length: 32 characters (IEEE 802.11 standard).
    pub ssid: heapless::String<32>,
    
    /// Network password/passphrase
    /// 
    /// For WPA2/WPA3 networks, this is the network passphrase.
    /// Minimum length: 8 characters, Maximum length: 63 characters.
    pub password: heapless::String<64>,
}

/// WiFi connection status and network information
/// 
/// Contains detailed information about the current WiFi connection,
/// including IP configuration obtained through DHCP.
/// 
/// # Examples
/// 
/// ```rust
/// # use wifi_embassy::{WiFiManager, ConnectionInfo};
/// # let wifi_manager: WiFiManager = unimplemented!();
/// if let Some(info) = wifi_manager.get_connection_info() {
///     println!("IP: {}", info.ip_address);
///     println!("Gateway: {:?}", info.gateway);
///     println!("Subnet: /{}", info.subnet_prefix);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// IP address assigned to this device via DHCP
    /// 
    /// This is the IPv4 address that other devices can use to
    /// communicate with this ESP32-C3 device.
    pub ip_address: embassy_net::Ipv4Address,
    
    /// Gateway IP address (typically the router)
    /// 
    /// This is the IP address of the router/gateway that provides
    /// internet access. `None` if no gateway is configured.
    pub gateway: Option<embassy_net::Ipv4Address>,
    
    /// DNS server addresses
    /// 
    /// List of DNS servers provided by DHCP, used for domain
    /// name resolution. Maximum of 3 servers supported.
    pub dns_servers: heapless::Vec<embassy_net::Ipv4Address, 3>,
    
    /// Subnet prefix length (e.g., 24 for /24 subnet)
    /// 
    /// Indicates the size of the local network subnet.
    /// Common values: 24 (/24 = 255.255.255.0), 16 (/16 = 255.255.0.0)
    pub subnet_prefix: u8,
}

/// WiFi-specific error types
/// 
/// These errors provide specific context for different types of WiFi failures,
/// making it easier to diagnose and handle connection issues.
/// 
/// # Error Categories
/// 
/// - **Hardware**: Issues with ESP32-C3 WiFi hardware or drivers
/// - **Configuration**: Invalid network settings or credentials
/// - **Connection**: Network communication failures
/// - **DHCP**: IP address assignment failures
/// 
/// # Examples
/// 
/// ```rust
/// use wifi_embassy::{WiFiManager, WiFiError};
/// 
/// match WiFiManager::new(/* ... */).await {
///     Err(WiFiError::Configuration(msg)) => {
///         println!("Check WiFi credentials: {}", msg);
///     }
///     Err(WiFiError::Connection(msg)) => {
///         println!("Network connection failed: {}", msg);
///     }
///     Ok(manager) => {
///         println!("WiFi connected successfully");
///     }
///     Err(e) => {
///         println!("WiFi error: {}", e);
///     }
/// }
/// ```
#[derive(Debug)]
pub enum WiFiError {
    /// Hardware initialization failed
    /// 
    /// Indicates problems with:
    /// - ESP32-C3 WiFi hardware
    /// - Driver initialization
    /// - Memory allocation for WiFi stack
    /// - Timer or RNG peripheral issues
    HardwareInit(&'static str),
    
    /// WiFi configuration failed
    /// 
    /// Indicates problems with:
    /// - Invalid SSID or password
    /// - Unsupported security type
    /// - Network configuration errors
    /// - Embassy initialization issues
    Configuration(&'static str),
    
    /// Connection to WiFi network failed
    /// 
    /// Indicates problems with:
    /// - Network unreachable or weak signal
    /// - Authentication failure (wrong password)
    /// - Network capacity (too many connected devices)
    /// - Incompatible network settings
    Connection(&'static str),
    
    /// DHCP IP address assignment failed
    /// 
    /// Indicates problems with:
    /// - DHCP server not responding
    /// - No available IP addresses in DHCP pool
    /// - Network configuration conflicts
    /// - Router DHCP service disabled
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

/// WiFi connectivity manager using Embassy async framework
/// 
/// The `WiFiManager` handles all aspects of WiFi connectivity including:
/// - Hardware initialization and configuration
/// - Network connection establishment
/// - DHCP IP address acquisition
/// - Connection monitoring and status reporting
/// - Network stack management for TCP/UDP operations
/// 
/// # Lifecycle
/// 
/// 1. **Initialization**: Hardware setup and Embassy integration
/// 2. **Connection**: WiFi network authentication and association
/// 3. **DHCP**: IP address acquisition from router
/// 4. **Ready**: Network stack available for applications
/// 5. **Monitoring**: Continuous connection health monitoring
/// 
/// # Examples
/// 
/// ```no_run
/// use wifi_embassy::{WiFiManager, WiFiConfig};
/// use embassy_executor::Spawner;
/// 
/// #[esp_hal_embassy::main]
/// async fn main(spawner: Spawner) {
///     let peripherals = esp_hal::init(esp_hal::Config::default());
///     esp_hal_embassy::init(peripherals.TIMG1);
///     
///     let config = WiFiConfig {
///         ssid: env!("WIFI_SSID"),
///         password: env!("WIFI_PASSWORD"),
///     };
///     
///     let manager = WiFiManager::new(
///         spawner,
///         peripherals.TIMG0,
///         peripherals.WIFI,
///         peripherals.RNG,
///         config,
///     ).await.unwrap();
///     
///     // WiFi is connected and ready
///     let stack = manager.get_stack();
///     
///     // Use stack for network operations
/// }
/// ```
/// 
/// # Network Stack Access
/// 
/// The `WiFiManager` provides access to the Embassy network stack through
/// `get_stack()`, which can be used for:
/// 
/// - TCP socket operations
/// - UDP communications  
/// - DNS resolution
/// - MQTT client connections
/// - HTTP client requests
/// 
/// # Thread Safety
/// 
/// The `WiFiManager` is designed to be used from a single async task.
/// The network stack it provides can be shared across multiple tasks safely.
pub struct WiFiManager {
    /// Embassy network stack for TCP/UDP operations
    stack: &'static Stack<'static>,
    /// Current connection information (None if disconnected)
    connection_info: Option<ConnectionInfo>,
    /// WiFi network configuration
    config: WiFiConfig,
}

impl WiFiManager {
    /// Create a new WiFi manager
    /// 
    /// Based on the working examples from wifi-simple-embassy and wifi-simple-must-working.
    /// Follows the bme280-embassy pattern for hardware initialization.
    /// 
    /// # Arguments
    /// * `spawner` - Embassy task spawner
    /// * `timg0` - Timer group 0 peripheral (for WiFi)
    /// * `wifi` - WiFi peripheral
    /// * `rng_peripheral` - Random number generator peripheral
    /// * `config` - WiFi configuration (SSID and password)
    /// 
    /// # Note
    /// Embassy must be initialized before calling this function using esp_hal_embassy::init()
    pub async fn new(
        spawner: Spawner,
        timg0: TIMG0<'static>,
        wifi: WIFI<'static>,
        rng_peripheral: RNG<'static>,
        config: WiFiConfig,
    ) -> Result<Self, WiFiError> {
        rprintln!("[WIFI] Initializing WiFi manager");
        rprintln!("[WIFI] Target SSID: {}", config.ssid);

        // Initialize timers (following working examples)
        let timer_group0 = TimerGroup::new(timg0);
        
        // NOTE: Embassy must be initialized BEFORE calling this function
        // The caller should call esp_hal_embassy::init() before creating WiFiManager
        rprintln!("[WIFI] Using existing Embassy time driver");
        
        // Initialize WiFi with proper RNG (from working examples)
        let mut rng = Rng::new(rng_peripheral);
        let esp_wifi_ctrl = mk_static!(
            EspWifiController,
            init(timer_group0.timer0, rng.clone())
                .map_err(|_| WiFiError::HardwareInit("Failed to initialize esp-wifi"))?
        );
        rprintln!("[WIFI] WiFi hardware initialized");

        // Add a small delay to ensure WiFi peripheral is ready after restart
        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;

        // Create WiFi controller and interfaces (from working examples)
        rprintln!("[WIFI] Creating WiFi controller...");
        let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, wifi)
            .map_err(|e| {
                rprintln!("[WIFI] WiFi controller creation failed: {:?}", e);
                WiFiError::HardwareInit("Failed to create WiFi controller")
            })?;
        let device = interfaces.sta;
        rprintln!("[WIFI] WiFi controller created");

        // Initialize Embassy network stack with static allocation (from working examples)
        let stack_resources = mk_static!(StackResources<3>, StackResources::<3>::new());
        let seed = (rng.random() as u64) << 32 | rng.random() as u64;
        let (stack, runner) = embassy_net::new(
            device,
            NetConfig::dhcpv4(Default::default()),
            stack_resources,
            seed,
        );
        let stack = mk_static!(Stack<'static>, stack);
        
        rprintln!("[WIFI] Network stack created with DHCP");

        // Spawn background tasks (from working examples)
        spawner.spawn(wifi_connection_task(controller, config.ssid.clone(), config.password.clone()))
            .map_err(|_| WiFiError::Configuration("Failed to spawn WiFi task"))?;
        spawner.spawn(network_task(runner))
            .map_err(|_| WiFiError::Configuration("Failed to spawn network task"))?;

        rprintln!("[WIFI] Background tasks started");

        // Wait for link up (following examples timeout pattern)
        rprintln!("[WIFI] Waiting for WiFi connection...");
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
                rprintln!("[WIFI] Still waiting for connection... ({} seconds)", timeout_counter / 2);
            }
        }
        rprintln!("[WIFI] WiFi link established");
        
        // Wait for IP address (DHCP)
        rprintln!("[WIFI] Waiting for DHCP IP address...");
        timeout_counter = 0;
        loop {
            if let Some(config_v4) = stack.config_v4() {
                let connection_info = ConnectionInfo {
                    ip_address: config_v4.address.address(),
                    gateway: config_v4.gateway,
                    dns_servers: config_v4.dns_servers,
                    subnet_prefix: config_v4.address.prefix_len(),
                };
                
                rprintln!("[WIFI] SUCCESS: Connected successfully!");
                rprintln!("[WIFI] IP address: {}", connection_info.ip_address);
                rprintln!("[WIFI] Gateway: {:?}", connection_info.gateway);
                rprintln!("[WIFI] Device is now pingable!");
                
                return Ok(Self {
                    stack,
                    connection_info: Some(connection_info),
                    config,
                });
            }
            Timer::after(Duration::from_millis(500)).await;
            timeout_counter += 1;
            
            if timeout_counter > 60 {  // 30 second timeout
                return Err(WiFiError::Dhcp("DHCP timeout"));
            }
            
            if timeout_counter % 10 == 0 {
                rprintln!("[WIFI] Still waiting for DHCP... ({} seconds)", timeout_counter / 2);
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

    /// Get WiFi configuration
    pub fn get_config(&self) -> &WiFiConfig {
        &self.config
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

/// WiFi connection management task (from working examples)
#[embassy_executor::task]
async fn wifi_connection_task(
    mut controller: WifiController<'static>,
    ssid: heapless::String<32>,
    password: heapless::String<64>,
) {
    rprintln!("[WIFI] Starting connection task for SSID: {}", ssid);
    
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                // Wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                rprintln!("[WIFI] Disconnected from network");
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: alloc::string::String::from(ssid.as_str()),
                password: alloc::string::String::from(password.as_str()),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            rprintln!("[WIFI] Starting WiFi...");
            controller.start_async().await.unwrap();
            rprintln!("[WIFI] WiFi started");
        }
        
        rprintln!("[WIFI] Connecting to '{}'...", ssid);
        match controller.connect_async().await {
            Ok(_) => {
                rprintln!("[WIFI] Connected successfully to '{}'", ssid);
            }
            Err(e) => {
                rprintln!("[WIFI] Connection failed: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await;
            }
        }
    }
}

/// Network stack runner task (from working examples)
#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, WifiDevice<'static>>) -> ! {
    runner.run().await
}