# Development Guide

> Complete development setup and workflow guide for the ESP32-C3 IoT Environmental Monitoring System

## Development Environment Setup

### Prerequisites

#### 1. Rust Development Environment
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install ESP32-C3 target
rustup target add riscv32imc-unknown-none-elf

# Install nightly for advanced features (optional)
rustup install nightly
rustup component add rust-src --toolchain nightly
```

#### 2. ESP32-C3 Development Tools
```bash
# Install probe-rs for flashing and debugging
cargo install probe-rs --features cli

# Install additional development tools
cargo install cargo-expand        # Macro expansion debugging
cargo install cargo-audit         # Security vulnerability scanning
cargo install cargo-bloat         # Binary size analysis
cargo install cargo-watch        # Automatic rebuild on file changes

# Verify probe-rs installation
probe-rs --version
probe-rs list  # Should detect connected ESP32-C3
```

#### 3. MQTT Development Environment
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install mosquitto mosquitto-clients

# macOS
brew install mosquitto

# Start MQTT broker
sudo systemctl start mosquitto  # Linux
brew services start mosquitto   # macOS

# Test MQTT broker
mosquitto_pub -h localhost -t test -m "hello"
mosquitto_sub -h localhost -t test
```

### IDE Setup

#### Visual Studio Code (Recommended)
```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb
code --install-extension ms-vscode.hexeditor
```

**VS Code Settings** (`.vscode/settings.json`):
```json
{
    "rust-analyzer.cargo.target": "riscv32imc-unknown-none-elf",
    "rust-analyzer.check.overrideCommand": [
        "cargo", "check", "--workspace", "--message-format=json"
    ],
    "rust-analyzer.cargo.features": ["esp32c3"],
    "files.watcherExclude": {
        "**/target/**": true
    }
}
```

#### CLion / IntelliJ IDEA
- Install Rust plugin
- Configure build target: `riscv32imc-unknown-none-elf`
- Set up run configurations for each module

## Project Structure and Workflow

### Workspace Organization
```
workspace/
├── Core Modules/           # Production-ready modules
├── Applications/          # Integrated applications  
├── Reference/            # Example implementations
├── Documentation/        # Comprehensive docs
└── Configuration/        # Build and toolchain config
```

### Development Workflow

#### 1. Module Development Cycle
```bash
# Navigate to module directory
cd workspace/bme280-embassy/

# Development cycle
cargo check              # Fast syntax/type checking
cargo clippy             # Lint and suggestions
cargo fmt               # Code formatting
cargo test              # Run unit tests (if any)
cargo build --release   # Optimized build
cargo run --release     # Flash and run

# Clean build artifacts
cargo clean
```

#### 2. Cross-Module Development
```bash
# Work from workspace root for cross-module development
cd workspace/

# Check all modules
cargo check --workspace

# Build specific module from workspace
cargo build -p bme280-embassy --release
cargo build -p main-app --release

# Run integration tests
cargo run -p main-app --release
```

#### 3. Continuous Development
```bash
# Automatic rebuild on changes (requires cargo-watch)
cargo watch -x 'check --workspace'
cargo watch -x 'run -p main-app --release'
```

### Build Configuration

#### Module-Specific Configuration
Each module has its own `.cargo/config.toml` for environment-specific settings:

```toml
# Example: wifi-embassy/.cargo/config.toml
[build]
target = "riscv32imc-unknown-none-elf"

[target.riscv32imc-unknown-none-elf]
runner = "probe-rs run --chip esp32c3"

[env]
WIFI_SSID = "Development_Network"
WIFI_PASSWORD = "dev_password_here"
DEFMT_LOG = "info"  # Logging level for defmt
```

#### Development vs Production Builds
```bash
# Development build (faster compilation, debug symbols)
cargo build

# Production build (optimized, smaller binary)
cargo build --release

# Size-optimized build
cargo build --release --config 'profile.release.opt-level="z"'
```

## Testing Strategy

### Unit Testing
```bash
# Run unit tests for specific module
cd workspace/iot-common/
cargo test

# Run tests with no_std compatibility
cargo test --no-default-features

# Run doctests
cargo test --doc
```

### Integration Testing
```bash
# Test hardware integration (requires connected hardware)
cargo run -p blinky --release           # Basic hardware test
cargo run -p bme280-embassy --release   # Sensor integration
cargo run -p wifi-embassy --example wifi_test --release  # Network test
```

### Embedded Testing Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock hardware for unit testing
    struct MockI2c {
        responses: Vec<u8>,
    }
    
    #[tokio::test]  // For async tests
    async fn test_sensor_reading() {
        let mut mock_i2c = MockI2c::new();
        let mut sensor = BME280::new(mock_i2c);
        
        // Test sensor logic without hardware
        assert!(sensor.init().await.is_ok());
    }
}
```

## Debugging Techniques

### Real-Time Transfer (RTT) Debugging
RTT provides real-time bidirectional communication for debugging:

```rust
use rtt_target::{rprintln, rtt_init_print};

#[no_mangle]
pub fn main() -> ! {
    rtt_init_print!();
    
    rprintln!("System starting...");
    rprintln!("Debug value: {}", some_variable);
    
    // ... rest of code
}
```

#### RTT Debugging Commands
```bash
# Flash with RTT output
cargo run --release

# Manual RTT connection (if needed)
probe-rs attach --chip esp32c3
```

### Advanced Debugging

#### Hardware Debugging with probe-rs
```bash
# Attach debugger
probe-rs attach --chip esp32c3

# Debug with GDB (requires gdb-multiarch)
probe-rs gdb --chip esp32c3

# Memory inspection
probe-rs read --chip esp32c3 0x3FC80000 256
```

#### Network Debugging
```bash
# Monitor network traffic
tcpdump -i wlan0 host [ESP32_IP]

# MQTT traffic monitoring
mosquitto_sub -h [BROKER_IP] -t "esp32/#" -v

# WiFi signal debugging
iwconfig  # Linux
airport -s  # macOS
```

### Common Debugging Scenarios

#### 1. I2C Communication Issues
```rust
// Add I2C debugging
rprintln!("I2C Status: Writing to addr 0x{:02x}", address);
match i2c.write(address, &data).await {
    Ok(()) => rprintln!("I2C Write successful"),
    Err(e) => rprintln!("I2C Write failed: {:?}", e),
}
```

#### 2. WiFi Connection Problems
```rust
// WiFi status debugging
rprintln!("WiFi Controller Status: {:?}", controller.status());
rprintln!("Network Config: {:?}", stack.config_v4());
```

#### 3. Memory Usage Analysis
```bash
# Analyze binary size
cargo bloat --release --crates

# Memory layout inspection
cargo size --release -- -A

# Stack usage analysis (requires nightly)
RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release
```

## Performance Optimization

### Build Optimization
```toml
# Cargo.toml - Optimize for size and speed
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Enable link-time optimization
codegen-units = 1        # Better optimization, slower compile
panic = "abort"          # Smaller binary size
strip = true            # Remove debug symbols
```

### Runtime Optimization

#### Memory Management
```rust
// Use stack allocation where possible
use heapless::{String, Vec};

// Bounded collections for embedded
let mut buffer: String<64> = String::new();
let mut data: Vec<u8, 32> = Vec::new();

// Static allocation for frequently used data
use static_cell::StaticCell;
static WIFI_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();
```

#### Async Task Optimization
```rust
// Proper task sizing
#[embassy_executor::task(pool_size = 4)]  // Task pool for multiple instances
async fn sensor_task() {
    // Task implementation
}

// Task priority management
#[embassy_executor::task]
async fn high_priority_task() {
    // Critical system tasks
}
```

### Performance Profiling
```bash
# Runtime performance analysis
cargo run --release 2>&1 | grep "timing"

# Memory usage profiling
RUST_LOG=debug cargo run --release 2>&1 | grep "heap"
```

## Code Quality Standards

### Code Style and Formatting
```bash
# Apply standard Rust formatting
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Custom formatting rules (rustfmt.toml)
edition = "2021"
max_width = 100
tab_spaces = 4
```

### Linting and Analysis
```bash
# Standard linting
cargo clippy

# Strict linting for production
cargo clippy -- -W clippy::all -W clippy::pedantic

# Security audit
cargo audit

# License compliance check
cargo deny check
```

### Documentation Standards
```rust
/// Reads environmental data from BME280 sensor
/// 
/// # Arguments
/// * `address` - I2C address of the sensor (0x76 or 0x77)
/// 
/// # Returns
/// * `Ok(Measurements)` - Successfully read sensor data
/// * `Err(IoTError)` - I2C communication failure or invalid data
/// 
/// # Examples
/// ```no_run
/// let mut sensor = BME280::new(i2c);
/// let data = sensor.read_measurements().await?;
/// println!("Temperature: {:.2}°C", data.temperature);
/// ```
/// 
/// # Hardware Requirements
/// - BME280 sensor connected via I2C
/// - GPIO8 (SDA) and GPIO9 (SCL) properly configured
/// - 3.3V power supply stable
pub async fn read_measurements(&mut self) -> IoTResult<Measurements> {
    // Implementation
}
```

### Error Handling Standards
```rust
use iot_common::{IoTResult, IoTError, SensorError};

// Always use Result types for fallible operations
pub async fn sensor_operation() -> IoTResult<f32> {
    match hardware_read().await {
        Ok(value) => Ok(process_value(value)),
        Err(e) => {
            let sensor_error = SensorError::I2CError(
                error_message("Sensor communication failed")
            );
            Err(IoTError::sensor(sensor_error)
                .with_context("Temperature reading"))
        }
    }
}
```

## Git Workflow

### Branch Strategy
```bash
# Main development branch
git checkout master

# Feature development
git checkout -b feature/sensor-calibration
git push -u origin feature/sensor-calibration

# Bug fixes
git checkout -b bugfix/wifi-reconnection
```

### Commit Message Standards
```
type(scope): brief description

Detailed explanation of what changed and why.

- Specific change 1
- Specific change 2

Resolves: #123
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Pre-commit Hooks
```bash
# Install pre-commit hooks
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
EOF

chmod +x .git/hooks/pre-commit
```

## Troubleshooting Development Issues

### Common Build Issues
1. **Target not installed**
   ```bash
   rustup target add riscv32imc-unknown-none-elf
   ```

2. **probe-rs not found**
   ```bash
   cargo install probe-rs --features cli
   export PATH="$HOME/.cargo/bin:$PATH"
   ```

3. **Permission denied on serial port**
   ```bash
   sudo usermod -a -G dialout $USER  # Linux
   # Logout and login again
   ```

### Hardware Debugging Issues
1. **ESP32-C3 not detected**
   - Press BOOT button while connecting USB
   - Check USB cable (data, not charging only)
   - Try different USB port

2. **I2C communication failures**
   - Verify pull-up resistors (typically 4.7kΩ)
   - Check connections and power supply
   - Verify sensor address (0x76 or 0x77)

3. **WiFi connection issues**
   - Ensure 2.4GHz network (ESP32-C3 doesn't support 5GHz)
   - Check SSID and password in configuration
   - Verify network allows new device connections

### Performance Issues
1. **Slow compilation**
   ```bash
   # Enable incremental compilation
   export CARGO_INCREMENTAL=1
   
   # Use faster linker
   sudo apt install lld  # Linux
   ```

2. **Large binary size**
   ```toml
   # Optimize Cargo.toml
   [profile.release]
   opt-level = "z"
   lto = true
   ```

## Contributing Guidelines

### Setting Up Development Environment
1. Fork the repository
2. Clone your fork locally
3. Set up development environment as described above
4. Create feature branch
5. Make changes with tests
6. Submit pull request

### Code Review Checklist
- [ ] Code follows Rust formatting standards
- [ ] All clippy warnings addressed
- [ ] Documentation updated for public APIs
- [ ] Error handling uses iot-common patterns
- [ ] Hardware integration tested
- [ ] No regression in existing functionality

### Documentation Requirements
- All public functions have rustdoc comments
- Examples provided for complex APIs
- Hardware requirements clearly stated
- Error conditions documented
- Integration patterns explained

This development guide provides a comprehensive foundation for working with the ESP32-C3 IoT system. Follow these practices to maintain code quality and ensure successful development outcomes.