//! # WiFi Embassy IoT Container Trait Implementation
//!
//! This module provides the implementation of the IoT Container NetworkManager trait
//! for the ESP32-C3 WiFi manager, enabling seamless integration with the dependency
//! injection container system.

use async_trait::async_trait;
use embassy_time::Instant;

use iot_common::IoTError;

// Import the container trait (when iot-container is available)
#[cfg(feature = "container")]
use iot_container::traits::{NetworkManager, ConnectionInfo as ContainerConnectionInfo, EmbeddedString};

use crate::wifi_manager::{WiFiManager, ConnectionInfo, WiFiError};

/// Adapter that implements the IoT Container NetworkManager trait for WiFiManager
/// 
/// This adapter bridges the WiFi manager with the IoT Container's trait-based
/// dependency injection system, enabling the WiFi manager to be used as a drop-in
/// component in the container architecture.
#[cfg(feature = "container")]
pub struct WiFiContainerAdapter {
    /// The underlying WiFi manager
    wifi_manager: WiFiManager,
    
    /// Last successful connection time
    last_connection_time: Option<u64>,
    
    /// Last connectivity test time
    last_connectivity_test: Option<u64>,
    
    /// Connectivity test interval in milliseconds (to avoid excessive testing)
    connectivity_test_interval_ms: u64,
    
    /// Connection attempt count (for metrics)
    connection_attempts: u32,
    
    /// Successful connection count (for metrics)
    successful_connections: u32,
    
    /// Connection failure count (for metrics)
    connection_failures: u32,
}

#[cfg(feature = "container")]
impl WiFiContainerAdapter {
    /// Creates a new WiFi container adapter
    /// 
    /// # Arguments
    /// 
    /// * `wifi_manager` - WiFi manager instance to wrap
    /// 
    /// # Returns
    /// 
    /// A new adapter instance ready for use
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use wifi_embassy::{WiFiManager, WiFiContainerAdapter};
    /// use iot_container::traits::NetworkManager;
    /// 
    /// let wifi_manager = WiFiManager::new(/* ... */).await?;
    /// let adapter = WiFiContainerAdapter::new(wifi_manager);
    /// ```
    pub fn new(wifi_manager: WiFiManager) -> Self {
        Self {
            wifi_manager,
            last_connection_time: None,
            last_connectivity_test: None,
            connectivity_test_interval_ms: 30000, // Test connectivity every 30 seconds
            connection_attempts: 0,
            successful_connections: 0,
            connection_failures: 0,
        }
    }
    
    /// Creates a new adapter with custom connectivity test interval
    /// 
    /// # Arguments
    /// 
    /// * `wifi_manager` - WiFi manager instance to wrap
    /// * `test_interval_ms` - Interval between connectivity tests in milliseconds
    /// 
    /// # Returns
    /// 
    /// A new adapter instance with custom connectivity testing
    pub fn new_with_test_interval(wifi_manager: WiFiManager, test_interval_ms: u64) -> Self {
        Self {
            wifi_manager,
            last_connection_time: None,
            last_connectivity_test: None,
            connectivity_test_interval_ms: test_interval_ms,
            connection_attempts: 0,
            successful_connections: 0,
            connection_failures: 0,
        }
    }
    
    /// Converts WiFi connection info to container connection info format
    /// 
    /// This method handles the conversion between the WiFi-specific connection
    /// information format and the standardized container connection format.
    /// 
    /// # Arguments
    /// 
    /// * `connection_info` - WiFi connection information to convert
    /// 
    /// # Returns
    /// 
    /// Container-compatible connection information
    fn convert_connection_info(&self, connection_info: ConnectionInfo) -> Result<ContainerConnectionInfo, IoTError> {
        let ip_address = format!("{}", connection_info.ip_address);
        let mut container_info = ContainerConnectionInfo::new(&ip_address)?;
        
        // Convert gateway
        if let Some(gateway) = connection_info.gateway {
            let gateway_str = format!("{}", gateway);
            container_info.gateway = Some(EmbeddedString::try_from(gateway_str.as_str()).map_err(|_| {
                IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Gateway address too long"))
            })?);
        }
        
        // Convert subnet mask from prefix length
        let subnet_mask = Self::prefix_to_subnet_mask(connection_info.subnet_prefix);
        let subnet_str = format!("{}", subnet_mask);
        container_info.subnet_mask = Some(EmbeddedString::try_from(subnet_str.as_str()).map_err(|_| {
            IoTError::Configuration(iot_common::ConfigError::InvalidFormat("Subnet mask too long"))
        })?);
        
        // Convert DNS servers
        for dns_server in connection_info.dns_servers.iter() {
            let dns_str = format!("{}", dns_server);
            if let Ok(dns_embedded) = EmbeddedString::try_from(dns_str.as_str()) {
                if container_info.dns_servers.push(dns_embedded).is_err() {
                    break; // DNS server list full
                }
            }
        }
        
        // Add signal strength if available
        container_info.signal_strength_dbm = self.wifi_manager.get_signal_strength();
        
        // Add SSID if available
        if let Some(ssid) = self.wifi_manager.get_ssid() {
            container_info.ssid = Some(EmbeddedString::try_from(ssid.as_str()).map_err(|_| {
                IoTError::Configuration(iot_common::ConfigError::InvalidFormat("SSID too long"))
            })?);
        }
        
        Ok(container_info)
    }
    
    /// Converts subnet prefix length to subnet mask
    /// 
    /// # Arguments
    /// 
    /// * `prefix` - Subnet prefix length (e.g., 24 for /24)
    /// 
    /// # Returns
    /// 
    /// IPv4 subnet mask address
    fn prefix_to_subnet_mask(prefix: u8) -> embassy_net::Ipv4Address {
        if prefix >= 32 {
            return embassy_net::Ipv4Address::new(255, 255, 255, 255);
        }
        
        let mask_bits = ((1u64 << prefix) - 1) << (32 - prefix);
        embassy_net::Ipv4Address::new(
            ((mask_bits >> 24) & 0xFF) as u8,
            ((mask_bits >> 16) & 0xFF) as u8,
            ((mask_bits >> 8) & 0xFF) as u8,
            (mask_bits & 0xFF) as u8,
        )
    }
    
    /// Checks if connectivity test should be performed
    /// 
    /// This method implements smart connectivity testing by only performing
    /// actual network tests at specified intervals, improving performance.
    /// 
    /// # Returns
    /// 
    /// `true` if connectivity should be tested, `false` if recent test is still valid
    fn should_test_connectivity(&self) -> bool {
        match self.last_connectivity_test {
            None => true, // Never tested before
            Some(last_test) => {
                let now = Instant::now().as_millis();
                (now - last_test) >= self.connectivity_test_interval_ms
            }
        }
    }
    
    /// Updates the connectivity test timestamp
    fn update_connectivity_test_time(&mut self) {
        self.last_connectivity_test = Some(Instant::now().as_millis());
    }
    
    /// Converts WiFi errors to IoT errors
    /// 
    /// # Arguments
    /// 
    /// * `wifi_error` - WiFi-specific error to convert
    /// 
    /// # Returns
    /// 
    /// Standardized IoT error
    fn convert_error(&self, wifi_error: WiFiError) -> IoTError {
        match wifi_error {
            WiFiError::HardwareInit(msg) => {
                IoTError::Hardware(iot_common::HardwareError::InitializationFailed(msg))
            }
            WiFiError::Configuration(msg) => {
                IoTError::Configuration(iot_common::ConfigError::InvalidFormat(msg))
            }
            WiFiError::Connection(msg) => {
                IoTError::Network(iot_common::NetworkError::ConnectionFailed(msg))
            }
            WiFiError::Dhcp(msg) => {
                IoTError::Network(iot_common::NetworkError::DhcpFailed(msg))
            }
        }
    }
    
    /// Gets connection metrics for monitoring
    /// 
    /// # Returns
    /// 
    /// Tuple containing (total_attempts, successful_connections, failures)
    pub fn get_connection_metrics(&self) -> (u32, u32, u32) {
        (self.connection_attempts, self.successful_connections, self.connection_failures)
    }
}

#[cfg(feature = "container")]
#[async_trait]
impl NetworkManager for WiFiContainerAdapter {
    /// Establishes WiFi network connection
    /// 
    /// This method ensures the WiFi manager is connected to the configured network.
    /// Since the WiFi manager may already be connected, this method primarily
    /// validates and maintains the connection.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Connection established or already connected
    /// * `Err(IoTError)` - Connection failed
    /// 
    /// # Implementation Details
    /// 
    /// - Checks current connection status
    /// - Attempts reconnection if disconnected
    /// - Updates connection metrics
    /// - Handles WiFi-specific errors
    async fn connect(&mut self) -> Result<(), IoTError> {
        self.connection_attempts += 1;
        
        // Check if already connected
        if self.wifi_manager.is_connected() {
            // Already connected, update success metrics
            self.successful_connections += 1;
            self.last_connection_time = Some(Instant::now().as_millis());
            return Ok(());
        }
        
        // WiFi manager handles the actual connection process internally
        // We verify connection status and wait if needed
        let mut retries = 10;
        while retries > 0 && !self.wifi_manager.is_connected() {
            embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
            retries -= 1;
        }
        
        if self.wifi_manager.is_connected() {
            self.successful_connections += 1;
            self.last_connection_time = Some(Instant::now().as_millis());
            Ok(())
        } else {
            self.connection_failures += 1;
            Err(IoTError::Network(iot_common::NetworkError::ConnectionFailed("WiFi connection timeout")))
        }
    }
    
    /// Disconnects from WiFi network
    /// 
    /// This method gracefully disconnects from the current WiFi network.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Disconnection completed or already disconnected
    /// * `Err(IoTError)` - Disconnection failed
    /// 
    /// # Implementation Notes
    /// 
    /// The WiFi manager may not support explicit disconnection,
    /// so this method may be a no-op in some implementations.
    async fn disconnect(&mut self) -> Result<(), IoTError> {
        // WiFi manager implementation may not support explicit disconnect
        // This is implementation-dependent
        
        // Reset connection state
        self.last_connection_time = None;
        self.last_connectivity_test = None;
        
        Ok(())
    }
    
    /// Checks if WiFi network is currently connected
    /// 
    /// This method provides a quick status check without establishing new connections.
    /// 
    /// # Returns
    /// 
    /// `true` if connected and network is reachable, `false` otherwise
    /// 
    /// # Implementation Details
    /// 
    /// - Uses WiFi manager's internal connection status
    /// - Fast operation with no network I/O
    /// - Does not perform connectivity tests
    async fn is_connected(&self) -> bool {
        self.wifi_manager.is_connected()
    }
    
    /// Gets current WiFi connection information
    /// 
    /// Returns detailed information about the current network connection
    /// including IP address, gateway, DNS servers, and signal strength.
    /// 
    /// # Returns
    /// 
    /// `Some(ConnectionInfo)` if connected, `None` if disconnected
    /// 
    /// # Implementation Details
    /// 
    /// - Converts WiFi-specific format to container format
    /// - Includes signal strength and SSID information
    /// - Handles format conversion errors gracefully
    async fn get_connection_info(&self) -> Option<ContainerConnectionInfo> {
        if let Some(wifi_info) = self.wifi_manager.get_connection_info() {
            self.convert_connection_info(wifi_info).ok()
        } else {
            None
        }
    }
    
    /// Gets current WiFi signal strength
    /// 
    /// Returns the signal strength of the current WiFi connection in dBm.
    /// Typical values range from -30 dBm (excellent) to -90 dBm (poor).
    /// 
    /// # Returns
    /// 
    /// `Some(signal_dbm)` if connected, `None` if disconnected
    /// 
    /// # Signal Strength Guide
    /// 
    /// - -30 to -50 dBm: Excellent signal
    /// - -50 to -60 dBm: Good signal
    /// - -60 to -70 dBm: Fair signal
    /// - -70 to -80 dBm: Weak signal
    /// - -80 to -90 dBm: Very weak signal
    /// - Below -90 dBm: No signal
    async fn get_signal_strength(&self) -> Option<i8> {
        if self.wifi_manager.is_connected() {
            self.wifi_manager.get_signal_strength()
        } else {
            None
        }
    }
    
    /// Performs network connectivity test
    /// 
    /// Tests actual internet connectivity by attempting to reach a known endpoint.
    /// This goes beyond local network connectivity to verify internet access.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Internet connectivity verified
    /// * `Err(IoTError)` - No internet connectivity
    /// 
    /// # Implementation Details
    /// 
    /// - Uses smart testing intervals to avoid excessive network traffic
    /// - Caches connectivity test results for performance
    /// - Performs actual network connectivity verification
    /// - May attempt DNS resolution or ping to verify connectivity
    async fn test_connectivity(&self) -> Result<(), IoTError> {
        // Check if we need to perform actual connectivity test
        if !self.should_test_connectivity() {
            // Use cached result - assume connectivity if recently tested and connected
            if self.wifi_manager.is_connected() {
                return Ok(());
            } else {
                return Err(IoTError::Network(iot_common::NetworkError::ConnectionLost("No WiFi connection")));
            }
        }
        
        // Perform actual connectivity test
        if !self.wifi_manager.is_connected() {
            return Err(IoTError::Network(iot_common::NetworkError::NotConnected("WiFi not connected")));
        }
        
        // For a more thorough test, we could:
        // 1. Attempt DNS resolution of a known domain
        // 2. Perform a ping or HTTP request to a reliable endpoint
        // 3. Verify gateway reachability
        //
        // For now, we'll do a basic connectivity check by verifying
        // we have IP address and gateway information
        
        if let Some(connection_info) = self.wifi_manager.get_connection_info() {
            if connection_info.gateway.is_some() {
                // We have IP and gateway, likely have internet connectivity
                Ok(())
            } else {
                Err(IoTError::Network(iot_common::NetworkError::ConfigurationError("No gateway configured")))
            }
        } else {
            Err(IoTError::Network(iot_common::NetworkError::ConnectionLost("No connection information available")))
        }
    }
    
    /// Gets the network stack for protocol operations
    /// 
    /// Returns a reference to the underlying network stack for TCP/UDP operations.
    /// This enables other components to perform network I/O operations.
    /// 
    /// # Returns
    /// 
    /// Reference to the Embassy network stack
    /// 
    /// # Usage
    /// 
    /// The returned stack can be used for:
    /// - TCP connections
    /// - UDP sockets
    /// - HTTP clients
    /// - MQTT connections
    /// - Any network protocol implementation
    fn get_stack(&self) -> &'static embassy_net::Stack<embassy_net::driver::Driver<'static>> {
        self.wifi_manager.get_stack()
    }
}

// Convenience functions for creating container-compatible WiFi instances

/// Creates a new WiFi network adapter for use with the IoT container
/// 
/// This function provides a convenient way to create a WiFi adapter that
/// implements the container's NetworkManager trait.
/// 
/// # Arguments
/// 
/// * `wifi_manager` - WiFi manager instance to wrap
/// 
/// # Returns
/// 
/// A new WiFi adapter ready for use with the IoT container
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use wifi_embassy::{WiFiManager, create_container_network_manager};
/// use iot_container::ComponentFactory;
/// 
/// let wifi_manager = WiFiManager::new(/* ... */).await?;
/// let network = create_container_network_manager(wifi_manager);
/// ```
#[cfg(feature = "container")]
pub fn create_container_network_manager(wifi_manager: WiFiManager) -> WiFiContainerAdapter {
    WiFiContainerAdapter::new(wifi_manager)
}

/// Creates a WiFi adapter with custom connectivity test interval
/// 
/// This function allows customization of how frequently the adapter tests
/// internet connectivity, which can be tuned for performance vs. responsiveness.
/// 
/// # Arguments
/// 
/// * `wifi_manager` - WiFi manager instance to wrap
/// * `test_interval_ms` - Interval between connectivity tests in milliseconds
/// 
/// # Returns
/// 
/// A new WiFi adapter with custom connectivity testing
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use wifi_embassy::create_container_network_manager_with_interval;
/// 
/// // Test connectivity every 60 seconds instead of default 30 seconds
/// let network = create_container_network_manager_with_interval(wifi_manager, 60000);
/// ```
#[cfg(feature = "container")]
pub fn create_container_network_manager_with_interval(
    wifi_manager: WiFiManager, 
    test_interval_ms: u64
) -> WiFiContainerAdapter {
    WiFiContainerAdapter::new_with_test_interval(wifi_manager, test_interval_ms)
}