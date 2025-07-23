# Blinky - ESP32-C3 Basic LED Example

A minimal ESP32-C3 project that demonstrates basic GPIO control with Real-Time Transfer (RTT) console output.

## Purpose

This module serves as:
- **Template starting point** for new ESP32-C3 projects
- **Hardware verification** tool to ensure development environment is working
- **GPIO demonstration** showing basic digital output control
- **RTT console example** for debugging and logging

## Features

- LED blinking on GPIO8 (onboard LED)
- 500ms toggle interval
- Timestamped RTT console output
- Minimal dependencies with stable ESP-HAL
- Release build optimized for size and performance

## Hardware Requirements

- ESP32-C3 development board
- USB cable for programming and power
- Onboard LED connected to GPIO8 (standard on most dev boards)

## Dependencies

```toml
[dependencies]
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
rtt-target = "0.5"
panic-rtt-target = "0.1"
```

## Usage

### Build and Run
```bash
# Navigate to module
cd workspace/blinky

# Build release version (recommended)
cargo build --release

# Flash and run with live console output
cargo run --release
```

### Expected Output
```
10:02:47.191: esp32-c3 is booting!
10:02:47.703: status: High
10:02:48.216: status: Low  
10:02:48.728: status: High
...
```

### Customization

#### Change LED Pin
```rust
// Modify GPIO pin in src/main.rs
let mut led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
```

#### Adjust Timing
```rust
// Change delay interval
delay.delay_millis(1000); // 1 second blink
```

#### Add Console Messages
```rust
// Enhanced logging
rprintln!("{}: LED state: {}", time_ms, if led_state { "ON" } "OFF" });
```

## Code Structure

- `src/main.rs` - Main application with RTT initialization and LED control loop
- `build.rs` - Linker configuration for ESP32-C3 memory layout
- `Cargo.toml` - Project dependencies and metadata
- `.cargo/config.toml` - Cargo runner configuration for probe-rs

## Troubleshooting

### LED Not Blinking
- Verify ESP32-C3 board has onboard LED on GPIO8
- Check USB connection and power
- Ensure container has device access with `probe-rs list`

### No Console Output
- Verify RTT initialization with `rtt_init_print!()`
- Check probe-rs runner configuration in `.cargo/config.toml`
- Use `cargo objdump --release -- -s | grep SEGGER` to verify RTT symbols

### Build Errors
- Clean build artifacts: `cargo clean`
- Update dependencies: `cargo update`
- Verify Rust toolchain: `rustc --version`

## Integration Notes

This module can be used as a foundation for more complex projects:
- Add WiFi connectivity (see wifi-simple module)
- Implement sensor readings with I2C/SPI
- Create MQTT communication systems
- Build real-time control applications

## Performance Notes

- Release builds are significantly faster and smaller
- RTT output has minimal performance impact
- GPIO operations are optimized by ESP-HAL
- Consider using embassy-time for more precise timing