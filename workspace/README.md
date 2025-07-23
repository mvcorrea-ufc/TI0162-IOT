# ESP32-C3 Rust Development Workspace

This workspace contains modular ESP32-C3 projects demonstrating progressive embedded development with Rust, from basic GPIO control to WiFi connectivity.

## Module Overview

### blinky/
**Purpose**: Basic ESP32-C3 LED control and RTT console output  
**Level**: Beginner  
**Features**: GPIO control, RTT debugging, hardware verification  
**Use Cases**: Template starting point, development environment testing, GPIO learning

### wifi-simple/
**Purpose**: Modular WiFi connectivity with DHCP and network stack  
**Level**: Intermediate  
**Features**: WiFi connection, DHCP IP acquisition, network monitoring, TCP stack  
**Use Cases**: IoT connectivity, MQTT preparation, network applications

## Development Workflow

### Module Progression
1. **Start with blinky** - Verify hardware and development environment
2. **Move to wifi-simple** - Add network connectivity to your projects
3. **Combine modules** - Use wifi-simple as foundation for IoT applications

### Quick Start
```bash
# Test basic functionality
cd workspace/blinky
cargo run --release

# Add WiFi capability  
cd ../wifi-simple
# Configure WiFi credentials in .cargo/config.toml
cargo run --release
```

## Module Dependencies

### Blinky Dependencies
- **esp-hal**: Hardware abstraction layer
- **rtt-target**: Console debugging output
- **panic-rtt-target**: Panic handler with RTT

### WiFi-Simple Dependencies
- **esp-hal**: Hardware abstraction (git version for latest features)
- **esp-wifi**: WiFi connectivity support
- **smoltcp**: TCP/IP network stack
- **blocking-network-stack**: Synchronous network operations
- **esp-alloc**: Heap allocator for WiFi operations

## Configuration Files

### Global Configuration
- `Cargo.toml` - Workspace configuration and shared dependencies
- `rust-toolchain.toml` - Rust toolchain specification for ESP32-C3

### Module-Specific Configuration
- Each module has its own `Cargo.toml` with specific dependencies
- WiFi modules require `.cargo/config.toml` with network credentials

## Hardware Requirements

### Minimum Requirements
- ESP32-C3 development board (ESP32-C3-DevKitC-02 recommended)
- USB cable for programming and power
- Development container with probe-rs and Rust toolchain

### Additional for WiFi Modules
- Access to 2.4GHz WiFi network (ESP32-C3 doesn't support 5GHz)
- Router with DHCP enabled
- Network allowing new device connections

## Development Environment

### Container Setup
All modules run in a containerized development environment with:
- Ubuntu 22.04 base with Rust toolchain
- probe-rs for ESP32-C3 flashing and debugging  
- SSH access on port 2222
- USB device passthrough for hardware access
- RTT support for real-time console output

### Build Commands (All Modules)
```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized, recommended)
cargo build --release

# Flash and run with console
cargo run --release

# Clean build artifacts
cargo clean
```

## Module Integration Patterns

### Using Blinky as Base
```rust
// Copy blinky structure
// Add new peripherals (I2C, SPI, etc.)
// Extend with sensor readings
// Implement custom protocols
```

### Building on WiFi-Simple
```rust
// Use WiFi connection from wifi-simple
// Add MQTT client functionality
// Implement HTTP REST APIs
// Create IoT sensor networks
```

### Combining Modules
```rust
// Start with wifi-simple network stack
// Add blinky LED status indicators
// Implement application-specific logic
// Create complete IoT solutions
```

## Common Integration Examples

### MQTT Client Project
1. Copy wifi-simple as foundation
2. Add rust-mqtt dependency
3. Use existing network stack for TCP connections
4. Implement publish/subscribe functionality

### HTTP Client Project  
1. Start with wifi-simple module
2. Add reqwest or similar HTTP client
3. Use established IP connection
4. Implement REST API calls

### Sensor Data Logger
1. Begin with blinky for GPIO understanding
2. Add I2C/SPI sensor code
3. Integrate wifi-simple for connectivity
4. Implement data transmission protocols

## Best Practices

### Module Development
- Start simple with blinky patterns
- Add complexity incrementally
- Test each module independently
- Use consistent error handling
- Document module interfaces clearly

### Network Applications
- Always use release builds for stable WiFi
- Implement connection loss recovery
- Use blocking operations for MQTT compatibility
- Monitor network status continuously
- Handle DHCP lease renewal

### Code Organization
- Keep modules focused on single responsibilities  
- Use descriptive function names
- Implement proper error types
- Add comprehensive logging
- Create meaningful README files

## Testing Strategy

### Module Testing
```bash
# Test blinky first to verify hardware
cd blinky && cargo run --release

# Test WiFi connectivity
cd ../wifi-simple && cargo run --release

# Verify network accessibility
ping <displayed-ip-address>
```

### Integration Testing
```bash
# Test module combination
# Copy code between modules
# Verify compatibility
# Test error conditions
```

## Troubleshooting

### Common Issues
- **Probe not found**: Restart container and verify USB connections
- **WiFi connection fails**: Check credentials and 2.4GHz network
- **No IP address**: Verify DHCP router configuration
- **Build errors**: Clean build and update dependencies

### Debug Strategies
- Use RTT console output for real-time debugging
- Check container logs with `podman-compose logs`
- Verify device detection with `probe-rs list`
- Monitor network with ping and router logs

## Performance Considerations

### Build Optimization
- Always use `--release` for final builds
- Release builds are significantly faster and smaller
- Debug builds useful for development but slower on ESP32-C3

### Runtime Performance
- WiFi operations require heap allocation (configured in modules)
- Network operations are blocking by design for simplicity
- RTT output has minimal performance impact
- LED operations are optimized by ESP-HAL

## Future Expansion

### Planned Modules
- **mqtt-client**: MQTT connectivity building on wifi-simple
- **sensor-i2c**: I2C sensor reading with blinky foundation
- **web-server**: HTTP server using WiFi network stack
- **ble-simple**: Bluetooth Low Energy connectivity

### Module Templates
Each new module should follow this pattern:
- README.md with clear purpose and usage
- Cargo.toml with specific dependencies
- src/main.rs with documented examples  
- Integration examples and troubleshooting
- Performance notes and best practices

## Contributing

When adding new modules:
1. Follow existing naming conventions
2. Create comprehensive README files
3. Include working examples
4. Document integration patterns
5. Add troubleshooting sections
6. Test with both debug and release builds
7. Verify container compatibility