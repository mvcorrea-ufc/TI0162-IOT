# TI0162 - Internet of Things - Execution Plan

## Project Status: 🚀 In Development

## Completed Tasks ✅

1. **Initial Project Setup**
   - Clone of base repository rust-esp32-tmpl
   - Definition of `blinky` project as implementation base
   - Creation of documentation structure (CLAUDE.md, PLAN.md)
   - Definition of modular architecture based on blinky

## Tasks in Progress 🔄

*No tasks in progress at the moment*

## Pending Tasks 📋

2. **BME280 Module - Implementation**
   - Create `bme280-module/` directory based on `blinky` structure
   - Copy base configuration (Cargo.toml, build.rs) from blinky project
   - Implement BME280 driver using Embassy over esp-hal
   - Configure I2C for sensor communication
   - Implement temperature, humidity and pressure readings
   - Integrate with RTT system for debugging
   - Create data structures for sensor values

3. **BME280 Module - Validation**
   - Create test application for BME280
   - Implement value output to console
   - Verify reading accuracy
   - Document module interface

4. **WiFi Module - Implementation**
   - Create `wifi-module/` directory
   - Implement WiFi connection using Embassy
   - Configure connection to local access point
   - Implement automatic reconnection management

5. **WiFi Module - Validation**
   - Verify IP address acquisition via DHCP
   - Implement ping test to validate connectivity
   - Create connection status logs
   - Document network configuration

6. **Web Server - Implementation**
   - Create `web-server/` directory
   - Implement basic HTTP server
   - Create HTML page to display BME280 data
   - Integrate sensor data with web interface

7. **MQTT Module - Implementation**
   - Create `mqtt-module/` directory
   - Implement MQTT client using Embassy
   - Configure connection to Mosquitto broker
   - Implement JSON serialization of data

8. **MQTT Module - Validation**
   - Configure automatic sending every 10 seconds
   - Test connectivity with broker
   - Validate format of sent data
   - Implement connection error handling

## Future Expansions 🔮

- Implementation of new sensors
- Advanced web dashboard
- Local data storage
- OTA updates
- Low power modes

## Development Notes 📝

**Implementation Base - `blinky` Project:**
- Use esp-hal v0.23.1 as base HAL
- Maintain RTT structure for debugging (rprintln!)
- Copy base configurations (Cargo.toml, build.rs)
- Preserve esp-hal peripheral initialization

**Modular Development:**
- Each module should be independent and reusable
- Use async/await extensively with Embassy over esp-hal
- Implement robust error handling
- Maintain structured logging via RTT for debugging
- Follow Rust code conventions

**Inherited Standard Configuration:**
```toml
[dependencies]
esp-hal = { version = "0.23.1", features = ["esp32c3"] }
esp-rom-sys = { version = "0.1", features = ["esp32c3"] }
defmt = "0.3"
rtt-target = "0.5"  
panic-rtt-target = "0.1"
```

---

**Last update**: 2025-09-12  
**Next review**: After BME280 module completion