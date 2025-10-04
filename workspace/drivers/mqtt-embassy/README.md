# MQTT Embassy - Asynchronous MQTT Client

## 📨 Description

Complete and functional MQTT client module for the **TI0162 Internet of Things** course project. Implements asynchronous MQTT client using the Embassy framework for ESP32-C3, via Embassy TCP sockets with support for JSON publication of sensor data, status, and heartbeat.

**Project**: TI0162 IoT Environmental Monitoring System (UFC)  
**Status**: ✅ Implemented and validated on real hardware

## 🚀 Features

- ✅ **Asynchronous MQTT Client**: Via Embassy TCP sockets
- ✅ **MQTT 3.1.1 Protocol**: Complete protocol implementation
- ✅ **Configurable Broker**: Broker support via environment variables (tested: 10.10.10.210:1883)
- ✅ **JSON Publication**: Structured sensor data, status, and heartbeat
- ✅ **Environment Configuration**: Secure credentials via .cargo/config.toml
- ✅ **WiFi Integration**: Works perfectly with wifi-embassy
- ✅ **Robust Reconnection**: Creates new connection for each publication cycle
- ✅ **Complete IoT Pipeline**: ESP32-C3 → WiFi → MQTT → Subscribers

## 🏗️ Architecture

### Project Structure

```
mqtt-embassy/
├── src/
│   ├── lib.rs              # Module public interface
│   ├── mqtt_client.rs      # Main MQTT client
│   └── message.rs          # JSON message structures
├── examples/
│   ├── mqtt_test.rs        # Basic MQTT test
│   └── mqtt_test_working.rs # Integrated test with WiFi
├── .cargo/
│   └── config.toml         # Broker configuration via env vars
└── Cargo.toml              # Embassy dependencies
```

### Data Flow

```
ESP32-C3 → WiFi → MQTT Broker → Mosquitto Subscribers
         ↑                    ↑
   wifi-embassy        mqtt-embassy
```

## ⚙️ Configuration

### MQTT Broker

Edit `.cargo/config.toml` to configure the broker:

```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
MQTT_BROKER_IP = "10.10.10.210"
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-iot"
MQTT_TOPIC_PREFIX = "esp32"
```

### Mosquitto Broker

```bash
# Install Mosquitto
sudo apt install mosquitto mosquitto-clients

# Start broker
sudo systemctl start mosquitto

# Configure to accept remote connections
sudo nano /etc/mosquitto/mosquitto.conf
# Add:
# listener 1883 0.0.0.0
# allow_anonymous true

# Restart
sudo systemctl restart mosquitto
```

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust target for ESP32-C3
rustup target add riscv32imc-unknown-none-elf

# Install probe-rs
cargo install probe-rs --features cli

# Check connected device
probe-rs list

# Check MQTT broker availability
ping 10.10.10.210
```

## Build Instructions

### Building from Workspace Root
```bash
# Navigate to workspace root
cd workspace/

# Build mqtt-embassy module from workspace
cargo build -p mqtt-embassy --release

# Build specific examples from workspace (requires features)
cargo build -p mqtt-embassy --example basic_mqtt --features examples --release
cargo build -p mqtt-embassy --example mqtt_test_working --features examples --release
cargo build -p mqtt-embassy --example mqtt_test --features examples --release

# Run examples from workspace
cargo run -p mqtt-embassy --example basic_mqtt --features examples --release
cargo run -p mqtt-embassy --example mqtt_test_working --features examples --release
cargo run -p mqtt-embassy --example mqtt_test --features examples --release
```

### Building from Module Folder
```bash
# Navigate to mqtt-embassy module
cd workspace/mqtt-embassy/

# Build library module from module folder
cargo build --release

# Build examples from module folder (requires features)
cargo build --example basic_mqtt --features examples --release
cargo build --example mqtt_test_working --features examples --release
cargo build --example mqtt_test --features examples --release

# Run examples from module folder
cargo run --example basic_mqtt --features examples --release
cargo run --example mqtt_test_working --features examples --release
cargo run --example mqtt_test --features examples --release
```

### Integration into Your Project

#### Method 1: Add as Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
mqtt-embassy = { path = "../mqtt-embassy" }

# Required MQTT dependencies
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }
wifi-embassy = { path = "../wifi-embassy" }
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embedded-io-async = "0.6"
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"
heapless = "0.8"
```

Configure MQTT broker in your `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
MQTT_BROKER_IP = "10.10.10.210"
MQTT_BROKER_PORT = "1883"
MQTT_CLIENT_ID = "esp32-c3-iot"
MQTT_TOPIC_PREFIX = "esp32"
```

#### Method 2: Copy Source Files
```bash
# Copy MQTT components to your project
cp workspace/mqtt-embassy/src/mqtt_client.rs your-project/src/
cp workspace/mqtt-embassy/src/message.rs your-project/src/

# Add to your main.rs:
mod mqtt_client;
mod message;
use mqtt_client::{MqttClient, MqttConfig};
use message::{SensorData, DeviceStatus};
```

#### Method 3: Use as Library Module
```rust
use mqtt_embassy::{MqttClient, MqttConfig, SensorData};
use wifi_embassy::WiFiManager;

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    let stack = wifi_manager.get_stack();
    
    let sensor_data = SensorData::new(23.5, 68.2, 1013.8);
    
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
        Ok(mut socket) => {
            client.publish_sensor_data(&mut socket, &sensor_data).await.unwrap();
        }
        Err(e) => rprintln!("MQTT Error: {}", e),
    }
}
```

## Testing Instructions

### MQTT Broker Setup
```bash
# 1. Install Mosquitto MQTT broker
sudo apt install mosquitto mosquitto-clients    # Linux
brew install mosquitto                           # macOS

# 2. Configure broker to accept connections
sudo nano /etc/mosquitto/mosquitto.conf
# Add these lines:
listener 1883 0.0.0.0
allow_anonymous true

# 3. Start broker
sudo systemctl start mosquitto    # Linux
brew services start mosquitto     # macOS

# 4. Test broker connectivity
ping [BROKER_IP]
telnet [BROKER_IP] 1883
```

### Build Verification
```bash
# Test workspace build
cd workspace/
cargo check -p mqtt-embassy
cargo build -p mqtt-embassy --release
cargo build -p mqtt-embassy --example mqtt_test_working --features examples --release

# Test module build
cd workspace/mqtt-embassy/
cargo check
cargo build --release
cargo build --example mqtt_test_working --features examples --release
```

### Runtime Testing
```bash
# Terminal 1: Start MQTT monitor (before running ESP32)
mosquitto_sub -h [BROKER_IP] -p 1883 -t "esp32/#" -v

# Terminal 2: Run MQTT test (choose one)
cargo run --example basic_mqtt --features examples --release          # Simple MQTT with simulated data
cargo run --example mqtt_test_working --features examples --release   # Full integration test

# Expected ESP32 output:
# WiFi Connected Successfully!
# MQTT Task: ✅ Connected to MQTT broker successfully!
# MQTT Task: ✅ Sensor data published to topic 'esp32/sensor/bme280'

# Expected MQTT monitor output:
# esp32/sensor/bme280 {"temperature":22.1,"humidity":68.0,"pressure":1013.3,"reading":1}
# esp32/heartbeat ping
# esp32/status {"status":"online","uptime":300,"free_heap":48000,"wifi_rssi":-38}
```

### Integration Testing with WiFi
```bash
# Test complete WiFi + MQTT integration
cd workspace/wifi-embassy/
cargo run --example wifi_mqtt_test --release

# Expected: Full WiFi connection + MQTT publishing pipeline
```

### MQTT Protocol Testing
```bash
# Monitor all topics
mosquitto_sub -h [BROKER_IP] -t "#" -v

# Monitor specific topic
mosquitto_sub -h [BROKER_IP] -t "esp32/sensor/bme280" -v

# Test broker with manual publish
mosquitto_pub -h [BROKER_IP] -t "test/topic" -m "hello world"
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
use mqtt_embassy::{MqttClient, MqttConfig, SensorData, DeviceStatus};
use wifi_embassy::WiFiManager;

#[embassy_executor::task]
async fn mqtt_task(wifi_manager: &'static WiFiManager) {
    // Configure MQTT
    let mqtt_config = MqttConfig::default();
    let client = MqttClient::new(mqtt_config);
    
    // Get network stack from WiFi
    let stack = wifi_manager.get_stack();
    
    // Create sensor data
    let sensor_data = SensorData::new(23.5, 68.2, 1013.8);
    
    // Buffers for TCP connection
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];
    
    // Connect and publish
    match client.connect(stack, &mut rx_buffer, &mut tx_buffer).await {
        Ok(mut socket) => {
            // Publish sensor data
            client.publish_sensor_data(&mut socket, &sensor_data).await?;
            
            // Publish heartbeat
            client.publish_heartbeat(&mut socket).await?;
        }
        Err(e) => rprintln!("MQTT Error: {}", e),
    }
}
```

## 📊 Published Messages

### Sensor Data (esp32/sensor/bme280)

```json
{
  "temperature": 23.5,
  "humidity": 68.2,
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

## 📊 Expected Output

### ESP32 Console

```
🚀 ESP32-C3 MQTT Embassy Test
📡 WiFi + MQTT Integration Test
Target SSID: FamiliaFeliz-2Ghz
MQTT Broker: 10.10.10.210:1883
✅ Embassy time driver initialized successfully
✅ WiFi manager initialized successfully!

🎉 WiFi Connected Successfully!
📡 Network Details:
  📍 IP Address: 10.10.10.214
  🌐 Gateway: Some(10.10.10.1)
  🔧 Subnet: /24

MQTT Task: Reading #1 - T: 22.1°C, H: 68.0%, P: 1013.3 hPa
MQTT Task: ✅ Connected to MQTT broker successfully!
MQTT Task: ✅ Sensor data published to topic 'esp32/sensor/bme280'
```

### Mosquitto Monitor

```bash
$ mosquitto_sub -h 10.10.10.210 -p 1883 -t "esp32/#" -v

esp32/sensor/bme280 {"temperature":22.1,"humidity":68.0,"pressure":1013.3,"reading":1}
esp32/sensor/bme280 {"temperature":22.2,"humidity":67.8,"pressure":1013.4,"reading":2}
esp32/heartbeat ping
esp32/status {"status":"online","uptime":300,"free_heap":48000,"wifi_rssi":-38}
```

## 🔗 Tested Integration

### With WiFi Embassy

Functional example available in `wifi-embassy/examples/wifi_mqtt_test.rs`:

```rust
// Complete WiFi + MQTT system
let wifi_manager = WiFiManager::new(/* params */).await?;
let stack = wifi_manager.get_stack();

// Direct MQTT publication via TCP sockets
let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
let broker_addr = ("10.10.10.210".parse().unwrap(), 1883);
socket.connect(broker_addr).await?;

// Send MQTT CONNECT and PUBLISH
socket.write_all(&connect_packet).await?;
socket.write_all(&publish_packet).await?;
```

### With BME280 Embassy

```rust
// Integration with real sensor
let measurements = bme280.read_measurements().await?;
let sensor_data = SensorData::new(
    measurements.temperature,
    measurements.humidity,
    measurements.pressure
);
client.publish_sensor_data(&mut socket, &sensor_data).await?;
```

## 📦 Dependencies

```toml
[dependencies]
# ESP32-C3 Hardware Abstraction Layer
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-hal-embassy = { version = "0.9.0", features = ["esp32c3"] }

# WiFi Embassy (integration)
wifi-embassy = { path = "../wifi-embassy" }

# Embassy Async Framework
embassy-executor = { version = "0.7", features = ["task-arena-size-32768"] }
embassy-time = { version = "0.4" }
embedded-io-async = "0.6"

# JSON and utilities
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde-json-core = "0.6"
heapless = "0.8"
```

## 🐛 Troubleshooting

### Common Issues

1. **MQTT Broker not accessible**:
   ```bash
   # Check connectivity
   ping 10.10.10.210
   telnet 10.10.10.210 1883
   
   # Check Mosquitto configuration
   sudo systemctl status mosquitto
   sudo journalctl -u mosquitto
   ```

2. **Messages don't appear in subscriber**:
   ```bash
   # Check MQTT packet format
   # Add hex debug in code
   rprintln!("MQTT Packet: {:02X?}", &publish_packet);
   
   # Check topics
   mosquitto_sub -h 10.10.10.210 -t "#" -v
   ```

3. **WiFi connected but MQTT fails**:
   ```bash
   # Check network stack
   let stack = wifi_manager.get_stack();
   rprintln!("Stack status: {:?}", stack.config_v4());
   ```

4. **Build fails**:
   ```bash
   cargo clean
   cargo build --example mqtt_test_working --features examples --release
   ```

### MQTT Debug

```rust
// Detailed MQTT protocol debug
rprintln!("MQTT CONNECT packet: {:02X?}", &connect_packet);
rprintln!("MQTT PUBLISH packet: {:02X?}", &publish_packet[..20]);
rprintln!("Socket state: {:?}", socket.state());
```

## 📋 MQTT Specifications

- **Protocol**: MQTT 3.1.1
- **QoS**: 0 (Fire and forget)
- **Retain**: false
- **Keep Alive**: 60 seconds
- **Clean Session**: true
- **Client ID**: Configurable via env var

### Packet Format

```
CONNECT:  [0x10, length, protocol_name, version, flags, keep_alive, client_id]
PUBLISH:  [0x30, length, topic_length, topic, payload]
```

## 🔄 Publication Cycle

1. **Sensor Data**: Every 30 seconds
2. **Heartbeat**: Every 5 cycles (2.5 minutes)
3. **Device Status**: Every 10 cycles (5 minutes)

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>