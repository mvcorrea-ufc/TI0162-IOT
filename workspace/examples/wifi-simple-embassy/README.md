# WiFi Simple Embassy

A simple, robust WiFi library for ESP32-C3 using Embassy async framework.

## Features

- ✅ **ESP32-C3 WiFi connectivity** using latest esp-hal 1.0.0-rc.0
- ✅ **Embassy async framework** for non-blocking operations  
- ✅ **DHCP IP address acquisition** with pingable connectivity
- ✅ **Compatible with vanilla esp-hal** (no custom toolchains)
- ✅ **Clean, maintainable API** following proven patterns
- ✅ **Comprehensive error handling** and logging
- ✅ **Easy integration** in other projects

## Quick Start

### 1. Add to your Cargo.toml

```toml
[dependencies]
wifi-simple-embassy = { path = "../wifi-simple-embassy" }
```

### 2. Set WiFi credentials

Create `.cargo/config.toml` in your project:

```toml
[env]
SSID = "YourWiFiNetwork"
PASSWORD = "YourWiFiPassword"
```

### 3. Basic usage

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use wifi_simple_embassy::WiFiManager;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    let wifi_manager = WiFiManager::new(
        spawner,
        peripherals.TIMG0,
        peripherals.TIMG1,
        peripherals.WIFI,
        peripherals.RNG,
        env!("SSID"),
        env!("PASSWORD"),
    ).await.unwrap();
    
    // WiFi is connected and ready!
    let stack = wifi_manager.get_stack();
    
    loop {
        // Your application code here
    }
}
```

## Examples

Run the basic example:

```bash
cargo run --example basic_wifi --release
```

## API Overview

### WiFiManager

The main interface for WiFi functionality:

```rust
impl WiFiManager {
    // Initialize WiFi and connect to network
    async fn new(/* parameters */) -> Result<Self, WiFiError>
    
    // Get the embassy network stack
    fn get_stack(&self) -> &'static Stack<'static>
    
    // Check connection status
    fn is_connected(&self) -> bool
    
    // Get current IP address
    fn get_ip_address(&self) -> Option<Ipv4Address>
    
    // Get detailed connection info
    fn get_connection_info(&self) -> Option<&ConnectionInfo>
    
    // Wait for network to be ready
    async fn wait_for_ready(&self)
}
```

### ConnectionInfo

Detailed connection information:

```rust
pub struct ConnectionInfo {
    pub ip_address: Ipv4Address,
    pub gateway: Option<Ipv4Address>,
    pub dns_servers: Vec<Ipv4Address, 3>,
    pub subnet_prefix: u8,
}
```

## Error Handling

The library provides comprehensive error types:

```rust
pub enum WiFiError {
    HardwareInit(&'static str),
    Configuration(&'static str),
    Connection(&'static str),
    Dhcp(&'static str),
}
```

## Requirements

- ESP32-C3 microcontroller
- Rust nightly toolchain
- espflash for flashing

## Version Compatibility

- esp-hal: 1.0.0-rc.0
- esp-wifi: 0.15.0
- embassy: 0.7.x

## License

MIT OR Apache-2.0