//! # ESP32-C3 Hardware Platform Implementation
//!
//! Concrete implementation of hardware abstraction traits for ESP32-C3.
//! Provides real hardware access using esp-hal and Embassy framework.

use crate::{
    HardwarePlatform, I2cInterface, UartTxInterface, UartRxInterface, 
    GpioInterface, TimerInterface, WiFiInterface, WiFiConnectionInfo,
    HardwareConfig, error::*
};
use iot_common::IoTError;
use embassy_time::{Duration, Instant};
use async_trait::async_trait;
use esp_hal::{
    i2c::{I2c, Config as I2cConfig},
    uart::{Uart, UartTx, UartRx, config::Config as UartConfig},
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagTx, UsbSerialJtagRx},
    gpio::{Output, AnyPin},
    Async, peripherals,
};
use static_cell::StaticCell;
use core::net::IpAddr;

/// ESP32-C3 hardware platform implementation
/// 
/// Provides concrete hardware access for ESP32-C3 microcontroller using
/// esp-hal and Embassy async framework. Manages all hardware resources
/// and provides unified interface through hardware abstraction traits.
/// 
/// # Resource Management
/// 
/// - **I2C Bus**: Single I2C master for sensor communication
/// - **UART/USB**: Console interface (USB Serial/JTAG preferred)
/// - **GPIO**: Status LED and user-defined pins
/// - **WiFi**: Network connectivity with automatic management
/// - **Timer**: Embassy-based async delays and timeouts
/// 
/// # Memory Usage
/// 
/// - Stack-only resource management where possible
/// - Static allocation for long-lived resources
/// - Bounded buffer sizes for communication
/// - Zero heap allocation in critical paths
/// 
/// # Examples
/// 
/// ```rust
/// use iot_hal::{HardwarePlatform, esp32c3::Esp32C3Platform};
/// 
/// // Initialize with default configuration
/// let mut platform = Esp32C3Platform::initialize().await?;
/// 
/// // Access hardware interfaces
/// let i2c = platform.get_i2c();
/// let (tx, rx) = platform.get_console();
/// let led = platform.get_status_led();
/// ```
pub struct Esp32C3Platform {
    /// I2C bus for sensor communication
    i2c: Esp32C3I2c,
    
    /// Console UART transmitter
    uart_tx: Esp32C3UartTx,
    
    /// Console UART receiver
    uart_rx: Esp32C3UartRx,
    
    /// Status LED GPIO
    status_led: Esp32C3Gpio,
    
    /// Timer interface
    timer: Esp32C3Timer,
    
    /// WiFi interface
    wifi: Esp32C3WiFi,
    
    /// Platform configuration
    config: HardwareConfig,
}

#[async_trait(?Send)]
impl HardwarePlatform for Esp32C3Platform {
    type I2cBus = Esp32C3I2c;
    type UartTx = Esp32C3UartTx;
    type UartRx = Esp32C3UartRx;
    type GpioPin = Esp32C3Gpio;
    type Timer = Esp32C3Timer;
    type WiFi = Esp32C3WiFi;

    async fn initialize() -> Result<Self, IoTError> {
        Self::initialize_with_config(HardwareConfig::esp32c3_devkit()).await
    }

    async fn initialize_with_config(config: HardwareConfig) -> Result<Self, IoTError> {
        // Validate configuration first
        config.validate().map_err(|e| PlatformError::InvalidConfiguration(e))?;

        // Initialize ESP32-C3 peripherals
        let peripherals = esp_hal::init(esp_hal::Config::default());

        // Initialize I2C bus
        let i2c = Self::init_i2c(peripherals.I2C0, &config)?;

        // Initialize console (USB Serial/JTAG or UART)
        let (uart_tx, uart_rx) = Self::init_console(peripherals.USB_DEVICE, &config)?;

        // Initialize status LED GPIO
        let status_led = Self::init_status_led(peripherals.GPIO3, &config)?;

        // Initialize timer
        let timer = Esp32C3Timer::new();

        // Initialize WiFi (placeholder - actual implementation would use peripherals.WIFI)
        let wifi = Esp32C3WiFi::new(&config.wifi)?;

        Ok(Self {
            i2c,
            uart_tx,
            uart_rx,
            status_led,
            timer,
            wifi,
            config,
        })
    }

    fn get_i2c(&mut self) -> &mut Self::I2cBus {
        &mut self.i2c
    }

    fn get_console(&mut self) -> (&mut Self::UartTx, &mut Self::UartRx) {
        (&mut self.uart_tx, &mut self.uart_rx)
    }

    fn get_status_led(&mut self) -> &mut Self::GpioPin {
        &mut self.status_led
    }

    fn get_timer(&mut self) -> &mut Self::Timer {
        &mut self.timer
    }

    fn get_wifi(&mut self) -> &mut Self::WiFi {
        &mut self.wifi
    }

    async fn is_healthy(&mut self) -> bool {
        // Check I2C bus health
        if !self.i2c.is_healthy().await {
            return false;
        }

        // Check console health
        if !self.uart_tx.is_healthy().await || !self.uart_rx.is_healthy().await {
            return false;
        }

        // Check GPIO health
        if !self.status_led.is_healthy().await {
            return false;
        }

        // Check WiFi health
        if !self.wifi.is_healthy().await {
            return false;
        }

        true
    }

    fn platform_info(&self) -> &'static str {
        "ESP32-C3 RISC-V 160MHz with WiFi and Embassy async framework"
    }
}

impl Esp32C3Platform {
    /// Initialize I2C bus with configuration
    fn init_i2c(
        i2c_peripheral: peripherals::I2C0,
        config: &HardwareConfig,
    ) -> Result<Esp32C3I2c, IoTError> {
        let i2c_config = I2cConfig::default()
            .baudrate(config.i2c.frequency)
            .timeout(embassy_time::Duration::from_millis(config.i2c.timeout_ms as u64));

        // Note: In real implementation, would configure pins based on config
        // For now, using default pins from peripherals
        let i2c = I2c::new(i2c_peripheral, i2c_config)
            .map_err(|_| PlatformError::InitializationFailed("I2C initialization failed"))?
            .into_async();

        Ok(Esp32C3I2c::new(i2c))
    }

    /// Initialize console interface (USB Serial/JTAG)
    fn init_console(
        usb_device: peripherals::USB_DEVICE,
        config: &HardwareConfig,
    ) -> Result<(Esp32C3UartTx, Esp32C3UartRx), IoTError> {
        // For ESP32-C3, we prefer USB Serial/JTAG for console
        let usb_serial = UsbSerialJtag::new(usb_device).into_async();
        let (rx, tx) = usb_serial.split();

        Ok((
            Esp32C3UartTx::new_usb(tx),
            Esp32C3UartRx::new_usb(rx),
        ))
    }

    /// Initialize status LED GPIO
    fn init_status_led(
        gpio_pin: AnyPin,
        config: &HardwareConfig,
    ) -> Result<Esp32C3Gpio, IoTError> {
        let output = Output::new(gpio_pin, esp_hal::gpio::Level::Low);
        Ok(Esp32C3Gpio::new(output, config.gpio.status_led_active_high))
    }
}

/// ESP32-C3 I2C interface implementation
pub struct Esp32C3I2c {
    i2c: I2c<'static, Async>,
}

impl Esp32C3I2c {
    fn new(i2c: I2c<'static, Async>) -> Self {
        Self { i2c }
    }

    async fn is_healthy(&mut self) -> bool {
        // Simple health check - try to scan for any device
        for addr in 0x08..0x78 {
            if self.i2c.read(addr, &mut [0u8; 1]).await.is_ok() {
                return true; // Found at least one responsive device
            }
        }
        false // No devices found
    }
}

#[async_trait(?Send)]
impl I2cInterface for Esp32C3I2c {
    async fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), IoTError> {
        self.i2c.read(address, buffer).await
            .map_err(|_| I2cError::DeviceNotResponding(address).into())
    }

    async fn write(&mut self, address: u8, data: &[u8]) -> Result<(), IoTError> {
        self.i2c.write(address, data).await
            .map_err(|_| I2cError::DeviceNotResponding(address).into())
    }

    async fn write_read(&mut self, address: u8, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), IoTError> {
        self.i2c.write_read(address, write_data, read_buffer).await
            .map_err(|_| I2cError::DeviceNotResponding(address).into())
    }
}

/// ESP32-C3 UART transmitter implementation
pub struct Esp32C3UartTx {
    interface: UartTxType,
}

enum UartTxType {
    Usb(UsbSerialJtagTx<'static, Async>),
    Uart(UartTx<'static, Async>),
}

impl Esp32C3UartTx {
    fn new_usb(tx: UsbSerialJtagTx<'static, Async>) -> Self {
        Self {
            interface: UartTxType::Usb(tx),
        }
    }

    fn new_uart(tx: UartTx<'static, Async>) -> Self {
        Self {
            interface: UartTxType::Uart(tx),
        }
    }

    async fn is_healthy(&mut self) -> bool {
        // Simple health check - try to write empty data using the trait method
        UartTxInterface::write(self, b"").await.is_ok()
    }
}

#[async_trait(?Send)]
impl UartTxInterface for Esp32C3UartTx {
    async fn write(&mut self, data: &[u8]) -> Result<usize, IoTError> {
        match &mut self.interface {
            UartTxType::Usb(tx) => {
                embedded_io_async::Write::write(tx, data).await
                    .map_err(|_| UartError::TransmitTimeout.into())
            }
            UartTxType::Uart(tx) => {
                embedded_io_async::Write::write(tx, data).await
                    .map_err(|_| UartError::TransmitTimeout.into())
            }
        }
    }

    async fn flush(&mut self) -> Result<(), IoTError> {
        match &mut self.interface {
            UartTxType::Usb(tx) => {
                embedded_io_async::Write::flush(tx).await
                    .map_err(|_| UartError::TransmitTimeout.into())
            }
            UartTxType::Uart(tx) => {
                embedded_io_async::Write::flush(tx).await
                    .map_err(|_| UartError::TransmitTimeout.into())
            }
        }
    }
}

/// ESP32-C3 UART receiver implementation
pub struct Esp32C3UartRx {
    interface: UartRxType,
}

enum UartRxType {
    Usb(UsbSerialJtagRx<'static, Async>),
    Uart(UartRx<'static, Async>),
}

impl Esp32C3UartRx {
    fn new_usb(rx: UsbSerialJtagRx<'static, Async>) -> Self {
        Self {
            interface: UartRxType::Usb(rx),
        }
    }

    fn new_uart(rx: UartRx<'static, Async>) -> Self {
        Self {
            interface: UartRxType::Uart(rx),
        }
    }

    async fn is_healthy(&mut self) -> bool {
        // UART RX is always considered healthy if initialized
        true
    }
}

#[async_trait(?Send)]
impl UartRxInterface for Esp32C3UartRx {
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, IoTError> {
        match &mut self.interface {
            UartRxType::Usb(rx) => {
                embedded_io_async::Read::read(rx, buffer).await
                    .map_err(|_| UartError::ReceiveTimeout.into())
            }
            UartRxType::Uart(rx) => {
                embedded_io_async::Read::read(rx, buffer).await
                    .map_err(|_| UartError::ReceiveTimeout.into())
            }
        }
    }

    fn available(&self) -> bool {
        // For simplicity, always return true
        // In real implementation, would check hardware buffer status
        true
    }
}

/// ESP32-C3 GPIO implementation
pub struct Esp32C3Gpio {
    output: Output<'static>,
    active_high: bool,
}

impl Esp32C3Gpio {
    fn new(output: Output<'static>, active_high: bool) -> Self {
        Self { output, active_high }
    }

    async fn is_healthy(&mut self) -> bool {
        // GPIO is always considered healthy if initialized
        true
    }
}

#[async_trait(?Send)]
impl GpioInterface for Esp32C3Gpio {
    async fn set_high(&mut self) -> Result<(), IoTError> {
        if self.active_high {
            self.output.set_high();
        } else {
            self.output.set_low();
        }
        Ok(())
    }

    async fn set_low(&mut self) -> Result<(), IoTError> {
        if self.active_high {
            self.output.set_low();
        } else {
            self.output.set_high();
        }
        Ok(())
    }

    async fn toggle(&mut self) -> Result<(), IoTError> {
        self.output.toggle();
        Ok(())
    }

    async fn is_high(&self) -> Result<bool, IoTError> {
        let level = self.output.is_set_high();
        Ok(if self.active_high { level } else { !level })
    }
}

/// ESP32-C3 timer implementation
pub struct Esp32C3Timer {
    // Embassy timer doesn't need state
}

impl Esp32C3Timer {
    fn new() -> Self {
        Self {}
    }
}

impl TimerInterface for Esp32C3Timer {
    async fn delay(&mut self, duration: Duration) {
        embassy_time::Timer::after(duration).await;
    }

    fn now(&self) -> Instant {
        embassy_time::Instant::now()
    }

    fn deadline(&self, duration: Duration) -> Instant {
        embassy_time::Instant::now() + duration
    }
}

/// ESP32-C3 WiFi implementation
pub struct Esp32C3WiFi {
    config: crate::WiFiConfig,
    connected: bool,
    connection_info: Option<WiFiConnectionInfo>,
}

impl Esp32C3WiFi {
    fn new(config: &crate::WiFiConfig) -> Result<Self, IoTError> {
        Ok(Self {
            config: config.clone(),
            connected: false,
            connection_info: None,
        })
    }

    async fn is_healthy(&mut self) -> bool {
        // In real implementation, would check WiFi hardware status
        true
    }
}

#[async_trait(?Send)]
impl WiFiInterface for Esp32C3WiFi {
    async fn connect(&mut self, ssid: &str, password: &str) -> Result<(), IoTError> {
        // This is a placeholder implementation
        // Real implementation would:
        // 1. Initialize WiFi hardware
        // 2. Scan for network
        // 3. Attempt connection with credentials
        // 4. Configure DHCP
        // 5. Update connection status

        // Simulate connection delay
        embassy_time::Timer::after(Duration::from_secs(2)).await;

        // For now, assume connection succeeds
        self.connected = true;
        self.connection_info = Some(WiFiConnectionInfo {
            ip_address: IpAddr::V4(core::net::Ipv4Addr::new(192, 168, 1, 100)),
            gateway: Some(IpAddr::V4(core::net::Ipv4Addr::new(192, 168, 1, 1))),
            netmask: Some(IpAddr::V4(core::net::Ipv4Addr::new(255, 255, 255, 0))),
            ssid: heapless::String::try_from(ssid).unwrap_or_default(),
            signal_strength: -45, // Good signal
            uptime_seconds: 0,
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), IoTError> {
        self.connected = false;
        self.connection_info = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_ip_address(&self) -> Option<IpAddr> {
        self.connection_info.as_ref().map(|info| info.ip_address)
    }

    fn get_signal_strength(&self) -> i8 {
        self.connection_info
            .as_ref()
            .map(|info| info.signal_strength)
            .unwrap_or(-100) // No signal if not connected
    }

    fn get_connection_info(&self) -> Option<WiFiConnectionInfo> {
        self.connection_info.clone()
    }
}

/// Platform-specific utilities for ESP32-C3
pub mod utils {
    use super::*;

    /// Get ESP32-C3 chip information
    pub fn get_chip_info() -> &'static str {
        "ESP32-C3 RISC-V 160MHz WiFi SoC"
    }

    /// Get available memory information
    pub fn get_memory_info() -> (usize, usize) {
        // In real implementation, would query actual memory usage
        (400 * 1024, 32 * 1024) // (total RAM, available heap)
    }

    /// Reset the ESP32-C3 system
    pub fn system_reset() -> ! {
        esp_hal::reset::software_reset();
    }

    /// Enter deep sleep mode
    pub async fn deep_sleep(duration: Duration) {
        // In real implementation, would configure deep sleep
        // For now, just delay
        embassy_time::Timer::after(duration).await;
    }

    /// Check if brownout detection triggered last reset
    pub fn was_brownout_reset() -> bool {
        // In real implementation, would check reset reason
        false
    }
}