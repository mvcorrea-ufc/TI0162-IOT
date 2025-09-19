# Contributing Guide

> Guidelines for contributing to the ESP32-C3 IoT Environmental Monitoring System

## Welcome Contributors!

We welcome contributions to this production-ready IoT system! This guide will help you understand our development process, coding standards, and how to submit high-quality contributions.

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for all contributors. Please be respectful in all interactions and follow professional software development practices.

## Getting Started

### 1. Development Setup
Before contributing, ensure you have the complete development environment set up:

```bash
# Clone the repository
git clone <repository-url>
cd workspace/

# Set up Rust environment
rustup target add riscv32imc-unknown-none-elf
cargo install probe-rs --features cli

# Verify setup
cargo check --workspace
probe-rs list  # Should detect ESP32-C3 if connected
```

### 2. Understanding the Architecture
Review the project documentation:
- Read [`README.md`](../README.md) for system overview
- Study [`ARCHITECTURE_ANALYSIS.md`](../ARCHITECTURE_ANALYSIS.md) for design decisions
- Review module-specific READMEs for implementation details
- Check [`docs/DEVELOPMENT.md`](DEVELOPMENT.md) for development workflow

### 3. Types of Contributions

We welcome several types of contributions:

#### Code Contributions
- **Bug fixes**: Address issues in existing functionality
- **Feature enhancements**: Improve existing modules
- **New modules**: Add new sensor or connectivity modules
- **Performance optimizations**: Improve efficiency or memory usage
- **Error handling improvements**: Enhance the unified error system

#### Documentation Contributions
- **API documentation**: Improve rustdoc comments
- **Usage examples**: Add practical implementation examples
- **Hardware guides**: Document new sensor integrations
- **Troubleshooting guides**: Help others resolve common issues

#### Testing Contributions
- **Unit tests**: Add test coverage for business logic
- **Integration tests**: Test module interactions
- **Hardware testing**: Validate on different hardware configurations
- **Mock implementations**: Create test doubles for hardware interfaces

## Development Workflow

### 1. Issue-Based Development
All contributions should address a specific issue:

```bash
# Check existing issues first
# Create new issue if none exists
# Reference issue in your branch name
git checkout -b fix/issue-123-wifi-reconnection
```

### 2. Branch Naming Convention
Use descriptive branch names that indicate the type and scope of changes:

- `feat/module-new-feature` - New feature implementation
- `fix/issue-###-description` - Bug fix referencing issue number
- `docs/api-documentation` - Documentation improvements
- `refactor/error-handling` - Code refactoring
- `test/sensor-mock-implementation` - Testing additions

### 3. Development Standards

#### Code Quality Requirements
All code must meet these standards before submission:

```bash
# Code formatting
cargo fmt --check

# Linting without warnings
cargo clippy -- -D warnings

# All tests passing
cargo test --workspace

# Documentation building without warnings
cargo doc --no-deps --document-private-items
```

#### Embedded Rust Best Practices
- **Use `#![no_std]`** for all embedded modules
- **Prefer stack allocation** over heap allocation
- **Use bounded collections** from `heapless` crate
- **Implement proper error handling** using `iot-common` types
- **Follow async/await patterns** with Embassy framework
- **Document hardware requirements** clearly

### 4. Error Handling Standards
All modules must use the unified error handling system:

```rust
use iot_common::{IoTResult, IoTError, SensorError, error::utils};

// Good: Using unified error types
pub async fn read_sensor(&mut self) -> IoTResult<f32> {
    match self.hardware_read().await {
        Ok(value) => Ok(value),
        Err(_) => {
            let error = SensorError::I2CError(
                utils::error_message("Hardware communication failed")
            );
            Err(IoTError::sensor(error)
                .with_context("BME280 temperature reading"))
        }
    }
}

// Bad: Module-specific error types
pub async fn read_sensor(&mut self) -> Result<f32, CustomSensorError> {
    // This doesn't integrate with the unified error system
}
```

### 5. Documentation Standards

#### Rustdoc Requirements
All public APIs must have comprehensive documentation:

```rust
/// Initializes the BME280 sensor with specified I2C address
/// 
/// This function performs a complete sensor initialization including:
/// - Chip ID verification
/// - Calibration coefficient reading
/// - Sensor configuration for continuous measurement
/// 
/// # Arguments
/// * `i2c` - Async I2C interface for sensor communication
/// * `address` - I2C address (0x76 or 0x77)
/// 
/// # Returns
/// * `Ok(BME280)` - Successfully initialized sensor instance
/// * `Err(IoTError)` - Initialization failure with error context
/// 
/// # Hardware Requirements
/// - BME280 sensor connected via I2C
/// - Pull-up resistors on SDA/SCL lines (typically 4.7kΩ)
/// - Stable 3.3V power supply
/// - GPIO pins properly configured for I2C operation
/// 
/// # Examples
/// ```no_run
/// use bme280_embassy::BME280;
/// use esp_hal::i2c::I2c;
/// 
/// let mut sensor = BME280::new(i2c, 0x76).await?;
/// let measurements = sensor.read_measurements().await?;
/// println!("Temperature: {:.2}°C", measurements.temperature);
/// ```
/// 
/// # Error Conditions
/// - `SensorError::InitializationFailed` - Chip ID verification failed
/// - `SensorError::I2CError` - Communication timeout or failure
/// - `HardwareError::GPIOError` - I2C pin configuration invalid
pub async fn new<I2C>(i2c: I2C, address: u8) -> IoTResult<Self>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    // Implementation
}
```

#### README Requirements
Each new module must include:
- **Purpose and features** clearly described
- **Hardware requirements** and wiring diagrams
- **Installation and usage** instructions
- **Integration examples** with other modules
- **Troubleshooting section** for common issues

## Testing Requirements

### 1. Unit Testing
All business logic must have unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iot_common::testing::MockI2c;
    
    #[tokio::test]
    async fn test_sensor_initialization() {
        let mut mock_i2c = MockI2c::new();
        mock_i2c.expect_read(0x76, 0xD0, &[0x60]); // BME280 chip ID
        
        let sensor = BME280::new(mock_i2c, 0x76).await;
        assert!(sensor.is_ok());
    }
    
    #[tokio::test]
    async fn test_invalid_chip_id() {
        let mut mock_i2c = MockI2c::new();
        mock_i2c.expect_read(0x76, 0xD0, &[0x00]); // Invalid chip ID
        
        let result = BME280::new(mock_i2c, 0x76).await;
        assert!(matches!(result, Err(IoTError::Sensor(_))));
    }
}
```

### 2. Integration Testing
Test module interactions and hardware integration:

```rust
// Integration test example
#[tokio::test]
async fn test_wifi_mqtt_integration() {
    let wifi_manager = setup_wifi_test().await;
    let mqtt_client = MqttClient::new(wifi_manager.get_stack()).await?;
    
    // Test complete data flow
    let test_data = create_test_sensor_data();
    let result = mqtt_client.publish("test/topic", &test_data).await;
    assert!(result.is_ok());
}
```

### 3. Hardware Testing
Include instructions for hardware validation:

```markdown
## Hardware Testing Checklist

Before submitting hardware-related changes:

- [ ] Test on actual ESP32-C3 hardware
- [ ] Verify with both BME280 I2C addresses (0x76 and 0x77)
- [ ] Test WiFi connectivity on 2.4GHz network
- [ ] Validate MQTT publishing to external broker
- [ ] Check RTT debug output for error messages
- [ ] Measure memory usage and ensure it's within limits
```

## Submission Process

### 1. Pull Request Preparation

Before creating a pull request:

```bash
# Ensure your branch is up-to-date
git checkout master
git pull origin master
git checkout your-feature-branch
git rebase master

# Run full test suite
cargo test --workspace
cargo clippy -- -D warnings
cargo fmt --check

# Build documentation
cargo doc --no-deps
```

### 2. Pull Request Template

Use this template for all pull requests:

```markdown
## Description
Brief summary of the changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing Checklist
- [ ] Unit tests pass
- [ ] Integration tests pass  
- [ ] Hardware validation completed (if applicable)
- [ ] Documentation updated
- [ ] Error handling tested

## Hardware Requirements
List any specific hardware requirements or setup needed to test this change.

## Related Issues
Fixes #(issue number)

## Screenshots/Logs
Include RTT output, MQTT message examples, or other relevant logs.
```

### 3. Code Review Process

All contributions go through code review:

1. **Automated Checks**: CI runs tests and linting
2. **Technical Review**: Maintainer reviews code quality and design
3. **Hardware Validation**: Testing on actual hardware if needed
4. **Documentation Review**: Ensure documentation is complete and accurate

### 4. Review Criteria

Code reviews evaluate:

#### Technical Aspects
- **Correctness**: Does the code work as intended?
- **Safety**: Are there potential memory safety issues?
- **Performance**: Is the implementation efficient for embedded systems?
- **Integration**: Does it work well with existing modules?

#### Quality Aspects
- **Readability**: Is the code easy to understand and maintain?
- **Documentation**: Are APIs properly documented?
- **Error Handling**: Are errors handled consistently?
- **Testing**: Is there adequate test coverage?

#### Embedded-Specific Aspects
- **Memory Usage**: Does it fit within ESP32-C3 constraints?
- **Real-Time Behavior**: Are timing requirements met?
- **Power Consumption**: Does it impact battery life considerations?
- **Hardware Integration**: Is hardware abstraction appropriate?

## Special Considerations

### 1. Hardware Dependencies
When contributing hardware-related code:
- **Abstract hardware interfaces** using traits where possible
- **Provide mock implementations** for testing
- **Document hardware requirements** clearly
- **Include wiring diagrams** for new sensors
- **Test on actual hardware** before submission

### 2. Memory Constraints
ESP32-C3 has limited memory, so:
- **Use stack allocation** where possible
- **Prefer `heapless` collections** over standard library
- **Avoid unnecessary cloning** of large data structures
- **Monitor binary size** and memory usage

### 3. Real-Time Requirements
For time-critical code:
- **Use appropriate async patterns** with Embassy
- **Avoid blocking operations** in async contexts
- **Document timing constraints** clearly
- **Test latency and jitter** characteristics

### 4. Security Considerations
For network and security-related contributions:
- **Validate all inputs** from network sources
- **Use secure communication** where possible
- **Handle credentials securely** (no hardcoded secrets)
- **Follow embedded security best practices**

## Getting Help

### Resources
- **Documentation**: Check existing docs before asking questions
- **Issues**: Search existing issues for similar problems
- **Examples**: Look at existing modules for implementation patterns
- **Community**: Engage respectfully with other contributors

### Contact
- **Technical Questions**: Create an issue with detailed description
- **Design Discussions**: Use issue discussions for architectural questions
- **Bug Reports**: Include RTT logs, hardware setup, and reproduction steps

## Recognition

Contributors are recognized in several ways:
- **Git commit history** preserves authorship
- **Pull request acknowledgment** in release notes
- **Documentation contributions** credited in relevant sections
- **Issue resolution** tracked and appreciated

## Conclusion

Thank you for contributing to this ESP32-C3 IoT project! Your contributions help create a robust, well-documented, and production-ready embedded system. Follow these guidelines to ensure your contributions can be integrated smoothly and benefit the entire community.

By contributing, you're helping build a high-quality IoT system that serves as both a practical implementation and an educational resource for embedded Rust development.