# IoT Configuration Management

A comprehensive platform-agnostic configuration management system for embedded IoT applications providing unified configuration handling across different microcontroller platforms, environments, and storage backends.

## Overview

The `iot-config` module provides a centralized configuration system that supports multiple storage backends (embedded flash, file systems, memory), different deployment environments (development, testing, production), and modular feature configuration for IoT components across various embedded platforms.

## Key Features

- **Platform-Agnostic API**: Single interface for configuration operations across all embedded platforms
- **Multiple Storage Backends**: Support for embedded flash (ESP32-C3, STM32, etc.), file systems, EEPROM, and memory-based config
- **Cross-Platform Compatibility**: Works on ESP32-C3, ARM Cortex-M, AVR, RISC-V, and other embedded architectures
- **Environment-Specific**: Different configurations for development, testing, and production deployments
- **Feature Flags**: Modular configuration for WiFi, MQTT, LoRa, BLE, console, performance monitoring
- **No-std Compatible**: Designed for resource-constrained embedded systems with minimal memory overhead
- **Validation**: Built-in configuration validation and error handling across platforms
- **Serialization**: JSON-based configuration persistence with platform-specific optimizations

## Architecture

### Configuration Backends

```rust
// Platform-specific flash storage backends
iot_config::esp32c3::FlashConfig::load()?;          // ESP32-C3 NVS flash
iot_config::cortex_m::FlashConfig::load()?;         // ARM Cortex-M internal flash
iot_config::atmega::EepromConfig::load()?;          // AVR EEPROM storage

// File system backend (development/desktop)
iot_config::JsonConfig::from_file("config.json")?;

// Memory backend (testing/simulation)
iot_config::MemoryConfig::new()?;

// Universal backend (platform-agnostic)
iot_config::UniversalConfig::load()?;
```

### Environment Support

- **Development**: Local configuration with debug features enabled
- **Testing**: Controlled configuration for unit and integration tests  
- **Production**: Optimized configuration for deployment

### Feature Flags

The module supports granular feature configuration:

- `wifi` - WiFi connectivity configuration
- `bluetooth` - Bluetooth/BLE connectivity configuration  
- `lora` - LoRaWAN configuration for long-range communication
- `mqtt` - MQTT broker and messaging configuration
- `coap` - CoAP protocol configuration
- `console` - Serial/USB console and debugging configuration
- `performance` - Performance monitoring configuration
- `container` - Dependency injection container configuration
- `sensors` - Sensor-specific configuration (I2C, SPI, analog)
- `actuators` - Actuator and output device configuration

## Module Structure

```
iot-config/
├── src/
│   ├── lib.rs              # Main platform-agnostic configuration API
│   ├── unified.rs          # Unified configuration interface
│   ├── embedded.rs         # Generic embedded configuration
│   ├── flash_integration.rs # Platform-specific flash storage backends
│   ├── validation.rs       # Cross-platform configuration validation
│   ├── feature_flags.rs    # Feature flag management
│   ├── esp32c3.rs          # ESP32-C3 specific implementation
│   ├── cortex_m.rs         # ARM Cortex-M specific implementation
│   └── atmega.rs           # AVR ATmega specific implementation
├── config/
│   ├── default.json        # Default configuration template
│   ├── development.json    # Development environment config
│   ├── production.json     # Production environment config
│   └── platform/           # Platform-specific configuration templates
│       ├── esp32c3.json
│       ├── cortex_m.json
│       └── atmega.json
├── examples/
│   └── json_config_demo.rs # Configuration usage examples
├── Cargo.toml              # Module dependencies
└── README.md               # This documentation
```

## Usage Examples

### Basic Configuration Loading

```rust
use iot_config::{ConfigManager, ConfigBackend, Environment};

// Load configuration for current environment
let config = ConfigManager::load(Environment::Development)?;

// Access configuration values
let wifi_ssid = config.wifi().ssid();
let mqtt_broker = config.mqtt().broker_address();
```

### Environment-Specific Configuration

```rust
// Development configuration with debug features
let dev_config = ConfigManager::load_environment("development")?;

// Production configuration with optimizations
let prod_config = ConfigManager::load_environment("production")?;

// Testing configuration with mocks
let test_config = ConfigManager::load_environment("testing")?;
```

### Platform-Specific Flash Storage Backends

```rust
// ESP32-C3 NVS flash storage
use iot_config::esp32c3::Esp32C3Config;
let config = Esp32C3Config::load_from_nvs()?;
config.save_to_nvs()?;

// ARM Cortex-M internal flash storage  
use iot_config::cortex_m::CortexMConfig;
let config = CortexMConfig::load_from_flash()?;
config.save_to_flash()?;

// AVR ATmega EEPROM storage
use iot_config::atmega::AtmegaConfig;
let config = AtmegaConfig::load_from_eeprom()?;
config.save_to_eeprom()?;

// Generic embedded storage (platform-agnostic)
use iot_config::embedded::EmbeddedConfig;
let config = EmbeddedConfig::load_from_storage()?;
config.save_to_storage()?;
```

### Feature Flag Configuration

```rust
// Enable specific features based on configuration
if config.features().wifi_enabled() {
    initialize_wifi(&config.wifi())?;
}

if config.features().mqtt_enabled() {
    initialize_mqtt(&config.mqtt())?;
}

if config.features().performance_enabled() {
    initialize_performance_monitoring(&config.performance())?;
}
```

## Configuration Schema

### WiFi Configuration

```rust
pub struct WiFiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
    pub timeout_ms: u32,
    pub retry_attempts: u8,
}
```

### MQTT Configuration

```rust
pub struct MqttConfig {
    pub broker_address: String<64>,
    pub broker_port: u16,
    pub client_id: String<32>,
    pub username: Option<String<32>>,
    pub password: Option<String<64>>,
    pub keep_alive_seconds: u16,
}
```

### Performance Configuration

```rust
pub struct PerformanceConfig {
    pub monitoring_enabled: bool,
    pub sampling_interval_ms: u32,
    pub memory_tracking: bool,
    pub timing_analysis: bool,
}
```

## Dependencies

### Core Dependencies
- **iot-common**: Error handling and common utilities
- **serde**: Configuration serialization
- **serde-json-core**: JSON processing for no-std
- **heapless**: No-std collections

### Optional Dependencies
- **esp-storage**: ESP32-C3 flash storage backend (feature: `esp32c3`)

## Build Features

```toml
# Platform-specific configuration support
iot-config = { path = "../core/iot-config", features = ["esp32c3", "wifi", "mqtt"] }
iot-config = { path = "../core/iot-config", features = ["cortex-m", "bluetooth", "lora"] }
iot-config = { path = "../core/iot-config", features = ["atmega", "sensors"] }

# Multi-platform support
iot-config = { path = "../core/iot-config", features = ["esp32c3", "cortex-m", "wifi", "mqtt", "bluetooth"] }
```

Available platform features:
- `esp32c3`: Enable ESP32-C3 NVS flash storage backend
- `cortex-m`: Enable ARM Cortex-M flash storage backend
- `atmega`: Enable AVR ATmega EEPROM storage backend
- `storage`: Enable generic storage backends

Available protocol features:
- `wifi`: Include WiFi configuration support
- `bluetooth`: Include Bluetooth/BLE configuration support
- `lora`: Include LoRaWAN configuration support
- `mqtt`: Include MQTT configuration support
- `coap`: Include CoAP configuration support
- `console`: Include console configuration support
- `performance`: Include performance monitoring configuration
- `container`: Include dependency injection configuration
- `sensors`: Include sensor configuration support
- `actuators`: Include actuator configuration support

## Error Handling

The module uses the `iot-common` error system for consistent error handling:

```rust
use iot_config::{ConfigError, ConfigResult};

match config.load() {
    Ok(cfg) => println!("Configuration loaded successfully"),
    Err(ConfigError::ValidationFailed) => eprintln!("Configuration validation failed"),
    Err(ConfigError::StorageError) => eprintln!("Storage backend error"),
    Err(ConfigError::SerializationError) => eprintln!("JSON parsing error"),
}
```

## Testing

```bash
# Run all tests
cargo test

# Test specific features
cargo test --features "esp32c3,wifi,mqtt"

# Test with different backends
cargo test --features "storage"
```

## Integration

### With Main Applications

```rust
// In main applications
use iot_config::ConfigManager;

#[esp_hal::main]
async fn main() -> ! {
    let config = ConfigManager::load_embedded().expect("Failed to load configuration");
    
    // Initialize modules with configuration
    let wifi_manager = WiFiManager::new(config.wifi()).await?;
    let mqtt_client = MqttClient::new(config.mqtt()).await?;
    
    // Application logic...
}
```

### With IoT Container

```rust
// Register configuration in dependency injection container
container.register_singleton(|| ConfigManager::load_embedded())?;
```

## Development Guidelines

### Adding New Configuration Options

1. **Define the configuration struct** in appropriate module
2. **Add validation rules** in `validation.rs`
3. **Update JSON schemas** in `config/` directory
4. **Add feature flag** if component-specific
5. **Update documentation** and examples

### Configuration Best Practices

- **Use typed configuration**: Leverage Rust's type system for validation
- **Environment separation**: Keep development, testing, and production configs separate
- **Validation early**: Validate configuration at startup, not during operation
- **Feature flags**: Use feature flags to minimize binary size
- **Documentation**: Document all configuration options and their defaults

## Production Deployment

### Platform-Specific Storage Configuration

Different platforms use optimized storage mechanisms:

```rust
// ESP32-C3 with NVS flash storage
cargo build --release --features "esp32c3,storage,wifi,mqtt"

// ARM Cortex-M with internal flash
cargo build --release --features "cortex-m,storage,bluetooth,lora"

// AVR ATmega with EEPROM
cargo build --release --features "atmega,storage,sensors"

// Multi-platform build
cargo build --release --features "esp32c3,cortex-m,wifi,mqtt,bluetooth"
```

### Configuration Security

- **Sensitive data**: Use secure storage for credentials in production
- **Validation**: Always validate configuration before use
- **Defaults**: Provide sensible defaults for all optional parameters
- **Error recovery**: Implement fallback configuration for production

---

**Module Type**: Core Infrastructure  
**Target Environment**: Multi-platform embedded systems (ESP32-C3, ARM Cortex-M, AVR, RISC-V)  
**Integration**: Used by all IoT applications and modules across platforms  
**Dependencies**: iot-common, serde ecosystem, platform-specific storage backends