# IoT Performance Monitoring System

A platform-agnostic performance monitoring and profiling system for embedded IoT applications, providing comprehensive performance analysis, memory tracking, timing analysis, and system optimization insights.

## Overview

The `iot-performance` module provides a unified performance monitoring framework that can be used across different embedded platforms. It offers real-time performance metrics, memory usage analysis, timing profiling, and optimization recommendations for IoT systems.

## Key Features

- **Platform Agnostic**: Works across different embedded architectures and platforms
- **Real-time Monitoring**: Live performance metrics during system operation
- **Memory Analysis**: Heap, stack, and static memory usage tracking
- **Timing Profiling**: Function execution time and critical path analysis
- **Performance Baselines**: Establish and compare against performance benchmarks
- **Regression Detection**: Automatic detection of performance degradation
- **Optimization Insights**: Actionable recommendations for performance improvements
- **No-std Compatible**: Designed for resource-constrained embedded systems

## Architecture

### Performance Monitoring Components

```rust
use iot_performance::{
    PerformanceMonitor, MemoryAnalyzer, TimingProfiler,
    MetricsCollector, BaselineManager
};
```

### Platform Abstraction

The module provides platform-specific implementations through a trait-based system:

```rust
// Platform-agnostic traits
use iot_performance::{
    PlatformMetrics, MemoryProvider, TimingProvider,
    SystemInfoProvider
};

// Platform-specific implementations
use iot_performance::{
    esp32c3::Esp32C3Metrics,    // ESP32-C3 implementation
    cortex_m::CortexMMetrics,   // ARM Cortex-M implementation
    riscv::RiscVMetrics,        // RISC-V implementation
    mock::MockMetrics,          // Testing implementation
};
```

## Module Structure

```
iot-performance/
├── src/
│   ├── lib.rs              # Main performance monitoring API
│   ├── monitor.rs          # Core performance monitor
│   ├── memory.rs           # Memory usage analysis
│   ├── timing.rs           # Timing and profiling
│   ├── analysis.rs         # Performance analysis algorithms
│   ├── baseline.rs         # Performance baseline management
│   ├── regression.rs       # Performance regression detection
│   └── esp32c3.rs          # ESP32-C3 specific implementation
├── Cargo.toml              # Module dependencies and features
└── README.md               # This documentation
```

## Core Performance Metrics

### Memory Metrics

```rust
use iot_performance::{MemoryMetrics, MemoryAnalyzer};

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub heap_used: usize,
    pub heap_free: usize,
    pub heap_total: usize,
    pub stack_used: usize,
    pub stack_free: usize,
    pub static_memory: usize,
    pub fragmentation_ratio: f32,
}

// Usage
let memory_analyzer = MemoryAnalyzer::new();
let metrics = memory_analyzer.current_metrics().await?;
```

### Timing Metrics

```rust
use iot_performance::{TimingMetrics, TimingProfiler};

#[derive(Debug, Clone)]
pub struct TimingMetrics {
    pub function_name: &'static str,
    pub execution_time_us: u64,
    pub call_count: u32,
    pub average_time_us: u64,
    pub max_time_us: u64,
    pub min_time_us: u64,
}

// Usage with timing macros
use iot_performance::profile_function;

#[profile_function]
async fn critical_function() {
    // Function implementation
}
```

### System Metrics

```rust
use iot_performance::{SystemMetrics, SystemMonitor};

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_utilization: f32,
    pub uptime_ms: u64,
    pub task_count: usize,
    pub interrupt_count: u64,
    pub context_switches: u64,
    pub temperature: Option<f32>,
}
```

## Usage Examples

### Basic Performance Monitoring

```rust
use iot_performance::{PerformanceMonitor, MonitorConfig};

// Initialize performance monitor
let config = MonitorConfig {
    sampling_interval_ms: 1000,
    memory_tracking: true,
    timing_profiling: true,
    baseline_comparison: true,
};

let mut monitor = PerformanceMonitor::new(config).await?;

// Start monitoring
monitor.start_monitoring().await?;

// Get current metrics
let metrics = monitor.current_metrics().await?;
println!("Memory usage: {}KB", metrics.memory.heap_used / 1024);
println!("CPU utilization: {:.1}%", metrics.system.cpu_utilization);
```

### Function Timing Profiling

```rust
use iot_performance::{TimingProfiler, profile_block};

// Profile a specific code block
let profiler = TimingProfiler::new();

{
    let _timer = profile_block!("sensor_reading");
    // Critical sensor reading code
    let sensor_data = read_bme280_sensor().await?;
}

// Profile an entire function
#[profile_function]
async fn process_mqtt_message(message: &[u8]) -> Result<(), Error> {
    // Message processing logic
}

// Get profiling results
let timing_report = profiler.generate_report().await?;
```

### Memory Analysis

```rust
use iot_performance::{MemoryAnalyzer, MemoryAnalysisConfig};

let config = MemoryAnalysisConfig {
    track_allocations: true,
    detect_leaks: true,
    fragmentation_analysis: true,
};

let mut analyzer = MemoryAnalyzer::new(config);

// Monitor memory over time
analyzer.start_continuous_monitoring().await?;

// Analyze memory patterns
let analysis = analyzer.analyze_patterns().await?;
if analysis.potential_leak_detected {
    println!("Warning: Potential memory leak detected");
}
```

### Performance Baseline Management

```rust
use iot_performance::{BaselineManager, PerformanceBaseline};

// Establish performance baseline
let mut baseline_manager = BaselineManager::new();

// Record baseline during optimal conditions
baseline_manager.record_baseline("startup").await?;
baseline_manager.record_baseline("steady_state").await?;

// Compare current performance against baseline
let comparison = baseline_manager
    .compare_against_baseline("steady_state")
    .await?;

if comparison.is_regression_detected() {
    println!("Performance regression detected: {}", comparison.summary());
}
```

### Regression Detection

```rust
use iot_performance::{RegressionDetector, RegressionConfig};

let config = RegressionConfig {
    memory_threshold_percent: 10.0,  // Alert if memory usage increases by 10%
    timing_threshold_percent: 20.0,  // Alert if timing increases by 20%
    cpu_threshold_percent: 15.0,     // Alert if CPU usage increases by 15%
};

let mut detector = RegressionDetector::new(config);

// Continuously monitor for regressions
detector.start_monitoring().await?;

// Check for regressions
if let Some(regression) = detector.check_for_regression().await? {
    println!("Regression detected: {}", regression.description());
    
    // Take corrective action
    handle_performance_regression(regression).await?;
}
```

## Platform-Specific Implementations

### ESP32-C3 Implementation

```rust
use iot_performance::esp32c3::{Esp32C3Metrics, Esp32C3Config};

let config = Esp32C3Config {
    enable_hardware_counters: true,
    temperature_monitoring: true,
    wifi_performance_tracking: true,
};

let metrics_provider = Esp32C3Metrics::new(config).await?;
let monitor = PerformanceMonitor::with_provider(metrics_provider).await?;
```

### Generic ARM Cortex-M Implementation

```rust
use iot_performance::cortex_m::{CortexMMetrics, CortexMConfig};

let config = CortexMConfig {
    systick_frequency: 1000,
    dwt_cycle_counter: true,
    mpu_monitoring: false,
};

let metrics_provider = CortexMMetrics::new(config).await?;
```

### Mock Implementation for Testing

```rust
use iot_performance::mock::{MockMetrics, MockConfig};

let config = MockConfig::with_deterministic_values();
let metrics_provider = MockMetrics::new(config);

// Configure mock responses for testing
metrics_provider.set_memory_usage(1024, 512);
metrics_provider.set_cpu_utilization(45.0);
```

## Configuration Features

### Feature Flags

```toml
# Basic performance monitoring
iot-performance = { path = "../core/iot-performance" }

# With ESP32-C3 specific features
iot-performance = { path = "../core/iot-performance", features = ["esp32c3"] }

# With detailed profiling capabilities
iot-performance = { path = "../core/iot-performance", features = ["detailed-profiling"] }

# With memory allocation tracking
iot-performance = { path = "../core/iot-performance", features = ["alloc"] }

# All features enabled
iot-performance = { path = "../core/iot-performance", features = ["esp32c3", "detailed-profiling", "alloc"] }
```

Available features:
- `esp32c3`: ESP32-C3 specific performance counters and metrics
- `detailed-profiling`: Advanced profiling capabilities with higher overhead
- `alloc`: Memory allocation tracking and leak detection
- `flash-analysis`: Performance analysis stored in flash memory

## Performance Analysis Types

### Real-time Analysis

```rust
// Continuous monitoring during operation
let monitor = PerformanceMonitor::new(config).await?;
monitor.start_realtime_analysis().await?;

// Get instant metrics
let instant_metrics = monitor.snapshot().await?;
```

### Historical Analysis

```rust
// Store performance data over time
let analyzer = PerformanceAnalyzer::new();
analyzer.enable_historical_tracking().await?;

// Analyze trends over time
let trend_analysis = analyzer.analyze_trends().await?;
println!("Memory usage trend: {}", trend_analysis.memory_trend);
```

### Comparative Analysis

```rust
// Compare different system configurations
let comparator = PerformanceComparator::new();

let config_a_metrics = monitor.collect_metrics_for_duration(60).await?;
let config_b_metrics = monitor.collect_metrics_for_duration(60).await?;

let comparison = comparator.compare(config_a_metrics, config_b_metrics).await?;
```

## Integration with IoT Systems

### Main Application Integration

```rust
use iot_performance::{PerformanceMonitor, MonitorConfig};

#[esp_hal::main]
async fn main() -> ! {
    // Initialize performance monitoring early
    let perf_config = MonitorConfig::for_production();
    let mut perf_monitor = PerformanceMonitor::new(perf_config)
        .await
        .expect("Failed to initialize performance monitoring");
    
    perf_monitor.start_monitoring().await.expect("Failed to start monitoring");
    
    // Regular application initialization
    let wifi_manager = WiFiManager::new().await?;
    let mqtt_client = MqttClient::new().await?;
    
    // Main application loop with performance monitoring
    loop {
        // Application logic...
        
        // Periodic performance checks
        if let Some(regression) = perf_monitor.check_regression().await? {
            handle_performance_issue(regression).await?;
        }
        
        embassy_time::Timer::after(Duration::from_secs(1)).await;
    }
}
```

### Task-based Monitoring

```rust
// Monitor specific Embassy tasks
#[embassy_executor::task]
async fn sensor_task(perf_monitor: PerformanceMonitor) {
    loop {
        let _timer = perf_monitor.profile_block("sensor_reading");
        
        // Sensor reading logic
        let data = read_sensors().await;
        
        // Check if this task is performing within expectations
        perf_monitor.validate_task_performance("sensor_task").await;
        
        embassy_time::Timer::after(Duration::from_secs(30)).await;
    }
}
```

## Testing and Development

### Unit Testing with Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use iot_performance::mock::MockMetrics;
    
    #[tokio::test]
    async fn test_performance_monitoring() {
        let mock_metrics = MockMetrics::new_with_defaults();
        let monitor = PerformanceMonitor::with_provider(mock_metrics).await.unwrap();
        
        // Test monitoring functionality
        let metrics = monitor.current_metrics().await.unwrap();
        assert!(metrics.memory.heap_used > 0);
    }
}
```

### Performance Regression Testing

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_no_performance_regression() {
        let monitor = PerformanceMonitor::new(MonitorConfig::for_testing()).await.unwrap();
        
        // Load baseline performance
        let baseline = load_performance_baseline().await.unwrap();
        
        // Run performance-critical code
        let metrics = measure_critical_path().await.unwrap();
        
        // Verify no significant regression
        assert!(metrics.execution_time_us <= baseline.execution_time_us * 1.1);
        assert!(metrics.memory_usage <= baseline.memory_usage * 1.05);
    }
}
```

## Best Practices

### Production Monitoring

- **Low Overhead**: Configure monitoring with minimal performance impact
- **Critical Metrics Only**: Focus on metrics that impact user experience
- **Adaptive Sampling**: Reduce sampling frequency during normal operation
- **Threshold Alerts**: Set up automatic alerts for performance degradation

### Development Profiling

- **Detailed Analysis**: Enable detailed profiling during development
- **Baseline Establishment**: Create performance baselines for different scenarios
- **Continuous Integration**: Include performance testing in CI/CD pipeline
- **Optimization Guidance**: Use analysis results to guide optimization efforts

---

**Module Type**: Core Infrastructure  
**Platform Support**: Cross-platform with specific implementations  
**Overhead**: Configurable from minimal (production) to detailed (development)  
**Integration**: Seamless integration with Embassy async runtime