# IoT Applications

This directory contains application-level code that integrates the core modules and drivers to create complete IoT solutions.

## Directory Structure

```
apps/
├── main-app/        # Main IoT environmental monitoring application
├── minimal-app/     # Minimal sensor-only application (future)
└── test-apps/       # Hardware testing applications (future)
```

## Architecture Overview

Applications in this directory integrate:

- **Core modules** (`../core/`): Configuration, storage, HAL abstractions
- **Drivers** (`../drivers/`): Hardware drivers and connectivity  
- **Common utilities** (`../iot-common/`): Error handling and shared code

## Available Applications

### main-app

The primary IoT environmental monitoring system that demonstrates the complete architecture:

**Features:**
- BME280 sensor data collection (temperature, humidity, pressure)
- WiFi connectivity with automatic reconnection
- MQTT publishing to cloud services
- Interactive serial console for configuration
- Performance monitoring and optimization
- Flash storage for configuration persistence

**Usage:**
```bash
# Build the main application
cargo build -p main-app --release

# Flash to ESP32-C3
cargo run -p main-app --release

# Monitor via RTT
cargo run -p main-app --release | rtt-target
```

**Architecture:**
```rust
// Simplified integration pattern
let sensor = BME280::new(i2c).await?;
let wifi = WiFiManager::new().await?;
let mqtt = MqttClient::new(wifi.get_stack()).await?;
let console = Console::new().await?;

// Main application loop
loop {
    let reading = sensor.read_all().await?;
    mqtt.publish("sensors/data", &reading).await?;
    console.process_commands().await?;
    embassy_time::Timer::after_secs(60).await;
}
```

## Future Applications

### minimal-app (Planned)

A lightweight sensor-only application for resource-constrained deployments:
- BME280 sensor only
- Local data logging
- Minimal flash usage (~150KB vs 243KB)
- Battery-optimized sleep patterns

### test-apps (Planned)

Hardware validation and testing applications:
- Individual driver tests
- Communication protocol verification
- Performance benchmarking
- Factory calibration

## Development Guidelines

### Application Structure

```rust
// apps/your-app/src/main.rs
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use esp_hal::entry;

// Import drivers and core modules
use bme280_embassy::BME280;
use wifi_embassy::WiFiManager;
use iot_common::IoTResult;

#[main]
async fn main(spawner: Spawner) -> ! {
    // Initialize hardware
    // Configure drivers  
    // Run application logic
}
```

### Dependency Management

Applications should use relative paths to workspace modules:

```toml
[dependencies]
# Drivers
bme280-embassy = { path = "../../drivers/bme280-embassy" }
wifi-embassy = { path = "../../drivers/wifi-embassy" }
mqtt-embassy = { path = "../../drivers/mqtt-embassy" }

# Core modules
iot-hal = { path = "../../iot-hal" }
iot-common = { path = "../../iot-common" }

# Workspace dependencies
esp-hal = { workspace = true }
embassy-executor = { workspace = true }
```

## Building Applications

```bash
# Build all applications
cargo build --workspace

# Build specific application
cargo build -p main-app --release

# Build with features
cargo build -p main-app --features "development" --release

# Cross-compilation for ESP32-C3
cargo build -p main-app --target riscv32imc-unknown-none-elf --release
```

## Deployment

Applications can be deployed using various methods:

```bash
# Direct flashing (development)
cargo run -p main-app --release

# Production deployment with Docker
docker-compose up deployment

# OTA updates (future)
./deploy.sh main-app production
```

See the individual application README files for specific build and deployment instructions.