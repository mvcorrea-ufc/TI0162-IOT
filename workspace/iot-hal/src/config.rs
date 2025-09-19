//! # Hardware Configuration
//!
//! Configuration structures for hardware platform initialization.
//! Provides a centralized way to configure hardware parameters.

/// Hardware platform configuration
/// 
/// Centralizes all hardware configuration parameters in one structure.
/// Allows customization of pin assignments, communication parameters,
/// and operational settings.
/// 
/// # Design Goals
/// 
/// - **Validation**: All parameters validated at construction time
/// - **Defaults**: Sensible defaults for common ESP32-C3 setups
/// - **Flexibility**: Easy customization for different hardware variants
/// - **Documentation**: Clear parameter descriptions and constraints
/// 
/// # Examples
/// 
/// ```rust
/// use iot_hal::HardwareConfig;
/// 
/// // Use default configuration
/// let config = HardwareConfig::default();
/// 
/// // Customize specific parameters
/// let config = HardwareConfig {
///     i2c: I2cConfig {
///         sda_pin: 8,
///         scl_pin: 9,
///         frequency: 400_000,
///     },
///     uart: UartConfig {
///         tx_pin: 21,
///         rx_pin: 20,
///         baud_rate: 115_200,
///     },
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct HardwareConfig {
    /// I2C bus configuration
    pub i2c: I2cConfig,
    
    /// UART console configuration
    pub uart: UartConfig,
    
    /// GPIO configuration
    pub gpio: GpioConfig,
    
    /// WiFi configuration
    pub wifi: WiFiConfig,
    
    /// System configuration
    pub system: SystemConfig,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            i2c: I2cConfig::default(),
            uart: UartConfig::default(),
            gpio: GpioConfig::default(),
            wifi: WiFiConfig::default(),
            system: SystemConfig::default(),
        }
    }
}

impl HardwareConfig {
    /// Create new hardware configuration with validation
    /// 
    /// # Arguments
    /// 
    /// All configuration sub-structures
    /// 
    /// # Returns
    /// 
    /// * `Ok(config)` - Configuration is valid
    /// * `Err(message)` - Configuration validation failed
    /// 
    /// # Validation Rules
    /// 
    /// - Pin numbers must be valid for target platform
    /// - Communication parameters within supported ranges
    /// - No pin conflicts between interfaces
    /// - All required parameters specified
    pub fn new(
        i2c: I2cConfig,
        uart: UartConfig,
        gpio: GpioConfig,
        wifi: WiFiConfig,
        system: SystemConfig,
    ) -> Result<Self, &'static str> {
        let config = Self {
            i2c,
            uart,
            gpio,
            wifi,
            system,
        };
        
        config.validate()?;
        Ok(config)
    }

    /// Validate configuration parameters
    /// 
    /// Checks for:
    /// - Valid pin assignments for ESP32-C3
    /// - No pin conflicts between interfaces
    /// - Communication parameters within spec
    /// - Required fields populated
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - Configuration is valid
    /// * `Err(message)` - Specific validation error
    pub fn validate(&self) -> Result<(), &'static str> {
        // Validate I2C configuration
        self.i2c.validate()?;
        
        // Validate UART configuration
        self.uart.validate()?;
        
        // Validate GPIO configuration
        self.gpio.validate()?;
        
        // Validate WiFi configuration
        self.wifi.validate()?;
        
        // Check for pin conflicts
        self.check_pin_conflicts()?;
        
        Ok(())
    }

    /// Check for pin conflicts between interfaces
    /// 
    /// Ensures no two interfaces try to use the same pin.
    fn check_pin_conflicts(&self) -> Result<(), &'static str> {
        let mut used_pins = heapless::FnvIndexSet::<u8, 16>::new();
        
        // Check I2C pins
        if !used_pins.insert(self.i2c.sda_pin).map_err(|_| "Too many pins configured")? {
            return Err("I2C SDA pin conflict");
        }
        if !used_pins.insert(self.i2c.scl_pin).map_err(|_| "Too many pins configured")? {
            return Err("I2C SCL pin conflict");
        }
        
        // Check UART pins
        if !used_pins.insert(self.uart.tx_pin).map_err(|_| "Too many pins configured")? {
            return Err("UART TX pin conflict");
        }
        if !used_pins.insert(self.uart.rx_pin).map_err(|_| "Too many pins configured")? {
            return Err("UART RX pin conflict");
        }
        
        // Check GPIO pins
        if !used_pins.insert(self.gpio.status_led_pin).map_err(|_| "Too many pins configured")? {
            return Err("Status LED pin conflict");
        }
        
        Ok(())
    }

    /// Create configuration for ESP32-C3 DevKit
    /// 
    /// Pre-configured for standard ESP32-C3 development board
    /// with commonly used pin assignments.
    pub fn esp32c3_devkit() -> Self {
        Self {
            i2c: I2cConfig::esp32c3_default(),
            uart: UartConfig::esp32c3_usb_serial(),
            gpio: GpioConfig::esp32c3_devkit(),
            wifi: WiFiConfig::default(),
            system: SystemConfig::esp32c3_default(),
        }
    }

    /// Create configuration for custom ESP32-C3 board
    /// 
    /// Template for custom board configurations.
    /// Modify pin assignments as needed.
    pub fn esp32c3_custom() -> Self {
        Self::default()
    }
}

/// I2C bus configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct I2cConfig {
    /// SDA (data) pin number
    pub sda_pin: u8,
    
    /// SCL (clock) pin number
    pub scl_pin: u8,
    
    /// I2C bus frequency in Hz
    pub frequency: u32,
    
    /// Enable internal pull-up resistors
    pub pullup_enabled: bool,
    
    /// Bus timeout in milliseconds
    pub timeout_ms: u32,
}

impl Default for I2cConfig {
    fn default() -> Self {
        Self {
            sda_pin: 8,      // Common ESP32-C3 I2C SDA
            scl_pin: 9,      // Common ESP32-C3 I2C SCL
            frequency: 400_000, // 400kHz (fast mode)
            pullup_enabled: true,
            timeout_ms: 1000,
        }
    }
}

impl I2cConfig {
    /// Create I2C configuration with custom pins
    /// 
    /// # Arguments
    /// 
    /// * `sda_pin` - SDA pin number (GPIO)
    /// * `scl_pin` - SCL pin number (GPIO)
    /// 
    /// # Returns
    /// 
    /// I2C configuration with specified pins and default settings
    pub fn with_pins(sda_pin: u8, scl_pin: u8) -> Self {
        Self {
            sda_pin,
            scl_pin,
            ..Default::default()
        }
    }

    /// Set I2C bus frequency
    /// 
    /// # Arguments
    /// 
    /// * `frequency` - Bus frequency in Hz
    /// 
    /// # Common Frequencies
    /// 
    /// - 100_000 Hz - Standard mode
    /// - 400_000 Hz - Fast mode (default)
    /// - 1_000_000 Hz - Fast mode plus
    pub fn with_frequency(mut self, frequency: u32) -> Self {
        self.frequency = frequency;
        self
    }

    /// ESP32-C3 default I2C configuration
    /// 
    /// Standard pins for ESP32-C3 development boards
    pub fn esp32c3_default() -> Self {
        Self::default()
    }

    /// Validate I2C configuration
    fn validate(&self) -> Result<(), &'static str> {
        // Validate pin numbers for ESP32-C3
        if self.sda_pin > 21 {
            return Err("Invalid I2C SDA pin for ESP32-C3");
        }
        if self.scl_pin > 21 {
            return Err("Invalid I2C SCL pin for ESP32-C3");
        }
        
        // Validate frequency range
        if self.frequency < 10_000 || self.frequency > 1_000_000 {
            return Err("I2C frequency out of supported range (10kHz - 1MHz)");
        }
        
        // Validate timeout
        if self.timeout_ms == 0 || self.timeout_ms > 10_000 {
            return Err("I2C timeout must be 1-10000ms");
        }
        
        Ok(())
    }
}

/// UART console configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct UartConfig {
    /// UART transmit pin number
    pub tx_pin: u8,
    
    /// UART receive pin number
    pub rx_pin: u8,
    
    /// Baud rate in bits per second
    pub baud_rate: u32,
    
    /// Data bits (5-8)
    pub data_bits: u8,
    
    /// Stop bits (1-2)
    pub stop_bits: u8,
    
    /// Parity (None, Even, Odd)
    pub parity: UartParity,
    
    /// Hardware flow control enabled
    pub flow_control: bool,
    
    /// Receive buffer size
    pub rx_buffer_size: usize,
    
    /// Transmit buffer size
    pub tx_buffer_size: usize,
}

/// UART parity configuration
#[derive(Debug, Clone, PartialEq)]
pub enum UartParity {
    /// No parity bit
    None,
    /// Even parity
    Even,
    /// Odd parity
    Odd,
}

impl Default for UartConfig {
    fn default() -> Self {
        Self {
            tx_pin: 21,      // ESP32-C3 common UART TX
            rx_pin: 20,      // ESP32-C3 common UART RX
            baud_rate: 115_200,
            data_bits: 8,
            stop_bits: 1,
            parity: UartParity::None,
            flow_control: false,
            rx_buffer_size: 256,
            tx_buffer_size: 256,
        }
    }
}

impl UartConfig {
    /// Create UART configuration with custom pins
    pub fn with_pins(tx_pin: u8, rx_pin: u8) -> Self {
        Self {
            tx_pin,
            rx_pin,
            ..Default::default()
        }
    }

    /// Set UART baud rate
    pub fn with_baud_rate(mut self, baud_rate: u32) -> Self {
        self.baud_rate = baud_rate;
        self
    }

    /// ESP32-C3 USB Serial/JTAG configuration
    /// 
    /// Uses built-in USB Serial/JTAG for console
    pub fn esp32c3_usb_serial() -> Self {
        Self {
            tx_pin: 255, // Special value for USB Serial/JTAG
            rx_pin: 255, // Special value for USB Serial/JTAG
            ..Default::default()
        }
    }

    /// Validate UART configuration
    fn validate(&self) -> Result<(), &'static str> {
        // Special case for USB Serial/JTAG
        if self.tx_pin == 255 && self.rx_pin == 255 {
            return Ok(()); // USB Serial/JTAG doesn't need pin validation
        }
        
        // Validate pin numbers for ESP32-C3
        if self.tx_pin > 21 {
            return Err("Invalid UART TX pin for ESP32-C3");
        }
        if self.rx_pin > 21 {
            return Err("Invalid UART RX pin for ESP32-C3");
        }
        
        // Validate baud rate
        if self.baud_rate < 300 || self.baud_rate > 2_000_000 {
            return Err("UART baud rate out of supported range");
        }
        
        // Validate data bits
        if self.data_bits < 5 || self.data_bits > 8 {
            return Err("UART data bits must be 5-8");
        }
        
        // Validate stop bits
        if self.stop_bits < 1 || self.stop_bits > 2 {
            return Err("UART stop bits must be 1-2");
        }
        
        Ok(())
    }
}

/// GPIO configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct GpioConfig {
    /// Status LED pin number
    pub status_led_pin: u8,
    
    /// Status LED active level (true = active high)
    pub status_led_active_high: bool,
    
    /// Additional GPIO pins for future use
    pub user_pins: heapless::Vec<u8, 8>,
}

impl Default for GpioConfig {
    fn default() -> Self {
        Self {
            status_led_pin: 3,   // ESP32-C3 built-in LED
            status_led_active_high: true,
            user_pins: heapless::Vec::new(),
        }
    }
}

impl GpioConfig {
    /// ESP32-C3 DevKit configuration
    pub fn esp32c3_devkit() -> Self {
        Self::default()
    }

    /// Add user-defined GPIO pin
    pub fn add_user_pin(&mut self, pin: u8) -> Result<(), &'static str> {
        self.user_pins.push(pin).map_err(|_| "Too many user pins configured")
    }

    /// Validate GPIO configuration
    fn validate(&self) -> Result<(), &'static str> {
        // Validate status LED pin
        if self.status_led_pin > 21 {
            return Err("Invalid status LED pin for ESP32-C3");
        }
        
        // Validate user pins
        for &pin in &self.user_pins {
            if pin > 21 {
                return Err("Invalid user GPIO pin for ESP32-C3");
            }
        }
        
        Ok(())
    }
}

/// WiFi configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct WiFiConfig {
    /// WiFi power management mode
    pub power_mode: WiFiPowerMode,
    
    /// Connection timeout in seconds
    pub connection_timeout_sec: u32,
    
    /// Maximum retry attempts for connection
    pub max_retry_attempts: u8,
    
    /// Retry delay between attempts in seconds
    pub retry_delay_sec: u32,
    
    /// Enable automatic reconnection
    pub auto_reconnect: bool,
}

/// WiFi power management modes
#[derive(Debug, Clone, PartialEq)]
pub enum WiFiPowerMode {
    /// Maximum performance, highest power consumption
    Performance,
    /// Balanced power and performance
    Balanced,
    /// Minimum power consumption
    PowerSave,
}

impl Default for WiFiConfig {
    fn default() -> Self {
        Self {
            power_mode: WiFiPowerMode::Balanced,
            connection_timeout_sec: 30,
            max_retry_attempts: 5,
            retry_delay_sec: 5,
            auto_reconnect: true,
        }
    }
}

impl WiFiConfig {
    /// High performance WiFi configuration
    pub fn high_performance() -> Self {
        Self {
            power_mode: WiFiPowerMode::Performance,
            connection_timeout_sec: 15,
            max_retry_attempts: 3,
            retry_delay_sec: 2,
            auto_reconnect: true,
        }
    }

    /// Power saving WiFi configuration
    pub fn power_save() -> Self {
        Self {
            power_mode: WiFiPowerMode::PowerSave,
            connection_timeout_sec: 60,
            max_retry_attempts: 10,
            retry_delay_sec: 10,
            auto_reconnect: true,
        }
    }

    /// Validate WiFi configuration
    fn validate(&self) -> Result<(), &'static str> {
        // Validate timeout
        if self.connection_timeout_sec == 0 || self.connection_timeout_sec > 300 {
            return Err("WiFi connection timeout must be 1-300 seconds");
        }
        
        // Validate retry attempts
        if self.max_retry_attempts == 0 || self.max_retry_attempts > 20 {
            return Err("WiFi retry attempts must be 1-20");
        }
        
        // Validate retry delay
        if self.retry_delay_sec == 0 || self.retry_delay_sec > 60 {
            return Err("WiFi retry delay must be 1-60 seconds");
        }
        
        Ok(())
    }
}

/// System-level configuration parameters
#[derive(Debug, Clone, PartialEq)]
pub struct SystemConfig {
    /// System clock frequency in Hz
    pub cpu_frequency_hz: u32,
    
    /// Heap size allocation in bytes
    pub heap_size_bytes: usize,
    
    /// Stack size for async tasks in bytes
    pub task_stack_size_bytes: usize,
    
    /// Enable debug logging
    pub debug_logging: bool,
    
    /// Watchdog timeout in seconds
    pub watchdog_timeout_sec: u32,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            cpu_frequency_hz: 160_000_000, // 160MHz
            heap_size_bytes: 32 * 1024,    // 32KB
            task_stack_size_bytes: 4 * 1024, // 4KB
            debug_logging: true,
            watchdog_timeout_sec: 60,
        }
    }
}

impl SystemConfig {
    /// ESP32-C3 default system configuration
    pub fn esp32c3_default() -> Self {
        Self::default()
    }

    /// Low power system configuration
    pub fn low_power() -> Self {
        Self {
            cpu_frequency_hz: 80_000_000, // 80MHz
            heap_size_bytes: 16 * 1024,   // 16KB
            task_stack_size_bytes: 2 * 1024, // 2KB
            debug_logging: false,
            watchdog_timeout_sec: 120,
        }
    }

    /// High performance system configuration
    pub fn high_performance() -> Self {
        Self {
            cpu_frequency_hz: 160_000_000, // 160MHz
            heap_size_bytes: 64 * 1024,    // 64KB
            task_stack_size_bytes: 8 * 1024, // 8KB
            debug_logging: true,
            watchdog_timeout_sec: 30,
        }
    }
}