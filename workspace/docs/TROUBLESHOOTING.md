# Troubleshooting Guide

> Comprehensive troubleshooting guide for ESP32-C3 IoT Environmental Monitoring System

## Quick Reference

### Emergency Checklist
When the system isn't working, check these first:

1. **Power**: Is the ESP32-C3 getting stable 3.3V power?
2. **Connection**: Is the device detected by `probe-rs list`?
3. **Build**: Does `cargo build --release` complete without errors?
4. **Network**: Is the WiFi network 2.4GHz and accessible?
5. **Broker**: Is the MQTT broker running and reachable?

## Hardware Issues

### ESP32-C3 Device Problems

#### Device Not Detected
**Symptoms**: `probe-rs list` shows no devices

**Diagnostic Steps**:
```bash
# Check USB connection
lsusb | grep -i "espressif\|serial"

# Check device permissions (Linux)
groups $USER | grep dialout
ls -l /dev/ttyUSB* /dev/ttyACM*

# Try different USB ports/cables
# Use data-capable USB cable (not charging-only)
```

**Solutions**:
```bash
# Add user to dialout group (Linux)
sudo usermod -a -G dialout $USER
# Logout and login again

# Reset ESP32-C3 into download mode
# Hold BOOT button, press RESET, release RESET, release BOOT

# Install/update probe-rs
cargo install --force probe-rs --features cli
```

#### Device Detected But Won't Flash
**Symptoms**: Device appears in `probe-rs list` but flashing fails

**Diagnostic Commands**:
```bash
# Verify chip detection
probe-rs info --chip esp32c3

# Try manual reset sequence
probe-rs reset --chip esp32c3

# Check flash memory
probe-rs read --chip esp32c3 0x0000 16
```

**Solutions**:
```bash
# Erase flash completely
probe-rs erase --chip esp32c3

# Use slower flash speed
probe-rs run --chip esp32c3 --speed 1000 your_binary

# Try different USB cable or port
# Ensure stable power supply (not just USB power)
```

#### Random Resets or Crashes
**Symptoms**: Device resets unexpectedly, RTT output stops

**Diagnostic Approach**:
```rust
// Add reset cause detection
use esp_hal::reset::*;

#[no_mangle]
pub fn main() -> ! {
    let reset_reason = get_reset_reason();
    rprintln!("Reset reason: {:?}", reset_reason);
    
    // Continue with normal initialization
}
```

**Common Causes & Solutions**:
- **Brown-out**: Improve power supply stability
- **Watchdog**: Increase watchdog timeout or feed it more frequently
- **Stack overflow**: Increase stack size in `.cargo/config.toml`
- **Memory corruption**: Check for buffer overruns

### Sensor Communication Issues

#### BME280 Not Responding
**Symptoms**: I2C timeout errors, sensor initialization fails

**Diagnostic Script**:
```bash
# Create I2C diagnostic script
cat > check_i2c.py << 'EOF'
#!/usr/bin/env python3
import smbus
import time

bus = smbus.SMBus(1)  # I2C bus 1

# Try both BME280 addresses
addresses = [0x76, 0x77]

for addr in addresses:
    try:
        # Read chip ID register
        chip_id = bus.read_byte_data(addr, 0xD0)
        print(f"Address 0x{addr:02X}: Chip ID = 0x{chip_id:02X}")
        if chip_id == 0x60:
            print("  -> BME280 detected!")
        else:
            print("  -> Unknown chip")
    except Exception as e:
        print(f"Address 0x{addr:02X}: No response ({e})")
EOF

python3 check_i2c.py
```

**Hardware Checks**:
```
1. Wiring verification:
   ESP32-C3    BME280
   --------    ------
   GPIO8   --> SDA
   GPIO9   --> SCL
   3.3V    --> VCC
   GND     --> GND

2. Pull-up resistors:
   - 4.7kΩ resistors on SDA and SCL lines
   - Connected between signal lines and 3.3V

3. Power supply:
   - Stable 3.3V ±5%
   - Sufficient current capability (>100mA)
   - Clean power (add 100nF + 10μF capacitors)
```

**Software Diagnostics**:
```rust
// Add I2C debugging to your code
use rtt_target::rprintln;

async fn diagnose_i2c<I2C>(i2c: &mut I2C) -> IoTResult<()>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    rprintln!("Starting I2C diagnostics...");
    
    // Test both BME280 addresses
    for addr in [0x76, 0x77] {
        rprintln!("Testing address 0x{:02X}...", addr);
        
        let mut chip_id = [0u8; 1];
        match i2c.write_read(addr, &[0xD0], &mut chip_id).await {
            Ok(()) => {
                rprintln!("  Chip ID: 0x{:02X}", chip_id[0]);
                if chip_id[0] == 0x60 {
                    rprintln!("  -> BME280 detected!");
                    return Ok(());
                }
            },
            Err(e) => rprintln!("  Error: {:?}", e),
        }
    }
    
    Err(IoTError::sensor(SensorError::NotResponding(
        error_message("BME280 not found on I2C bus")
    )))
}
```

#### Incorrect Sensor Readings
**Symptoms**: Temperature/humidity/pressure values seem wrong

**Validation Steps**:
```rust
// Add sensor reading validation
fn validate_measurements(measurements: &Measurements) -> IoTResult<()> {
    // Temperature range check
    if measurements.temperature < -40.0 || measurements.temperature > 85.0 {
        return Err(IoTError::sensor(SensorError::InvalidData(
            error_message(&format!("Temperature out of range: {:.2}°C", measurements.temperature))
        )));
    }
    
    // Humidity range check  
    if measurements.humidity < 0.0 || measurements.humidity > 100.0 {
        return Err(IoTError::sensor(SensorError::InvalidData(
            error_message(&format!("Humidity out of range: {:.2}%", measurements.humidity))
        )));
    }
    
    // Pressure range check
    if measurements.pressure < 300.0 || measurements.pressure > 1100.0 {
        return Err(IoTError::sensor(SensorError::InvalidData(
            error_message(&format!("Pressure out of range: {:.2} hPa", measurements.pressure))
        )));
    }
    
    Ok(())
}
```

**Calibration Issues**:
```rust
// Debug calibration data
async fn debug_calibration<I2C>(sensor: &mut BME280<I2C>) -> IoTResult<()>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let cal_data = sensor.get_calibration_data().await?;
    
    rprintln!("Calibration Data:");
    rprintln!("  dig_T1: {}", cal_data.dig_t1);
    rprintln!("  dig_T2: {}", cal_data.dig_t2);
    rprintln!("  dig_T3: {}", cal_data.dig_t3);
    rprintln!("  dig_H1: {}", cal_data.dig_h1);
    rprintln!("  dig_H2: {}", cal_data.dig_h2);
    
    // Check for obviously invalid calibration
    if cal_data.dig_t1 == 0 || cal_data.dig_t1 == 0xFFFF {
        return Err(IoTError::sensor(SensorError::CalibrationError(
            error_message("Invalid temperature calibration data")
        )));
    }
    
    Ok(())
}
```

## Network Issues

### WiFi Connection Problems

#### Can't Connect to WiFi
**Symptoms**: WiFi connection timeout, no IP address assigned

**Diagnostic Steps**:
```rust
// Add detailed WiFi diagnostics
use esp_wifi::wifi::*;

async fn diagnose_wifi_connection(wifi: &mut WifiController) -> IoTResult<()> {
    rprintln!("WiFi Diagnostics:");
    
    // Check WiFi controller status
    rprintln!("  Controller status: {:?}", wifi.status());
    
    // Scan for networks
    let scan_results = wifi.scan().await?;
    rprintln!("  Available networks:");
    
    for result in scan_results {
        rprintln!("    SSID: {}, RSSI: {} dBm, Channel: {}", 
                  result.ssid, result.signal_strength, result.channel);
        
        if result.ssid == env!("WIFI_SSID") {
            rprintln!("    -> Target network found!");
            rprintln!("    -> Security: {:?}", result.auth_mode);
            rprintln!("    -> Channel: {}", result.channel);
        }
    }
    
    Ok(())
}
```

**Common Issues & Solutions**:

1. **Wrong Network Type**:
   ```
   Issue: Trying to connect to 5GHz network
   Solution: ESP32-C3 only supports 2.4GHz
   Check: Ensure network broadcasts on 2.4GHz
   ```

2. **Authentication Failure**:
   ```bash
   # Check credentials
   echo "WIFI_SSID: $WIFI_SSID"
   echo "WIFI_PASSWORD: (length: ${#WIFI_PASSWORD})"
   
   # Verify SSID is exact match (case-sensitive)
   # Check for hidden characters in password
   ```

3. **Network Security**:
   ```
   Supported: WPA2-Personal, WPA3-Personal
   Not supported: WPA2-Enterprise, hidden networks
   Check: Router security settings
   ```

#### WiFi Connects But No Internet
**Symptoms**: WiFi connected, IP assigned, but can't reach internet

**Network Diagnostics**:
```rust
// Add network connectivity tests
async fn diagnose_network_connectivity(stack: &embassy_net::Stack<WifiDevice>) -> IoTResult<()> {
    rprintln!("Network Diagnostics:");
    
    // Check IP configuration
    if let Some(config) = stack.config_v4() {
        rprintln!("  IP Address: {:?}", config.address);
        rprintln!("  Gateway: {:?}", config.gateway);
        rprintln!("  DNS: {:?}", config.dns_servers);
    }
    
    // Test gateway connectivity
    if let Some(gateway) = stack.config_v4().and_then(|c| c.gateway) {
        match ping_host(stack, gateway).await {
            Ok(time) => rprintln!("  Gateway ping: {} ms", time),
            Err(e) => rprintln!("  Gateway unreachable: {:?}", e),
        }
    }
    
    // Test DNS resolution
    match stack.dns_query("google.com", embassy_net::dns::DnsQueryType::A).await {
        Ok(addresses) => rprintln!("  DNS resolution: OK ({:?})", addresses),
        Err(e) => rprintln!("  DNS resolution failed: {:?}", e),
    }
    
    Ok(())
}
```

**Solutions**:
```bash
# Check router/network configuration
ping [ESP32_IP]  # Should respond if device is online
ping [GATEWAY_IP]  # Check if router is reachable

# Check DHCP settings on router
# Ensure sufficient IP addresses in DHCP pool
# Verify no MAC address filtering
```

### MQTT Connection Issues

#### Can't Connect to MQTT Broker
**Symptoms**: MQTT connection timeouts, broker unreachable

**Diagnostic Steps**:
```bash
# Test MQTT broker from development machine
mosquitto_pub -h [BROKER_IP] -p 1883 -t test -m "hello"
mosquitto_sub -h [BROKER_IP] -p 1883 -t test

# Check if broker is listening
netstat -tlnp | grep 1883
telnet [BROKER_IP] 1883

# Check firewall rules
sudo ufw status
sudo iptables -L
```

**MQTT Client Diagnostics**:
```rust
// Add MQTT connection diagnostics
async fn diagnose_mqtt_connection(stack: &embassy_net::Stack<WifiDevice>) -> IoTResult<()> {
    let broker_ip = env!("MQTT_BROKER_IP");
    let broker_port: u16 = env!("MQTT_BROKER_PORT").parse().unwrap_or(1883);
    
    rprintln!("MQTT Diagnostics:");
    rprintln!("  Broker: {}:{}", broker_ip, broker_port);
    
    // Test TCP connection to broker
    let mut socket = TcpSocket::new(stack, &mut [0; 1024], &mut [0; 1024]);
    
    let broker_addr = (broker_ip.parse::<IpAddr>()?, broker_port);
    rprintln!("  Connecting to {:?}...", broker_addr);
    
    match socket.connect(broker_addr).await {
        Ok(()) => {
            rprintln!("  TCP connection successful!");
            socket.close();
        },
        Err(e) => {
            rprintln!("  TCP connection failed: {:?}", e);
            return Err(IoTError::network(NetworkError::TCPConnectionFailed(
                error_message("MQTT broker unreachable")
            )));
        }
    }
    
    Ok(())
}
```

#### MQTT Messages Not Publishing
**Symptoms**: Connection successful but messages don't appear in subscribers

**Message Debugging**:
```rust
// Add message publishing diagnostics
async fn debug_mqtt_publish(client: &mut MqttClient, topic: &str, payload: &[u8]) -> IoTResult<()> {
    rprintln!("Publishing MQTT message:");
    rprintln!("  Topic: {}", topic);
    rprintln!("  Payload length: {} bytes", payload.len());
    rprintln!("  Payload: {:?}", core::str::from_utf8(payload).unwrap_or("<binary>"));
    
    match client.publish(topic, payload).await {
        Ok(()) => {
            rprintln!("  Publish successful!");
            Ok(())
        },
        Err(e) => {
            rprintln!("  Publish failed: {:?}", e);
            Err(e)
        }
    }
}
```

**Common Publishing Issues**:
```bash
# Check topic permissions
mosquitto_sub -h [BROKER_IP] -t "esp32/#" -v

# Verify message format
echo "Test message" | mosquitto_pub -h [BROKER_IP] -t esp32/test -l

# Check broker logs
sudo tail -f /var/log/mosquitto/mosquitto.log

# Monitor network traffic
sudo tcpdump -i any -A 'port 1883'
```

## Build and Development Issues

### Compilation Problems

#### Target Not Found
**Error**: `error: target 'riscv32imc-unknown-none-elf' not found`

**Solution**:
```bash
# Install the correct target
rustup target add riscv32imc-unknown-none-elf

# Verify installation
rustup target list --installed | grep riscv32imc

# Update rust if needed
rustup update
```

#### Dependency Conflicts
**Error**: Version conflicts in Cargo dependencies

**Diagnostic Steps**:
```bash
# Check dependency tree
cargo tree

# Look for conflicting versions
cargo tree --duplicates

# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build --release
```

**ESP32-C3 Specific Fixes**:
```toml
# Add to Cargo.toml to resolve portable-atomic conflict
[dependencies]
portable-atomic = { version = "1.5", features = ["unsafe-assume-single-core"] }

# Force specific esp-hal version
esp-hal = { version = "=1.0.0-rc.0", features = ["esp32c3", "unstable"] }
```

#### Memory Issues During Build
**Error**: Out of memory during compilation

**Solutions**:
```bash
# Reduce parallel jobs
CARGO_BUILD_JOBS=1 cargo build --release

# Increase system swap (Linux)
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Use faster linker
sudo apt install lld  # Linux
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
```

### Runtime Issues

#### Stack Overflow
**Symptoms**: Device resets immediately after start, stack overflow panic

**Diagnostic**:
```rust
// Add stack monitoring
use esp_hal::system::*;

fn monitor_stack_usage() {
    let free_stack = get_free_stack_size();
    rprintln!("Free stack: {} bytes", free_stack);
    
    if free_stack < 1024 {
        rprintln!("WARNING: Low stack space!");
    }
}
```

**Solutions**:
```toml
# Increase stack size in .cargo/config.toml
[env]
ESP_STACK_SIZE = "8192"  # Increase from default 4096

# Or in build.rs
println!("cargo:rustc-link-arg=-Wl,--defsym,_stack_size=8192");
```

#### Heap Exhaustion
**Symptoms**: Memory allocation failures, system instability

**Memory Monitoring**:
```rust
use esp_alloc::*;

fn check_memory_usage() {
    let heap_info = heap_caps_get_info(MALLOC_CAP_8BIT);
    
    rprintln!("Heap status:");
    rprintln!("  Total: {} bytes", heap_info.total_free_bytes + heap_info.total_allocated_bytes);
    rprintln!("  Free: {} bytes", heap_info.total_free_bytes);
    rprintln!("  Largest block: {} bytes", heap_info.largest_free_block);
    rprintln!("  Min ever free: {} bytes", heap_info.minimum_free_bytes);
    
    if heap_info.total_free_bytes < 5000 {
        rprintln!("WARNING: Low memory!");
    }
}
```

**Memory Optimization**:
```rust
// Use bounded collections
use heapless::{String, Vec};

// Stack-allocated strings
let mut buffer: String<64> = String::new();

// Stack-allocated vectors
let mut data: Vec<u8, 32> = Vec::new();

// Static allocation for large buffers
use static_cell::StaticCell;
static BUFFER: StaticCell<[u8; 1024]> = StaticCell::new();

let buffer = BUFFER.init([0u8; 1024]);
```

## System Integration Issues

### Task Scheduling Problems

#### Embassy Task Hangs
**Symptoms**: One or more tasks stop responding

**Task Monitoring**:
```rust
// Add task watchdog
use embassy_time::{Timer, Duration, Instant};

#[embassy_executor::task]
async fn watchdog_task() {
    let mut last_sensor_update = Instant::now();
    let mut last_mqtt_update = Instant::now();
    
    loop {
        Timer::after(Duration::from_secs(30)).await;
        
        let now = Instant::now();
        
        if now.duration_since(last_sensor_update).as_secs() > 300 {
            rprintln!("WARNING: Sensor task hasn't updated in 5 minutes");
        }
        
        if now.duration_since(last_mqtt_update).as_secs() > 600 {
            rprintln!("WARNING: MQTT task hasn't updated in 10 minutes");
        }
    }
}
```

**Task Debugging**:
```rust
// Add task heartbeats
use embassy_sync::channel::Channel;

static TASK_STATUS: Channel<ThreadModeRawMutex, TaskStatus, 10> = Channel::new();

#[derive(Clone, Copy)]
enum TaskStatus {
    SensorReading,
    MqttPublishing,
    NetworkConnecting,
}

#[embassy_executor::task]
async fn sensor_task() {
    loop {
        TASK_STATUS.send(TaskStatus::SensorReading).await;
        
        // Sensor reading logic
        let measurements = read_sensor().await?;
        
        Timer::after(Duration::from_secs(30)).await;
    }
}
```

### Error Handling Issues

#### Error Information Lost
**Symptoms**: Generic error messages without context

**Enhanced Error Context**:
```rust
use iot_common::{IoTResult, result::IoTResultExt};

// Add context throughout call chain
async fn system_operation() -> IoTResult<()> {
    init_hardware()
        .with_context("Hardware initialization phase")?;
    
    connect_wifi()
        .with_context("Network connectivity phase")?;
    
    setup_mqtt()
        .with_context("MQTT broker setup phase")?;
    
    start_data_collection()
        .with_context("Data collection startup")?;
    
    Ok(())
}

// Result includes full context chain:
// "I2C timeout [Context: Data collection startup <- MQTT broker setup <- Network connectivity phase <- Hardware initialization phase]"
```

#### Silent Failures
**Symptoms**: Operations fail but no error indication

**Comprehensive Error Logging**:
```rust
// Add error logging throughout system
use rtt_target::rprintln;

async fn robust_operation() -> IoTResult<T> {
    match risky_operation().await {
        Ok(result) => {
            rprintln!("Operation successful");
            Ok(result)
        },
        Err(e) => {
            rprintln!("Operation failed: {}", e);
            rprintln!("Error code: {}", e.error_code());
            rprintln!("Error category: {}", e.category());
            
            // Attempt recovery if possible
            if let Some(recovery_result) = attempt_recovery(&e).await {
                rprintln!("Recovery successful");
                Ok(recovery_result)
            } else {
                rprintln!("Recovery failed, propagating error");
                Err(e)
            }
        }
    }
}
```

## Performance Issues

### Slow Response Times

#### Sensor Reading Delays
**Symptoms**: Sensor readings take too long

**Performance Measurement**:
```rust
use embassy_time::Instant;

async fn measure_sensor_performance<I2C>(sensor: &mut BME280<I2C>) -> IoTResult<Measurements>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let start = Instant::now();
    
    let measurements = sensor.read_measurements().await?;
    
    let duration = start.elapsed();
    rprintln!("Sensor reading took {} ms", duration.as_millis());
    
    if duration.as_millis() > 500 {
        rprintln!("WARNING: Slow sensor reading!");
    }
    
    Ok(measurements)
}
```

**Optimization Strategies**:
```rust
// Use faster I2C speed
let i2c = I2c::new(peripherals.I2C0, Config::new().frequency(400_000.Hz()));

// Cache calibration data
struct CachedBME280<I2C> {
    i2c: I2C,
    calibration: CalibrationData,
}

// Read multiple measurements in batch
async fn read_batch_measurements(&mut self, count: usize) -> IoTResult<Vec<Measurements, 10>> {
    let mut results = Vec::new();
    
    for _ in 0..count {
        let measurement = self.read_raw_measurements().await?;
        results.push(self.compensate_measurements(measurement)).ok();
    }
    
    Ok(results)
}
```

#### Network Latency Issues
**Symptoms**: High network operation times

**Network Performance Testing**:
```rust
async fn measure_network_performance(stack: &embassy_net::Stack<WifiDevice>) -> IoTResult<()> {
    let start = Instant::now();
    
    // Test DNS resolution
    let dns_start = Instant::now();
    let _addr = stack.dns_query("google.com", DnsQueryType::A).await?;
    let dns_time = dns_start.elapsed();
    
    // Test TCP connection
    let tcp_start = Instant::now();
    let mut socket = TcpSocket::new(stack, &mut [0; 1024], &mut [0; 1024]);
    socket.connect(("8.8.8.8".parse().unwrap(), 53)).await?;
    let tcp_time = tcp_start.elapsed();
    socket.close();
    
    rprintln!("Network Performance:");
    rprintln!("  DNS resolution: {} ms", dns_time.as_millis());
    rprintln!("  TCP connection: {} ms", tcp_time.as_millis());
    
    if dns_time.as_millis() > 5000 {
        rprintln!("WARNING: Slow DNS resolution!");
    }
    
    if tcp_time.as_millis() > 10000 {
        rprintln!("WARNING: Slow TCP connection!");
    }
    
    Ok(())
}
```

### High Memory Usage

#### Memory Leak Detection
**Symptoms**: Memory usage increases over time

**Memory Tracking**:
```rust
use embassy_time::{Timer, Duration};

#[embassy_executor::task]
async fn memory_monitor_task() {
    let mut min_free = u32::MAX;
    
    loop {
        let current_free = heap_caps_get_free_size(MALLOC_CAP_8BIT);
        
        if current_free < min_free {
            min_free = current_free;
            rprintln!("New minimum free memory: {} bytes", min_free);
        }
        
        if current_free < 10000 {
            rprintln!("CRITICAL: Low memory warning - {} bytes free", current_free);
            // Trigger garbage collection or emergency cleanup
        }
        
        Timer::after(Duration::from_secs(60)).await;
    }
}
```

## Recovery Procedures

### Device Recovery

#### Soft Reset
```rust
// Graceful system restart
pub fn graceful_restart() -> ! {
    rprintln!("Initiating graceful restart...");
    
    // Close network connections
    // Save critical state
    // Clean up resources
    
    Timer::after(Duration::from_secs(1)).await;
    esp_restart();
}
```

#### Hard Reset
```bash
# Physical reset via probe-rs
probe-rs reset --chip esp32c3

# Erase flash and reprogram
probe-rs erase --chip esp32c3
cargo run --release
```

### Configuration Recovery
```rust
// Reset to default configuration
pub fn reset_configuration() -> IoTResult<()> {
    let default_config = SystemConfiguration::default();
    default_config.save_to_flash()?;
    
    rprintln!("Configuration reset to defaults");
    graceful_restart();
}
```

### Emergency Procedures
```bash
#!/bin/bash
# emergency_recovery.sh

echo "=== Emergency Recovery Procedure ==="

# 1. Stop any running instances
pkill -f cargo
pkill -f probe-rs

# 2. Reset hardware
echo "Resetting ESP32-C3..."
probe-rs reset --chip esp32c3

# 3. Erase flash
echo "Erasing flash..."
probe-rs erase --chip esp32c3

# 4. Deploy known good firmware
echo "Deploying backup firmware..."
cd workspace/main-app/
cargo build --release
cargo run --release

echo "Emergency recovery completed"
```

This troubleshooting guide provides systematic approaches to identify, diagnose, and resolve common issues in the ESP32-C3 IoT system. Use it as a reference during development and deployment to quickly resolve problems and maintain system reliability.