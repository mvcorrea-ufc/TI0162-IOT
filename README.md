# ESP32-C3 RTT Development Template

A clean, minimal template for ESP32-C3 embedded development in Rust with Real-Time Transfer (RTT) console output.

## Features

- **Direct console output** in `cargo run` via RTT - no helper scripts needed
- **Minimal dependencies** with stable ESP-HAL 0.23.1
- **Container-based development** with full USB hardware access
- **LED blinking example** on GPIO8 with 500ms timing
- **Timestamped console output** for easy debugging

## Hardware Requirements

- ESP32-C3 development board (ESP32-C3-DevKitC-02 or compatible)
- USB cable for programming and power

## Software Requirements

- **Remote Linux Server** with Podman/Docker installed
- **Rust toolchain** (installed in container)
- **probe-rs** for flashing (installed in container)
- **VSCode** with Remote-SSH extension (for IDE development)

## Quick Start

### 1. Repository Setup

```bash
# Clone this template
git clone <repository-url> my-esp32c3-project
cd my-esp32c3-project

# Initialize as new repository
rm -rf .git
git init
git add .
git commit -m "Initial ESP32-C3 RTT project from template"
```

### 2. Development Environment

**On Remote Server (10.10.10.217):**

```bash
# Navigate to project directory
cd podman/new_tmpl/

# Build and start development container
podman-compose up --build -d

# Verify ESP32-C3 detection
podman-compose exec test probe-rs list
# Should show: ESP JTAG -- 303a:1001:F0:F5:BD:C9:4A:90 (EspJtag)
```

### 3. Build and Run

```bash
# Enter development container
podman-compose exec test bash

# Navigate to project
cd /workspace/blinky

# Build project
cargo build --release

# Flash and run with live console output
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

## Project Structure

```
new_tmpl/
├── .vscode/                # VSCode configuration
│   ├── settings.json       # Rust-analyzer and editor settings
│   ├── extensions.json     # Recommended extensions
│   ├── launch.json         # Debug configurations
│   └── tasks.json          # Build and run tasks
├── workspace/
│   ├── blinky/             # Main ESP32-C3 project
│   │   ├── .cargo/
│   │   │   └── config.toml # Cargo runner configuration
│   │   ├── src/
│   │   │   └── main.rs     # Main application with RTT
│   │   ├── Cargo.toml      # Project dependencies
│   │   └── build.rs        # Linker configuration
│   ├── Cargo.toml          # Workspace configuration
│   ├── .gitignore          # Git ignore patterns
│   └── README.md           # This file
└── [other files...]        # Container and Docker configs
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
podman-compose exec test probe-rs list

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
ssh root@localhost -p2222  # password: rootpass

# Restart if needed
podman-compose restart test
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
3. **SSH access** to your remote server (10.10.10.217)

### Step 1: Configure SSH Connection

Add to your local `~/.ssh/config`:
```
Host esp32-dev
    HostName 10.10.10.217
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
6. **Open folder** `/workspace/new_tmpl` in the remote VSCode window

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
ssh root@10.10.10.217 -p 2222

# Restart container if connection fails
podman-compose restart test
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
ssh root@10.10.10.217 -p 2222

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

## Next Steps

1. **Explore ESP-HAL**: Check [esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) for more peripheral examples
2. **Add Sensors**: Integrate I2C/SPI sensors with RTT logging
3. **WiFi Connectivity**: Use esp-wifi crate for network features
4. **Real-time Features**: Implement embassy-time for async operations
5. **Custom Bootloader**: Configure secure boot and OTA updates

## License

This template is provided as-is for educational and development purposes.