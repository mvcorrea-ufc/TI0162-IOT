# IoT Drivers

This directory contains all hardware drivers, connectivity modules, and interface implementations for the ESP32-C3 IoT system.

## Directory Structure

```
drivers/
├── bme280-embassy/          # BME280 sensor driver with Embassy async support
├── wifi-embassy/            # WiFi connectivity management
├── mqtt-embassy/            # MQTT client implementation  
├── serial-console-embassy/  # Console interface for system configuration
└── sensor-fusion/           # Multi-sensor support (future)
```

## Consolidated Architecture

According to Phase 3 architectural improvements, all drivers, connectivity, and interface modules have been consolidated into this single `drivers/` folder for:

- **Clear organization**: All hardware-related code in one location
- **Simplified dependencies**: Easier cross-module references
- **Better maintainability**: Single location for driver updates
- **Consistent patterns**: All drivers follow Embassy async patterns

## Available Modules

### Hardware Drivers

- **bme280-embassy**: Temperature, humidity, and pressure sensor
  - Async I2C interface using Embassy
  - Calibrated readings with compensation
  - Error handling and recovery

### Connectivity

- **wifi-embassy**: WiFi network management
  - WPA2/WPA3 support
  - Automatic reconnection
  - Network scanning and status

- **mqtt-embassy**: MQTT messaging client
  - QoS 0/1/2 support
  - Topic subscription/publishing
  - Connection management

### Interfaces

- **serial-console-embassy**: Interactive console system
  - Command-line interface
  - Configuration management
  - System diagnostics

## Usage

All modules follow consistent dependency patterns:

```toml
# In your Cargo.toml
bme280-embassy = { path = "../drivers/bme280-embassy" }
wifi-embassy = { path = "../drivers/wifi-embassy" }
mqtt-embassy = { path = "../drivers/mqtt-embassy" }
serial-console-embassy = { path = "../drivers/serial-console-embassy" }
```

## Building

```bash
# Build all drivers
cargo build --workspace

# Build specific driver
cargo build -p bme280-embassy
cargo build -p wifi-embassy
cargo build -p mqtt-embassy
cargo build -p serial-console-embassy

# Run examples
cargo run -p bme280-embassy --example basic_reading
cargo run -p wifi-embassy --example simple_connect
```

## Integration

All drivers are designed to work together in the main IoT application:

```rust
use bme280_embassy::BME280;
use wifi_embassy::WiFiManager;
use mqtt_embassy::MqttClient;
use serial_console_embassy::Console;

// Complete IoT system integration
```

See the `apps/main-app` for a complete integration example.