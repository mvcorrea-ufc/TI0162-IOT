# WiFi Embassy - Asynchronous WiFi Connectivity

## 📡 Description

Complete and functional WiFi connectivity module for the **TI0162 Internet of Things** course project. Implements robust WiFi connection using the Embassy framework for ESP32-C3, with automatic reconnection, DHCP acquisition, and complete network stack for TCP/UDP operations.

**Project**: TI0162 IoT Environmental Monitoring System (UFC)  
**Status**: ✅ Implemented and validated on real hardware

## 🚀 Features

- ✅ **Robust WiFi Connectivity**: Automatic connection with retry logic
- ✅ **DHCP Support**: Automatic IP address acquisition (tested: 10.10.10.214)
- ✅ **Embassy Integration**: Complete async/await support with Embassy framework
- ✅ **Automatic Reconnection**: Gracefully manages network disconnections
- ✅ **Connection Monitoring**: Real-time status verification and reporting
- ✅ **Network Stack Access**: Provides embassy-net stack for TCP/UDP operations
- ✅ **Proven Architecture**: Based on functional examples from the workspace
- ✅ **Environment Credentials**: Secure configuration via .cargo/config.toml

## 🏗️ Architecture

This module follows the established patterns from:
- **bme280-embassy**: Hardware initialization and Embassy integration
- **wifi-simple-embassy**: Clean API design and error handling
- **wifi-simple-must-working**: Proven asynchronous connection management

### Project Structure

```
wifi-embassy/
├── src/
│   ├── lib.rs              # Module public interface
│   └── wifi_manager.rs     # Main WiFi manager
├── examples/
│   ├── wifi_test.rs        # Basic connectivity test
│   ├── wifi_test_new.rs    # Test with detailed information
│   └── wifi_mqtt_test.rs   # Complete WiFi + MQTT integration
├── .cargo/
│   └── config.toml         # WiFi credentials via environment variables
└── Cargo.toml              # Embassy dependencies

## ⚙️ Configuration

### WiFi Credentials

Edit `.cargo/config.toml` to configure your credentials:

```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust target for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs
cargo install probe-rs --features cli

# Verify connected device
probe-rs list
```

## Build Instructions

### Building from Workspace Root
```bash
# Navigate to workspace root
cd workspace/

# Build wifi-embassy module from workspace
cargo build -p wifi-embassy --release

# Build specific examples from workspace
cargo build -p wifi-embassy --example simple_connect --release
cargo build -p wifi-embassy --example wifi_test --release
cargo build -p wifi-embassy --example wifi_test_new --release
cargo build -p wifi-embassy --example wifi_mqtt_test --release

# Run examples from workspace
cargo run -p wifi-embassy --example simple_connect --release
cargo run -p wifi-embassy --example wifi_test --release
cargo run -p wifi-embassy --example wifi_test_new --release
cargo run -p wifi-embassy --example wifi_mqtt_test --release
```

### Building from Module Folder
```bash
# Navigate to wifi-embassy module
cd workspace/wifi-embassy/

# Build library module from module folder
cargo build --release

# Build examples from module folder
cargo build --example simple_connect --release
cargo build --example wifi_test --release
cargo build --example wifi_test_new --release
cargo build --example wifi_mqtt_test --release

# Run examples from module folder
cargo run --example simple_connect --release
cargo run --example wifi_test --release
cargo run --example wifi_test_new --release
cargo run --example wifi_mqtt_test --release
```

### Integration into Your Project

#### Method 1: Add as Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
wifi-embassy = { path = "../wifi-embassy" }

# Required WiFi dependencies
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
esp-alloc = { version = "0.8.0" }
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-net = { version = "0.7", features = ["tcp", "udp", "dhcpv4", "medium-ethernet"] }
embassy-time = { version = "0.4" }
```

Configure WiFi credentials in your `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
```

#### Method 2: Copy Source Files
```bash
# Copy WiFi manager to your project
cp workspace/wifi-embassy/src/wifi_manager.rs your-project/src/

# Add to your main.rs:
mod wifi_manager;
use wifi_manager::{WiFiManager, WiFiConfig};
```

#### Method 3: Use as Library Module
```rust
use wifi_embassy::{WiFiManager, WiFiConfig};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };
    
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await.unwrap();
    
    // WiFi is connected and ready to use
    let stack = wifi_manager.get_stack();
}
```

## Testing Instructions

### Network Setup Test
```bash
# 1. Configure your WiFi credentials
# Edit .cargo/config.toml with your network details

# 2. Verify network requirements
# - 2.4GHz WiFi network (ESP32-C3 doesn't support 5GHz)
# - DHCP server available on router
# - No MAC address filtering blocking ESP32-C3
```

### Build Verification
```bash
# Test workspace build
cd workspace/
cargo check -p wifi-embassy
cargo build -p wifi-embassy --release
cargo build -p wifi-embassy --example wifi_test_new --release

# Test module build
cd workspace/wifi-embassy/
cargo check
cargo build --release
cargo build --example wifi_test_new --release
```

### Runtime Testing
```bash
# Test basic WiFi connectivity
cargo run --example wifi_test_new --release

# Expected output:
# WiFi Connected Successfully!
# IP Address: 10.10.10.214
# Gateway: Some(10.10.10.1)
# Subnet: /24

# Test WiFi + MQTT integration
cargo run --example wifi_mqtt_test --release

# Expected: WiFi connection + MQTT broker connection + data publishing
```

### Network Integration Testing
```bash
# Test network connectivity from host
ping [ESP32_IP_ADDRESS]  # Should respond if WiFi connected

# Monitor MQTT traffic (if testing MQTT)
mosquitto_sub -h [BROKER_IP] -p 1883 -t "esp32/#" -v
```

### Code Quality
```bash
# Code verification
cargo clippy  # Check for warnings
cargo fmt     # Format code
cargo clean   # Clean build artifacts
```

### Programmatic Usage

```rust
use wifi_embassy::{WiFiManager, WiFiConfig};
use embassy_executor::Spawner;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Initialize ESP32-C3
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Configure WiFi via environment variables
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
    };
    
    // Create WiFi manager
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.WIFI,
        peripherals.RNG,
        wifi_config,
    ).await?;
    
    // WiFi connected and ready!
    let stack = wifi_manager.get_stack();
    
    // Use stack for TCP/UDP operations
}
```

### Connection Monitoring

```rust
// Check connection status
if let Some(connection_info) = wifi_manager.get_connection_info() {
    rprintln!("📍 IP Address: {}", connection_info.ip_address);
    rprintln!("🌐 Gateway: {:?}", connection_info.gateway);
    rprintln!("🔧 Subnet: /{}", connection_info.subnet_prefix);
}
```

## 📊 Expected Output

```
🚀 ESP32-C3 WiFi Embassy Test
📡 Target SSID: FamiliaFeliz-2Ghz
✅ Embassy time driver initialized
🔧 Hardware initialized, starting WiFi connection...
✅ WiFi manager initialized successfully!

🎉 WiFi Connected Successfully!
📡 Network Details:
  📍 IP Address: 10.10.10.214
  🌐 Gateway: Some(10.10.10.1)
  🔧 Subnet: /24
```

## 🔗 Integration with Other Modules

### With BME280 Embassy

```rust
// Initialize WiFi and BME280 together
let wifi_manager = WiFiManager::new(/* params */).await?;
let bme280 = BME280::new(&mut i2c);

// Use both modules together
let stack = wifi_manager.get_stack();
let measurements = bme280.read_measurements().await?;

// Send sensor data via network
```

### Network Stack for MQTT/HTTP

```rust
let stack = wifi_manager.get_stack();

// The stack can be used with:
// - embassy-net TcpSocket for HTTP clients
// - MQTT clients that accept embassy-net stack
// - Custom TCP/UDP applications
```

## 📋 Hardware Requirements

- **ESP32-C3**: Main target microcontroller
- **WiFi Network**: 2.4GHz network (5GHz not supported by ESP32-C3)
- **Power Supply**: Stable 3.3V
- **Antenna**: Integrated PCB antenna or external

## 📦 Dependencies

```toml
[dependencies]
# ESP32-C3 Hardware Abstraction Layer
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# WiFi Hardware and Network Stack
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
esp-alloc = { version = "0.8.0" }

# Embassy Async Framework
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-net = { version = "0.7", features = ["tcp", "udp", "dhcpv4", "medium-ethernet"] }
embassy-time = { version = "0.4" }
```

## 🐛 Troubleshooting

### Common Issues

1. **WiFi doesn't connect**:
   ```bash
   # Check credentials in .cargo/config.toml
   # Verify network is 2.4GHz (not 5GHz)
   # Confirm exact SSID (case-sensitive)
   ```

2. **DHCP fails**:
   ```bash
   # Check router/gateway is working
   # Confirm DHCP pool is available
   # Test with mobile device first
   ```

3. **Embassy time driver not initialized**:
   ```bash
   # Error: schedule_wake called before esp_hal_embassy::init()
   # Solution: Call esp_hal_embassy::init() before WiFiManager::new()
   ```

4. **Build fails**:
   ```bash
   cargo clean
   cargo build --release
   ```

### WiFi Debug

```rust
// Add detailed debug
rprintln!("WiFi Status: {:?}", wifi_controller.status());
rprintln!("IP Config: {:?}", stack.config_v4());
```

## 🔗 Tested Integration

This module has been tested and integrates perfectly with:

- **mqtt-embassy**: MQTT publishing via WiFi (example wifi_mqtt_test.rs)
- **Mosquitto Broker**: MQTT broker at 10.10.10.210:1883
- **Network Stack**: embassy-net for TCP/UDP

### MQTT Integration Example

```rust
// Functional example in examples/wifi_mqtt_test.rs
let stack = wifi_manager.get_stack();
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);

// Connect to MQTT broker
let broker_addr = ("10.10.10.210".parse().unwrap(), 1883);
socket.connect(broker_addr).await?;

// Publish data via MQTT
let json_payload = format!(r#"{{"temperature":{:.1},"humidity":{:.1}}}"#, temp, hum);
socket.write_all(&mqtt_publish_packet).await?;
```

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>