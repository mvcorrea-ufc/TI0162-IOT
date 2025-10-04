//! # IoT Dependency Injection Container
//!
//! This module provides the main dependency injection container for the ESP32-C3 IoT system.
//! The container manages all system components and orchestrates their interactions while
//! maintaining clean separation of concerns and enabling comprehensive testing.

use embassy_time::{Duration, Timer, Instant};
use embassy_sync::signal::Signal;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use heapless::Deque;

use iot_common::{IoTError, IoTResult};
use iot_hal::HardwarePlatform;

use crate::traits::{
    SensorReader, NetworkManager, MessagePublisher, ConsoleInterface,
    Measurements, SensorData, DeviceStatus, EmbeddedString
};
use crate::config::{SystemConfiguration, OperatingMode, LogLevel};

/// Maximum number of measurements to buffer
const MAX_MEASUREMENT_BUFFER: usize = 16;

/// Maximum number of pending console commands
#[allow(dead_code)]
const MAX_CONSOLE_COMMANDS: usize = 8;

/// System state shared between tasks
#[derive(Debug, Clone, Copy)]
pub struct SystemState {
    /// Is the sensor subsystem active and reading data
    pub sensor_active: bool,
    
    /// Is the network subsystem connected
    pub network_connected: bool,
    
    /// Is the message publisher connected
    pub publisher_connected: bool,
    
    /// Is the console interface active
    pub console_active: bool,
    
    /// Total number of sensor readings taken
    pub sensor_readings_count: u32,
    
    /// Total number of messages published
    pub messages_published_count: u32,
    
    /// System uptime in seconds
    pub uptime_seconds: u32,
    
    /// Last error code (0 = no error)
    pub last_error_code: u32,
    
    /// Free heap memory in bytes
    pub free_heap_bytes: u32,
}

impl SystemState {
    /// Creates a new default system state
    pub const fn new() -> Self {
        Self {
            sensor_active: false,
            network_connected: false,
            publisher_connected: false,
            console_active: false,
            sensor_readings_count: 0,
            messages_published_count: 0,
            uptime_seconds: 0,
            last_error_code: 0,
            free_heap_bytes: 0,
        }
    }
}

/// Shared system state accessible by all tasks
static SYSTEM_STATE: Mutex<CriticalSectionRawMutex, SystemState> = 
    Mutex::new(SystemState::new());

/// Signal for sharing sensor data between tasks
static SENSOR_DATA_SIGNAL: Signal<CriticalSectionRawMutex, Measurements> = Signal::new();

/// Signal for sharing console commands between tasks
#[allow(dead_code)]
static CONSOLE_COMMAND_SIGNAL: Signal<CriticalSectionRawMutex, EmbeddedString> = Signal::new();

/// IoT Dependency Injection Container
/// 
/// The main container that manages all system components and orchestrates their interactions.
/// This container implements the dependency injection pattern to enable clean architecture,
/// comprehensive testing, and flexible configuration.
/// 
/// # Type Parameters
/// 
/// * `P` - Hardware platform implementation
/// * `S` - Sensor reader implementation  
/// * `N` - Network manager implementation
/// * `M` - Message publisher implementation
/// * `C` - Console interface implementation
/// 
/// # Examples
/// 
/// ```rust,no_run
/// use iot_container::{IoTContainer, ComponentFactory, SystemConfiguration};
/// use iot_hal::Esp32C3Platform;
/// 
/// // Create container with real hardware components
/// let platform = Esp32C3Platform::initialize().await?;
/// let sensor = ComponentFactory::create_sensor(&mut platform, &config.sensor).await?;
/// let network = ComponentFactory::create_network(&mut platform, &config.wifi).await?;
/// let publisher = ComponentFactory::create_publisher(&network, &config.mqtt).await?;
/// let console = ComponentFactory::create_console(&mut platform, &config.console).await?;
/// 
/// let mut container = IoTContainer::new(
///     platform, sensor, network, publisher, console, config
/// ).await?;
/// 
/// container.run_system().await?;
/// ```
pub struct IoTContainer<P, S, N, M, C>
where
    P: HardwarePlatform,
    S: SensorReader,
    N: NetworkManager,
    M: MessagePublisher,
    C: ConsoleInterface,
{
    /// Hardware platform abstraction
    #[allow(dead_code)]
    platform: P,
    
    /// Environmental sensor reader
    sensor: S,
    
    /// Network connectivity manager
    network: N,
    
    /// Message publisher for remote communication
    publisher: M,
    
    /// Interactive console interface
    console: C,
    
    /// System configuration
    config: SystemConfiguration,
    
    /// Measurement buffer for data smoothing
    measurement_buffer: Deque<Measurements, MAX_MEASUREMENT_BUFFER>,
    
    /// System start time for uptime calculation
    start_time: Instant,
    
    /// Device identifier for published messages
    device_id: EmbeddedString,
}

impl<P, S, N, M, C> IoTContainer<P, S, N, M, C>
where
    P: HardwarePlatform,
    S: SensorReader,
    N: NetworkManager,
    M: MessagePublisher,
    C: ConsoleInterface,
{
    /// Creates a new IoT container with the specified components
    /// 
    /// This constructor initializes the container with all required dependencies
    /// and prepares it for system operation.
    /// 
    /// # Arguments
    /// 
    /// * `platform` - Hardware platform implementation
    /// * `sensor` - Sensor reader implementation
    /// * `network` - Network manager implementation
    /// * `publisher` - Message publisher implementation
    /// * `console` - Console interface implementation
    /// * `config` - System configuration
    /// 
    /// # Returns
    /// 
    /// * `Ok(IoTContainer)` - Container created successfully
    /// * `Err(IoTError)` - Container creation failed
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// let container = IoTContainer::new(
    ///     platform, sensor, network, publisher, console, config
    /// ).await?;
    /// ```
    pub async fn new(
        platform: P,
        mut sensor: S,
        mut network: N,
        mut publisher: M,
        console: C,
        config: SystemConfiguration,
    ) -> IoTResult<Self> {
        // Initialize device ID
        let device_id = EmbeddedString::try_from(config.device_id.as_str())
            .map_err(|_| IoTError::configuration(
                iot_common::ConfigError::ValidationError("Device ID too long".try_into().unwrap_or_default())
            ))?;
        
        // Initialize components
        Self::log(&config, LogLevel::Info, "Initializing IoT container components").await;
        
        // Initialize sensor
        if let Err(e) = sensor.initialize().await {
            Self::log(&config, LogLevel::Warning, "Sensor initialization failed").await;
            if config.operation_mode == OperatingMode::Production {
                return Err(e);
            }
        }
        
        // Initialize network connection
        if let Err(_e) = network.connect().await {
            Self::log(&config, LogLevel::Warning, "Network connection failed").await;
            if config.operation_mode == OperatingMode::Production {
                return Err(_e);
            }
        }
        
        // Initialize message publisher
        if let Err(_e) = publisher.connect().await {
            Self::log(&config, LogLevel::Warning, "Message publisher connection failed").await;
        }
        
        let container = Self {
            platform,
            sensor,
            network,
            publisher,
            console,
            config,
            measurement_buffer: Deque::new(),
            start_time: Instant::now(),
            device_id,
        };
        
        Self::log(&container.config, LogLevel::Info, "IoT container initialized successfully").await;
        
        Ok(container)
    }
    
    /// Runs the complete IoT system
    /// 
    /// This method starts all system tasks and runs the main application loop.
    /// It orchestrates sensor readings, network communications, message publishing,
    /// and console interactions.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - System shut down gracefully
    /// * `Err(IoTError)` - System encountered fatal error
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// container.run_system().await?;
    /// ```
    pub async fn run_system(&mut self) -> IoTResult<()> {
        Self::log(&self.config, LogLevel::Info, "Starting IoT system operation").await;
        
        // Initialize all subsystems
        self.initialize_all().await?;
        
        // Start main operation loop
        self.run_main_loop().await?;
        
        Ok(())
    }
    
    /// Initializes all system components
    async fn initialize_all(&mut self) -> IoTResult<()> {
        Self::log(&self.config, LogLevel::Debug, "Initializing all system components").await;
        
        // Mark console as active
        {
            let mut state = SYSTEM_STATE.lock().await;
            state.console_active = true;
        }
        
        // Test sensor availability
        if self.sensor.is_available().await {
            let mut state = SYSTEM_STATE.lock().await;
            state.sensor_active = true;
            Self::log(&self.config, LogLevel::Info, "Sensor subsystem active").await;
        } else {
            Self::log(&self.config, LogLevel::Warning, "Sensor not available").await;
        }
        
        // Test network connectivity
        if self.network.is_connected().await {
            let mut state = SYSTEM_STATE.lock().await;
            state.network_connected = true;
            Self::log(&self.config, LogLevel::Info, "Network subsystem connected").await;
        } else {
            Self::log(&self.config, LogLevel::Warning, "Network not connected").await;
        }
        
        // Test publisher connectivity
        if self.publisher.is_connected().await {
            let mut state = SYSTEM_STATE.lock().await;
            state.publisher_connected = true;
            Self::log(&self.config, LogLevel::Info, "Publisher subsystem connected").await;
        } else {
            Self::log(&self.config, LogLevel::Warning, "Publisher not connected").await;
        }
        
        Ok(())
    }
    
    /// Runs the main system operation loop
    async fn run_main_loop(&mut self) -> IoTResult<()> {
        Self::log(&self.config, LogLevel::Info, "Starting main operation loop").await;
        
        let mut cycle_count = 0u32;
        let mut last_status_report = Instant::now();
        let operation_interval = Duration::from_secs(self.config.sensor_read_interval_secs);
        let status_report_interval = Duration::from_secs(self.config.status_report_interval_secs);
        
        loop {
            let cycle_start = Instant::now();
            
            // Update system uptime
            {
                let mut state = SYSTEM_STATE.lock().await;
                state.uptime_seconds = self.start_time.elapsed().as_secs() as u32;
            }
            
            // Perform sensor reading cycle
            if let Err(e) = self.sensor_reading_cycle().await {
                Self::log(&self.config, LogLevel::Error, "Sensor reading cycle failed").await;
                self.handle_error(e).await;
            }
            
            // Perform network operations cycle
            if let Err(_e) = self.network_operations_cycle().await {
                Self::log(&self.config, LogLevel::Error, "Network operations cycle failed").await;
                self.handle_error(_e).await;
            }
            
            // Perform console operations cycle
            if let Err(_e) = self.console_operations_cycle().await {
                Self::log(&self.config, LogLevel::Debug, "Console operations cycle completed").await;
            }
            
            // Periodic status reporting
            if cycle_start.duration_since(last_status_report) >= status_report_interval {
                if let Err(_e) = self.status_report_cycle().await {
                    Self::log(&self.config, LogLevel::Warning, "Status report cycle failed").await;
                }
                last_status_report = cycle_start;
            }
            
            cycle_count += 1;
            
            // Calculate time to next cycle
            let cycle_duration = Instant::now().duration_since(cycle_start);
            if cycle_duration < operation_interval {
                Timer::after(operation_interval - cycle_duration).await;
            }
            
            // Periodic logging in debug mode
            if cycle_count % 10 == 0 && self.config.log_level == LogLevel::Debug {
                Self::log(&self.config, LogLevel::Debug, "Main loop cycle completed").await;
            }
        }
    }
    
    /// Performs a sensor reading cycle
    async fn sensor_reading_cycle(&mut self) -> IoTResult<()> {
        if !self.sensor.is_available().await {
            // Mark sensor as inactive
            let mut state = SYSTEM_STATE.lock().await;
            state.sensor_active = false;
            return Err(IoTError::sensor(iot_common::SensorError::NotResponding("Sensor not responding".try_into().unwrap_or_default())));
        }
        
        // Read measurements
        match self.sensor.read_measurements().await {
            Ok(measurements) => {
                // Validate measurements
                if !measurements.is_valid() {
                    Self::log(&self.config, LogLevel::Warning, "Invalid sensor measurements").await;
                    return Ok(());
                }
                
                // Update measurement buffer
                if self.measurement_buffer.len() >= MAX_MEASUREMENT_BUFFER {
                    self.measurement_buffer.pop_front();
                }
                let _ = self.measurement_buffer.push_back(measurements.clone());
                
                // Signal new measurement available
                SENSOR_DATA_SIGNAL.signal(measurements);
                
                // Update system state
                {
                    let mut state = SYSTEM_STATE.lock().await;
                    state.sensor_active = true;
                    state.sensor_readings_count += 1;
                }
                
                Self::log(&self.config, LogLevel::Debug, "Sensor reading completed").await;
                Ok(())
            }
            Err(e) => {
                let mut state = SYSTEM_STATE.lock().await;
                state.sensor_active = false;
                Err(e)
            }
        }
    }
    
    /// Performs network operations cycle
    async fn network_operations_cycle(&mut self) -> IoTResult<()> {
        // Check network connectivity
        if !self.network.is_connected().await {
            Self::log(&self.config, LogLevel::Warning, "Network disconnected, attempting reconnection").await;
            
            if let Err(e) = self.network.connect().await {
                let mut state = SYSTEM_STATE.lock().await;
                state.network_connected = false;
                return Err(e);
            }
        }
        
        // Update network status
        {
            let mut state = SYSTEM_STATE.lock().await;
            state.network_connected = self.network.is_connected().await;
        }
        
        // Attempt to publish pending sensor data
        if let Some(measurements) = SENSOR_DATA_SIGNAL.try_take() {
            if let Err(e) = self.publish_sensor_data(measurements.clone()).await {
                Self::log(&self.config, LogLevel::Warning, "Failed to publish sensor data").await;
                // Put the measurement back for retry
                SENSOR_DATA_SIGNAL.signal(measurements);
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    /// Performs console operations cycle
    async fn console_operations_cycle(&mut self) -> IoTResult<()> {
        // Check if console command is available
        if let Ok(Some(command)) = self.console.read_command().await {
            Self::log(&self.config, LogLevel::Debug, "Processing console command").await;
            
            // Process the command
            match self.console.handle_command(&command).await {
                Ok(response) => {
                    if let Err(e) = self.console.write_line(&response).await {
                        Self::log(&self.config, LogLevel::Warning, "Failed to write console response").await;
                        return Err(e);
                    }
                }
                Err(e) => {
                    let error_msg = "Command processing failed";
                    let _ = self.console.write_line(error_msg).await;
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Performs status reporting cycle
    async fn status_report_cycle(&mut self) -> IoTResult<()> {
        let state = SYSTEM_STATE.lock().await;
        
        // Create device status
        let status = match DeviceStatus::new(
            &self.device_id,
            if state.sensor_active && state.network_connected { "operational" } else { "degraded" },
            state.uptime_seconds,
            state.free_heap_bytes,
            self.network.get_signal_strength().await.unwrap_or(-99),
        ) {
            Ok(mut status) => {
                status.sensor_readings_count = state.sensor_readings_count;
                status.mqtt_messages_count = state.messages_published_count;
                status.last_error_code = state.last_error_code;
                status
            }
            Err(e) => return Err(e),
        };
        
        drop(state);
        
        // Publish status if publisher is connected
        if self.publisher.is_connected().await {
            if let Err(e) = self.publisher.publish_status(&status).await {
                Self::log(&self.config, LogLevel::Warning, "Failed to publish status").await;
                return Err(e);
            }
            Self::log(&self.config, LogLevel::Debug, "Status report published").await;
        }
        
        Ok(())
    }
    
    /// Publishes sensor data
    async fn publish_sensor_data(&mut self, measurements: Measurements) -> IoTResult<()> {
        // Create sensor data
        let sensor_data = SensorData::new(&self.device_id, measurements)?;
        
        // Ensure publisher is connected
        if !self.publisher.is_connected().await {
            if let Err(e) = self.publisher.connect().await {
                let mut state = SYSTEM_STATE.lock().await;
                state.publisher_connected = false;
                return Err(e);
            }
        }
        
        // Publish the data
        match self.publisher.publish_sensor_data(&sensor_data).await {
            Ok(()) => {
                let mut state = SYSTEM_STATE.lock().await;
                state.publisher_connected = true;
                state.messages_published_count += 1;
                Self::log(&self.config, LogLevel::Debug, "Sensor data published successfully").await;
                Ok(())
            }
            Err(e) => {
                let mut state = SYSTEM_STATE.lock().await;
                state.publisher_connected = false;
                Err(e)
            }
        }
    }
    
    /// Handles system errors
    async fn handle_error(&self, error: IoTError) {
        // Update error state
        {
            let mut state = SYSTEM_STATE.lock().await;
            state.last_error_code = error.error_code() as u32;
        }
        
        // Log error based on configuration
        if self.config.log_level as u8 >= LogLevel::Error as u8 {
            Self::log(&self.config, LogLevel::Error, "System error occurred").await;
        }
    }
    
    /// Logs a message if logging level permits
    async fn log(config: &SystemConfiguration, level: LogLevel, message: &str) {
        if config.log_level as u8 >= level as u8 {
            // Use RTT for embedded logging
            #[cfg(feature = "esp32c3")]
            rtt_target::rprintln!("[{}] {}", level.as_str(), message);
            
            // Use println for testing
            #[cfg(feature = "mock")]
            println!("[{}] {}", level.as_str(), message);
        }
    }
    
    /// Runs a single operation cycle (useful for testing)
    pub async fn run_single_cycle(&mut self) -> IoTResult<()> {
        let _ = self.sensor_reading_cycle().await;
        let _ = self.network_operations_cycle().await;
        let _ = self.console_operations_cycle().await;
        Ok(())
    }
    
    /// Gets current system state (useful for testing and monitoring)
    pub async fn get_system_state(&self) -> SystemState {
        *SYSTEM_STATE.lock().await
    }
    
    /// Gets buffered measurements (useful for testing and data analysis)
    pub fn get_measurement_buffer(&self) -> &Deque<Measurements, MAX_MEASUREMENT_BUFFER> {
        &self.measurement_buffer
    }
}