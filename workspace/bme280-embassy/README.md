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

### Build Commands

```bash
# Navigate to the module
cd bme280-embassy/

# Build only (check compilation)
cargo build --release

# Build + Flash + Monitor (main application)
cargo run --release

# Build + Flash + Monitor (basic example)
cargo run --example basic_reading --release

# Build cleanup
cargo clean

# Code verification
cargo clippy
cargo fmt
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