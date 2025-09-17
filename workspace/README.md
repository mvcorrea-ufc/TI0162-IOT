# TI0162 - Internet of Things - Complete IoT Project

This workspace contains a complete and functional IoT project developed in Rust for ESP32-C3 using the Embassy framework. The system implements environmental data collection via BME280 sensor, WiFi connectivity, and MQTT transmission, forming a robust and modular IoT pipeline.

**Project Status**: ✅ Fully functional and operational IoT system

## 🏗️ Implemented Modular Architecture

### ✅ bme280-embassy/ - BME280 Sensor + Embassy
**Status**: Implemented and tested  
**Function**: Asynchronous reading of temperature, humidity, and pressure  
**Technology**: Embassy async + I2C + custom BME280  
**Hardware**: GPIO8(SDA), GPIO9(SCL), GPIO3(LED)  
**Output**: RTT debugging with compensated values

### ✅ wifi-embassy/ - WiFi Connectivity
**Status**: Implemented and tested  
**Function**: Robust WiFi connection with automatic reconnection  
**Technology**: Embassy + esp-wifi + DHCP  
**Tested IP**: 10.10.10.214  
**Features**: Complete network stack for TCP/UDP

### ✅ mqtt-embassy/ - MQTT Client
**Status**: Implemented and tested  
**Function**: Asynchronous MQTT publishing via TCP sockets  
**Technology**: Embassy + MQTT 3.1.1 protocol  
**Tested Broker**: 10.10.10.210:1883  
**Messages**: Structured JSON for sensors, status, and heartbeat

### ✅ Integrated System - Complete IoT Pipeline
**Status**: Operational and validated  
**Flow**: ESP32-C3 → BME280 → WiFi → MQTT → Mosquitto → Subscribers  
**Example**: wifi-embassy/examples/wifi_mqtt_test.rs  
**Periodicity**: 30s sensor, 2.5min heartbeat, 5min status

## 🚀 Quick Start - Complete IoT System

### Prerequisites

```bash
# Install Rust + ESP32-C3 target
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs
cargo install probe-rs --features cli

# Verify ESP32-C3 connection
probe-rs list
```

### Credentials Configuration

Each module has `.cargo/config.toml` for configuration via environment variables:

```toml
# Example: wifi-embassy/.cargo/config.toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
```

### Complete System Test

```bash
# 1. Test BME280 sensor
cd bme280-embassy/
cargo run --release

# 2. Test WiFi connectivity
cd ../wifi-embassy/
cargo run --example wifi_test_new --release

# 3. Setup MQTT broker
sudo apt install mosquitto mosquitto-clients
sudo systemctl start mosquitto

# 4. MQTT monitor (separate terminal)
mosquitto_sub -h [YOUR_IP] -p 1883 -t "esp32/#" -v

# 5. Complete IoT system
cargo run --example wifi_mqtt_test --release
```

## 📊 MQTT Published Data

### BME280 Sensor (esp32/sensor/bme280)
```json
{
  "temperature": 23.2,
  "humidity": 68.5,
  "pressure": 1013.8,
  "reading": 1
}
```

### Device Status (esp32/status)
```json
{
  "status": "online",
  "uptime": 300,
  "free_heap": 45000,
  "wifi_rssi": -42
}
```

### Heartbeat (esp32/heartbeat)
```
ping
```

## 📂 File Structure

```
workspace/
├── bme280-embassy/          # 🌡️ Temperature/humidity/pressure sensor
│   ├── src/                 # Custom BME280 driver + Embassy
│   ├── examples/            # Reading examples
│   └── README.md           # Detailed documentation
├── wifi-embassy/            # 📡 Robust WiFi connectivity
│   ├── src/                 # WiFi manager + Embassy network stack
│   ├── examples/            # WiFi tests + MQTT integration
│   └── README.md           # Detailed documentation
├── mqtt-embassy/            # 📨 Asynchronous MQTT client
│   ├── src/                 # MQTT client + JSON structures
│   ├── examples/            # MQTT tests
│   └── README.md           # Detailed documentation
├── examples/                # 📚 External reference projects
├── blinky/                 # 🏗️ Base template (basic esp-hal)
├── CLAUDE.md               # 📖 Complete project documentation
├── .gitignore              # Git exclusions (target/, logs, etc.)
└── README.md               # This documentation
```

## 🛠️ Technologies and Dependencies

### Main Technology Stack
- **Language**: Rust (stable)
- **Target**: riscv32imc-unknown-none-elf (ESP32-C3)
- **Async Framework**: Embassy (executor 0.7 + time 0.4)
- **HAL**: esp-hal 1.0.0-rc.0 (ESP32-C3 unstable features)
- **WiFi**: esp-wifi 0.15.0 + smoltcp network stack
- **Debugging**: RTT (Real-Time Transfer) via rtt-target

### Dependencies by Module

#### BME280 Embassy
```toml
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embedded-hal-async = "1.0"
```

#### WiFi Embassy
```toml
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
embassy-net = { version = "0.7", features = ["tcp", "udp", "dhcpv4"] }
esp-alloc = { version = "0.8.0" }
static_cell = "2.0"
```

#### MQTT Embassy
```toml
wifi-embassy = { path = "../wifi-embassy" }
embedded-io-async = "0.6"
serde = { version = "1.0", default-features = false }
serde-json-core = "0.6"
heapless = "0.8"
```

## 📋 Hardware Requirements

### ESP32-C3 DevKit
- **Microcontroller**: ESP32-C3 (RISC-V single-core 160MHz)
- **Connectivity**: WiFi 2.4GHz (does not support 5GHz)
- **GPIO**: 22 available digital pins
- **I2C**: GPIO8(SDA), GPIO9(SCL) for BME280 sensor
- **Power**: 3.3V via USB or external supply
- **Flash**: 4MB minimum recommended

### BME280 Sensor (Optional)
- **Interface**: I2C (address 0x76 or 0x77)
- **Measurements**: Temperature (-40°C to +85°C), Humidity (0-100% RH), Pressure (300-1100 hPa)
- **Precision**: ±1°C (temp), ±3% (humidity), ±1 hPa (pressure)
- **Power**: 3.3V, ~3.4μA sleep mode

### Network Infrastructure
- **WiFi**: 2.4GHz network with DHCP enabled
- **MQTT Broker**: Mosquitto or similar (tested: 10.10.10.210:1883)
- **Monitoring**: mosquitto_sub client to view messages

### Development Environment
- **OS**: Linux/macOS/Windows with USB support
- **Rust**: stable toolchain + target riscv32imc-unknown-none-elf
- **Debugging**: probe-rs for flash and RTT
- **USB**: Data cable (not just charging)

## 🔧 Development Commands

### Build and Flash (All Modules)
```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized, recommended for ESP32)
cargo build --release

# Flash + RTT monitor (main application)
cargo run --release

# Flash + RTT monitor (specific example)
cargo run --example [EXAMPLE_NAME] --release

# Clean artifacts
cargo clean

# Code verification
cargo clippy
cargo fmt
```

### Module-Specific Commands

#### BME280 Embassy
```bash
cd bme280-embassy/
cargo run --release                         # Main app
cargo run --example basic_reading --release # Basic test
```

#### WiFi Embassy
```bash
cd wifi-embassy/
cargo run --example wifi_test --release      # Basic WiFi test
cargo run --example wifi_test_new --release  # Detailed test
cargo run --example wifi_mqtt_test --release # Complete system
```

#### MQTT Embassy
```bash
cd mqtt-embassy/
cargo run --example mqtt_test_working --features examples --release
```

## 🔗 Implemented Integration Patterns

### Complete IoT System
The project demonstrates complete integration between all modules:

```rust
// Functional example in wifi-embassy/examples/wifi_mqtt_test.rs

// 1. Initialize Embassy + WiFi
let wifi_manager = WiFiManager::new(spawner, /* ... */).await?;
let stack = wifi_manager.get_stack();

// 2. Create sensor data (mock or real BME280)
let temperature = 23.5;
let humidity = 68.2;
let pressure = 1013.8;

// 3. Connect to MQTT broker
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
socket.connect(("10.10.10.210".parse().unwrap(), 1883)).await?;

// 4. Publish data via MQTT
let json_payload = format!(
    r#"{{"temperature":{:.1},"humidity":{:.1},"pressure":{:.1}}}"#,
    temperature, humidity, pressure
);
socket.write_all(&mqtt_publish_packet).await?;
```

### Established Code Patterns

#### Embassy Async Tasks
```rust
#[embassy_executor::task]
async fn sensor_task() {
    loop {
        let data = sensor.read().await;
        rprintln!("Sensor: {:?}", data);
        Timer::after(Duration::from_secs(30)).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    // Periodic MQTT publishing
}
```

#### Environment Configuration
```rust
const WIFI_SSID: &str = env!("WIFI_SSID", "Configure in .cargo/config.toml");
const MQTT_BROKER: &str = env!("MQTT_BROKER_IP", "192.168.1.100");
```

#### Robust Error Handling
```rust
match socket.connect(broker_addr).await {
    Ok(()) => rprintln!("✅ Connected to broker"),
    Err(e) => {
        rprintln!("❌ Connection failed: {:?}", e);
        return; // Retry on next cycle
    }
}
```

## 🐛 Troubleshooting

### Common Problems and Solutions

#### Hardware and Connectivity
1. **ESP32-C3 won't connect**:
   ```bash
   probe-rs list  # Should show the device
   # If not shown: check USB cable (data), press BOOT+RST
   ```

2. **WiFi won't connect**:
   ```bash
   # Check credentials in .cargo/config.toml
   # Confirm 2.4GHz network (ESP32-C3 doesn't support 5GHz)
   # Test case-sensitive SSID
   ```

3. **DHCP fails**:
   ```bash
   # Check router functioning
   # Confirm available DHCP pool
   # Test connectivity with another device
   ```

#### Development and Build
4. **Build fails**:
   ```bash
   cargo clean
   rustup target add riscv32imc-unknown-none-elf
   cargo build --release
   ```

5. **Embassy time driver error**:
   ```bash
   # Error: schedule_wake called before esp_hal_embassy::init()
   # Solution: Call esp_hal_embassy::init() before WiFiManager::new()
   ```

#### MQTT and Network
6. **MQTT broker unreachable**:
   ```bash
   ping 10.10.10.210
   telnet 10.10.10.210 1883
   sudo systemctl status mosquitto
   ```

7. **MQTT messages don't appear**:
   ```bash
   # Check topics: mosquitto_sub -h [BROKER] -t "#" -v
   # Debug packet format in ESP32 code
   ```

### Debug Strategies

#### RTT Debugging
```rust
// Add detailed debug
rprintln!("WiFi Status: {:?}", wifi_status);
rprintln!("IP Config: {:?}", stack.config_v4());
rprintln!("MQTT Packet: {:02X?}", &packet[..20]);
```

#### Modular Testing
```bash
# 1. Check basic hardware
cd blinky/ && cargo run --release

# 2. Test sensor (if available)
cd ../bme280-embassy/ && cargo run --release

# 3. Test WiFi in isolation
cd ../wifi-embassy/ && cargo run --example wifi_test_new --release

# 4. Complete system
cargo run --example wifi_mqtt_test --release
```

#### Network Monitor
```bash
# Terminal 1: MQTT Monitor
mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v

# Terminal 2: Run ESP32
cd wifi-embassy/
cargo run --example wifi_mqtt_test --release

# Terminal 3: Connectivity monitor
ping 10.10.10.214  # ESP32 IP
```

## 📈 Performance and Optimization

### Build and Runtime
- **Release mandatory**: Always use `--release` for ESP32-C3 (debug builds are very slow)
- **Heap allocation**: 72KB configured for WiFi operations
- **RTT minimal overhead**: RTT debugging has minimal performance impact
- **Network stack**: Operations are synchronous by design for MQTT compatibility

### IoT System Timing
- **Sensor data**: Publishing every 30 seconds
- **Heartbeat**: Every 2.5 minutes (5 cycles)
- **Device status**: Every 5 minutes (10 cycles)
- **WiFi reconnect**: Automatic on disconnection
- **MQTT reconnect**: New TCP connection each cycle (robust)

## 🔮 Future Expansion

### Planned Modules
- **web-server**: Web interface for real-time monitoring
- **main-app**: Final application integrating all modules
- **sensor-advanced**: Multiple I2C/SPI sensors
- **ble-simple**: Bluetooth Low Energy connectivity as backup

### Potential Improvements
- **Persistent MQTT**: Persistent MQTT connections (vs. reconnect each cycle)
- **Deep Sleep**: Power saving between readings
- **OTA Updates**: Over-the-air updates
- **Data buffering**: Local buffer for temporary disconnection cases
- **Time sync**: Time synchronization via NTP
- **TLS/SSL**: Secure MQTT connections

## 🎯 Project Status

### ✅ Implemented and Tested
- [x] BME280 sensor with corrected compensation
- [x] Robust WiFi connectivity with DHCP
- [x] Complete MQTT client with JSON
- [x] End-to-end functional IoT pipeline
- [x] Complete documentation of all modules
- [x] Functional examples for each component

### 📊 Validated Results
- **Hardware**: ESP32-C3 DevKit working perfectly
- **Sensor**: BME280 with accurate readings (T: 23°C, H: 68%, P: 1013hPa)
- **WiFi**: Stable connection with IP 10.10.10.214
- **MQTT**: Messages successfully delivered to broker 10.10.10.210:1883
- **Subscribers**: mosquitto_sub receiving structured JSON data

### 🏆 Achieved Objectives
1. **Modularity**: Each component works independently
2. **Robustness**: System resilient to disconnections and failures
3. **Scalability**: Architecture prepared for expansion
4. **Documentation**: Detailed READMEs in each module
5. **Testability**: Functional examples for validation

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>

**Project TI0162 - Internet of Things**  
**Complete IoT System with ESP32-C3 + Rust + Embassy**