# Serial Console Embassy - Interactive Serial Console

## 💻 Description

Interactive serial console module for ESP32-C3 using the Embassy framework. Provides a command-line interface via **USB Serial/JTAG** (direct access) or UART for IoT system configuration and monitoring, enabling dynamic configuration of WiFi credentials, MQTT settings, and real-time status visualization.

**Supported Hardware**: WeAct ESP32-C3 via USB-C (no external converters required)

**Status**: ✅ Implemented and tested

## 🚀 Features

- ✅ **USB Serial/JTAG Console**: Direct access via WeAct ESP32-C3 USB-C port
- ✅ **Async UART Interface**: Alternative serial console via UART
- ✅ **Command System**: Robust parser with command validation
- ✅ **Dynamic Configuration**: WiFi and MQTT configurable via commands
- ✅ **Real-time Monitoring**: System and module status display
- ✅ **Embassy Integration**: Async tasks for non-blocking I/O
- ✅ **Persistence**: Save/load configurations (prepared for flash storage)
- ✅ **Modularity**: Optional features for selective integration
- ✅ **Minimal Hardware**: No external USB-serial converters needed

## 🔌 Hardware and Connection

### WeAct ESP32-C3 USB-C
**✅ PRIMARY SOLUTION**: Console accessible directly via USB-C port

```
WeAct ESP32-C3    Host Computer
--------------    -------------
USB-C         <-> USB-C/USB-A
                   /dev/ttyACM0 (Linux/macOS)
                   COMx (Windows)
```

- **Interface**: USB Serial/JTAG (built-in)
- **Access**: `/dev/ttyACM0` or equivalent COM port
- **Speed**: 115200 baud
- **Additional Hardware**: None required

### Alternative UART Configuration (Optional)
For development with external USB-serial converter:

```
ESP32-C3        USB-Serial Converter
--------        -------------------
GPIO20 (RX) <-- TX
GPIO21 (TX) --> RX
GND         --- GND
```

## Build Instructions

### Building from Workspace Root
```bash
# Navigate to workspace root
cd workspace/

# Build serial-console-embassy module from workspace
cargo build -p serial-console-embassy --release

# Build specific examples from workspace
cargo build -p serial-console-embassy --example simple_working_console --release
cargo build -p serial-console-embassy --example direct_usb_console --release
cargo build -p serial-console-embassy --example basic_console --release

# Run examples from workspace
cargo run -p serial-console-embassy --example simple_working_console --release
cargo run -p serial-console-embassy --example direct_usb_console --release
cargo run -p serial-console-embassy --example basic_console --release

# Run with full features from workspace
cargo run -p serial-console-embassy --example system_console --features full --release
```

### Building from Module Folder
```bash
# Navigate to serial-console-embassy module
cd workspace/serial-console-embassy/

# Build library module from module folder
cargo build --release

# Build examples from module folder
cargo build --example simple_working_console --release
cargo build --example direct_usb_console --release
cargo build --example basic_console --release

# Run examples from module folder
cargo run --example simple_working_console --release
cargo run --example direct_usb_console --release
cargo run --example basic_console --release

# Run with features from module folder
cargo run --example system_console --features full --release
```

### Integration into Your Project

#### Method 1: Add as Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
serial-console-embassy = { path = "../serial-console-embassy" }

# Required Embassy dependencies
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embassy-sync = { version = "0.7" }
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
heapless = "0.8"
embedded-io-async = "0.6"
```

Configure environment in your `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "ESP32-Test"
WIFI_PASSWORD = "password123"
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
```

#### Method 2: Copy Source Files
```bash
# Copy console components to your project
cp workspace/serial-console-embassy/src/console.rs your-project/src/
cp workspace/serial-console-embassy/src/commands.rs your-project/src/
cp workspace/serial-console-embassy/src/config.rs your-project/src/

# Add to your main.rs:
mod console;
mod commands;
mod config;
use console::SerialConsole;
```

#### Method 3: Use as Library Module
```rust
use serial_console_embassy::{SerialConsole, ConsoleConfig};

#[embassy_executor::task]
async fn console_task(uart: uart::Uart<'static, esp_hal::peripherals::UART0, uart::Async>) {
    let config = ConsoleConfig::default();
    let mut console = SerialConsole::new(uart, config);
    console.run().await;
}
```

## Testing Instructions

### Hardware Setup Test
```bash
# 1. Connect ESP32-C3 via USB-C
# Direct USB Serial/JTAG access (no additional hardware needed)

# 2. Verify device detection
ls /dev/ttyACM*    # Linux/macOS
# OR check Device Manager on Windows for COM ports
```

### Build Verification
```bash
# Test workspace build
cd workspace/
cargo check -p serial-console-embassy
cargo build -p serial-console-embassy --release
cargo build -p serial-console-embassy --example simple_working_console --release

# Test module build
cd workspace/serial-console-embassy/
cargo check
cargo build --release
cargo build --example simple_working_console --release
```

### Runtime Testing
```bash
# Test basic console functionality
cargo run --example simple_working_console --release

# Expected: Console starts, waits for serial input

# Test USB Serial/JTAG console (recommended)
cargo run --example direct_usb_console --release

# Expected: Interactive console with command prompt
```

### Serial Terminal Testing
```bash
# Connect to console via serial terminal
picocom /dev/ttyACM0 -b 115200

# Test basic commands in console:
# help          - Shows available commands
# status        - Shows system status
# info          - Shows hardware information
# clear         - Clears screen
```

### Integration Testing
```bash
# Test console with full IoT system integration
cargo run --example system_console --features full --release

# Expected: Console with WiFi/MQTT/sensor integration commands
```

### Code Quality
```bash
# Code verification
cargo clippy  # Check for warnings
cargo fmt     # Format code
cargo clean   # Clean build artifacts
```

## 🚀 Quick Start

### Connect via Serial Terminal

```bash
# ✅ WeAct ESP32-C3 via USB-C (PRIMARY)
picocom /dev/ttyACM0 -b 115200

# Linux/macOS alternatives
screen /dev/ttyACM0 115200
minicom -D /dev/ttyACM0 -b 115200

# Windows
putty -serial COM3 -serspeed 115200
```

## 📋 Available Commands

### System Commands
```bash
help, h, ?          # Show complete help
status, stat        # Current system status
info, i             # Detailed hardware information
clear, cls          # Clear terminal screen
restart, reset      # Restart system
```

### WiFi Commands
```bash
wifi show           # Show current configuration
wifi ssid <name>    # Configure network SSID
wifi pass <password> # Configure WiFi password
```

### MQTT Commands
```bash
mqtt show           # Show MQTT configuration
mqtt broker <ip>    # Configure broker IP
mqtt port <port>    # Configure port (default: 1883)
mqtt client <id>    # Configure client ID
mqtt prefix <prefix> # Configure topic prefix
```

### Configuration Commands
```bash
save                # Save configuration to flash
load                # Load configuration from flash
```

## 📊 Session Example

```
+==============================================================+
|              ESP32-C3 IoT System Console                     |
|                    Embassy Framework                         |
|                  Direct USB Serial/JTAG                      |
+==============================================================+

Type 'help' for available commands

esp32> status
=== System Status ===
WiFi: Connected (10.10.10.214)
MQTT: Connected
Sensor: Active

esp32> wifi show
=== WiFi Configuration ===
SSID: MyNetworkWiFi
Password: ********
Status: Valid

esp32> mqtt show
=== MQTT Configuration ===
Broker: 10.10.10.210:1883
Client ID: esp32-c3-console
Topic Prefix: esp32
Status: Valid

esp32> wifi ssid NewNetwork
WiFi SSID set to: NewNetwork

esp32> save
Configuration saved to flash

esp32> info
=== System Information ===
Chip: ESP32-C3
Framework: Embassy
Build: Release
Free Heap: 48KB
```

## 🏗️ Module Architecture

### File Structure

```
serial-console-embassy/
├── src/
│   ├── lib.rs              # Public module interface
│   ├── console.rs          # Async UART console
│   ├── commands.rs         # Command parser and handler
│   └── config.rs           # Configuration structures
├── examples/
│   ├── basic_console.rs          # Basic UART console
│   ├── direct_usb_console.rs     # ✅ Direct USB Serial/JTAG console
│   ├── simple_working_console.rs # Simple UART console for testing
│   ├── usb_bridge_console.rs     # UART↔USB bridge console
│   └── system_console.rs         # IoT-integrated console
├── .cargo/
│   └── config.toml         # Build configuration and env vars
├── Cargo.toml              # Dependencies and features
└── README.md               # This documentation
```

### Available Implementations

1. **`direct_usb_console.rs`** ✅ **RECOMMENDED**
   - Direct console via ESP32-C3 USB Serial/JTAG
   - Access at `/dev/ttyACM0` without additional hardware
   - Complete command interface
   - Tested and functional

2. **`basic_console.rs`**
   - Traditional console via UART
   - Requires external USB-serial converter
   - For development with additional hardware

3. **`usb_bridge_console.rs`**
   - Software bridge between UART and USB Serial/JTAG
   - Experimental, for specific use cases

4. **`system_console.rs`**
   - Console integrated with WiFi/MQTT/BME280 modules
   - Requires enabled features (`--features full`)

### Available Features

```toml
[features]
default = []
wifi = ["dep:wifi-embassy"]      # WiFi integration
mqtt = ["dep:mqtt-embassy"]      # MQTT integration
sensor = ["dep:bme280-embassy"]  # Sensor integration
usb = ["dep:embassy-usb"]        # USB console (future)
full = ["wifi", "mqtt", "sensor"] # All features
```

## 🔧 Configuration

### Main Dependencies

```toml
[dependencies]
# ESP32-C3 HAL + Embassy
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# Embassy Framework  
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embassy-sync = { version = "0.7" }

# String processing
heapless = "0.8"
embedded-io-async = "0.6"
```

### Development Environment

```toml
# .cargo/config.toml
[env]
WIFI_SSID = "ESP32-Test"
WIFI_PASSWORD = "password123"
MQTT_BROKER_IP = "192.168.1.100"
MQTT_BROKER_PORT = "1883"
```

## 📚 Integration with Other Modules

### With WiFi Embassy
```rust
use wifi_embassy::{WiFiManager, WiFiConfig};
use serial_console_embassy::SerialConsole;

// Update WiFi status in console
console.update_system_status(true, false, true, Some("10.10.10.214")).await;
```

### With MQTT Embassy
```rust
use mqtt_embassy::MqttClient;

// Configure MQTT via console and use in client
let config = console.get_config().await;
let mqtt_client = MqttClient::new_from_console_config(&config.mqtt);
```

### With BME280 Embassy
```rust
use bme280_embassy::BME280;

// Monitor sensor and report status
let sensor_active = bme280.check_id().await.is_ok();
console.update_system_status(wifi_ok, mqtt_ok, sensor_active, ip).await;
```

## 🐛 Troubleshooting

### Common Issues

1. **USB console doesn't appear at /dev/ttyACM0**:
   ```bash
   # Check if USB-C is connected properly
   # Try different USB ports
   # Verify ESP32-C3 drivers on system
   lsusb | grep -i esp
   ```

2. **Console doesn't respond**:
   ```bash
   # Check if picocom is connected to correct port
   # Confirm baud rate 115200
   # Reset ESP32-C3 (reset button)
   ```

3. **Corrupted characters**:
   ```bash
   # Check terminal speed (115200)
   # Test with different clients (screen, minicom, picocom)
   # Verify USB-C cable
   ```

4. **Build fails**:
   ```bash
   cargo clean
   cargo build --example direct_usb_console --release
   ```

5. **Features not available**:
   ```bash
   # Use correct features
   cargo run --example system_console --features full --release
   ```

### Console Debug

```rust
// RTT logs for console debugging
rprintln!("[CONSOLE] Command received: {}", command);
rprintln!("[CONSOLE] Status updated: WiFi={}, MQTT={}", wifi, mqtt);
```

## 🔮 Future Extensions

### Planned Features
- **Flash Storage**: Real configuration persistence
- **Command History**: Command history with arrow keys
- **Auto-completion**: Automatic command completion
- **Web Console**: Web interface for remote command
- **Scripting**: Command script execution
- **Console Integration**: Complete integration with WiFi/MQTT modules

### Additional Commands
- **log level**: Configure logging level
- **network scan**: Scan available WiFi networks
- **sensor calibrate**: Manual sensor calibration
- **system update**: OTA update via console

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>

**Project TI0162 - Internet of Things**  
**Interactive Serial Console for ESP32-C3 IoT System**