# BME280 Algorithms Test Suite

> **Suggested Module Name**: `bme280-algorithms-test` (more descriptive than `bme280-tests`)

## Overview

This module provides comprehensive host-based testing for BME280 environmental sensor algorithms. It validates the mathematical calculations and calibration algorithms without requiring actual ESP32-C3 hardware or embedded dependencies.

## Purpose

- **Algorithm Validation**: Tests BME280 temperature, humidity, and pressure calculation algorithms
- **Host Testing**: Runs on development machine (x86_64) for fast iteration
- **Mathematical Verification**: Validates sensor calibration and compensation formulas
- **Regression Testing**: Ensures algorithm correctness across code changes

## Module Contents

### Core Components
- `src/lib.rs` - Algorithm implementations and data structures
- `tests/algorithm_tests.rs` - Comprehensive test suite for all BME280 algorithms

### Test Coverage
- BME280 sensor constants validation
- Temperature compensation algorithms
- Pressure compensation algorithms  
- Humidity compensation algorithms
- Calibration data structure verification
- Edge case and boundary testing

## Key Features

- ✅ **Zero Dependencies** - Pure Rust algorithm testing
- ✅ **Fast Execution** - Host-based tests run in milliseconds
- ✅ **Comprehensive Coverage** - Tests all BME280 calculation paths
- ✅ **Hardware Independent** - No embedded dependencies required
- ✅ **Mathematical Precision** - Validates sensor compensation formulas

## Usage

### Running Tests
```bash
# Run all algorithm tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_bme280_constants
```

### Building
```bash
# Check compilation
cargo check

# Build for testing
cargo build --tests
```

## Test Structure

The test suite validates:

1. **Constants**: BME280 I2C addresses, chip ID, register addresses
2. **Calibration**: Calibration data structure and parsing
3. **Temperature**: Temperature compensation algorithm accuracy
4. **Pressure**: Pressure compensation algorithm accuracy  
5. **Humidity**: Humidity compensation algorithm accuracy
6. **Integration**: End-to-end calculation chains

## Dependencies

This module intentionally has **zero dependencies** to:
- Enable fast compilation and testing
- Avoid version conflicts with embedded dependencies
- Focus purely on algorithm validation
- Support continuous integration environments

## Architecture

```
bme280-algorithms-test/
├── src/
│   └── lib.rs              # Algorithm implementations
├── tests/
│   └── algorithm_tests.rs  # Comprehensive test suite
├── Cargo.toml              # Zero-dependency configuration
└── README.md               # This documentation
```

## Integration with Main Project

This test module validates algorithms used in:
- `drivers/bme280-embassy` - Production BME280 driver
- `apps/main-app` - Complete IoT application
- `supporting/modules/simple-iot` - Standalone IoT implementation

## Development Workflow

1. Modify BME280 algorithms in production code
2. Run this test suite to validate mathematical correctness
3. Run embedded tests on actual hardware
4. Deploy to production with confidence

## Mathematical Background

The BME280 sensor requires complex calibration and compensation algorithms:
- **Temperature**: 2-stage compensation with trimming parameters
- **Pressure**: Temperature-dependent compensation
- **Humidity**: Non-linear compensation with multiple calibration factors

This test suite ensures these algorithms maintain mathematical precision across all sensor reading ranges.

---

**Target Hardware**: Host development machine (x86_64)  
**Dependencies**: None (pure algorithm testing)  
**Test Framework**: Built-in Rust test framework  
**Execution Time**: < 100ms for complete test suite