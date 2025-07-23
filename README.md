# ESP32-C3 Rust Development Template

A comprehensive template for ESP32-C3 embedded development in Rust with modular examples from basic GPIO to WiFi connectivity.

## Features

- **Modular project structure** with progressive complexity levels
- **Direct console output** via RTT - no helper scripts needed
- **Container-based development** with full USB hardware access
- **WiFi connectivity module** with DHCP and network stack
- **Clean modular architecture** ready for MQTT and IoT applications
- **Comprehensive documentation** with usage examples and troubleshooting

## Hardware Requirements

- ESP32-C3 development board (ESP32-C3-DevKitC-02 or compatible)
- USB cable for programming and power

## Software Requirements

- **Remote or Local Linux Server** with Podman/Docker installed (from now we will call it HOST)
- **Rust toolchain** (will be installed in container)
- **probe-rs** for flashing (will be installed in container)
- **VSCode** with Remote-SSH extension (for IDE development)

## Available Modules

### blinky/
**Basic ESP32-C3 LED control and RTT console**
- Hardware verification and GPIO control
- RTT debugging setup
- Perfect starting point for new projects

### wifi-simple/  
**Modular WiFi connectivity with network stack**
- DHCP IP acquisition with real IP display
- Network monitoring and status reporting
- Ready for MQTT and TCP applications
- Clean modular architecture without complex lifetimes

## Quick Start

### 1. Repository Setup

```bash
# Clone this template (on the HOST)
git clone https://github.com/mvcorrea-ufc/rust-esp32-tmpl.git my-esp32c3-project
cd my-esp32c3-project

# Initialize as new repository
rm -rf .git
git init
git add .
git commit -m "Initial ESP32-C3 project from template"
```

### 2. Development Environment

**On HOST server (always where you got podman/docker installed):**

```bash
# Navigate to project root directory (where Dockerfile and podman-compose.yml are)
cd my-esp32c3-project

# Build and start development container (or use docker-compose with '-f podman-compose.yml')
podman-compose up --build -d

# Then you should have your container up and running

# To connect to it you could
podman-compose exec esp32dev bash
or via ssh
ssh root@HOST -p2222 (where HOST is the place podman/docker is installed)

# Verify ESP32-C3 detection
podman-compose exec esp32dev probe-rs list
# Should show: ESP JTAG -- 303a:1001:F0:F5:BD:C9:4A:90 (EspJtag)
# from time to time we loose the /dev/tty* access, then you should restart the container 'podman-compose restart'
```

### 3. Choose Your Starting Point

#### Option A: Basic Hardware Test (Recommended First)
```bash
# Enter development container
podman-compose exec esp32dev bash

# Navigate to blinky module
cd /workspace/blinky

# Build and run basic LED example
cargo run --release
```

**Expected Output:**
```
10:02:47.191: esp32-c3 is booting!
10:02:47.703: status: High
10:02:48.216: status: Low  
10:02:48.728: status: High
...
```

#### Option B: WiFi Connectivity
```bash
# Enter development container
podman-compose exec esp32dev bash

# Configure WiFi credentials first
cd /workspace/wifi-simple
# Edit .cargo/config.toml with your WiFi network details

# Build and run WiFi example
cargo run --release
```

**Expected Output:**
```
ESP32-C3 WiFi Simple - Clean Modular Implementation
Config: SSID=YourNetwork Hostname=ESP32-C3-WiFi-Test
WiFi: Scanning for networks...
WiFi: Connected successfully
DHCP: IP address acquired successfully
Network: IP=192.168.1.100 Gateway=192.168.1.1 Subnet=/24 DNS=Some(8.8.8.8)
NETWORK READY
Status: CONNECTED IP=192.168.1.100 GW=192.168.1.1
```

## Project Structure

```
rust-esp32-tmpl/
├── .vscode/                # VSCode configuration
│   ├── settings.json       # Rust-analyzer and editor settings
│   ├── extensions.json     # Recommended extensions
│   ├── launch.json         # Debug configurations
│   └── tasks.json          # Build and run tasks
├── workspace/              # Development modules (see workspace/README.md)
│   ├── blinky/             # Basic LED control example
│   │   ├── src/main.rs     # GPIO control with RTT console
│   │   ├── Cargo.toml      # Basic ESP-HAL dependencies
│   │   ├── build.rs        # Linker configuration
│   │   └── README.md       # Module documentation
│   ├── wifi-simple/        # WiFi connectivity module
│   │   ├── src/
│   │   │   ├── main.rs     # WiFi application example
│   │   │   └── wifi.rs     # Modular WiFi helper functions
│   │   ├── .cargo/
│   │   │   └── config.toml # WiFi credentials configuration
│   │   ├── Cargo.toml      # WiFi and networking dependencies
│   │   ├── build.rs        # Linker configuration
│   │   └── README.md       # WiFi module documentation
│   ├── Cargo.toml          # Workspace configuration
│   ├── rust-toolchain.toml # Rust toolchain specification
│   └── README.md           # Workspace and module overview
├── Dockerfile              # Container build configuration
├── podman-compose.yml      # Container orchestration
└── README.md               # This file
```

## Key Files Explained

### `main.rs` - RTT Implementation
```rust
use rtt_target::{rprintln, rtt_init_print};

#[main]
fn main() -> ! {
    // Initialize RTT for console output
    rtt_init_print!();
    
    // Your application code...
    rprintln!("Hello ESP32-C3!");
}
```

### `Cargo.toml` - Dependencies
```toml
[dependencies]
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
rtt-target = "0.5"
panic-rtt-target = "0.1"
```

### `.cargo/config.toml` - Runner Configuration
```toml
[target.riscv32imc-unknown-none-elf]
runner = "probe-rs run --chip=esp32c3 --preverify --always-print-stacktrace --no-location --catch-hardfault"
```

## Common Commands

### Build Commands
```bash
# Debug build (faster compilation, larger binary)
cargo build

# Release build (optimized for size/performance)
cargo build --release

# Clean build artifacts
cargo clean
```

### Run Commands
```bash
# Flash and run with RTT console output
cargo run --release

# Just flash without console (background)
probe-rs run --chip=esp32c3 target/riscv32imc-unknown-none-elf/release/blinky
```

### Debug Commands
```bash
# List connected ESP32 devices
probe-rs list

# Check if RTT symbols are compiled into binary
cargo objdump --release -- -s | grep SEGGER

# Attach to running target for RTT debugging
probe-rs attach --chip esp32c3 target/riscv32imc-unknown-none-elf/release/blinky
```

## Development Workflow

### 1. Code Changes
```bash
# Edit source code
vim src/main.rs

# Build and test
cargo run --release

# Ctrl+C to stop and flash new changes
```

### 2. Adding Dependencies
```bash
# Add new crate to Cargo.toml
[dependencies]
serde = { version = "1.0", default-features = false }

# Update and build
cargo build
```

### 3. GPIO Configuration
```rust
// Configure different GPIO pin
let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

// Use different delay timing
delay.delay_millis(1000); // 1 second
```

## Troubleshooting

### "Probe not found" Error
```bash
# Restart container to refresh USB connections
podman-compose down && podman-compose up -d

# Verify device detection
podman-compose exec esp32dev probe-rs list

# Check USB device mapping in host
lsusb | grep Espressif
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update

# Check toolchain
rustc --version --verbose
```

### No Console Output
```bash
# Verify RTT is initialized
grep -n "rtt_init_print" src/main.rs

# Check for SEGGER symbols in binary
cargo objdump --release -- -s | grep SEGGER

# Test with simple rprintln
rprintln!("Debug: RTT working!");
```

### Container Access Issues
```bash
# Verify container is running
podman-compose ps

# Check SSH access
ssh root@HOST -p 2222  # password: rootpass

# Restart if needed
podman-compose restart esp32dev
```

## Customization

### Change LED Pin
```rust
// In main.rs, modify GPIO pin
let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());
```

### Add More Peripherals
```rust
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    i2c::I2C,
    spi::Spi,
};

// Configure I2C, SPI, etc.
```

### Custom Console Output
```rust
// Format messages
rprintln!("Temperature: {:.2}°C", temp);

// Debug with multiple values
rprintln!("GPIO: {} | Counter: {}", pin_state, counter);

// Conditional output
if error_condition {
    rprintln!("ERROR: {}", error_msg);
}
```

## VSCode Remote Development Setup

### Prerequisites
1. **VSCode** installed on your local machine
2. **Remote-SSH extension** installed in VSCode
3. **SSH access** to your remote server (HOST)

### Step 1: Configure SSH Connection

Add to your local `~/.ssh/config`:
```
Host esp32-dev
    HostName <HOST IP>
    Port 2222
    User root
    PasswordAuthentication yes
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
```

### Step 2: Connect with VSCode

1. **Open VSCode** on your local machine
2. **Press** `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
3. **Type** "Remote-SSH: Connect to Host"
4. **Select** `esp32-dev` from the list
5. **Enter password** when prompted: `rootpass`
6. **Open folder** `/workspace` in the remote VSCode window

### Step 3: Install Recommended Extensions

VSCode will automatically suggest installing the recommended extensions defined in `.vscode/extensions.json`:

- **rust-analyzer** - Rust language support
- **probe-rs-debugger** - ESP32 debugging support
- **Embedded Tools** - Additional embedded development tools
- **Hex Editor** - For binary file inspection

### Step 4: Development Workflow in VSCode

#### Building and Running
- **Build Debug**: `Ctrl+Shift+P` → "Tasks: Run Task" → "cargo-build-debug"
- **Build Release**: `Ctrl+Shift+P` → "Tasks: Run Task" → "cargo-build-release"
- **Flash and Run**: `Ctrl+Shift+P` → "Tasks: Run Task" → "cargo-run-release"
- **Clean**: `Ctrl+Shift+P` → "Tasks: Run Task" → "cargo-clean"

#### Debugging
1. **Set breakpoints** in `workspace/blinky/src/main.rs`
2. **Press** `F5` or go to Run and Debug panel
3. **Select** "ESP32-C3 Debug" or "ESP32-C3 Release Debug"
4. **Debug session** will start with probe-rs debugger

#### Terminal Access
- **Open Terminal**: `Ctrl+`` (backtick)
- **Navigate to project**: Terminal opens in `/workspace/blinky` by default
- **Run commands**: All cargo and probe-rs commands work directly

### Step 5: Key VSCode Features

#### Rust-Analyzer Integration
- **Auto-completion** for ESP-HAL functions
- **Error highlighting** and quick fixes
- **Type hints** and parameter suggestions
- **Go to definition** for ESP32-C3 peripherals
- **Documentation on hover** for HAL functions

#### Embedded Development
- **Device detection**: Use Command Palette → "probe-rs-list"
- **RTT output**: Integrated console shows RTT messages during debugging
- **Memory viewer**: Inspect ESP32-C3 memory during debugging
- **Register viewer**: Monitor peripheral registers

#### Project Navigation
- **File Explorer**: Navigate project structure
- **Symbol search**: `Ctrl+Shift+O` for functions and types
- **Global search**: `Ctrl+Shift+F` across entire codebase
- **Git integration**: Built-in version control

### Troubleshooting VSCode Setup

#### Connection Issues
```bash
# Test SSH connection manually
ssh root@HOST -p 2222

# Restart container if connection fails
podman-compose restart esp32dev
```

#### Extension Issues
- **Rust-analyzer not working**: Restart VSCode Remote session
- **Debug not starting**: Verify ESP32-C3 is connected with "probe-rs-list" task
- **RTT not showing**: Check that `rtt_init_print!()` is called in main.rs

#### Performance Tips
- **Exclude target folder**: Already configured in `.vscode/settings.json`
- **Disable unused extensions**: Only install recommended embedded extensions
- **Use release builds**: Debug builds are slower on ESP32-C3

### Alternative: Terminal-Only Development

If you prefer terminal-based development:
```bash
# SSH directly to container
ssh root@HOST -p 2222

# Use vim/nano for editing
cd /workspace/blinky
vim src/main.rs

# Build and run as normal
cargo run --release
```

## Container Configuration

The development environment uses a privileged Podman container with:
- **USB device mapping** for ESP32-C3 access
- **SSH server** on port 2222 (password: `rootpass`)
- **Rust toolchain** with RISC-V target
- **probe-rs tools** for flashing and debugging
- **Git version control** for project management

## Module Development Workflow

### Starting New Projects
1. **Begin with blinky** - Verify hardware and development environment
2. **Progress to wifi-simple** - Add network connectivity to your application
3. **Combine functionality** - Use wifi-simple as foundation for IoT projects

### Common Integration Patterns
```rust
// MQTT Client: wifi-simple + rust-mqtt
// HTTP Client: wifi-simple + reqwest  
// Sensor Network: blinky + wifi-simple + sensor libraries
// IoT Dashboard: wifi-simple + web server + sensor data
```

## WiFi Configuration

For wifi-simple module, configure your network in `.cargo/config.toml`:
```toml
[env]
# WiFi credentials - Replace with your network details
WIFI_SSID = "YourNetworkName"
WIFI_PASSWORD = "YourNetworkPassword"
CARGO_CFG_PORTABLE_ATOMIC_UNSAFE_ASSUME_SINGLE_CORE = ""
```

**Important**: Replace the placeholder values with your actual WiFi network credentials before building the wifi-simple module.

## Next Steps

1. **Start with blinky** - Verify your hardware setup works correctly
2. **Try wifi-simple** - Add network connectivity to your projects  
3. **Read module READMEs** - Each module has detailed documentation
4. **Build MQTT applications** - Use wifi-simple as foundation
5. **Add sensors** - Integrate I2C/SPI sensors with existing modules
6. **Create IoT solutions** - Combine modules for complete applications

## License

This template is provided as-is for educational and development purposes.
