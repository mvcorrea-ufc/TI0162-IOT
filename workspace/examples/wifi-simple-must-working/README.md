# WiFi-Simple - ESP32-C3 Modular WiFi Implementation

A clean, modular WiFi connectivity module for ESP32-C3 that provides DHCP IP acquisition and network stack management with simple helper functions.

## Purpose

This module provides:
- **Modular WiFi functions** with clean separation of concerns
- **DHCP IP address acquisition** with actual IP display
- **Network stack management** compatible with TCP/MQTT applications
- **Connection monitoring** and status reporting
- **Simple error handling** without complex lifetime management

## Features

- Modular WiFi helper functions
- DHCP client with real IP address display
- Network scanning and connection management
- Status LED indication during connection process
- Network stack creation for TCP/MQTT applications
- Clean error handling with descriptive messages
- No complex Rust lifetimes - kept simple and maintainable

## Hardware Requirements

- ESP32-C3 development board with WiFi capability
- Access to 2.4GHz WiFi network
- LED on GPIO8 for status indication

## Dependencies

```toml
[dependencies]
esp-hal = { git = "https://github.com/esp-rs/esp-hal", features = ["esp32c3", "unstable"] }
esp-wifi = { git = "https://github.com/esp-rs/esp-hal", features = ["esp32c3", "wifi", "smoltcp"] }
esp-alloc = { git = "https://github.com/esp-rs/esp-hal" }
smoltcp = { version = "0.12.0", default-features = false, features = ["medium-ethernet", "proto-dhcpv4", "proto-ipv4", "socket-dhcpv4", "socket-tcp", "socket-udp"] }
blocking-network-stack = { git = "https://github.com/bjoernQ/blocking-network-stack.git", rev = "b3ecefc222d8806edd221f266999ca339c52d34e", default-features = false, features = ["dhcpv4", "tcp"] }
rtt-target = "0.5"
panic-rtt-target = "0.1"
```

## Configuration

WiFi credentials are configured via environment variables in `.cargo/config.toml`:
```toml
[env]
# WiFi credentials - Replace with your network details
WIFI_SSID = "YourWiFiNetwork"
WIFI_PASSWORD = "YourWiFiPassword"
CARGO_CFG_PORTABLE_ATOMIC_UNSAFE_ASSUME_SINGLE_CORE = ""
```

**Note**: Replace `YourWiFiNetwork` and `YourWiFiPassword` with your actual WiFi network credentials before building.

## Module Functions

### WiFi Configuration
```rust
pub struct WiFiConfig {
    pub ssid: &'static str,
    pub password: &'static str, 
    pub hostname: &'static str,
}
```

### Core Functions
- `create_interface()` - Creates smoltcp network interface
- `create_dhcp_socket()` - Sets up DHCP socket for IP acquisition
- `create_stack()` - Creates blocking network stack for applications
- `configure_wifi()` - Configures and starts WiFi connection
- `scan_networks()` - Scans and displays available networks
- `wait_for_connection()` - Waits for WiFi association
- `wait_for_ip()` - Waits for DHCP IP address assignment
- `get_status()` - Gets current connection status and IP information

### WiFi Connection Info
```rust
pub struct WiFiConnection {
    pub ip: core::net::Ipv4Addr,
    pub gateway: core::net::Ipv4Addr,
    pub subnet_mask: u8,
    pub dns_primary: Option<core::net::Ipv4Addr>,
    pub dns_secondary: Option<core::net::Ipv4Addr>,
}
```

## Usage

### Build and Run
```bash
# Navigate to module
cd workspace/wifi-simple

# Build release version (recommended)
cargo build --release

# Flash and run with live console output
cargo run --release
```

### Expected Output
```
ESP32-C3 WiFi Simple - Clean Modular Implementation
Config: SSID=YourWiFiNetwork Hostname=ESP32-C3-WiFi-Test
Hardware: WiFi initialized
Hardware: WiFi controller created
Network: Stack created
WiFi: Configuration complete
WiFi: Scanning for networks...
Found 5 networks:
  0: YourWiFiNetwork (-45)
  1: NeighborNetwork (-67)
WiFi: Waiting for connection...
WiFi: Connected successfully
DHCP: Waiting for IP address...
DHCP: IP address acquired successfully
Network: IP=10.10.10.214 Gateway=10.10.10.1 Subnet=/24 DNS=Some(8.8.8.8)
Device: Hostname=ESP32-C3-WiFi-Test SSID=YourWiFiNetwork
Status: Device is now accessible on network
NETWORK READY
Connection: WiFiConnection { ip: 10.10.10.214, gateway: 10.10.10.1, subnet_mask: 24, dns_primary: Some(8.8.8.8), dns_secondary: None }
Test: ping 10.10.10.214
Status: Starting network monitoring loop
Status: CONNECTED IP=10.10.10.214 GW=10.10.10.1
```

### Integration Example
```rust
use wifi::{WiFiConfig, configure_wifi, wait_for_connection, wait_for_ip};

// Configure WiFi using environment variables
let wifi_config = WiFiConfig {
    ssid: env!("WIFI_SSID"),
    password: env!("WIFI_PASSWORD"), 
    hostname: "ESP32-Device",
};

// Set up WiFi connection
configure_wifi(&mut controller, &wifi_config)?;
wait_for_connection(&mut controller, &mut led)?;
let connection = wait_for_ip(&mut stack, &mut led, &wifi_config)?;

// Use connection info
rprintln!("Connected: {:?}", connection);
```

## LED Status Indicators

- **Fast blink** - Connecting to WiFi network
- **Medium blink** - Waiting for DHCP IP address
- **Slow blink** - Connected and operational
- **Solid off** - Network ready, normal operation

## Network Stack Usage

The created network stack is compatible with TCP applications and MQTT clients:
```rust
// Create stack using module functions
let mut stack = create_stack(iface, device, socket_set);

// Use stack for TCP connections, MQTT, etc.
// Stack provides blocking network operations suitable for MQTT
```

## Troubleshooting

### Connection Issues
- Verify WiFi credentials in `.cargo/config.toml`
- Check 2.4GHz network compatibility (ESP32-C3 doesn't support 5GHz)
- Ensure network allows new device connections
- Check signal strength during scanning

### No IP Address
- Verify DHCP is enabled on router
- Check for IP address conflicts
- Try different hostname in WiFiConfig
- Monitor router DHCP lease table

### Build Issues
- Use git versions of esp-hal and esp-wifi for compatibility
- Set `CARGO_CFG_PORTABLE_ATOMIC_UNSAFE_ASSUME_SINGLE_CORE` environment variable
- Clean build: `cargo clean && cargo build --release`

### Container Issues
- Restart container: `podman-compose restart`
- Verify probe access: `probe-rs list`
- Check USB device permissions

## Integration with MQTT

This module is designed to integrate seamlessly with MQTT clients:
1. **Network stack** - Compatible with TCP socket requirements
2. **IP information** - Provides connection details for broker setup
3. **Status monitoring** - Can detect connection loss for reconnection
4. **Simple design** - No complex lifetimes that interfere with MQTT timing

## Performance Notes

- Release builds recommended for stable WiFi performance
- Network stack operations are blocking - suitable for MQTT applications
- DHCP client handles lease renewal automatically
- Connection monitoring has minimal overhead
- RTT logging can be removed for production builds

## Future Enhancements

- WiFi reconnection on connection loss
- Multiple network configuration profiles
- WPA Enterprise support
- WiFi AP mode capability
- Signal strength monitoring and reporting