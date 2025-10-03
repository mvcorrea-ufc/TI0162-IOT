# IoT Hardware Abstraction Layer (HAL)

A comprehensive platform-agnostic hardware abstraction layer providing unified interfaces for embedded IoT applications across different microcontroller architectures, with support for both real hardware implementations and mock testing environments.

## Overview

The `iot-hal` module defines common hardware interfaces and provides platform-specific implementations for various embedded microcontrollers (ESP32-C3, ARM Cortex-M, RISC-V, etc.) as well as mock implementations for testing. This enables consistent hardware access patterns across diverse IoT platforms and systems.

## Key Features

- **Platform-Agnostic Interfaces**: Common traits for I2C, GPIO, timers, and system resources across all platforms
- **Multiple Platform Support**: Implementations for ESP32-C3, ARM Cortex-M, RISC-V, and other embedded architectures
- **Mock Implementation**: Complete testing infrastructure with simulated hardware for any platform
- **Async Support**: Full async/await support for non-blocking hardware operations on supported platforms
- **Error Handling**: Comprehensive platform-independent error types and result handling
- **Resource Management**: Safe hardware resource allocation and lifecycle management across platforms
- **Testing Infrastructure**: Universal mock implementations for unit and integration testing

## Architecture

### Hardware Trait System

```rust
// Core platform-agnostic hardware traits
use iot_hal::{
    I2cBus, GpioPin, Timer, SystemInfo,
    HardwareProvider, HardwareError
};

// Platform-specific implementations
use iot_hal::esp32c3::Esp32C3Hardware;        // ESP32-C3 RISC-V implementation
use iot_hal::cortex_m::CortexMHardware;       // ARM Cortex-M implementation
use iot_hal::atmega::AtmegaHardware;          // AVR ATmega implementation
use iot_hal::riscv::RiscVHardware;            // Generic RISC-V implementation

// Universal mock implementation for testing any platform
use iot_hal::mock::MockHardware;
```

### Supported Hardware Interfaces

- **I2C Bus**: Platform-agnostic async I2C communication for sensors and peripherals
- **GPIO**: Universal digital input/output control with interrupt support
- **Timers**: Precise timing and delay operations across different timer architectures
- **System Info**: Hardware identification, status, and platform capabilities
- **Memory Management**: Cross-platform heap and stack monitoring
- **Power Management**: Platform-specific sleep modes and power optimization
- **ADC/DAC**: Analog-to-digital and digital-to-analog conversion interfaces
- **SPI**: Serial Peripheral Interface for high-speed device communication

## Module Structure

```
iot-hal/
├── src/
│   ├── lib.rs              # Main HAL traits and exports
│   ├── traits.rs           # Platform-agnostic hardware abstraction traits
│   ├── error.rs            # HAL-specific error types
│   ├── config.rs           # Hardware configuration structures
│   ├── esp32c3.rs          # ESP32-C3 RISC-V implementation
│   ├── cortex_m.rs         # ARM Cortex-M implementation  
│   ├── atmega.rs           # AVR ATmega implementation
│   ├── riscv.rs            # Generic RISC-V implementation
│   └── mock.rs             # Universal mock implementation for testing
├── Cargo.toml              # Module dependencies and features
└── README.md               # This documentation
```

## Hardware Abstractions

### I2C Bus Interface

```rust
use iot_hal::{I2cBus, I2cAddress};

#[async_trait]
pub trait I2cBus {
    async fn write(&mut self, address: I2cAddress, data: &[u8]) -> Result<(), HardwareError>;
    async fn read(&mut self, address: I2cAddress, buffer: &mut [u8]) -> Result<(), HardwareError>;
    async fn write_read(&mut self, address: I2cAddress, write_data: &[u8], read_buffer: &mut [u8]) -> Result<(), HardwareError>;
}
```

### GPIO Interface

```rust
use iot_hal::{GpioPin, PinState, PinDirection};

#[async_trait]
pub trait GpioPin {
    async fn set_direction(&mut self, direction: PinDirection) -> Result<(), HardwareError>;
    async fn set_state(&mut self, state: PinState) -> Result<(), HardwareError>;
    async fn get_state(&self) -> Result<PinState, HardwareError>;
    async fn enable_interrupt(&mut self, trigger: InterruptTrigger) -> Result<(), HardwareError>;
}
```

### Timer Interface

```rust
use iot_hal::{Timer, Duration};

#[async_trait]
pub trait Timer {
    async fn delay(&mut self, duration: Duration) -> Result<(), HardwareError>;
    async fn delay_ms(&mut self, milliseconds: u32) -> Result<(), HardwareError>;
    async fn delay_us(&mut self, microseconds: u32) -> Result<(), HardwareError>;
}
```

### System Information Interface

```rust
use iot_hal::{SystemInfo, ChipInfo, MemoryInfo};

#[async_trait]
pub trait SystemInfo {
    async fn chip_info(&self) -> Result<ChipInfo, HardwareError>;
    async fn memory_info(&self) -> Result<MemoryInfo, HardwareError>;
    async fn uptime_ms(&self) -> Result<u64, HardwareError>;
    async fn temperature(&self) -> Result<f32, HardwareError>;
}
```

## Implementation Examples

### Platform-Specific Hardware Providers

#### ESP32-C3 RISC-V Implementation
```rust
use iot_hal::esp32c3::{Esp32C3Hardware, Esp32C3Config};

let config = Esp32C3Config {
    i2c_sda_pin: 8, i2c_scl_pin: 9, i2c_frequency: 100_000, status_led_pin: 3,
};
let mut hardware = Esp32C3Hardware::new(config).await?;
```

#### ARM Cortex-M Implementation
```rust
use iot_hal::cortex_m::{CortexMHardware, CortexMConfig};

let config = CortexMConfig {
    i2c_instance: 1, sda_pin: PinId::PA8, scl_pin: PinId::PA9, led_pin: PinId::PC13,
};
let mut hardware = CortexMHardware::new(config).await?;
```

#### AVR ATmega Implementation
```rust
use iot_hal::atmega::{AtmegaHardware, AtmegaConfig};

let config = AtmegaConfig {
    twi_bitrate: 100_000, led_pin: PortPin::PB5,
};
let mut hardware = AtmegaHardware::new(config).await?;
```

#### Universal Usage Pattern
```rust
// All platforms use the same interface
let mut i2c = hardware.i2c_bus().await?;
i2c.write(0x76, &[0xD0]).await?; // Read BME280 chip ID

let mut led = hardware.gpio_pin(led_pin_id).await?;
led.set_direction(PinDirection::Output).await?;
led.set_state(PinState::High).await?;
```

### Mock Hardware for Testing

```rust
use iot_hal::mock::{MockHardware, MockConfig};

// Initialize mock hardware for testing
let config = MockConfig::default();
let mut hardware = MockHardware::new(config);

// Configure mock responses
hardware.i2c_bus().expect_read(0x76, vec![0x60]); // Mock BME280 chip ID

// Use same interfaces as real hardware
let mut i2c = hardware.i2c_bus().await?;
let mut buffer = [0u8; 1];
i2c.read(0x76, &mut buffer).await?;
assert_eq!(buffer[0], 0x60); // Verify mock response
```

### Hardware Provider Pattern

```rust
use iot_hal::{HardwareProvider, HardwareType};

// Abstract hardware provider
#[async_trait]
pub trait HardwareProvider {
    async fn i2c_bus(&mut self) -> Result<Box<dyn I2cBus>, HardwareError>;
    async fn gpio_pin(&mut self, pin: u8) -> Result<Box<dyn GpioPin>, HardwareError>;
    async fn timer(&mut self) -> Result<Box<dyn Timer>, HardwareError>;
    async fn system_info(&self) -> Result<Box<dyn SystemInfo>, HardwareError>;
    fn hardware_type(&self) -> HardwareType;
}

// Usage with dependency injection
async fn initialize_sensor<T: HardwareProvider>(hardware: &mut T) -> Result<(), HardwareError> {
    let mut i2c = hardware.i2c_bus().await?;
    // Sensor initialization logic...
    Ok(())
}
```

## Features and Build Configuration

### Feature Flags

```toml
# Platform-specific hardware support
iot-hal = { path = "../core/iot-hal", features = ["esp32c3"] }      # ESP32-C3 RISC-V
iot-hal = { path = "../core/iot-hal", features = ["cortex-m"] }     # ARM Cortex-M
iot-hal = { path = "../core/iot-hal", features = ["atmega"] }       # AVR ATmega
iot-hal = { path = "../core/iot-hal", features = ["riscv"] }        # Generic RISC-V

# Multiple platform support
iot-hal = { path = "../core/iot-hal", features = ["esp32c3", "cortex-m"] }

# Universal mock for testing any platform
iot-hal = { path = "../core/iot-hal", features = ["mock"] }

# All platforms with testing support
iot-hal = { path = "../core/iot-hal", features = ["esp32c3", "cortex-m", "atmega", "mock"] }
```

Available features:
- `esp32c3`: Enable ESP32-C3 RISC-V hardware implementation
- `cortex-m`: Enable ARM Cortex-M hardware implementation  
- `atmega`: Enable AVR ATmega hardware implementation
- `riscv`: Enable generic RISC-V hardware implementation
- `mock`: Enable universal mock hardware implementation for testing
- `default`: No specific platform (traits only)

### Dependencies by Feature

**ESP32-C3 Feature (`esp32c3`)**:
- esp-hal: ESP32-C3 hardware abstraction
- esp-hal-embassy: Embassy integration for ESP32-C3
- embassy-time: Async timing support
- static_cell: Static memory allocation

**Mock Feature (`mock`)**:
- tokio: Async runtime for testing
- Simulated hardware implementations

## Error Handling

```rust
use iot_hal::{HardwareError, HardwareResult};

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareError {
    // I2C errors
    I2cBusError(I2cErrorKind),
    I2cDeviceNotFound(u8),
    I2cTimeout,
    
    // GPIO errors
    GpioConfigurationError,
    GpioInvalidPin(u8),
    
    // Timer errors
    TimerNotAvailable,
    InvalidDuration,
    
    // System errors
    SystemResourceUnavailable,
    HardwareNotInitialized,
    
    // General errors
    ConfigurationError(String),
    OperationFailed(String),
}

pub type HardwareResult<T> = Result<T, HardwareError>;
```

## Testing Infrastructure

### Unit Testing with Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iot_hal::mock::MockHardware;
    
    #[tokio::test]
    async fn test_sensor_initialization() {
        let mut hardware = MockHardware::new(MockConfig::default());
        
        // Configure mock expectations
        hardware.i2c_bus()
            .expect_write_read(0x76, vec![0xD0], vec![0x60]);
        
        // Test sensor initialization
        let result = initialize_bme280_sensor(&mut hardware).await;
        assert!(result.is_ok());
    }
}
```

### Integration Testing

```rust
#[cfg(feature = "esp32c3")]
#[cfg(test)]
mod integration_tests {
    use super::*;
    use iot_hal::esp32c3::Esp32C3Hardware;
    
    #[tokio::test]
    async fn test_real_hardware_i2c() {
        let config = Esp32C3Config::default();
        let mut hardware = Esp32C3Hardware::new(config).await.unwrap();
        
        // Test real I2C communication
        let mut i2c = hardware.i2c_bus().await.unwrap();
        let result = i2c.write(0x76, &[0xD0]).await;
        
        // Verify hardware responds correctly
        assert!(result.is_ok());
    }
}
```

## Usage in Applications

### Main Application Integration

```rust
use iot_hal::esp32c3::{Esp32C3Hardware, Esp32C3Config};

#[esp_hal::main]
async fn main() -> ! {
    // Initialize hardware abstraction layer
    let hardware_config = Esp32C3Config {
        i2c_sda_pin: 8,
        i2c_scl_pin: 9,
        i2c_frequency: 100_000,
        status_led_pin: 3,
    };
    
    let mut hardware = Esp32C3Hardware::new(hardware_config)
        .await
        .expect("Failed to initialize hardware");
    
    // Initialize drivers using HAL
    let bme280 = BME280Driver::new(hardware.i2c_bus().await?).await?;
    let status_led = StatusLed::new(hardware.gpio_pin(3).await?).await?;
    
    // Application logic...
}
```

### Driver Implementation Using HAL

```rust
use iot_hal::{I2cBus, HardwareError};

pub struct BME280Driver {
    i2c: Box<dyn I2cBus>,
    address: u8,
}

impl BME280Driver {
    pub async fn new(mut i2c: Box<dyn I2cBus>) -> Result<Self, HardwareError> {
        let address = 0x76;
        
        // Verify chip ID using HAL
        let mut chip_id = [0u8; 1];
        i2c.write_read(address, &[0xD0], &mut chip_id).await?;
        
        if chip_id[0] != 0x60 {
            return Err(HardwareError::I2cDeviceNotFound(address));
        }
        
        Ok(Self { i2c, address })
    }
    
    pub async fn read_temperature(&mut self) -> Result<f32, HardwareError> {
        // Implementation using HAL I2C interface
        let mut temp_data = [0u8; 3];
        self.i2c.write_read(self.address, &[0xFA], &mut temp_data).await?;
        
        // Temperature calculation...
        Ok(temperature)
    }
}
```

## Performance Considerations

### Memory Usage

- **Trait Objects**: Minimal overhead using Box<dyn Trait>
- **Static Allocation**: ESP32-C3 implementation uses static_cell
- **Zero-Copy**: Interfaces designed for zero-copy operations where possible

### Async Performance

- **Non-blocking**: All operations use async/await for non-blocking execution
- **Embassy Integration**: Optimized for Embassy async runtime
- **Interrupt-driven**: GPIO and timer operations use interrupts when available

### Resource Management

- **RAII**: Automatic resource cleanup using Rust ownership
- **Exclusive Access**: Hardware resources protected by ownership rules
- **Graceful Degradation**: Fallback behavior for resource contention

---

**Module Type**: Core Infrastructure  
**Target Environment**: ESP32-C3 with Embassy async framework  
**Testing Support**: Complete mock infrastructure for unit testing  
**Integration**: Used by all hardware drivers and applications