//! # IoT Performance Monitoring and Optimization
//!
//! A comprehensive performance monitoring system designed specifically for 
//! ESP32-C3 IoT environmental monitoring systems. This crate provides 
//! real-time performance tracking, memory usage analysis, and optimization 
//! validation without impacting system performance.
//!
//! ## Features
//!
//! - **Zero-Cost Performance Tracking**: Minimal overhead measurement infrastructure
//! - **Memory Usage Analysis**: Heap, stack, and flash usage monitoring
//! - **Real-Time Constraint Validation**: Timing analysis for critical operations
//! - **Embassy Integration**: Async task performance monitoring
//! - **Build Performance Analysis**: Compilation time and binary size tracking
//! - **Regression Detection**: Automated performance regression testing
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Performance Monitor                          │
//! ├─────────────────────────────────────────────────────────────┤
//! │ CycleTimer │ MemoryTracker │ TaskProfiler │ FlashAnalyzer   │
//! ├─────────────────────────────────────────────────────────────┤
//! │              Performance Data Aggregation                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │               Baseline Comparison                           │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use iot_performance::{PerformanceMonitor, TimingCategory};
//! use embassy_time::Instant;
//!
//! #[embassy_executor::task]
//! async fn sensor_task() {
//!     let mut perf_monitor = PerformanceMonitor::new();
//!     
//!     loop {
//!         let start_time = Instant::now();
//!         
//!         // Your sensor reading code
//!         let measurements = read_sensor().await;
//!         
//!         // Record performance metrics
//!         perf_monitor.record_cycle_time(
//!             TimingCategory::SensorReading,
//!             start_time.elapsed()
//!         );
//!         
//!         // Check if performance is within acceptable bounds
//!         if let Some(report) = perf_monitor.check_performance_thresholds() {
//!             rprintln!("[PERF] Performance alert: {:?}", report);
//!         }
//!         
//!         embassy_time::Timer::after(embassy_time::Duration::from_secs(30)).await;
//!     }
//! }
//! ```
//!
//! ## Memory Usage Tracking
//!
//! ```rust,no_run
//! use iot_performance::{MemoryTracker, MemoryRegion};
//!
//! let mut memory_tracker = MemoryTracker::new();
//!
//! // Record memory usage at critical points
//! memory_tracker.snapshot(MemoryRegion::Heap);
//! memory_tracker.snapshot(MemoryRegion::Stack);
//!
//! // Analyze memory patterns
//! let analysis = memory_tracker.analyze_usage_patterns();
//! rprintln!("[PERF] Memory analysis: {:?}", analysis);
//! ```
//!
//! ## Performance Baselines
//!
//! Established performance baselines for Phase 2 validation:
//!
//! | Metric | Phase 0 Baseline | Phase 2 Target | Current |
//! |--------|------------------|----------------|---------|
//! | Boot Time | ~2.3s | <2.5s | TBD |
//! | Sensor Cycle | ~450μs | <500μs | TBD |
//! | Memory Usage | ~48KB heap | <52KB heap | TBD |
//! | Network Connect | ~3-4s | <5s | TBD |
//! | MQTT Publish | <500ms | <500ms | TBD |

#![no_std]

// Only require alloc for detailed profiling features
#[cfg(feature = "alloc")]
extern crate alloc;

// Core modules
pub mod monitor;
pub mod timing;
pub mod memory;
pub mod analysis;
pub mod baseline;
pub mod regression;

// Platform-specific performance counters
#[cfg(feature = "esp32c3")]
pub mod esp32c3;

// Re-export main types
pub use monitor::{PerformanceMonitor, PerformanceReport, PerformanceAlert};
pub use timing::{TimingCategory, CycleTimer, TimingData, TimingStatistics};
pub use memory::{MemoryTracker, MemoryRegion, MemorySnapshot, MemoryAnalysis};
pub use analysis::{PerformanceAnalyzer, TrendAnalysis, PerformanceTrend};
pub use baseline::{PerformanceBaseline, BaselineComparison, BaselineStatus};
pub use regression::{RegressionTester, RegressionResult, PerformanceThresholds};

// Re-export commonly used types
pub use embassy_time::{Duration, Instant};
pub use heapless::{Vec, String};
pub use iot_common::{IoTError, IoTResult};

/// Current version of the iot-performance library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum number of performance samples to keep in memory
pub const MAX_PERFORMANCE_SAMPLES: usize = 64;

/// Maximum number of timing measurements per category
pub const MAX_TIMING_MEASUREMENTS: usize = 32;

/// Maximum number of memory snapshots to retain
pub const MAX_MEMORY_SNAPSHOTS: usize = 16;

/// Performance monitoring update interval in milliseconds
pub const PERF_UPDATE_INTERVAL_MS: u64 = 1000;

/// Memory usage threshold for alerts (bytes)
pub const MEMORY_ALERT_THRESHOLD: usize = 50 * 1024; // 50KB

/// Timing threshold for alerts (microseconds)
pub const TIMING_ALERT_THRESHOLD_US: u64 = 600; // 600μs

// Performance baseline targets for Phase 2
/// Boot time target in milliseconds
pub const BOOT_TIME_TARGET_MS: u64 = 2500;

/// Sensor reading cycle time target in microseconds 
pub const SENSOR_CYCLE_TARGET_US: u64 = 500;

/// Heap usage target in bytes
pub const HEAP_USAGE_TARGET_BYTES: usize = 52 * 1024;

/// Network connection time target in milliseconds
pub const NETWORK_CONNECT_TARGET_MS: u64 = 5000;

/// MQTT publish time target in milliseconds
pub const MQTT_PUBLISH_TARGET_MS: u64 = 500;