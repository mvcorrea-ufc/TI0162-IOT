//! # Component Factory for IoT Container
//!
//! This module provides factory methods for creating system components based on configuration.
//! The factory pattern enables configuration-driven component selection and facilitates
//! testing with mock implementations.

use alloc::boxed::Box;
use iot_common::{IoTError, IoTResult};
use iot_hal::HardwarePlatform;

use crate::traits::{SensorReader, NetworkManager, MessagePublisher, ConsoleInterface};
use crate::config::{SensorConfig, WiFiConfig, MqttConfig, ConsoleConfig};

#[cfg(feature = "mock")]
use crate::mocks::{MockSensorReader, MockNetworkManager, MockMessagePublisher, MockConsoleInterface};

/// Component factory for creating system components
/// 
/// This factory creates components based on configuration settings, enabling
/// different implementations to be selected at runtime. In production, it creates
/// real hardware-backed components. In testing, it can create mock implementations.
pub struct ComponentFactory;

impl ComponentFactory {
    /// Creates a sensor reader based on configuration
    /// 
    /// This method examines the sensor configuration and creates the appropriate
    /// sensor implementation. Currently supports BME280 sensors with plans for
    /// additional sensor types.
    /// 
    /// # Arguments
    /// 
    /// * `platform` - Hardware platform providing I2C interface
    /// * `config` - Sensor configuration specifying type and parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn SensorReader>)` - Sensor reader implementation
    /// * `Err(IoTError)` - Sensor creation failed
    /// 
    /// # Supported Sensor Types
    /// 
    /// - `"BME280"` - Bosch BME280 environmental sensor
    /// - `"MOCK"` - Mock sensor for testing (requires `mock` feature)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use iot_container::{ComponentFactory, SensorConfig};
    /// use iot_hal::Esp32C3Platform;
    /// 
    /// let mut platform = Esp32C3Platform::initialize().await?;
    /// let sensor_config = SensorConfig::default();
    /// let sensor = ComponentFactory::create_sensor(&mut platform, &sensor_config).await?;
    /// ```
    pub async fn create_sensor<P: HardwarePlatform>(
        _platform: &mut P,
        config: &SensorConfig,
    ) -> IoTResult<Box<dyn SensorReader + Send + Sync>> {
        match config.sensor_type.as_str() {
            "BME280" => {
                // NOTE: Direct BME280 creation disabled due to circular dependency
                // Concrete sensors should be created externally and injected
                // #[cfg(feature = "esp32c3")]
                // {
                //     use bme280_embassy::BME280;
                //     
                //     let i2c = platform.get_i2c();
                //     let mut sensor = BME280::new(i2c);
                //     
                //     // Configure sensor based on config parameters
                //     // Note: BME280 driver may need updates to accept configuration parameters
                //     sensor.initialize().await?;
                //     
                //     Ok(Box::new(SensorAdapter::new(sensor)))
                // }
                // 
                // #[cfg(not(feature = "esp32c3"))]
                Err(IoTError::configuration(
                    iot_common::ConfigError::ValidationError("BME280 sensor must be injected externally".try_into().unwrap_or_default())
                ))
            }
            
            #[cfg(feature = "mock")]
            "MOCK" => {
                let sensor = MockSensorReader::new_with_config(config);
                Ok(Box::new(sensor))
            }
            
            _ => Err(IoTError::configuration(
                iot_common::ConfigError::InvalidParameter("Unsupported sensor type".try_into().unwrap_or_default())
            )),
        }
    }
    
    /// Creates a network manager based on configuration
    /// 
    /// This method creates the appropriate network manager implementation based on
    /// the WiFi configuration. Currently supports ESP32-C3 WiFi with plans for
    /// additional network interfaces.
    /// 
    /// # Arguments
    /// 
    /// * `platform` - Hardware platform providing WiFi interface
    /// * `config` - WiFi configuration specifying network parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn NetworkManager>)` - Network manager implementation
    /// * `Err(IoTError)` - Network manager creation failed
    /// 
    /// # Supported Network Types
    /// 
    /// - ESP32-C3 WiFi (default)
    /// - Mock network for testing (requires `mock` feature)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// let mut platform = Esp32C3Platform::initialize().await?;
    /// let wifi_config = WiFiConfig::default();
    /// let network = ComponentFactory::create_network_manager(&mut platform, &wifi_config).await?;
    /// ```
    pub async fn create_network_manager<P: HardwarePlatform>(
        _platform: &mut P,
        _config: &WiFiConfig,
    ) -> IoTResult<Box<dyn NetworkManager + Send + Sync>> {
        // NOTE: Direct WiFi creation disabled due to circular dependency
        // Concrete network managers should be created externally and injected
        // #[cfg(feature = "esp32c3")]
        // {
        //     use wifi_embassy::WiFiManager;
        //     
        //     // Create WiFi configuration from our config
        //     let wifi_config = wifi_embassy::WiFiConfig {
        //         ssid: config.ssid.as_str(),
        //         password: config.password.as_str(),
        //     };
        //     
        //     // Get required hardware components from platform
        //     let wifi_hw = platform.get_wifi();
        //     let timer = platform.get_timer();
        //     let rng = platform.get_rng();
        //     let spawner = platform.get_spawner();
        //     
        //     let wifi_manager = WiFiManager::new(spawner, timer, wifi_hw, rng, wifi_config).await?;
        //     
        //     Ok(Box::new(NetworkAdapter::new(wifi_manager)))
        // }
        #[cfg(feature = "esp32c3")]
        {
            Err(IoTError::configuration(
                iot_common::ConfigError::ValidationError("WiFi manager must be injected externally".try_into().unwrap_or_default())
            ))
        }
        
        #[cfg(feature = "mock")]
        {
            let network = MockNetworkManager::new_with_config(config);
            Ok(Box::new(network))
        }
        
        #[cfg(not(any(feature = "esp32c3", feature = "mock")))]
        Err(IoTError::configuration(
            iot_common::ConfigError::UnsupportedFeature("No network implementation available")
        ))
    }
    
    /// Creates a message publisher based on configuration
    /// 
    /// This method creates the appropriate message publisher implementation based on
    /// the MQTT configuration. Currently supports MQTT over TCP with plans for
    /// additional messaging protocols.
    /// 
    /// # Arguments
    /// 
    /// * `network` - Network manager providing connectivity
    /// * `config` - MQTT configuration specifying broker and parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn MessagePublisher>)` - Message publisher implementation
    /// * `Err(IoTError)` - Message publisher creation failed
    /// 
    /// # Supported Publisher Types
    /// 
    /// - MQTT over TCP (default)
    /// - Mock publisher for testing (requires `mock` feature)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// let mqtt_config = MqttConfig::default();
    /// let publisher = ComponentFactory::create_message_publisher(&network, &mqtt_config).await?;
    /// ```
    pub async fn create_message_publisher(
        _network: &dyn NetworkManager,
        _config: &MqttConfig,
    ) -> IoTResult<Box<dyn MessagePublisher + Send + Sync>> {
        // NOTE: Direct MQTT creation disabled due to circular dependency  
        // Concrete message publishers should be created externally and injected
        // #[cfg(feature = "esp32c3")]
        // {
        //     use mqtt_embassy::{MqttClient, MqttConfig as MqttClientConfig};
        //     
        //     // Create MQTT configuration from our config
        //     let mqtt_config = MqttClientConfig {
        //         broker_ip: config.broker_host.as_str(),
        //         broker_port: config.broker_port,
        //         client_id: config.client_id.as_str(),
        //         username: config.username.as_ref().map(|s| s.as_str()),
        //         password: config.password.as_ref().map(|s| s.as_str()),
        //         keep_alive_secs: config.keep_alive_secs,
        //         qos_level: config.qos_level,
        //     };
        //     
        //     let mqtt_client = MqttClient::new(mqtt_config);
        //     
        //     Ok(Box::new(PublisherAdapter::new(mqtt_client, config.topic_prefix.clone())))
        // }
        #[cfg(feature = "esp32c3")]
        {
            Err(IoTError::configuration(
                iot_common::ConfigError::ValidationError("Message publisher must be injected externally".try_into().unwrap_or_default())
            ))
        }
        
        #[cfg(feature = "mock")]
        {
            let publisher = MockMessagePublisher::new_with_config(config);
            Ok(Box::new(publisher))
        }
        
        #[cfg(not(any(feature = "esp32c3", feature = "mock")))]
        Err(IoTError::configuration(
            iot_common::ConfigError::UnsupportedFeature("No message publisher implementation available")
        ))
    }
    
    /// Creates a console interface based on configuration
    /// 
    /// This method creates the appropriate console interface implementation based on
    /// the console configuration. Currently supports USB Serial/JTAG with plans for
    /// additional console interfaces.
    /// 
    /// # Arguments
    /// 
    /// * `platform` - Hardware platform providing console interface
    /// * `config` - Console configuration specifying interface and parameters
    /// 
    /// # Returns
    /// 
    /// * `Ok(Box<dyn ConsoleInterface>)` - Console interface implementation
    /// * `Err(IoTError)` - Console interface creation failed
    /// 
    /// # Supported Console Types
    /// 
    /// - `"USB"` - USB Serial/JTAG interface (ESP32-C3)
    /// - `"UART"` - UART serial interface
    /// - `"MOCK"` - Mock console for testing (requires `mock` feature)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// let mut platform = Esp32C3Platform::initialize().await?;
    /// let console_config = ConsoleConfig::default();
    /// let console = ComponentFactory::create_console(&mut platform, &console_config).await?;
    /// ```
    pub async fn create_console<P: HardwarePlatform>(
        _platform: &mut P,
        config: &ConsoleConfig,
    ) -> IoTResult<Box<dyn ConsoleInterface + Send + Sync>> {
        match config.interface_type.as_str() {
            "USB" => {
                // NOTE: Console adapter disabled due to circular dependency
                // Console interfaces should be created externally and injected
                Err(IoTError::configuration(
                    iot_common::ConfigError::ValidationError("Console interface must be injected externally".try_into().unwrap_or_default())
                ))
            }
            
            "UART" => {
                // NOTE: Console adapter disabled due to circular dependency  
                // Console interfaces should be created externally and injected
                Err(IoTError::configuration(
                    iot_common::ConfigError::ValidationError("Console interface must be injected externally".try_into().unwrap_or_default())
                ))
            }
            
            #[cfg(feature = "mock")]
            "MOCK" => {
                let console = MockConsoleInterface::new_with_config(config);
                Ok(Box::new(console))
            }
            
            _ => Err(IoTError::configuration(
                iot_common::ConfigError::ValidationError("Unsupported console interface type".try_into().unwrap_or_default())
            )),
        }
    }
}

// ============================================================================
// ADAPTER IMPLEMENTATIONS DISABLED DUE TO CIRCULAR DEPENDENCY
// 
// The following adapter implementations are commented out to avoid circular
// dependency issues. In the dependency injection pattern, concrete implementations
// should be created externally and injected via the constructor rather than
// created by the factory.
// 
// This approach allows:
// 1. Breaking circular dependencies between iot-container and driver modules
// 2. More flexible dependency injection where implementations can be swapped
// 3. Better testability with easy mock injection
// 4. Cleaner separation of concerns
// ============================================================================

/*
// Adapter implementations to bridge concrete types to trait objects
// These adapters will be implemented when updating the concrete modules

/// Adapter for BME280 sensor to implement SensorReader trait
/// NOTE: Disabled due to circular dependency - sensors should be injected externally
// #[cfg(feature = "esp32c3")]
// pub struct SensorAdapter<I2C> {
//     sensor: bme280_embassy::BME280<I2C>,
//     last_measurement_time: Option<u64>,
// }

#[cfg(feature = "esp32c3")]
impl<I2C> SensorAdapter<I2C>
where
    I2C: iot_hal::I2cInterface,
{
    pub fn new(sensor: bme280_embassy::BME280<I2C>) -> Self {
        Self {
            sensor,
            last_measurement_time: None,
        }
    }
}

#[cfg(feature = "esp32c3")]
#[async_trait::async_trait]
impl<I2C> SensorReader for SensorAdapter<I2C>
where
    I2C: iot_hal::I2cInterface + Send + Sync,
{
    async fn read_measurements(&mut self) -> Result<crate::traits::Measurements, IoTError> {
        let measurements = self.sensor.read_measurements().await?;
        let timestamp = embassy_time::Instant::now().as_millis();
        
        self.last_measurement_time = Some(timestamp);
        
        Ok(crate::traits::Measurements {
            temperature: measurements.temperature,
            pressure: measurements.pressure,
            humidity: measurements.humidity,
            timestamp_ms: timestamp,
        })
    }
    
    async fn is_available(&self) -> bool {
        // Note: BME280 may need a method to check availability without mutable reference
        true // Placeholder - implement proper availability check
    }
    
    async fn initialize(&mut self) -> Result<(), IoTError> {
        self.sensor.init().await
    }
    
    fn get_sensor_type(&self) -> &'static str {
        "BME280"
    }
    
    fn get_last_measurement_time(&self) -> Option<u64> {
        self.last_measurement_time
    }
    
    async fn self_test(&mut self) -> Result<(), IoTError> {
        // Perform BME280-specific self-test
        self.sensor.check_id().await.map(|valid| {
            if valid {
                Ok(())
            } else {
                Err(IoTError::Sensor(iot_common::SensorError::SelfTestFailed("Invalid chip ID")))
            }
        })?
    }
}

/// Adapter for WiFi manager to implement NetworkManager trait
#[cfg(feature = "esp32c3")]
pub struct NetworkAdapter {
    wifi_manager: wifi_embassy::WiFiManager,
}

#[cfg(feature = "esp32c3")]
impl NetworkAdapter {
    pub fn new(wifi_manager: wifi_embassy::WiFiManager) -> Self {
        Self { wifi_manager }
    }
}

#[cfg(feature = "esp32c3")]
#[async_trait::async_trait]
impl NetworkManager for NetworkAdapter {
    async fn connect(&mut self) -> Result<(), IoTError> {
        // WiFi manager handles connection internally
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<(), IoTError> {
        // Implement disconnect if supported by WiFi manager
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        self.wifi_manager.is_connected()
    }
    
    async fn get_connection_info(&self) -> Option<crate::traits::ConnectionInfo> {
        self.wifi_manager.get_connection_info().map(|info| {
            crate::traits::ConnectionInfo::new(&info.ip_address).ok()
        }).flatten()
    }
    
    async fn get_signal_strength(&self) -> Option<i8> {
        // Implement signal strength reading if supported
        Some(-50) // Placeholder
    }
    
    async fn test_connectivity(&self) -> Result<(), IoTError> {
        // Implement connectivity test (ping, DNS lookup, etc.)
        if self.is_connected().await {
            Ok(())
        } else {
            Err(IoTError::Network(iot_common::NetworkError::ConnectionLost("No connectivity")))
        }
    }
    
    fn get_stack(&self) -> &'static embassy_net::Stack<embassy_net::driver::Driver<'static>> {
        self.wifi_manager.get_stack()
    }
}

/// Adapter for MQTT client to implement MessagePublisher trait
#[cfg(feature = "esp32c3")]
pub struct PublisherAdapter {
    mqtt_client: mqtt_embassy::MqttClient,
    topic_prefix: crate::config::ConfigString,
    message_count: u32,
    error_count: u32,
    start_time: embassy_time::Instant,
}

#[cfg(feature = "esp32c3")]
impl PublisherAdapter {
    pub fn new(mqtt_client: mqtt_embassy::MqttClient, topic_prefix: crate::config::ConfigString) -> Self {
        Self {
            mqtt_client,
            topic_prefix,
            message_count: 0,
            error_count: 0,
            start_time: embassy_time::Instant::now(),
        }
    }
}

#[cfg(feature = "esp32c3")]
#[async_trait::async_trait]
impl MessagePublisher for PublisherAdapter {
    async fn publish_sensor_data(&mut self, data: &crate::traits::SensorData) -> Result<(), IoTError> {
        // Convert to MQTT sensor data format
        let sensor_data = mqtt_embassy::SensorData::new(
            data.measurements.temperature,
            data.measurements.humidity,
            data.measurements.pressure,
        );
        
        // Create socket buffers (these should be persistent in real implementation)
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];
        
        // Connect and publish
        match self.mqtt_client.connect(self.get_stack(), &mut rx_buffer, &mut tx_buffer).await {
            Ok(mut socket) => {
                match self.mqtt_client.publish_sensor_data(&mut socket, &sensor_data).await {
                    Ok(()) => {
                        self.message_count += 1;
                        Ok(())
                    }
                    Err(e) => {
                        self.error_count += 1;
                        Err(IoTError::Network(iot_common::NetworkError::PublishFailed("MQTT publish failed")))
                    }
                }
            }
            Err(e) => {
                self.error_count += 1;
                Err(IoTError::Network(iot_common::NetworkError::ConnectionFailed("MQTT connect failed")))
            }
        }
    }
    
    async fn publish_status(&mut self, status: &crate::traits::DeviceStatus) -> Result<(), IoTError> {
        // Convert to MQTT device status format
        let device_status = mqtt_embassy::DeviceStatus::new(
            &status.status,
            status.uptime_seconds,
            status.free_heap_bytes,
            status.wifi_signal_dbm,
        );
        
        // Similar implementation to publish_sensor_data
        // ... (implementation details)
        
        self.message_count += 1;
        Ok(())
    }
    
    async fn is_connected(&self) -> bool {
        // Check MQTT connection status
        true // Placeholder
    }
    
    async fn connect(&mut self) -> Result<(), IoTError> {
        // MQTT client handles connection
        Ok(())
    }
    
    async fn publish_heartbeat(&mut self) -> Result<(), IoTError> {
        // Implement heartbeat publishing
        self.message_count += 1;
        Ok(())
    }
    
    fn get_metrics(&self) -> (u32, u32, u32) {
        let uptime = self.start_time.elapsed().as_secs() as u32;
        (self.message_count, self.error_count, uptime)
    }
}

/// Adapter for console interfaces to implement ConsoleInterface trait
#[cfg(feature = "esp32c3")]
pub struct ConsoleAdapter<TX, RX> {
    uart_tx: TX,
    uart_rx: RX,
    input_buffer: heapless::Vec<u8, 128>,
    session_start: embassy_time::Instant,
}

#[cfg(feature = "esp32c3")]
impl<TX, RX> ConsoleAdapter<TX, RX>
where
    TX: iot_hal::UartTxInterface,
    RX: iot_hal::UartRxInterface,
{
    pub fn new(uart_tx: TX, uart_rx: RX, _config: &ConsoleConfig) -> Self {
        Self {
            uart_tx,
            uart_rx,
            input_buffer: heapless::Vec::new(),
            session_start: embassy_time::Instant::now(),
        }
    }
}

#[cfg(feature = "esp32c3")]
#[async_trait::async_trait]
impl<TX, RX> ConsoleInterface for ConsoleAdapter<TX, RX>
where
    TX: iot_hal::UartTxInterface + Send + Sync,
    RX: iot_hal::UartRxInterface + Send + Sync,
{
    async fn write_line(&mut self, message: &str) -> Result<(), IoTError> {
        self.uart_tx.write(message.as_bytes()).await?;
        self.uart_tx.write(b"\r\n").await?;
        Ok(())
    }
    
    async fn read_command(&mut self) -> Result<Option<crate::traits::EmbeddedString>, IoTError> {
        // Implement command reading logic
        // This is a simplified placeholder
        Ok(None)
    }
    
    async fn handle_command(&mut self, command: &str) -> Result<crate::traits::EmbeddedString, IoTError> {
        // Implement command processing logic
        match command.trim() {
            "help" => Ok(crate::traits::EmbeddedString::try_from("Available commands: help, status, info").unwrap()),
            "status" => Ok(crate::traits::EmbeddedString::try_from("System operational").unwrap()),
            "info" => Ok(crate::traits::EmbeddedString::try_from("ESP32-C3 IoT System v1.0").unwrap()),
            _ => Ok(crate::traits::EmbeddedString::try_from("Unknown command").unwrap()),
        }
    }
    
    async fn is_ready(&self) -> bool {
        true
    }
    
    fn get_session_info(&self) -> crate::traits::EmbeddedString {
        let uptime = self.session_start.elapsed().as_secs();
        crate::traits::EmbeddedString::try_from(&format!("Session uptime: {}s", uptime)[..])
            .unwrap_or_else(|_| crate::traits::EmbeddedString::try_from("Session active").unwrap())
    }
    
    async fn show_prompt(&mut self) -> Result<(), IoTError> {
        self.uart_tx.write(b"iot> ").await?;
        Ok(())
    }
}
*/

// ============================================================================
// END OF COMMENTED ADAPTER IMPLEMENTATIONS
// ============================================================================