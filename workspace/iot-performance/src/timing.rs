//! Timing Analysis - High-precision operation timing measurement
//!
//! Provides comprehensive timing analysis for embedded IoT operations with
//! minimal performance overhead and detailed statistical analysis.

use embassy_time::{Duration, Instant};
use heapless::{Vec, FnvIndexMap};
use core::fmt::Debug;

/// Integer square root implementation for no-std
fn int_sqrt(value: u64) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}

/// Categories of operations that can be timed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingCategory {
    /// BME280 sensor reading operations
    SensorReading,
    
    /// WiFi connection and network operations
    NetworkOperation,
    
    /// MQTT publishing operations
    MqttPublish,
    
    /// Console command processing
    ConsoleCommand,
    
    /// System initialization and boot
    SystemBoot,
    
    /// Memory allocation operations
    MemoryAllocation,
    
    /// I2C bus operations
    I2cOperation,
    
    /// Task switching and scheduling
    TaskScheduling,
    
    /// Interrupt handling
    InterruptHandling,
    
    /// Overall system cycle time
    SystemCycle,
}

/// Single timing measurement
#[derive(Debug, Clone, Copy)]
pub struct TimingMeasurement {
    /// Duration of the operation
    pub duration: Duration,
    
    /// Timestamp when measurement was taken
    pub timestamp: Instant,
    
    /// Optional context information
    pub context: u32,
}

/// Collection of timing measurements for statistical analysis
#[derive(Debug, Clone)]
pub struct TimingData {
    /// Measurements by category
    measurements: FnvIndexMap<TimingCategory, Vec<TimingMeasurement, 32>, 16>,
    
    /// Total number of measurements recorded
    total_measurements: u32,
    
    /// Time when timing data collection started
    start_time: Instant,
}

/// Statistical analysis of timing measurements
#[derive(Debug, Clone)]
pub struct TimingStatistics {
    /// Statistics per timing category
    category_stats: FnvIndexMap<TimingCategory, CategoryStatistics, 16>,
    
    /// Overall timing analysis
    overall_stats: OverallStatistics,
    
    /// Analysis timestamp
    _analysis_time: Instant,
}

/// Statistical data for a specific timing category
#[derive(Debug, Clone, Copy)]
pub struct CategoryStatistics {
    /// Number of measurements in this category
    pub count: u32,
    
    /// Minimum observed duration
    pub min_duration: Duration,
    
    /// Maximum observed duration
    pub max_duration: Duration,
    
    /// Average duration
    pub average_duration: Duration,
    
    /// Standard deviation (approximated for no-std)
    pub std_deviation: Duration,
    
    /// 95th percentile duration
    pub p95_duration: Duration,
    
    /// 99th percentile duration
    pub p99_duration: Duration,
    
    /// Most recent measurement
    pub last_measurement: Duration,
    
    /// Trend analysis (positive = getting slower)
    pub trend_us_per_second: f32,
}

/// Overall system timing statistics
#[derive(Debug, Clone, Copy)]
pub struct OverallStatistics {
    /// Total number of timed operations
    pub total_operations: u32,
    
    /// Total time spent in timed operations
    pub total_time: Duration,
    
    /// Average time per operation across all categories
    pub average_operation_time: Duration,
    
    /// System performance efficiency ratio (0.0 to 1.0)
    pub efficiency_ratio: f32,
    
    /// Time since statistics collection started
    pub collection_duration: Duration,
}

/// High-precision cycle timer for measuring critical operations
pub struct CycleTimer {
    start_time: Option<Instant>,
    category: TimingCategory,
    context: u32,
}

impl TimingData {
    /// Create new timing data collection
    pub fn new() -> Self {
        Self {
            measurements: FnvIndexMap::new(),
            total_measurements: 0,
            start_time: Instant::now(),
        }
    }
    
    /// Record a timing measurement for a specific category
    pub fn record_measurement(&mut self, category: TimingCategory, duration: Duration) {
        let measurement = TimingMeasurement {
            duration,
            timestamp: Instant::now(),
            context: 0,
        };
        
        // Get or create measurements vector for this category
        let measurements = match self.measurements.entry(category) {
            heapless::Entry::Occupied(entry) => entry.into_mut(),
            heapless::Entry::Vacant(entry) => entry.insert(Vec::new()).expect("Failed to insert timing measurement"),
        };
        
        // Add measurement, removing oldest if at capacity
        if measurements.is_full() {
            measurements.remove(0);
        }
        let _ = measurements.push(measurement);
        
        self.total_measurements = self.total_measurements.saturating_add(1);
    }
    
    /// Record a measurement with additional context
    pub fn record_measurement_with_context(
        &mut self, 
        category: TimingCategory, 
        duration: Duration, 
        context: u32
    ) {
        let measurement = TimingMeasurement {
            duration,
            timestamp: Instant::now(),
            context,
        };
        
        let measurements = match self.measurements.entry(category) {
            heapless::Entry::Occupied(entry) => entry.into_mut(),
            heapless::Entry::Vacant(entry) => entry.insert(Vec::new()).expect("Failed to insert timing measurement"),
        };
        
        if measurements.is_full() {
            measurements.remove(0);
        }
        let _ = measurements.push(measurement);
        
        self.total_measurements = self.total_measurements.saturating_add(1);
    }
    
    /// Get statistical analysis of all timing data
    pub fn get_statistics(&self) -> TimingStatistics {
        let mut category_stats = FnvIndexMap::new();
        let mut total_operations = 0;
        let mut total_time = Duration::from_millis(0);
        
        // Calculate statistics for each category
        for (category, measurements) in &self.measurements {
            if !measurements.is_empty() {
                let stats = self.calculate_category_statistics(measurements);
                let _ = category_stats.insert(*category, stats);
                
                total_operations += stats.count;
                total_time = total_time + Duration::from_micros(
                    (stats.average_duration.as_micros() as u64) * (stats.count as u64)
                );
            }
        }
        
        let collection_duration = self.start_time.elapsed();
        let average_operation_time = if total_operations > 0 {
            Duration::from_micros(total_time.as_micros() as u64 / total_operations as u64)
        } else {
            Duration::from_millis(0)
        };
        
        // Calculate efficiency ratio (time spent in measured operations vs total time)
        let efficiency_ratio = if collection_duration.as_micros() > 0 {
            (total_time.as_micros() as f32) / (collection_duration.as_micros() as f32)
        } else {
            0.0
        }.min(1.0);
        
        let overall_stats = OverallStatistics {
            total_operations,
            total_time,
            average_operation_time,
            efficiency_ratio,
            collection_duration,
        };
        
        TimingStatistics {
            category_stats,
            overall_stats,
            _analysis_time: Instant::now(),
        }
    }
    
    /// Calculate statistics for a specific category
    fn calculate_category_statistics(&self, measurements: &Vec<TimingMeasurement, 32>) -> CategoryStatistics {
        if measurements.is_empty() {
            return CategoryStatistics {
                count: 0,
                min_duration: Duration::from_millis(0),
                max_duration: Duration::from_millis(0),
                average_duration: Duration::from_millis(0),
                std_deviation: Duration::from_millis(0),
                p95_duration: Duration::from_millis(0),
                p99_duration: Duration::from_millis(0),
                last_measurement: Duration::from_millis(0),
                trend_us_per_second: 0.0,
            };
        }
        
        let count = measurements.len() as u32;
        let mut durations: Vec<u64, 32> = Vec::new();
        let mut sum_micros = 0u64;
        
        // Collect duration data
        for measurement in measurements {
            let micros = measurement.duration.as_micros() as u64;
            let _ = durations.push(micros);
            sum_micros += micros;
        }
        
        // Sort for percentile calculations
        durations.sort_unstable();
        
        let min_duration = Duration::from_micros(durations[0]);
        let max_duration = Duration::from_micros(durations[durations.len() - 1]);
        let average_duration = Duration::from_micros(sum_micros / count as u64);
        
        // Calculate standard deviation (simplified for embedded)
        let avg_micros = sum_micros / count as u64;
        let mut variance_sum = 0u64;
        for &duration_micros in &durations {
            let diff = if duration_micros > avg_micros {
                duration_micros - avg_micros
            } else {
                avg_micros - duration_micros
            };
            variance_sum += diff * diff;
        }
        let std_deviation = Duration::from_micros(
            int_sqrt(variance_sum / count as u64)
        );
        
        // Calculate percentiles
        let p95_index = ((count as f32 * 0.95) as usize).min(durations.len() - 1);
        let p99_index = ((count as f32 * 0.99) as usize).min(durations.len() - 1);
        let p95_duration = Duration::from_micros(durations[p95_index]);
        let p99_duration = Duration::from_micros(durations[p99_index]);
        
        let last_measurement = measurements.last().unwrap().duration;
        
        // Calculate trend (simplified linear regression)
        let trend_us_per_second = self.calculate_trend(measurements);
        
        CategoryStatistics {
            count,
            min_duration,
            max_duration,
            average_duration,
            std_deviation,
            p95_duration,
            p99_duration,
            last_measurement,
            trend_us_per_second,
        }
    }
    
    /// Calculate performance trend for measurements
    fn calculate_trend(&self, measurements: &Vec<TimingMeasurement, 32>) -> f32 {
        if measurements.len() < 3 {
            return 0.0;
        }
        
        // Simple linear trend analysis
        let _first_time = measurements[0].timestamp;
        let mut sum_x = 0.0f32;
        let mut sum_y = 0.0f32;
        let mut sum_xy = 0.0f32;
        let mut sum_x2 = 0.0f32;
        let n = measurements.len() as f32;
        
        for (i, measurement) in measurements.iter().enumerate() {
            let x = i as f32;
            let y = measurement.duration.as_micros() as f32;
            
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-6 {
            return 0.0;
        }
        
        // Slope in microseconds per measurement
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        
        // Convert to microseconds per second (assuming measurements roughly every 30 seconds)
        slope / 30.0
    }
    
    /// Get measurements for a specific category
    pub fn get_measurements(&self, category: TimingCategory) -> Option<&Vec<TimingMeasurement, 32>> {
        self.measurements.get(&category)
    }
    
    /// Clear all measurements
    pub fn clear(&mut self) {
        self.measurements.clear();
        self.total_measurements = 0;
        self.start_time = Instant::now();
    }
}

impl TimingStatistics {
    /// Get average time for a specific category
    pub fn get_average_time(&self, category: TimingCategory) -> Option<Duration> {
        self.category_stats.get(&category).map(|stats| stats.average_duration)
    }
    
    /// Get maximum time for a specific category
    pub fn get_max_time(&self, category: TimingCategory) -> Option<Duration> {
        self.category_stats.get(&category).map(|stats| stats.max_duration)
    }
    
    /// Get measurement count for a specific category
    pub fn get_count(&self, category: TimingCategory) -> u32 {
        self.category_stats.get(&category).map_or(0, |stats| stats.count)
    }
    
    /// Check if any category exceeds performance thresholds
    pub fn has_performance_issues(&self) -> bool {
        for (category, stats) in &self.category_stats {
            let threshold = match category {
                TimingCategory::SensorReading => Duration::from_micros(crate::SENSOR_CYCLE_TARGET_US),
                TimingCategory::NetworkOperation => Duration::from_millis(crate::NETWORK_CONNECT_TARGET_MS),
                TimingCategory::MqttPublish => Duration::from_millis(crate::MQTT_PUBLISH_TARGET_MS),
                _ => Duration::from_millis(1000), // 1 second default threshold
            };
            
            if stats.average_duration > threshold {
                return true;
            }
        }
        false
    }
    
    /// Get overall statistics
    pub fn get_overall_stats(&self) -> &OverallStatistics {
        &self.overall_stats
    }
}

impl CycleTimer {
    /// Create a new cycle timer for a specific category
    pub fn new(category: TimingCategory) -> Self {
        Self {
            start_time: None,
            category,
            context: 0,
        }
    }
    
    /// Create a new cycle timer with context
    pub fn with_context(category: TimingCategory, context: u32) -> Self {
        Self {
            start_time: None,
            category,
            context,
        }
    }
    
    /// Start timing an operation
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }
    
    /// Stop timing and return the elapsed duration
    pub fn stop(&mut self) -> Option<Duration> {
        if let Some(start) = self.start_time.take() {
            Some(start.elapsed())
        } else {
            None
        }
    }
    
    /// Stop timing and record in timing data
    pub fn stop_and_record(&mut self, timing_data: &mut TimingData) -> Option<Duration> {
        if let Some(duration) = self.stop() {
            timing_data.record_measurement_with_context(self.category, duration, self.context);
            Some(duration)
        } else {
            None
        }
    }
    
    /// Get the category this timer is measuring
    pub fn category(&self) -> TimingCategory {
        self.category
    }
    
    /// Get the context value
    pub fn context(&self) -> u32 {
        self.context
    }
    
    /// Check if timer is currently running
    pub fn is_running(&self) -> bool {
        self.start_time.is_some()
    }
}

impl Default for TimingData {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for easy timing of operations
#[macro_export]
macro_rules! time_operation {
    ($timing_data:expr, $category:expr, $operation:expr) => {{
        let start = embassy_time::Instant::now();
        let result = $operation;
        let duration = start.elapsed();
        $timing_data.record_measurement($category, duration);
        result
    }};
}