# Supporting Modules Collection

This directory contains standalone ESP32-C3 modules that build independently from the main workspace. These modules serve as examples, templates, testing tools, and reference implementations for ESP32-C3 embedded development.

## Directory Structure

```
supporting/
├── modules/           # Standalone Rust modules for ESP32-C3
├── scripts/          # Automation and utility scripts
├── config/           # Configuration files
└── README.md         # This documentation
```

## Modules Overview

All modules in the `modules/` folder are **completely standalone** and build independently without workspace dependencies. They use direct dependency specifications and can be copied to other projects as templates.

### 🧪 Testing & Validation Modules

#### 1. bme280-algorithms-test
**Purpose**: Host-based testing for BME280 sensor algorithms  
**Target**: Development machine (x86_64)  
**Dependencies**: Zero - pure algorithm testing  

**Features**:
- Tests BME280 temperature, humidity, pressure calculation algorithms
- Validates mathematical compensation formulas  
- Fast execution on development machine
- Comprehensive test coverage for all sensor algorithms

**Build Instructions**:
```bash
cd supporting/modules/bme280-algorithms-test
cargo test                    # Run all algorithm tests
cargo test -- --nocapture     # Run with verbose output
```

#### 2. esp32c3-hardware-validator (formerly blinky)
**Purpose**: Hardware validation template and project starter  
**Target**: ESP32-C3 hardware  
**Dependencies**: Minimal - esp-hal, RTT debugging  

**Features**:
- Basic GPIO control (LED blinking)
- RTT console output for debugging
- Hardware validation tool
- Template for new ESP32-C3 projects

**Build Instructions**:
```bash
cd supporting/modules/esp32c3-hardware-validator
cargo build --release         # Build the project
cargo run --release          # Flash and run on ESP32-C3
```

**Expected Output**:
```
esp32-c3 is booting!
status: High
status: Low
status: High
...
```

### 🌐 WiFi Library Modules

#### 3. esp32c3-blocking-wifi (formerly wifi-synchronous)
**Purpose**: Synchronous WiFi connectivity for traditional programming  
**Target**: ESP32-C3 hardware  
**Dependencies**: esp-hal, blocking-network-stack, esp-wifi  

**Features**:
- Traditional blocking programming model
- DHCP IP address acquisition
- Network scanning capabilities
- Alternative to async Embassy framework

**Build Instructions**:
```bash
# Configure WiFi credentials in .cargo/config.toml
cd supporting/modules/esp32c3-blocking-wifi
cargo build --release
cargo run --example simple_wifi_sync --release
```

#### 4. esp32c3-embassy-wifi (formerly wifi-simple-embassy)
**Purpose**: Embassy async WiFi library  
**Target**: ESP32-C3 hardware  
**Dependencies**: Embassy framework, esp-hal, esp-wifi  

**Features**:
- Embassy async framework
- Non-blocking WiFi operations
- Clean, maintainable API
- Modern async/await programming

**Build Instructions**:
```bash
cd supporting/modules/esp32c3-embassy-wifi
cargo build --release
cargo run --example basic_wifi --release
```

#### 5. esp32c3-modular-wifi (formerly wifi-simple-must-working)
**Purpose**: Modular WiFi toolkit with helper functions  
**Target**: ESP32-C3 hardware  
**Dependencies**: esp-hal, esp-wifi, helper modules  

**Features**:
- Modular helper functions
- Clean separation of concerns
- Network stack management
- TCP/MQTT application compatibility

**Build Instructions**:
```bash
cd supporting/modules/esp32c3-modular-wifi
cargo build --release
cargo run --release  # Run main application
```

### 🚀 Complete IoT Systems

#### 6. esp32c3-environmental-monitor (formerly simple-iot)
**Purpose**: Complete IoT system with BME280 sensor + WiFi + MQTT  
**Target**: ESP32-C3 hardware with BME280 sensor  
**Dependencies**: Full IoT stack - Embassy, WiFi, MQTT, BME280  

**Features**:
- Complete environmental monitoring system
- BME280 temperature/humidity/pressure sensing
- WiFi connectivity with automatic reconnection
- MQTT data publishing in JSON format
- Two operational modes: console-only and full IoT

**Hardware Requirements**:
- ESP32-C3 development board
- BME280 sensor module connected via I2C (GPIO8=SDA, GPIO9=SCL)

**Build Instructions**:
```bash
# Configure WiFi and MQTT in .cargo/config.toml
cd supporting/modules/esp32c3-environmental-monitor

# Simple sensor reading (no WiFi)
cargo run --bin bme280_simple --release

# Complete IoT system (WiFi + MQTT)
cargo run --bin bme280_mqtt --release
```

**Expected Output (IoT mode)**:
```
🚀 ESP32-C3 Simple IoT System - OPTIMIZED
========================================
📡 WiFi SSID: YourNetwork
📡 MQTT Broker: 192.168.1.100:1883
🌡️ Sensor: External BME280/BMP280 on GPIO8/GPIO9
✅ WiFi manager initialized
[WIFI] ✅ Connected to WiFi!
✅ MQTT test successful!
[SENSOR] ✅ External sensor found at address 0x76
[BME280] ✅ REAL BME280 DATA: T=32.7°C, H=53.5%, P=1013.1hPa
[MQTT] ✅ Published REAL BME280 data
🎯 All tasks started - REAL BME280 data only
```

## General Build Requirements

### Development Environment
- **Rust**: Latest stable version with `riscv32imc-unknown-none-elf` target
- **probe-rs**: For flashing and debugging ESP32-C3
- **Hardware**: ESP32-C3 development board

### Setup Instructions
```bash
# Install Rust target for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs for flashing
cargo install probe-rs --features cli

# Verify ESP32-C3 connection
probe-rs list
```

### Common Build Commands
```bash
# Navigate to any module
cd supporting/modules/[module-name]

# Check code without building
cargo check --release

# Build for ESP32-C3
cargo build --release

# Flash and run on hardware
cargo run --release

# Build specific binary (for modules with multiple binaries)
cargo run --bin [binary-name] --release

# Build and run examples
cargo run --example [example-name] --release
```

## WiFi Configuration

Most WiFi-enabled modules require configuration in `.cargo/config.toml`:

```toml
[env]
# WiFi Network Settings
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"

# MQTT Broker Settings (for IoT modules)
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-device"
MQTT_TOPIC_PREFIX = "esp32"
```

## Module Independence Benefits

### Standalone Advantages
- **No Workspace Conflicts**: Each module has its own dependency versions
- **Easy Copying**: Can be copied to other projects as templates
- **Educational**: Perfect for learning ESP32-C3 development
- **Rapid Prototyping**: Quick setup for new projects
- **Version Isolation**: No dependency conflicts with main workspace

### Template Usage
Any module can be used as a project template:

```bash
# Copy module to new project
cp -r supporting/modules/esp32c3-hardware-validator ../my-new-project/
cd ../my-new-project/

# Customize for your needs
# - Update Cargo.toml with your project details
# - Modify src/main.rs with your application logic
# - Adjust .cargo/config.toml for your hardware
```

## Module Categories by Use Case

### 🎓 Learning & Education
- **esp32c3-hardware-validator**: Start here for basic ESP32-C3 concepts
- **esp32c3-environmental-monitor**: Complete IoT system example

### 🔧 Development Tools
- **bme280-algorithms-test**: Algorithm validation and testing
- **esp32c3-hardware-validator**: Hardware and environment validation

### 📚 Library Examples
- **esp32c3-blocking-wifi**: Traditional synchronous programming
- **esp32c3-embassy-wifi**: Modern async programming
- **esp32c3-modular-wifi**: Modular architecture patterns

### 🚀 Production Templates
- **esp32c3-environmental-monitor**: Full-featured IoT application
- **esp32c3-modular-wifi**: Scalable WiFi architecture

## Troubleshooting

### Common Issues

#### Module Won't Build
```bash
# Clean and retry
cargo clean
cargo build --release
```

#### ESP32-C3 Not Detected
```bash
# Check hardware connection
probe-rs list

# Verify device drivers are installed
```

#### WiFi Connection Fails
- Verify SSID and password in `.cargo/config.toml`
- Ensure 2.4GHz network (ESP32-C3 doesn't support 5GHz)
- Check network availability

#### Sensor Not Detected (for IoT modules)
- Verify I2C wiring (SDA=GPIO8, SCL=GPIO9)
- Check sensor power supply (3.3V)
- Try alternative I2C addresses (0x76, 0x77)

### Development Tips

1. **Start Simple**: Begin with `esp32c3-hardware-validator` to verify setup
2. **Test Incrementally**: Build complexity gradually
3. **Use RTT**: All modules support RTT debugging for real-time output
4. **Check Examples**: Each module includes working examples
5. **Monitor Output**: Use `cargo run --release` to see debug output

## Integration with Main Workspace

While these modules are standalone, they can inform and complement the main workspace:

- **Algorithm Testing**: Use `bme280-algorithms-test` to validate algorithms used in production drivers
- **Template Code**: Copy patterns from these modules to main workspace modules
- **Learning**: Understand concepts before implementing in production code
- **Rapid Prototyping**: Test ideas quickly before integration

## Contributing

When adding new supporting modules:

1. **Make it Standalone**: Use direct dependencies, not workspace dependencies
2. **Add Comprehensive README**: Document purpose, usage, and build instructions
3. **Include Examples**: Provide working examples for key functionality
4. **Test Independence**: Verify module builds without workspace
5. **Follow Naming**: Use descriptive names like `esp32c3-[purpose]`

---

**Total Modules**: 6 standalone modules  
**Target Hardware**: ESP32-C3 RISC-V microcontroller  
**Framework Support**: Both Embassy async and traditional blocking patterns  
**Development Stage**: Production-ready with comprehensive documentation