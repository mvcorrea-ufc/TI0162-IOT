//! Performance Monitor - Core monitoring infrastructure
//!
//! Provides the main PerformanceMonitor struct for tracking system performance
//! in real-time with minimal overhead suitable for embedded environments.

use embassy_time::{Duration, Instant};
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use heapless::{Vec, String};
use core::fmt::Write;

use crate::timing::{TimingCategory, TimingData, TimingStatistics};
use crate::memory::{MemoryTracker, MemorySnapshot, MemoryRegion};
use crate::baseline::{PerformanceBaseline, BaselineComparison};
use iot_common::{IoTError, IoTResult};

/// Main performance monitoring coordinator
/// 
/// Tracks timing, memory usage, and system performance with minimal overhead.
/// Designed to be embedded in IoT applications without affecting real-time behavior.
pub struct PerformanceMonitor {
    /// Timing measurements for different operation categories
    timing_data: Mutex<CriticalSectionRawMutex, TimingData>,
    
    /// Memory usage tracking
    memory_tracker: Mutex<CriticalSectionRawMutex, MemoryTracker>,
    
    /// Performance baseline for comparison
    baseline: PerformanceBaseline,
    
    /// System start time for uptime calculations
    start_time: Instant,
    
    /// Alert threshold configuration
    alert_config: AlertConfiguration,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Copy)]
pub struct AlertConfiguration {
    /// Maximum acceptable sensor cycle time (microseconds)
    pub max_sensor_cycle_us: u64,
    
    /// Maximum acceptable memory usage (bytes)
    pub max_memory_bytes: usize,
    
    /// Maximum acceptable network operation time (milliseconds)
    pub max_network_ms: u64,
    
    /// Enable detailed timing analysis
    pub detailed_timing: bool,
    
    /// Enable memory fragmentation tracking
    pub track_fragmentation: bool,
}

impl Default for AlertConfiguration {
    fn default() -> Self {
        Self {
            max_sensor_cycle_us: crate::SENSOR_CYCLE_TARGET_US,
            max_memory_bytes: crate::HEAP_USAGE_TARGET_BYTES,
            max_network_ms: crate::NETWORK_CONNECT_TARGET_MS,
            detailed_timing: true,
            track_fragmentation: true,
        }
    }
}

/// Performance monitoring report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// System uptime in seconds
    pub uptime_seconds: u64,
    
    /// Current timing statistics
    pub timing_stats: TimingStatistics,
    
    /// Current memory usage
    pub memory_usage: MemorySnapshot,
    
    /// Baseline comparison results
    pub baseline_comparison: BaselineComparison,
    
    /// Active performance alerts
    pub alerts: Vec<PerformanceAlert, 8>,
    
    /// Overall system performance status
    pub status: PerformanceStatus,
}

/// Performance alert information
#[derive(Debug, Clone, Copy)]
pub struct PerformanceAlert {
    /// Type of performance issue detected
    pub alert_type: AlertType,
    
    /// Severity level of the alert
    pub severity: AlertSeverity,
    
    /// Measured value that triggered the alert
    pub measured_value: u64,
    
    /// Expected threshold value
    pub threshold_value: u64,
    
    /// Timestamp when alert was generated
    pub timestamp: Instant,
}

/// Types of performance alerts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertType {
    /// Sensor reading cycle taking too long
    SlowSensorCycle,
    
    /// Memory usage exceeding threshold
    HighMemoryUsage,
    
    /// Network operations taking too long
    SlowNetworkOperation,
    
    /// Task scheduling delays detected
    TaskSchedulingDelay,
    
    /// Memory fragmentation detected
    MemoryFragmentation,
    
    /// Performance regression detected
    PerformanceRegression,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum AlertSeverity {
    /// Information only, within acceptable bounds
    Info,
    
    /// Warning, approaching threshold
    Warning,
    
    /// Error, threshold exceeded
    Error,
    
    /// Critical, system performance significantly degraded
    Critical,
}

/// Overall system performance status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceStatus {
    /// All metrics within acceptable bounds
    Optimal,
    
    /// Some metrics approaching thresholds
    Acceptable,
    
    /// Performance degraded but functional
    Degraded,
    
    /// Significant performance issues detected
    Critical,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(AlertConfiguration::default())
    }
    
    /// Create a new performance monitor with custom configuration
    pub fn with_config(config: AlertConfiguration) -> Self {
        Self {
            timing_data: Mutex::new(TimingData::new()),
            memory_tracker: Mutex::new(MemoryTracker::new()),
            baseline: PerformanceBaseline::phase_2_targets(),
            start_time: Instant::now(),
            alert_config: config,
        }
    }
    
    /// Record a timing measurement for a specific category
    pub async fn record_cycle_time(&self, category: TimingCategory, duration: Duration) {
        let mut timing_data = self.timing_data.lock().await;
        timing_data.record_measurement(category, duration);
    }
    
    /// Record current memory usage snapshot
    pub async fn record_memory_usage(&self, heap_used: usize, stack_peak: usize) -> IoTResult<()> {
        let mut tracker = self.memory_tracker.lock().await;
        tracker.record_snapshot(MemoryRegion::Heap, heap_used).map_err(|e| iot_common::IoTError::configuration(iot_common::ConfigError::InvalidParameter(heapless::String::try_from(e).unwrap_or_else(|_| heapless::String::new()))))?;
        tracker.record_snapshot(MemoryRegion::Stack, stack_peak).map_err(|e| iot_common::IoTError::configuration(iot_common::ConfigError::InvalidParameter(heapless::String::try_from(e).unwrap_or_else(|_| heapless::String::new()))))?;
        Ok(())
    }
    
    /// Track a specific operation and measure its performance
    pub async fn track_operation<F, R>(&self, category: TimingCategory, operation: F) -> R
    where
        F: core::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = operation.await;
        let duration = start.elapsed();
        
        // Record the timing asynchronously to minimize impact
        if let Ok(mut timing_data) = self.timing_data.try_lock() {
            timing_data.record_measurement(category, duration);
        }
        
        result
    }
    
    /// Generate comprehensive performance report
    pub async fn generate_report(&self) -> PerformanceReport {
        let timing_data = self.timing_data.lock().await;
        let memory_tracker = self.memory_tracker.lock().await;
        
        let uptime = self.start_time.elapsed();
        let timing_stats = timing_data.get_statistics();
        let memory_usage = memory_tracker.get_current_snapshot();
        let baseline_comparison = self.baseline.compare_current_performance(&timing_stats, &memory_usage);
        let alerts = self.check_performance_alerts(&timing_stats, &memory_usage).await;
        let status = self.calculate_overall_status(&alerts);
        
        PerformanceReport {
            uptime_seconds: uptime.as_secs(),
            timing_stats,
            memory_usage,
            baseline_comparison,
            alerts,
            status,
        }
    }
    
    /// Check for performance alerts and return any active alerts
    pub async fn check_performance_thresholds(&self) -> Option<Vec<PerformanceAlert, 8>> {
        let timing_data = self.timing_data.lock().await;
        let memory_tracker = self.memory_tracker.lock().await;
        
        let timing_stats = timing_data.get_statistics();
        let memory_usage = memory_tracker.get_current_snapshot();
        
        let alerts = self.check_performance_alerts(&timing_stats, &memory_usage).await;
        
        if alerts.is_empty() {
            None
        } else {
            Some(alerts)
        }
    }
    
    /// Internal method to check for performance alerts
    async fn check_performance_alerts(
        &self, 
        timing_stats: &TimingStatistics, 
        memory_usage: &MemorySnapshot
    ) -> Vec<PerformanceAlert, 8> {
        let mut alerts = Vec::new();
        let now = Instant::now();
        
        // Check sensor cycle time
        if let Some(sensor_time) = timing_stats.get_average_time(TimingCategory::SensorReading) {
            let sensor_time_us = sensor_time.as_micros() as u64;
            if sensor_time_us > self.alert_config.max_sensor_cycle_us {
                let severity = if sensor_time_us > self.alert_config.max_sensor_cycle_us * 2 {
                    AlertSeverity::Critical
                } else if sensor_time_us > self.alert_config.max_sensor_cycle_us * 3 / 2 {
                    AlertSeverity::Error
                } else {
                    AlertSeverity::Warning
                };
                
                let _ = alerts.push(PerformanceAlert {
                    alert_type: AlertType::SlowSensorCycle,
                    severity,
                    measured_value: sensor_time_us,
                    threshold_value: self.alert_config.max_sensor_cycle_us,
                    timestamp: now,
                });
            }
        }
        
        // Check memory usage
        if memory_usage.heap_used > self.alert_config.max_memory_bytes {
            let severity = if memory_usage.heap_used > self.alert_config.max_memory_bytes * 3 / 2 {
                AlertSeverity::Critical
            } else {
                AlertSeverity::Warning
            };
            
            let _ = alerts.push(PerformanceAlert {
                alert_type: AlertType::HighMemoryUsage,
                severity,
                measured_value: memory_usage.heap_used as u64,
                threshold_value: self.alert_config.max_memory_bytes as u64,
                timestamp: now,
            });
        }
        
        // Check network operation time
        if let Some(network_time) = timing_stats.get_average_time(TimingCategory::NetworkOperation) {
            let network_time_ms = network_time.as_millis() as u64;
            if network_time_ms > self.alert_config.max_network_ms {
                let _ = alerts.push(PerformanceAlert {
                    alert_type: AlertType::SlowNetworkOperation,
                    severity: AlertSeverity::Warning,
                    measured_value: network_time_ms,
                    threshold_value: self.alert_config.max_network_ms,
                    timestamp: now,
                });
            }
        }
        
        alerts
    }
    
    /// Calculate overall system performance status
    fn calculate_overall_status(&self, alerts: &Vec<PerformanceAlert, 8>) -> PerformanceStatus {
        if alerts.is_empty() {
            return PerformanceStatus::Optimal;
        }
        
        let mut max_severity = AlertSeverity::Info;
        for alert in alerts {
            if alert.severity > max_severity {
                max_severity = alert.severity;
            }
        }
        
        match max_severity {
            AlertSeverity::Info => PerformanceStatus::Optimal,
            AlertSeverity::Warning => PerformanceStatus::Acceptable,
            AlertSeverity::Error => PerformanceStatus::Degraded,
            AlertSeverity::Critical => PerformanceStatus::Critical,
        }
    }
    
    /// Get system uptime in seconds
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// Reset all performance counters and start fresh monitoring
    pub async fn reset_counters(&self) {
        let mut timing_data = self.timing_data.lock().await;
        let mut memory_tracker = self.memory_tracker.lock().await;
        
        *timing_data = TimingData::new();
        *memory_tracker = MemoryTracker::new();
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for formatting performance reports
impl PerformanceReport {
    /// Format report as a compact string for RTT output
    pub fn format_compact(&self) -> Result<String<256>, core::fmt::Error> {
        let mut output = String::new();
        
        write!(
            output,
            "[PERF] Uptime: {}s, Status: {:?}, Alerts: {}, Heap: {}KB",
            self.uptime_seconds,
            self.status,
            self.alerts.len(),
            self.memory_usage.heap_used / 1024
        )?;
        
        Ok(output)
    }
    
    /// Format detailed report for comprehensive analysis
    pub fn format_detailed(&self) -> Result<String<512>, core::fmt::Error> {
        let mut output = String::new();
        
        write!(
            output,
            "[PERF] === Performance Report ===\n\
             Uptime: {}s, Status: {:?}\n\
             Memory: Heap={} KB, Stack Peak={} KB\n\
             Alerts: {} active",
            self.uptime_seconds,
            self.status,
            self.memory_usage.heap_used / 1024,
            self.memory_usage.stack_peak / 1024,
            self.alerts.len()
        )?;
        
        for alert in &self.alerts {
            write!(
                output,
                "\n  - {:?}: {} (threshold: {})",
                alert.alert_type,
                alert.measured_value,
                alert.threshold_value
            )?;
        }
        
        Ok(output)
    }
}