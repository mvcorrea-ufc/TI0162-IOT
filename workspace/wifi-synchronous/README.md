# WiFi Synchronous - Synchronous WiFi Connectivity

## 📡 Description

Synchronous (blocking) WiFi connectivity module for ESP32-C3. Provides traditional blocking network stack programming model as an alternative to the async Embassy framework. Ideal for simple applications that don't need async/await complexity.

**Status**: ✅ Implemented and tested

## 🚀 Features

- ✅ **Synchronous WiFi Connectivity**: Traditional blocking programming model
- ✅ **Blocking Network Stack**: Uses blocking-network-stack crate
- ✅ **DHCP Support**: Automatic IP address acquisition with hostname
- ✅ **Network Scanning**: Scan for available WiFi networks
- ✅ **Connection Monitoring**: Real-time status verification
- ✅ **Modular API**: Clean helper functions for WiFi operations
- ✅ **Hardware Abstraction**: Works with esp-hal and esp-wifi
- ✅ **Environment Configuration**: WiFi credentials via .cargo/config.toml

## 🏗️ Migration-Friendly API

### API Comparison: wifi-embassy vs wifi-synchronous

| Operation | wifi-embassy (async) | wifi-synchronous (blocking) |
|-----------|---------------------|----------------------------|
| **Create Manager** | `WiFiManager::new(spawner, timg, wifi, rng, config).await?` | `WiFiManager::new(wifi, timg, rng, config)?` |
| **Connect** | `manager.get_connection_info()` (auto-connected) | `manager.connect()?` |
| **Get Info** | `manager.get_connection_info()` | `manager.get_connection_info()` |
| **Get Stack** | `manager.get_stack()` → `&Stack<'static>` | `manager.get_stack()` → `&blocking_network_stack::Stack` |
| **Types** | `WiFiConfig`, `ConnectionInfo`, `WiFiError` | Same types, same fields |

### Easy Migration Example

```rust
// BEFORE: wifi-embassy (async)
use wifi_embassy::{WiFiManager, WiFiConfig};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = WiFiConfig { ssid: "Net", password: "Pass" };
    let wifi_manager = WiFiManager::new(spawner, timg, wifi, rng, config).await?;
    let connection_info = wifi_manager.get_connection_info();
    let stack = wifi_manager.get_stack(); // embassy-net::Stack
}

// AFTER: wifi-synchronous (blocking) 
use wifi_synchronous::{WiFiManager, WiFiConfig};

#[main]
fn main() {
    let config = WiFiConfig { ssid: "Net", password: "Pass" }; // Same struct!
    let mut wifi_manager = WiFiManager::new(wifi, timg, rng, config)?; // No spawner needed
    let connection_info = wifi_manager.connect()?; // Explicit connect call  
    let stack = wifi_manager.get_stack(); // blocking_network_stack::Stack
}
```

## Build Instructions

### Building from Workspace Root
```bash
# Navigate to workspace root
cd workspace/

# Build wifi-synchronous module from workspace
cargo build -p wifi-synchronous --release

# Build examples from workspace
cargo build -p wifi-synchronous --example simple_wifi_sync --release
cargo build -p wifi-synchronous --example wifi_manager_sync --release

# Run examples from workspace
cargo run -p wifi-synchronous --example simple_wifi_sync --release        # Helper functions
cargo run -p wifi-synchronous --example wifi_manager_sync --release       # WiFiManager API
```

### Building from Module Folder
```bash
# Navigate to wifi-synchronous module
cd workspace/wifi-synchronous/

# Build library module from module folder
cargo build --release

# Build examples from module folder
cargo build --example simple_wifi_sync --release
cargo build --example wifi_manager_sync --release

# Run examples from module folder
cargo run --example simple_wifi_sync --release        # Helper functions
cargo run --example wifi_manager_sync --release       # WiFiManager API
```

### Integration into Your Project

#### Method 1: Add as Dependency
Add to your `Cargo.toml`:
```toml
[dependencies]
wifi-synchronous = { path = "../wifi-synchronous" }

# Required synchronous WiFi dependencies
esp-hal = { version = "1.0.0-rc.0", features = ["esp32c3", "unstable"] }
esp-wifi = { version = "0.15.0", features = ["esp32c3", "wifi", "smoltcp"] }
esp-alloc = { version = "0.8.0" }
smoltcp = { version = "0.12.0" }
blocking-network-stack = { git = "https://github.com/bjoernQ/blocking-network-stack.git" }
```

Configure WiFi credentials in your `.cargo/config.toml`:
```toml
[env]
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
```

#### Method 2: Use Library Functions
```rust
use wifi_synchronous::{
    WiFiConfig, configure_wifi, wait_for_connection, wait_for_ip
};

fn main() -> ! {
    let wifi_config = WiFiConfig {
        ssid: env!("WIFI_SSID"),
        password: env!("WIFI_PASSWORD"),
        hostname: "ESP32-Device",
    };
    
    // Traditional blocking approach - no async/await
    configure_wifi(&mut controller, &wifi_config).unwrap();
    wait_for_connection(&mut controller, &mut led).unwrap();
    let connection = wait_for_ip(&mut stack, &mut led, &wifi_config).unwrap();
    
    // Network ready for use
}
```

## Testing Instructions

### Runtime Testing
```bash
# Test synchronous WiFi connectivity
cargo run --example simple_wifi_sync --release

# Expected output:
# WiFi Connected Successfully!
# IP Address: 10.10.10.xxx
# Synchronous network stack ready for applications
```

## 📄 License

MIT OR Apache-2.0

## 👨‍💻 Author

Marcelo Correa <mvcorrea@gmail.com>