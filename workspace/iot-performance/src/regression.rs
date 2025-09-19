//! Performance Regression Testing - Automated detection of performance regressions
//!
//! Provides comprehensive regression testing infrastructure to ensure that
//! architectural improvements don't introduce performance regressions.

use embassy_time::{Duration, Instant};
use heapless::{Vec, String, FnvIndexMap};
use core::fmt::Write;

use crate::timing::{TimingStatistics, TimingCategory};
use crate::memory::MemorySnapshot;
use crate::baseline::{PerformanceBaseline, BaselineComparison, BaselineStatus};

/// Automated performance regression tester
pub struct RegressionTester {
    /// Reference baselines for regression comparison
    reference_baselines: Vec<PerformanceBaseline, 4>,
    
    /// Performance thresholds for regression detection
    thresholds: PerformanceThresholds,
    
    /// Historical test results
    test_history: Vec<RegressionTestResult, 16>,
    
    /// Configuration for regression testing
    config: RegressionConfig,
}

/// Configuration for regression testing
#[derive(Debug, Clone, Copy)]
pub struct RegressionConfig {
    /// Minimum degradation percentage to trigger regression alert
    pub regression_threshold_percent: f32,
    
    /// Number of consecutive failures needed to confirm regression
    pub consecutive_failure_threshold: u32,
    
    /// Enable statistical significance testing
    pub statistical_testing: bool,
    
    /// Confidence level for statistical tests (0.0 to 1.0)
    pub confidence_level: f32,
    
    /// Maximum acceptable p-value for statistical tests
    pub max_p_value: f32,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            regression_threshold_percent: 15.0, // 15% degradation threshold
            consecutive_failure_threshold: 3,   // 3 consecutive failures
            statistical_testing: true,
            confidence_level: 0.95,             // 95% confidence
            max_p_value: 0.05,                  // 5% p-value threshold
        }
    }
}

/// Performance thresholds for different metrics
#[derive(Debug, Clone, Copy)]
pub struct PerformanceThresholds {
    /// Timing thresholds by category
    pub timing_thresholds: [TimingThreshold; 8],
    
    /// Memory usage thresholds
    pub memory_thresholds: MemoryThresholds,
    
    /// System-level thresholds
    pub system_thresholds: SystemThresholds,
}

/// Timing threshold for a specific category
#[derive(Debug, Clone, Copy)]
pub struct TimingThreshold {
    /// Operation category
    pub category: TimingCategory,
    
    /// Baseline time for comparison
    pub baseline_time: Duration,
    
    /// Maximum acceptable time (for regression detection)
    pub max_time: Duration,
    
    /// Whether this threshold is enabled
    pub enabled: bool,
}

/// Memory usage thresholds
#[derive(Debug, Clone, Copy)]
pub struct MemoryThresholds {
    /// Maximum heap usage increase (bytes)
    pub max_heap_increase: usize,
    
    /// Maximum stack usage increase (bytes)
    pub max_stack_increase: usize,
    
    /// Maximum flash usage increase (bytes)
    pub max_flash_increase: usize,
    
    /// Maximum acceptable fragmentation increase (percentage)
    pub max_fragmentation_increase: f32,
}

/// System-level performance thresholds
#[derive(Debug, Clone, Copy)]
pub struct SystemThresholds {
    /// Maximum boot time increase
    pub max_boot_time_increase: Duration,
    
    /// Minimum acceptable efficiency
    pub min_efficiency: f32,
    
    /// Maximum acceptable alert frequency (per hour)
    pub max_alert_frequency: f32,
}

/// Result of a regression test
#[derive(Debug, Clone)]
pub struct RegressionTestResult {
    /// Test execution timestamp
    pub timestamp: Instant,
    
    /// Overall test result
    pub result: RegressionResult,
    
    /// Individual metric results
    pub metric_results: Vec<MetricRegressionResult, 16>,
    
    /// Statistical analysis results
    pub statistical_results: StatisticalAnalysis,
    
    /// Detected regressions
    pub regressions: Vec<DetectedRegression, 8>,
    
    /// Test execution time
    pub execution_time: Duration,
}

/// Overall regression test result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegressionResult {
    /// No regressions detected, performance maintained or improved
    Pass,
    
    /// Minor regressions detected, within acceptable bounds
    PassWithWarnings,
    
    /// Significant regressions detected
    Fail,
    
    /// Critical regressions that require immediate attention
    CriticalFail,
    
    /// Test inconclusive due to insufficient data
    Inconclusive,
}

/// Regression test result for individual metric
#[derive(Debug, Clone, Copy)]
pub struct MetricRegressionResult {
    /// Metric identifier
    pub metric: PerformanceMetric,
    
    /// Current measured value
    pub current_value: f32,
    
    /// Baseline reference value
    pub baseline_value: f32,
    
    /// Performance change percentage
    pub change_percent: f32,
    
    /// Whether this metric passed the regression test
    pub passed: bool,
    
    /// Severity of any detected regression
    pub regression_severity: RegressionSeverity,
}

/// Performance metrics that can be tested for regression
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceMetric {
    /// Sensor reading time
    SensorReadingTime,
    
    /// MQTT publish time
    MqttPublishTime,
    
    /// Network connection time
    NetworkConnectionTime,
    
    /// Boot time
    BootTime,
    
    /// Heap usage
    HeapUsage,
    
    /// Stack usage
    StackUsage,
    
    /// Flash usage
    FlashUsage,
    
    /// System efficiency
    SystemEfficiency,
    
    /// Console response time
    ConsoleResponseTime,
    
    /// Task scheduling latency
    TaskSchedulingLatency,
}

/// Severity of detected regression
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RegressionSeverity {
    /// No regression detected
    None,
    
    /// Minor regression, monitoring recommended
    Minor,
    
    /// Moderate regression, investigation recommended
    Moderate,
    
    /// Major regression, immediate attention required
    Major,
    
    /// Critical regression, blocking issue
    Critical,
}

/// Statistical analysis of regression test
#[derive(Debug, Clone, Copy)]
pub struct StatisticalAnalysis {
    /// Sample size used for analysis
    pub sample_size: u32,
    
    /// Statistical significance of results
    pub significance: f32,
    
    /// P-value for hypothesis testing
    pub p_value: f32,
    
    /// Confidence interval for performance change
    pub confidence_interval: (f32, f32),
    
    /// Whether statistical test passed
    pub statistical_test_passed: bool,
}

/// Detected performance regression
#[derive(Debug, Clone)]
pub struct DetectedRegression {
    /// Affected metric
    pub metric: PerformanceMetric,
    
    /// Regression severity
    pub severity: RegressionSeverity,
    
    /// Performance degradation percentage
    pub degradation_percent: f32,
    
    /// Expected vs actual values
    pub expected_value: f32,
    pub actual_value: f32,
    
    /// Recommended actions
    pub recommendations: Vec<RegressionRecommendation, 4>,
    
    /// Additional context
    pub description: String<128>,
}

/// Recommendations for addressing regressions
#[derive(Debug, Clone, Copy)]
pub enum RegressionRecommendation {
    /// Review recent code changes
    ReviewCodeChanges,
    
    /// Profile memory allocation patterns
    ProfileMemoryAllocation,
    
    /// Optimize critical path performance
    OptimizeCriticalPath,
    
    /// Review compiler optimization settings
    ReviewCompilerSettings,
    
    /// Check for resource leaks
    CheckResourceLeaks,
    
    /// Analyze task scheduling efficiency
    AnalyzeTaskScheduling,
    
    /// Review hardware configuration
    ReviewHardwareConfig,
    
    /// Consider performance tuning
    ConsiderPerformanceTuning,
}

impl RegressionTester {
    /// Create a new regression tester with default configuration
    pub fn new() -> Self {
        Self::with_config(RegressionConfig::default())
    }
    
    /// Create regression tester with custom configuration
    pub fn with_config(config: RegressionConfig) -> Self {
        let mut reference_baselines = Vec::new();
        let _ = reference_baselines.push(PerformanceBaseline::phase_0_baseline());
        let _ = reference_baselines.push(PerformanceBaseline::phase_2_targets());
        
        Self {
            reference_baselines,
            thresholds: Self::create_default_thresholds(),
            test_history: Vec::new(),
            config,
        }
    }
    
    /// Create default performance thresholds
    fn create_default_thresholds() -> PerformanceThresholds {
        let timing_thresholds = [
            TimingThreshold {
                category: TimingCategory::SensorReading,
                baseline_time: Duration::from_micros(450),
                max_time: Duration::from_micros(500),
                enabled: true,
            },
            TimingThreshold {
                category: TimingCategory::MqttPublish,
                baseline_time: Duration::from_millis(300),
                max_time: Duration::from_millis(500),
                enabled: true,
            },
            TimingThreshold {
                category: TimingCategory::NetworkOperation,
                baseline_time: Duration::from_millis(3500),
                max_time: Duration::from_millis(5000),
                enabled: true,
            },
            TimingThreshold {
                category: TimingCategory::SystemBoot,
                baseline_time: Duration::from_millis(2300),
                max_time: Duration::from_millis(2500),
                enabled: true,
            },
            TimingThreshold {
                category: TimingCategory::ConsoleCommand,
                baseline_time: Duration::from_millis(50),
                max_time: Duration::from_millis(100),
                enabled: false,
            },
            TimingThreshold {
                category: TimingCategory::I2cOperation,
                baseline_time: Duration::from_micros(100),
                max_time: Duration::from_micros(200),
                enabled: false,
            },
            TimingThreshold {
                category: TimingCategory::TaskScheduling,
                baseline_time: Duration::from_micros(10),
                max_time: Duration::from_micros(50),
                enabled: false,
            },
            TimingThreshold {
                category: TimingCategory::SystemCycle,
                baseline_time: Duration::from_secs(30),
                max_time: Duration::from_secs(35),
                enabled: false,
            },
        ];
        
        let memory_thresholds = MemoryThresholds {
            max_heap_increase: 4 * 1024,    // 4KB increase maximum
            max_stack_increase: 2 * 1024,   // 2KB increase maximum
            max_flash_increase: 32 * 1024,  // 32KB increase maximum
            max_fragmentation_increase: 10.0, // 10% fragmentation increase
        };
        
        let system_thresholds = SystemThresholds {
            max_boot_time_increase: Duration::from_millis(200),
            min_efficiency: 0.80,
            max_alert_frequency: 5.0,
        };
        
        PerformanceThresholds {
            timing_thresholds,
            memory_thresholds,
            system_thresholds,
        }
    }
    
    /// Execute comprehensive regression test
    pub fn execute_regression_test(
        &mut self,
        timing_stats: &TimingStatistics,
        memory_snapshot: &MemorySnapshot,
    ) -> RegressionTestResult {
        let start_time = Instant::now();
        
        let mut metric_results = Vec::new();
        let mut regressions = Vec::new();
        
        // Test timing metrics
        self.test_timing_metrics(timing_stats, &mut metric_results, &mut regressions);
        
        // Test memory metrics
        self.test_memory_metrics(memory_snapshot, &mut metric_results, &mut regressions);
        
        // Test system metrics
        self.test_system_metrics(timing_stats, memory_snapshot, &mut metric_results, &mut regressions);
        
        // Perform statistical analysis
        let statistical_results = self.perform_statistical_analysis(&metric_results);
        
        // Determine overall result
        let result = self.determine_overall_result(&metric_results, &regressions, &statistical_results);
        
        let execution_time = start_time.elapsed();
        
        let test_result = RegressionTestResult {
            timestamp: start_time,
            result,
            metric_results,
            statistical_results,
            regressions,
            execution_time,
        };
        
        // Store result in history
        if self.test_history.is_full() {
            self.test_history.remove(0);
        }
        let _ = self.test_history.push(test_result.clone());
        
        test_result
    }
    
    /// Test timing metrics for regressions
    fn test_timing_metrics(
        &self,
        timing_stats: &TimingStatistics,
        metric_results: &mut Vec<MetricRegressionResult, 16>,
        regressions: &mut Vec<DetectedRegression, 8>,
    ) {
        for threshold in &self.thresholds.timing_thresholds {
            if !threshold.enabled {
                continue;
            }
            
            if let Some(current_time) = timing_stats.get_average_time(threshold.category) {
                let current_value = current_time.as_micros() as f32;
                let baseline_value = threshold.baseline_time.as_micros() as f32;
                let change_percent = ((current_value - baseline_value) / baseline_value) * 100.0;
                
                let passed = current_time <= threshold.max_time;
                let regression_severity = self.calculate_regression_severity(change_percent);
                
                let metric = match threshold.category {
                    TimingCategory::SensorReading => PerformanceMetric::SensorReadingTime,
                    TimingCategory::MqttPublish => PerformanceMetric::MqttPublishTime,
                    TimingCategory::NetworkOperation => PerformanceMetric::NetworkConnectionTime,
                    TimingCategory::SystemBoot => PerformanceMetric::BootTime,
                    TimingCategory::ConsoleCommand => PerformanceMetric::ConsoleResponseTime,
                    TimingCategory::TaskScheduling => PerformanceMetric::TaskSchedulingLatency,
                    _ => continue,
                };
                
                let result = MetricRegressionResult {
                    metric,
                    current_value,
                    baseline_value,
                    change_percent,
                    passed,
                    regression_severity,
                };
                
                let _ = metric_results.push(result);
                
                // Create regression if detected
                if regression_severity > RegressionSeverity::None {
                    self.create_timing_regression(
                        metric,
                        regression_severity,
                        change_percent,
                        baseline_value,
                        current_value,
                        regressions,
                    );
                }
            }
        }
    }
    
    /// Test memory metrics for regressions
    fn test_memory_metrics(
        &self,
        memory_snapshot: &MemorySnapshot,
        metric_results: &mut Vec<MetricRegressionResult, 16>,
        regressions: &mut Vec<DetectedRegression, 8>,
    ) {
        // Get baseline memory usage (Phase 0 baseline)
        let baseline = &self.reference_baselines[0].memory_baseline;
        
        // Test heap usage
        let heap_change = memory_snapshot.heap_used as i32 - baseline.target_heap_bytes as i32;
        if heap_change > self.thresholds.memory_thresholds.max_heap_increase as i32 {
            let change_percent = (heap_change as f32 / baseline.target_heap_bytes as f32) * 100.0;
            let severity = self.calculate_regression_severity(change_percent);
            
            let result = MetricRegressionResult {
                metric: PerformanceMetric::HeapUsage,
                current_value: memory_snapshot.heap_used as f32,
                baseline_value: baseline.target_heap_bytes as f32,
                change_percent,
                passed: false,
                regression_severity: severity,
            };
            
            let _ = metric_results.push(result);
            
            if severity > RegressionSeverity::None {
                self.create_memory_regression(
                    PerformanceMetric::HeapUsage,
                    severity,
                    change_percent,
                    baseline.target_heap_bytes as f32,
                    memory_snapshot.heap_used as f32,
                    regressions,
                );
            }
        }
        
        // Test stack usage
        let stack_change = memory_snapshot.stack_used as i32 - baseline.target_stack_bytes as i32;
        if stack_change > self.thresholds.memory_thresholds.max_stack_increase as i32 {
            let change_percent = (stack_change as f32 / baseline.target_stack_bytes as f32) * 100.0;
            let severity = self.calculate_regression_severity(change_percent);
            
            let result = MetricRegressionResult {
                metric: PerformanceMetric::StackUsage,
                current_value: memory_snapshot.stack_used as f32,
                baseline_value: baseline.target_stack_bytes as f32,
                change_percent,
                passed: false,
                regression_severity: severity,
            };
            
            let _ = metric_results.push(result);
        }
    }
    
    /// Test system-level metrics for regressions
    fn test_system_metrics(
        &self,
        timing_stats: &TimingStatistics,
        _memory_snapshot: &MemorySnapshot,
        metric_results: &mut Vec<MetricRegressionResult, 16>,
        regressions: &mut Vec<DetectedRegression, 8>,
    ) {
        // Test system efficiency
        let current_efficiency = timing_stats.get_overall_stats().efficiency_ratio;
        let min_efficiency = self.thresholds.system_thresholds.min_efficiency;
        
        if current_efficiency < min_efficiency {
            let change_percent = ((min_efficiency - current_efficiency) / min_efficiency) * 100.0;
            let severity = self.calculate_regression_severity(change_percent);
            
            let result = MetricRegressionResult {
                metric: PerformanceMetric::SystemEfficiency,
                current_value: current_efficiency,
                baseline_value: min_efficiency,
                change_percent: -change_percent, // Negative because efficiency decreased
                passed: false,
                regression_severity: severity,
            };
            
            let _ = metric_results.push(result);
        }
    }
    
    /// Calculate regression severity based on performance change
    fn calculate_regression_severity(&self, change_percent: f32) -> RegressionSeverity {
        let abs_change = change_percent.abs();
        
        if abs_change < self.config.regression_threshold_percent / 3.0 {
            RegressionSeverity::None
        } else if abs_change < self.config.regression_threshold_percent / 2.0 {
            RegressionSeverity::Minor
        } else if abs_change < self.config.regression_threshold_percent {
            RegressionSeverity::Moderate
        } else if abs_change < self.config.regression_threshold_percent * 2.0 {
            RegressionSeverity::Major
        } else {
            RegressionSeverity::Critical
        }
    }
    
    /// Create timing regression detection
    fn create_timing_regression(
        &self,
        metric: PerformanceMetric,
        severity: RegressionSeverity,
        degradation_percent: f32,
        expected_value: f32,
        actual_value: f32,
        regressions: &mut Vec<DetectedRegression, 8>,
    ) {
        let mut recommendations = Vec::new();
        let _ = recommendations.push(RegressionRecommendation::ReviewCodeChanges);
        let _ = recommendations.push(RegressionRecommendation::OptimizeCriticalPath);
        
        if severity >= RegressionSeverity::Major {
            let _ = recommendations.push(RegressionRecommendation::ReviewCompilerSettings);
        }
        
        let mut description = String::new();
        let _ = write!(
            description,
            "{:?} degraded by {:.1}% (expected: {:.1}, actual: {:.1})",
            metric, degradation_percent, expected_value, actual_value
        );
        
        let regression = DetectedRegression {
            metric,
            severity,
            degradation_percent,
            expected_value,
            actual_value,
            recommendations,
            description,
        };
        
        let _ = regressions.push(regression);
    }
    
    /// Create memory regression detection
    fn create_memory_regression(
        &self,
        metric: PerformanceMetric,
        severity: RegressionSeverity,
        degradation_percent: f32,
        expected_value: f32,
        actual_value: f32,
        regressions: &mut Vec<DetectedRegression, 8>,
    ) {
        let mut recommendations = Vec::new();
        let _ = recommendations.push(RegressionRecommendation::ProfileMemoryAllocation);
        let _ = recommendations.push(RegressionRecommendation::CheckResourceLeaks);
        
        let mut description = String::new();
        let _ = write!(
            description,
            "{:?} increased by {:.1}% (expected: {:.0}B, actual: {:.0}B)",
            metric, degradation_percent, expected_value, actual_value
        );
        
        let regression = DetectedRegression {
            metric,
            severity,
            degradation_percent,
            expected_value,
            actual_value,
            recommendations,
            description,
        };
        
        let _ = regressions.push(regression);
    }
    
    /// Perform statistical analysis of regression test results
    fn perform_statistical_analysis(&self, metric_results: &Vec<MetricRegressionResult, 16>) -> StatisticalAnalysis {
        if !self.config.statistical_testing || metric_results.is_empty() {
            return StatisticalAnalysis {
                sample_size: 0,
                significance: 0.0,
                p_value: 1.0,
                confidence_interval: (0.0, 0.0),
                statistical_test_passed: false,
            };
        }
        
        let sample_size = metric_results.len() as u32;
        let failed_tests = metric_results.iter().filter(|r| !r.passed).count() as f32;
        let failure_rate = failed_tests / sample_size as f32;
        
        // Simplified statistical analysis
        let significance = if failure_rate > 0.5 { 0.95 } else { 0.5 };
        let p_value = failure_rate;
        let confidence_interval = (failure_rate * 0.8, failure_rate * 1.2);
        let statistical_test_passed = p_value <= self.config.max_p_value;
        
        StatisticalAnalysis {
            sample_size,
            significance,
            p_value,
            confidence_interval,
            statistical_test_passed,
        }
    }
    
    /// Determine overall regression test result
    fn determine_overall_result(
        &self,
        metric_results: &Vec<MetricRegressionResult, 16>,
        regressions: &Vec<DetectedRegression, 8>,
        statistical_results: &StatisticalAnalysis,
    ) -> RegressionResult {
        if metric_results.is_empty() {
            return RegressionResult::Inconclusive;
        }
        
        // Check for critical regressions
        if regressions.iter().any(|r| r.severity == RegressionSeverity::Critical) {
            return RegressionResult::CriticalFail;
        }
        
        // Check for major regressions
        if regressions.iter().any(|r| r.severity == RegressionSeverity::Major) {
            return RegressionResult::Fail;
        }
        
        // Check statistical test
        if self.config.statistical_testing && !statistical_results.statistical_test_passed {
            return RegressionResult::Fail;
        }
        
        // Check for moderate regressions
        if regressions.iter().any(|r| r.severity == RegressionSeverity::Moderate) {
            return RegressionResult::PassWithWarnings;
        }
        
        // Check for minor regressions
        if regressions.iter().any(|r| r.severity == RegressionSeverity::Minor) {
            return RegressionResult::PassWithWarnings;
        }
        
        RegressionResult::Pass
    }
    
    /// Get recent test history for trend analysis
    pub fn get_test_history(&self) -> &Vec<RegressionTestResult, 16> {
        &self.test_history
    }
    
    /// Check for regression trends over multiple test runs
    pub fn analyze_regression_trends(&self) -> Option<RegressionTrend> {
        if self.test_history.len() < 3 {
            return None;
        }
        
        let recent_failures = self.test_history.iter()
            .rev()
            .take(5)
            .filter(|r| matches!(r.result, RegressionResult::Fail | RegressionResult::CriticalFail))
            .count();
        
        let trend = if recent_failures >= 3 {
            TrendDirection::Worsening
        } else if recent_failures <= 1 {
            TrendDirection::Stable
        } else {
            TrendDirection::Unstable
        };
        
        Some(RegressionTrend { trend, recent_failures })
    }
}

/// Regression trend analysis
#[derive(Debug, Clone, Copy)]
pub struct RegressionTrend {
    pub trend: TrendDirection,
    pub recent_failures: usize,
}

/// Direction of regression trend
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    /// Performance improving over time
    Improving,
    
    /// Performance stable
    Stable,
    
    /// Performance unstable with occasional regressions
    Unstable,
    
    /// Performance consistently worsening
    Worsening,
}

impl Default for RegressionTester {
    fn default() -> Self {
        Self::new()
    }
}