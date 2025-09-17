# TI0162 - Internet of Things - Project

## Project Overview

This is an Internet of Things (IoT) project developed in Rust for ESP32-C3 using the Embassy framework. The project implements a modular sensing and connectivity system, focusing on environmental data collection and transmission via WiFi and MQTT.

**Implementation Base**: The project uses the `blinky` example as implementation base, leveraging its established configuration for ESP32-C3 with esp-hal.

## Technologies Used

- **Language**: Rust
- **Microcontroller**: ESP32-C3
- **HAL**: esp-hal v0.23.1 (Hardware Abstraction Layer)
- **Async Framework**: Embassy (async framework for embedded systems)
- **Base Template**: `blinky` project from rust-esp32-tmpl
- **Debugging**: RTT (Real-Time Transfer) via rtt-target
- **Sensor**: BME280 (temperature, humidity and pressure)
- **Connectivity**: WiFi + MQTT
- **MQTT Broker**: Mosquitto

## Architectural Decisions

### ⚡ esp-hal + Embassy (Chosen)
- **Advantages**: Lightweight, performant, full hardware control
- **Disadvantages**: More manual code, fewer abstractions
- **Usage**: Ideal for resource-constrained IoT projects

### ❌ esp-idf/FreeRTOS (Avoided)
- **Disadvantages**: Heavy framework, more memory overhead
- **Reason**: Unnecessary for simple IoT applications
- **Impact**: Reduces available resources for application logic

## Modular Architecture

The project was structured modularly based on the `blinky` template, expanding its functionalities:

```
workspace/
├── blinky/                 # 🏗️ BASE - Original template with esp-hal and RTT
├── examples/               # 📚 REFERENCES - Existing BME280 examples
│   └── simple-bme280-02/   #     BME280 driver with embedded-hal 1.0
├── bme280-embassy/         # 🌡️ IMPLEMENTED - BME280 + Embassy async
├── wifi-embassy/           # 📡 IMPLEMENTED - WiFi connectivity using Embassy
├── mqtt-embassy/           # 📨 IMPLEMENTED - MQTT client using Embassy
├── serial-console-embassy/ # 💻 IMPLEMENTED - Interactive serial console
├── web-server/             # 🌐 Web server for data display
└── main-app/               # 🎯 Main application integrating all modules
```

### Base Structure (blinky)
- **Cargo.toml**: Configuration with esp-hal 0.23.1, rtt-target for debugging
- **main.rs**: Base structure with RTT initialization and main loop
- **build.rs**: Linking configuration necessary for ESP32-C3
- **Functionality**: LED blinking with RTT output for validation

## Implemented Features

### 1. ✅ BME280 Embassy Module (`bme280-embassy/`)
- **Status**: Complete and functional
- Asynchronous reading of temperature (°C), humidity (%) and pressure (hPa) 
- Asynchronous I2C interface via Embassy
- Automatic BME280 sensor calibration
- Value compensation with corrected algorithms
- RTT output for debugging

### 2. ✅ WiFi Embassy Module (`wifi-embassy/`)
- **Status**: Complete and functional
- Automatic WiFi connection with credentials via environment variables
- IP acquisition via DHCP (tested: 10.10.10.214)
- Automatic reconnection in case of disconnection
- Embassy network stack with TCP/UDP support
- Simplified WiFi management interface

### 3. ✅ MQTT Embassy Module (`mqtt-embassy/`)
- **Status**: Complete and functional  
- Asynchronous MQTT client via Embassy TCP sockets
- Configurable broker support (tested: 10.10.10.210:1883)
- JSON publication of sensor data, status and heartbeat
- Configuration via environment variables
- Complete MQTT 3.1.1 protocol

### 4. ✅ Serial Console Embassy Module (`serial-console-embassy/`)
- **Status**: Complete and functional
- Interactive serial interface via UART using Embassy async
- Command system for configuration and monitoring
- Dynamic configuration of WiFi and MQTT credentials
- Real-time system information display
- Robust command parser with validation

### 5. ✅ WiFi + MQTT Integration
- **Status**: Pilot deployment ready with real connectivity
- Complete ESP32-C3 → WiFi → MQTT → Broker system
- Real WiFi radio communication and TCP socket MQTT publishing
- Periodic data publication (30s sensor, 6min heartbeat, 12min status)
- Structured JSON with real BME280 sensor data
- Zero mock data - fully functional IoT pipeline

## Data Structure

### MQTT Payload (JSON)
```json
{
  "timestamp": "2025-01-15T10:30:00Z",
  "sensor": "BME280",
  "data": {
    "temperature": 23.5,
    "humidity": 65.2,
    "pressure": 1013.25
  }
}
```

## 📖 Module and Examples Usage Guide

### Prerequisites

1. **Rust toolchain**: `rustup target add riscv32imc-unknown-none-elf`
2. **probe-rs**: `cargo install probe-rs --features cli`
3. **ESP32-C3**: Connected via USB with drivers installed
4. **WiFi**: Available network for connectivity tests
5. **MQTT Broker**: Mosquitto or similar for MQTT tests

### 🌡️ BME280 Embassy Module

**Location**: `bme280-embassy/`

```bash
# Navigate to module
cd bme280-embassy/

# Execute basic BME280 reading
cargo run --example basic_reading --release

# Execute main application (continuous reading)
cargo run --release
```

**Hardware Configuration**:
- BME280: SDA=GPIO8, SCL=GPIO9
- Status LED: GPIO3
- I2C Frequency: 100kHz

**Expected Output**:
```
BME280 Embassy: Sensor initialized successfully
BME280 Embassy: T: 23.2°C, H: 68.5%, P: 1013.8 hPa
BME280 Embassy: T: 23.1°C, H: 68.3%, P: 1013.9 hPa
```

### 📡 WiFi Embassy Module

**Location**: `wifi-embassy/`

**Configuration**: Edit `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
```

```bash
# Navigate to module
cd wifi-embassy/

# Basic WiFi connectivity test
cargo run --example wifi_test --release

# Complete test with network information
cargo run --example wifi_test_new --release

# WiFi + MQTT integration (requires MQTT broker)
cargo run --example wifi_mqtt_test --release
```

**Expected Output**:
```
WiFi Embassy: Connected to WiFi!
📍 IP Address: 10.10.10.214
🌐 Gateway: Some(10.10.10.1)
🔧 Subnet: /24
```

### 💻 Serial Console Embassy Module

**Location**: `serial-console-embassy/`

```bash
# Navigate to module
cd serial-console-embassy/

# Basic console (without integration)
cargo run --example basic_console --release

# Complete console with IoT (requires modules)
cargo run --example system_console --features full --release
```

**Serial Interface**: UART0 at 115200 baud
- **TX**: GPIO1 (connect to RX of USB-serial converter)
- **RX**: GPIO3 (connect to TX of USB-serial converter) 
- **GND**: Common between ESP32-C3 and converter

**Available Commands**:

```bash
# System commands
help, h, ?          # Show help
status, stat        # System status
info, i             # Detailed information
clear, cls          # Clear screen
restart, reset      # Restart system

# WiFi commands
wifi show           # Show WiFi configuration
wifi ssid <name>    # Configure SSID
wifi pass <password> # Configure password

# MQTT commands  
mqtt show           # Show MQTT configuration
mqtt broker <ip>    # Configure broker IP
mqtt port <port>    # Configure port
mqtt client <id>    # Configure client ID
mqtt prefix <pfx>   # Configure topic prefix

# Configuration commands
save                # Save config to flash
load                # Load config from flash
```

**Session Example**:
```
esp32> help
=== ESP32-C3 IoT System Console ===
Available commands:
[command list...]

esp32> status
=== System Status ===
WiFi: Connected (10.10.10.214)
MQTT: Connected
Sensor: Active

esp32> wifi ssid MyNetwork
WiFi SSID set to: MyNetwork

esp32> mqtt broker 192.168.1.100
MQTT broker set to: 192.168.1.100

esp32> save
Configuration saved to flash
```

### 📨 MQTT Embassy Module

**Location**: `mqtt-embassy/`

**Configuration**: Edit `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
MQTT_BROKER_IP = "192.168.1.100"  # Your broker IP
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-test"
MQTT_TOPIC_PREFIX = "esp32"
```

```bash
# Navigate to module
cd mqtt-embassy/

# Basic MQTT test (requires wifi-embassy)
cargo run --example mqtt_test_working --features examples --release
```

**MQTT Monitor** (separate terminal):
```bash
# Monitor all ESP32 messages
mosquitto_sub -h [YOUR_BROKER_IP] -p 1883 -t "esp32/#" -v

# Monitor specific topic
mosquitto_sub -h [YOUR_BROKER_IP] -p 1883 -t "esp32/sensor/bme280" -v
```

**Published MQTT Messages**:
```json
// esp32/sensor/bme280
{"temperature":23.5,"humidity":68.2,"pressure":1013.8,"reading":1}

// esp32/status  
{"status":"online","uptime":300,"free_heap":45000,"wifi_rssi":-42}

// esp32/heartbeat
ping
```

### 🚀 Integrated System (WiFi + MQTT)

**Recommended Example**: `wifi-embassy/examples/wifi_mqtt_test.rs`

```bash
cd wifi-embassy/
cargo run --example wifi_mqtt_test --release
```

**Features**:
- ✅ Automatic WiFi connection
- ✅ MQTT publication every 30 seconds  
- ✅ Heartbeat every 5 cycles (2.5 minutes)
- ✅ Device status every 10 cycles (5 minutes)
- ✅ Automatic WiFi and MQTT reconnection
- ✅ Structured JSON according to specification

### 🔧 Development Commands

```bash
# Build only (no flash)
cargo build --release

# Build and flash with RTT monitor
cargo run --release

# Build specific example
cargo build --example [EXAMPLE_NAME] --release

# Flash specific example
cargo run --example [EXAMPLE_NAME] --release

# Linting
cargo clippy

# Formatting
cargo fmt

# Cleanup
cargo clean
```

### 🐛 Debugging and Troubleshooting

**RTT Debugging**:
- All applications use `rtt-target` for real-time output
- Use `rprintln!()` instead of `println!()`
- Monitor via probe-rs automatically

**Common Issues**:

1. **WiFi doesn't connect**: Check credentials in `.cargo/config.toml`
2. **MQTT doesn't publish**: Check broker IP and firewall
3. **BME280 doesn't respond**: Check I2C pinout (SDA=GPIO8, SCL=GPIO9)
4. **Build fails**: Run `cargo clean` and try again

**Hardware Verification**:
```bash
# Check if ESP32-C3 is connected
probe-rs list

# Check Rust target
rustup target list | grep riscv32imc
```

### Debugging Structure (RTT)
The project uses RTT (Real-Time Transfer) for real-time debugging:
- **rtt-target**: Log output via RTT
- **panic-rtt-target**: Panic handler via RTT  
- **rprintln!()**: Macro for RTT print (replaces println!)

### Base Configuration (blinky) → Embassy Migration

**Current Dependencies (blinky)**:
```toml
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
esp-rom-sys = { version = "0.1", features = ["esp32c3"] }
defmt = "0.3"
rtt-target = "0.5"
panic-rtt-target = "0.1"
```

**Embassy Dependencies (2025)**:
```toml
# Base Embassy
embassy-executor = { version = "0.7", features = ["task-arena-size-20480"] }
embassy-time = "0.4.0"

# ESP32-C3 HAL + Embassy Integration  
esp-hal = { version = "0.23.1", features = ["esp32c3", "log"] }
esp-hal-embassy = { version = "0.6", features = ["esp32c3"] }

# WiFi Support
esp-wifi = { git = "https://github.com/esp-rs/esp-hal", features = ["esp32c3", "wifi", "embassy-net"] }

# Utilities
esp-backtrace = { version = "0.15.0", features = ["esp32c3", "exception-handler", "panic-handler", "println"] }
esp-println = { version = "0.13.0", features = ["esp32c3", "log"] }

# I2C Async Support
embedded-hal-async = "1.0"
```

### Key Differences
- **asynchronous executor** via embassy-executor
- **asynchronous timer** via embassy-time  
- **asynchronous I2C** via embedded-hal-async
- **asynchronous WiFi** via esp-wifi with embassy-net
- **integration** via esp-hal-embassy

## Development Conventions

### Commits
- **DO NOT mention Claude Code** in commits
- Commits should be written as if done manually
- Focus on what was implemented and why
- Concise and descriptive messages

### Build and Debug Tools
- **probe-rs**: Mandatory tool for flashing (not espflash)
- **RTT**: Mandatory system for debugging (not esp-println)
- **picocom**: For serial console access via /dev/ttyACM0

### Documentation
- **Mandatory README.md**: Every module MUST have a README.md file inside its folder
- **Language**: ALL documentation must be written in **ENGLISH**
- **Diagrams**: ALL diagrams must use **Mermaid** syntax (no ASCII art, no external tools)
- **No Emojis**: NO emojis in code or documentation (use clear text labels instead)
- **Exhaustive documentation**: The README.md must contain:
  - Complete description of the module and its functionalities
  - Installation and usage instructions
  - Code examples and commands
  - Hardware configuration (pinout, connections)
  - Common problem troubleshooting
  - Architecture and file structure
  - Dependencies and available features
- **Practical examples**: Include real usage sessions with commands and expected outputs
- **Implementation status**: Clearly mark what is implemented and tested

## Analysis of Existing Examples

### 📚 simple-bme280-02 Project (Analysis)
**Found structure**:
- Custom BME280 driver with embedded-hal 1.0
- Synchronous I2C implementation using esp-hal
- Basic (simplified) sensor calibration
- Modular interface with I2cDevice wrapper

**Key Points**:
- I2C addresses: 0x76 (primary), 0x77 (secondary)
- Registers: Temperature(0xFA), Pressure(0xF7), Humidity(0xFD)
- Expected Chip ID: 0x60
- Configured GPIO: SDA=GPIO8, SCL=GPIO9

### 🔍 Embassy Research (GitHub + 2025)
**Reference Project**: `claudiomattera/esp32c3-embassy`
- ESP32-C3 + BME280 + Embassy + I2C async
- Updated dependencies for 2025
- Complete implementation with deep sleep
- WiFi time synchronization

**Validated Embassy Dependencies (2025)**:
- embassy-executor 0.7 + task-arena-size-20480
- embassy-time 0.4.0  
- esp-hal 0.23.1 + esp32c3 features
- esp-hal-embassy 0.6 + esp32c3 features
- embedded-hal-async 1.0

## Complete Modular IoT System Implementation

### 🌡️ bme280-embassy Module (Implemented)
**Characteristics**:
- **Base**: Blinky template migrated to Embassy
- **Async Tasks**: Sensor reading + LED heartbeat
- **Async I2C**: embedded-hal-async + bme280-rs crate
- **Hardware**: GPIO8(SDA), GPIO9(SCL), GPIO3(LED)
- **Timing**: Readings every 30 seconds for IoT integration

### 📡 wifi-embassy Module (Implemented)
**Characteristics**:
- **WiFi Management**: Robust connection with auto-reconnection
- **Network Stack**: Complete TCP/UDP via Embassy networking
- **DHCP Integration**: Automatic IP configuration
- **Error Recovery**: Automatic retry on connection failures
- **Tested Networks**: Home and enterprise WiFi environments

### 📨 mqtt-embassy Module (Implemented)
**Characteristics**:
- **MQTT 3.1.1 Client**: Asynchronous broker communication
- **JSON Publishing**: Structured sensor data transmission
- **Message Types**: Sensor data, device status, heartbeat
- **Broker Compatibility**: Mosquitto, AWS IoT, Azure IoT
- **Automatic Reconnection**: Network failure recovery

### 💻 serial-console-embassy Module (Implemented)
**Characteristics**:
- **USB Serial/JTAG Console**: Direct WeAct ESP32-C3 USB-C access
- **Interactive Commands**: System monitoring and configuration
- **Real-time Status**: Live module status and sensor readings
- **No External Hardware**: Built-in USB Serial/JTAG controller
- **Command Interface**: help, status, info, sensor, wifi, mqtt

### 🚀 main-app Integrated System (Pilot Ready)
**Complete IoT System Architecture - Real Hardware Implementation**:

**Task Architecture**:
```rust
#[embassy_executor::task]
async fn sensor_task()        - BME280 readings every 30s
#[embassy_executor::task] 
async fn wifi_task()          - WiFi connection management
#[embassy_executor::task]
async fn mqtt_task()          - MQTT publishing pipeline
#[embassy_executor::task]
async fn console_task()       - USB Serial/JTAG interface
#[embassy_executor::task]
async fn system_monitor_task() - Health monitoring
#[esp_hal::main]
async fn main()               - Integrated system spawner
```

**Pilot Deployment Features**:
- **Real Hardware Integration**: Actual BME280 I2C sensor communication
- **Real Network Connectivity**: WiFi radio and TCP socket MQTT publishing
- **Production Error Handling**: Automatic reconnection and fault recovery
- **Environment Configuration**: WiFi and MQTT settings via config files
- **Zero Mock Data**: All sensor readings and network operations are real
- **Live System Monitoring**: Real-time status via USB Serial/JTAG console
- **Clean Build**: Zero warnings, zero compilation errors

**Real Data Flow Pipeline**:
```
Real BME280 Hardware → I2C GPIO8/9 → sensor_task() → Embassy Signal → mqtt_task() → TCP Socket → Real MQTT Broker
                                        ↓
                USB Serial/JTAG ← console_task() ← SYSTEM_STATE ← WiFi Status ← WiFi Radio
```

**Inter-module Communication Patterns**:
- **Embassy Signals**: Async data passing between tasks
- **Shared Mutex State**: System status coordination across modules
- **Event-driven Architecture**: Tasks respond to state changes
- **Non-blocking Operations**: All operations fully asynchronous
- **Resource Ownership**: Clear hardware peripheral allocation

### Established Code Patterns
- **NO EMOJIS** in production code (only in documentation)
- **esp-hal + Embassy** as standard stack (not esp-idf)
- **async/await** for all I/O operations
- **embedded-hal-async** for hardware abstraction
- **Task separation** for distinct responsibilities
- **Pluggable Architecture**: Each module independently testable
- **Signal-based IPC**: Embassy signals for inter-task communication
- **Shared State Management**: Mutex-protected system coordination
- **Resource Ownership**: Clear hardware resource allocation per module