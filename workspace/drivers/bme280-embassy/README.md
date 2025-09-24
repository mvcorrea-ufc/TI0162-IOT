# BME280 Embassy - Temperature, Humidity and Pressure Sensor

## 🌡️ Description

Complete and functional module for asynchronous reading of the BME280 sensor using the Embassy framework for ESP32-C3. This module implements a custom BME280 driver with corrected value compensation and automatic calibration.

**Status**: ✅ Implemented and tested

## 🚀 Features

- ✅ **Async/Await**: All I2C operations are asynchronous via Embassy
- ✅ **Embassy Framework**: embassy-executor 0.7 + embassy-time 0.4
- ✅ **ESP32-C3**: esp-hal v1.0.0-rc.0 with unstable features
- ✅ **Automatic Calibration**: Reading and application of calibration coefficients
- ✅ **Corrected Compensation**: Validated compensation algorithms
- ✅ **Dual Address**: Supports I2C addresses 0x76 and 0x77
- ✅ **RTT Debugging**: Real-time output via rtt-target
- ✅ **LED Heartbeat**: Visual indication of operation

## 🔌 Hardware Pinout

```
ESP32-C3        BME280
--------        ------
GPIO8    <-->   SDA (I2C data)
GPIO9    <-->   SCL (I2C clock)
3.3V     <-->   VCC (power supply)
GND      <-->   GND (ground)
GPIO3    <-->   LED (status indicator)
```

### 📋 BME280 Specifications

- **Temperature**: -40°C to +85°C (accuracy ±1°C)
- **Humidity**: 0-100% RH (accuracy ±3%)
- **Pressure**: 300-1100 hPa (accuracy ±1 hPa)
- **I2C Addresses**: 0x76 (primary), 0x77 (secondary)
- **I2C Frequency**: 100kHz (standard)
- **Power Supply**: 3.3V
- **Consumption**: ~3.4μA (sleep mode)

## Project Structure

```
bme280-embassy/
├── src/
│   ├── main.rs          # Main application with Embassy tasks
│   ├── lib.rs           # Library module
│   ├── bme280.rs        # Async BME280 driver
│   └── i2c_device.rs    # Async I2C wrapper
├── examples/
│   └── basic_reading.rs # Module test example
├── Cargo.toml           # Embassy dependencies
└── build.rs             # ESP32-C3 build configuration
```

## Dependencies

```toml
# Base Embassy
embassy-executor = { version = "0.7", features = ["task-arena-size-20480"] }
embassy-time = "0.4.0"

# ESP32-C3 HAL + Embassy Integration  
esp-hal = { version = "0.23.1", features = ["esp32c3", "log"] }
esp-hal-embassy = { version = "0.6", features = ["esp32c3"] }

# I2C Async Support
embedded-hal-async = "1.0"
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust target for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs for flashing and debugging
cargo install probe-rs --features cli

# Verify ESP32-C3 connection
probe-rs list
```

## Build Instructions

### Building from Workspace Root
```bash
# Navigate to workspace root
cd workspace/

# Build bme280-embassy module from workspace
cargo build -p bme280-embassy --release

# Build specific examples from workspace
cargo build -p bme280-embassy --example basic_reading --release
cargo build -p bme280-embassy --example full_system --release

# Run examples from workspace (no main application - library only)
cargo run -p bme280-embassy --example basic_reading --release
cargo run -p bme280-embassy --example full_system --release
```

### Building from Module Folder
```bash
# Navigate to bme280-embassy module
cd workspace/bme280-embassy/

# Build main application from module folder
cargo build --release

# Build examples from module folder
cargo build --example basic_reading --release
cargo build --example full_system --release

# Run examples from module folder (no main application - library only)
cargo run --example basic_reading --release
cargo run --example full_system --release
```

### Integration into Your Project

#### Method 1: Add as Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
bme280-embassy = { path = "../bme280-embassy" }

# Required Embassy dependencies
embassy-executor = { version = "0.7", features = ["task-arena-size-20480"] }
embassy-time = "0.4.0"
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
embedded-hal-async = "1.0"
```

#### Method 2: Copy Source Files
```bash
# Copy BME280 driver to your project
cp workspace/bme280-embassy/src/bme280.rs your-project/src/
cp workspace/bme280-embassy/src/i2c_device.rs your-project/src/

# Add to your main.rs:
mod bme280;
mod i2c_device;
use bme280::BME280;
```

#### Method 3: Use as Library Module
```rust
// In your main.rs or lib.rs
use bme280_embassy::{BME280, Measurements};

#[embassy_executor::task]
async fn sensor_task(mut i2c: I2c<'static, esp_hal::peripherals::I2C0>) {
    let mut bme280 = BME280::new(&mut i2c);
    let measurements = bme280.read_measurements().await.unwrap();
    // Use temperature, humidity, pressure data
}
```

## Testing Instructions

### Hardware Setup Test
```bash
# 1. Connect BME280 sensor to ESP32-C3
# GPIO8 -> SDA, GPIO9 -> SCL, 3.3V -> VCC, GND -> GND

# 2. Verify hardware connection
probe-rs list  # Check ESP32-C3 detection
```

### Build Verification
```bash
# Test workspace build
cd workspace/
cargo check -p bme280-embassy
cargo build -p bme280-embassy --release
cargo build -p bme280-embassy --example basic_reading --release

# Test module build  
cd workspace/bme280-embassy/
cargo check
cargo build --release
cargo build --example basic_reading --release
```

### Runtime Testing
```bash
# Test main application
cargo run --release

# Expected output:
# BME280 Embassy: Sensor initialized successfully
# BME280 Embassy: T: 23.2°C, H: 68.5%, P: 1013.8 hPa

# Test basic hardware detection example
cargo run --example basic_reading --release
# Expected: I2C scan, chip ID verification, LED blinking

# Test complete sensor system example
cargo run --example full_system --release
# Expected: Full async sensor readings with debug info
```

### Integration Testing
```bash
# Test with other modules (example)
cd workspace/
cargo build -p main-app --release  # Integrated system test
```

### Code Quality
```bash
# Code verification
cargo clippy  # Check for warnings
cargo fmt     # Format code
cargo clean   # Clean build artifacts
```

### Expected Output

```
BME280 Embassy: Initializing BME280 sensor...
BME280 Embassy: Sensor initialized successfully
BME280 Embassy: T: 23.2°C, H: 68.5%, P: 1013.8 hPa
BME280 Embassy: T: 23.1°C, H: 68.3%, P: 1013.9 hPa
BME280 Embassy: T: 23.0°C, H: 68.7%, P: 1013.7 hPa
```

## Module API

### BME280 Driver

```rust
use bme280_embassy::{BME280, Measurements};

// Initialize
let mut bme280 = BME280::new(&mut i2c);

// Check sensor
let detected = bme280.check_id().await?;

// Read processed data
let measurements = bme280.read_measurements().await?;
println!("Temp: {:.2}°C", measurements.temperature);

// Read raw data
let (temp, press, hum) = bme280.read_raw_data().await?;
```

### Embassy Tasks

```rust
#[embassy_executor::task]
async fn sensor_task(mut i2c: I2c<'static, esp_hal::peripherals::I2C0>) {
    let mut bme280 = BME280::new(&mut i2c);
    
    loop {
        let data = bme280.read_measurements().await?;
        // Process data...
        Timer::after(Duration::from_secs(2)).await;
    }
}
```

## Development Standards

- **NO EMOJIS** in production code
- **esp-hal + Embassy** as standard stack  
- **async/await** for all I/O operations
- **embedded-hal-async** for abstraction
- **Task separation** for responsibilities

## 🐛 Troubleshooting

### Common Issues

1. **Sensor not responding (I2C timeout)**:
   ```bash
   # Check pinout
   # GPIO8 = SDA, GPIO9 = SCL
   # Check 3.3V power supply
   # Test continuity with multimeter
   ```

2. **Incorrect humidity values (0-100%)**:
   ```bash
   # Normal after implemented corrections
   # Compensation algorithm was fixed
   # Wait for stabilization (~30 seconds)
   ```

3. **Build fails**:
   ```bash
   cargo clean
   rustup target add riscv32imc-unknown-none-elf
   cargo build --release
   ```

4. **ESP32-C3 doesn't connect**:
   ```bash
   probe-rs list  # Check device
   # Press BOOT + RST if necessary
   # Check USB cable (data, not just charging)
   ```

### RTT Debug

```rust
// Add custom debug
rprintln!("BME280 Debug: Temp raw = {}", temp_raw);
rprintln!("BME280 Debug: Calibration T1 = {}", cal_data.dig_t1);
```

## 🔗 Integration with Other Modules

This module can be integrated with:

- **wifi-embassy**: For WiFi data transmission
- **mqtt-embassy**: For MQTT sensor publishing
- **web-server**: Web interface for visualization
- **main-app**: Complete IoT application

### Integration Example

```rust
// In main-app/src/main.rs
use bme280_embassy::{BME280, Measurements};
use wifi_embassy::WiFiManager;
use mqtt_embassy::MqttClient;

#[embassy_executor::task]
async fn sensor_mqtt_task() {
    let measurements = bme280.read_measurements().await?;
    let json_data = format_sensor_data(&measurements);
    mqtt_client.publish("esp32/sensor/bme280", &json_data).await?;
}
```

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>