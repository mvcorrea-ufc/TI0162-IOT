//! Performance Analysis - Advanced performance data analysis and insights
//!
//! Provides sophisticated analysis capabilities for performance data including
//! trend analysis, pattern recognition, and optimization recommendations.

use embassy_time::{Duration, Instant};
use heapless::{Vec, String};
use core::fmt::Write;

use crate::timing::{TimingStatistics, TimingCategory};
use crate::memory::MemorySnapshot;

/// Comprehensive performance analyzer
pub struct PerformanceAnalyzer {
    /// Historical performance data for trend analysis
    historical_data: Vec<PerformanceDataPoint, 32>,
    
    /// Analysis configuration
    config: AnalysisConfig,
    
    /// Last analysis timestamp
    last_analysis: Option<Instant>,
}

/// Single performance data point for historical analysis
#[derive(Debug, Clone, Copy)]
pub struct PerformanceDataPoint {
    /// Timestamp of this data point
    pub timestamp: Instant,
    
    /// Average sensor reading time
    pub sensor_time_us: u32,
    
    /// Heap usage in bytes
    pub heap_usage: usize,
    
    /// Stack usage in bytes
    pub stack_usage: usize,
    
    /// Overall system efficiency (0.0 to 1.0)
    pub efficiency: f32,
    
    /// Number of performance alerts
    pub alert_count: u8,
}

/// Configuration for performance analysis
#[derive(Debug, Clone, Copy)]
pub struct AnalysisConfig {
    /// Minimum data points required for trend analysis
    pub min_trend_points: usize,
    
    /// Trend analysis window in seconds
    pub trend_window_secs: u64,
    
    /// Sensitivity for performance change detection (0.0 to 1.0)
    pub change_sensitivity: f32,
    
    /// Enable detailed pattern recognition
    pub pattern_recognition: bool,
    
    /// Enable predictive analysis
    pub predictive_analysis: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            min_trend_points: 5,
            trend_window_secs: 300, // 5 minutes
            change_sensitivity: 0.1, // 10% change threshold
            pattern_recognition: true,
            predictive_analysis: true,
        }
    }
}

/// Performance trend analysis results
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// Overall performance trend direction
    pub overall_trend: PerformanceTrend,
    
    /// Timing trend analysis
    pub timing_trends: Vec<TimingTrend, 8>,
    
    /// Memory usage trends
    pub memory_trends: MemoryTrends,
    
    /// System efficiency trend
    pub efficiency_trend: EfficiencyTrend,
    
    /// Confidence level in analysis (0.0 to 1.0)
    pub confidence: f32,
    
    /// Time period analyzed
    pub analysis_period: Duration,
}

/// Performance trend direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceTrend {
    /// Performance is improving significantly
    Improving,
    
    /// Performance is stable
    Stable,
    
    /// Performance is slowly degrading
    Degrading,
    
    /// Performance is rapidly degrading
    Critical,
    
    /// Insufficient data for analysis
    Unknown,
}

/// Timing trend for a specific category
#[derive(Debug, Clone, Copy)]
pub struct TimingTrend {
    /// Operation category
    pub category: TimingCategory,
    
    /// Trend direction
    pub trend: PerformanceTrend,
    
    /// Rate of change (microseconds per second)
    pub change_rate_us_per_sec: f32,
    
    /// Statistical significance of trend
    pub significance: f32,
}

/// Memory usage trend analysis
#[derive(Debug, Clone, Copy)]
pub struct MemoryTrends {
    /// Heap usage trend
    pub heap_trend: PerformanceTrend,
    
    /// Stack usage trend
    pub stack_trend: PerformanceTrend,
    
    /// Memory efficiency trend
    pub efficiency_trend: PerformanceTrend,
    
    /// Fragmentation trend
    pub fragmentation_trend: PerformanceTrend,
    
    /// Heap growth rate (bytes per second)
    pub heap_growth_rate: f32,
    
    /// Stack growth rate (bytes per second)
    pub stack_growth_rate: f32,
}

/// System efficiency trend analysis
#[derive(Debug, Clone, Copy)]
pub struct EfficiencyTrend {
    /// Overall efficiency trend
    pub trend: PerformanceTrend,
    
    /// Efficiency change rate (percentage per second)
    pub change_rate: f32,
    
    /// Current efficiency score (0.0 to 1.0)
    pub current_efficiency: f32,
    
    /// Predicted efficiency in 1 hour
    pub predicted_efficiency_1h: f32,
}

/// Performance pattern recognition results
#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    /// Detected performance patterns
    pub patterns: Vec<PerformancePattern, 8>,
    
    /// Pattern confidence scores
    pub pattern_confidence: Vec<f32, 8>,
    
    /// Recommendations based on patterns
    pub recommendations: Vec<PatternRecommendation, 8>,
}

/// Recognized performance patterns
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformancePattern {
    /// Memory leak pattern detected
    MemoryLeak,
    
    /// Performance degradation over time
    GradualDegradation,
    
    /// Periodic performance spikes
    PeriodicSpikes,
    
    /// Resource exhaustion pattern
    ResourceExhaustion,
    
    /// Task scheduling issues
    SchedulingProblems,
    
    /// Network-related performance issues
    NetworkBottleneck,
    
    /// Sensor reading optimization opportunity
    SensorOptimization,
    
    /// System initialization overhead
    InitializationOverhead,
}

/// Recommendations based on pattern analysis
#[derive(Debug, Clone, Copy)]
pub enum PatternRecommendation {
    /// Investigate memory allocation patterns
    InvestigateMemoryAllocation,
    
    /// Optimize sensor reading frequency
    OptimizeSensorFrequency,
    
    /// Review task priority configuration
    ReviewTaskPriorities,
    
    /// Implement caching for network operations
    ImplementNetworkCaching,
    
    /// Add memory pooling
    AddMemoryPooling,
    
    /// Optimize Embassy task scheduling
    OptimizeTaskScheduling,
    
    /// Review interrupt handler efficiency
    ReviewInterruptHandlers,
    
    /// Implement lazy initialization
    ImplementLazyInit,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new() -> Self {
        Self::with_config(AnalysisConfig::default())
    }
    
    /// Create analyzer with custom configuration
    pub fn with_config(config: AnalysisConfig) -> Self {
        Self {
            historical_data: Vec::new(),
            config,
            last_analysis: None,
        }
    }
    
    /// Add a performance data point for analysis
    pub fn add_data_point(
        &mut self,
        timing_stats: &TimingStatistics,
        memory_snapshot: &MemorySnapshot,
        alert_count: u8,
    ) {
        let sensor_time_us = timing_stats.get_average_time(TimingCategory::SensorReading)
            .map(|d| d.as_micros() as u32)
            .unwrap_or(0);
        
        let data_point = PerformanceDataPoint {
            timestamp: Instant::now(),
            sensor_time_us,
            heap_usage: memory_snapshot.heap_used,
            stack_usage: memory_snapshot.stack_used,
            efficiency: timing_stats.get_overall_stats().efficiency_ratio,
            alert_count,
        };
        
        // Add data point, removing oldest if at capacity
        if self.historical_data.is_full() {
            self.historical_data.remove(0);
        }
        let _ = self.historical_data.push(data_point);
    }
    
    /// Perform comprehensive trend analysis
    pub fn analyze_trends(&mut self) -> TrendAnalysis {
        self.last_analysis = Some(Instant::now());
        
        if self.historical_data.len() < self.config.min_trend_points {
            return TrendAnalysis {
                overall_trend: PerformanceTrend::Unknown,
                timing_trends: Vec::new(),
                memory_trends: MemoryTrends {
                    heap_trend: PerformanceTrend::Unknown,
                    stack_trend: PerformanceTrend::Unknown,
                    efficiency_trend: PerformanceTrend::Unknown,
                    fragmentation_trend: PerformanceTrend::Unknown,
                    heap_growth_rate: 0.0,
                    stack_growth_rate: 0.0,
                },
                efficiency_trend: EfficiencyTrend {
                    trend: PerformanceTrend::Unknown,
                    change_rate: 0.0,
                    current_efficiency: 0.0,
                    predicted_efficiency_1h: 0.0,
                },
                confidence: 0.0,
                analysis_period: Duration::from_secs(0),
            };
        }
        
        let analysis_period = self.calculate_analysis_period();
        let timing_trends = self.analyze_timing_trends();
        let memory_trends = self.analyze_memory_trends();
        let efficiency_trend = self.analyze_efficiency_trend();
        let overall_trend = self.determine_overall_trend(&timing_trends, &memory_trends, &efficiency_trend);
        let confidence = self.calculate_analysis_confidence();
        
        TrendAnalysis {
            overall_trend,
            timing_trends,
            memory_trends,
            efficiency_trend,
            confidence,
            analysis_period,
        }
    }
    
    /// Perform pattern recognition analysis
    pub fn analyze_patterns(&self) -> PatternAnalysis {
        if !self.config.pattern_recognition || self.historical_data.len() < 5 {
            return PatternAnalysis {
                patterns: Vec::new(),
                pattern_confidence: Vec::new(),
                recommendations: Vec::new(),
            };
        }
        
        let mut patterns = Vec::new();
        let mut pattern_confidence = Vec::new();
        let mut recommendations = Vec::new();
        
        // Detect memory leak pattern
        if let Some((confidence, _)) = self.detect_memory_leak_pattern() {
            if confidence > 0.7 {
                let _ = patterns.push(PerformancePattern::MemoryLeak);
                let _ = pattern_confidence.push(confidence);
                let _ = recommendations.push(PatternRecommendation::InvestigateMemoryAllocation);
            }
        }
        
        // Detect gradual degradation
        if let Some((confidence, _)) = self.detect_gradual_degradation() {
            if confidence > 0.6 {
                let _ = patterns.push(PerformancePattern::GradualDegradation);
                let _ = pattern_confidence.push(confidence);
                let _ = recommendations.push(PatternRecommendation::OptimizeSensorFrequency);
            }
        }
        
        // Detect periodic spikes
        if let Some((confidence, _)) = self.detect_periodic_spikes() {
            if confidence > 0.8 {
                let _ = patterns.push(PerformancePattern::PeriodicSpikes);
                let _ = pattern_confidence.push(confidence);
                let _ = recommendations.push(PatternRecommendation::ReviewTaskPriorities);
            }
        }
        
        PatternAnalysis {
            patterns,
            pattern_confidence,
            recommendations,
        }
    }
    
    /// Calculate analysis period from historical data
    fn calculate_analysis_period(&self) -> Duration {
        if self.historical_data.len() < 2 {
            return Duration::from_secs(0);
        }
        
        let first = &self.historical_data[0];
        let last = &self.historical_data[self.historical_data.len() - 1];
        last.timestamp.duration_since(first.timestamp)
    }
    
    /// Analyze timing trends across categories
    fn analyze_timing_trends(&self) -> Vec<TimingTrend, 8> {
        let mut trends = Vec::new();
        
        // Analyze sensor timing trend
        let sensor_trend = self.analyze_sensor_timing_trend();
        if let Some(trend) = sensor_trend {
            let _ = trends.push(trend);
        }
        
        trends
    }
    
    /// Analyze sensor timing trend specifically
    fn analyze_sensor_timing_trend(&self) -> Option<TimingTrend> {
        if self.historical_data.len() < 3 {
            return None;
        }
        
        let values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.sensor_time_us as f32)
            .collect();
        
        let (slope, significance) = self.calculate_linear_trend(&values)?;
        let change_rate_us_per_sec = slope; // Slope is already in desired units
        
        let trend = if significance > 0.8 {
            if slope > self.config.change_sensitivity * 100.0 {
                if slope > self.config.change_sensitivity * 300.0 {
                    PerformanceTrend::Critical
                } else {
                    PerformanceTrend::Degrading
                }
            } else if slope < -self.config.change_sensitivity * 50.0 {
                PerformanceTrend::Improving
            } else {
                PerformanceTrend::Stable
            }
        } else {
            PerformanceTrend::Unknown
        };
        
        Some(TimingTrend {
            category: TimingCategory::SensorReading,
            trend,
            change_rate_us_per_sec,
            significance,
        })
    }
    
    /// Analyze memory usage trends
    fn analyze_memory_trends(&self) -> MemoryTrends {
        if self.historical_data.len() < 3 {
            return MemoryTrends {
                heap_trend: PerformanceTrend::Unknown,
                stack_trend: PerformanceTrend::Unknown,
                efficiency_trend: PerformanceTrend::Unknown,
                fragmentation_trend: PerformanceTrend::Unknown,
                heap_growth_rate: 0.0,
                stack_growth_rate: 0.0,
            };
        }
        
        let heap_values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.heap_usage as f32)
            .collect();
        
        let stack_values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.stack_usage as f32)
            .collect();
        
        let (heap_slope, heap_significance) = self.calculate_linear_trend(&heap_values).unwrap_or((0.0, 0.0));
        let (stack_slope, stack_significance) = self.calculate_linear_trend(&stack_values).unwrap_or((0.0, 0.0));
        
        let heap_trend = self.slope_to_trend(heap_slope, heap_significance, 100.0); // 100 bytes threshold
        let stack_trend = self.slope_to_trend(stack_slope, stack_significance, 50.0); // 50 bytes threshold
        
        MemoryTrends {
            heap_trend,
            stack_trend,
            efficiency_trend: PerformanceTrend::Stable, // Simplified
            fragmentation_trend: PerformanceTrend::Stable, // Simplified
            heap_growth_rate: heap_slope,
            stack_growth_rate: stack_slope,
        }
    }
    
    /// Analyze system efficiency trends
    fn analyze_efficiency_trend(&self) -> EfficiencyTrend {
        if self.historical_data.len() < 3 {
            return EfficiencyTrend {
                trend: PerformanceTrend::Unknown,
                change_rate: 0.0,
                current_efficiency: 0.0,
                predicted_efficiency_1h: 0.0,
            };
        }
        
        let efficiency_values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.efficiency)
            .collect();
        
        let (slope, significance) = self.calculate_linear_trend(&efficiency_values).unwrap_or((0.0, 0.0));
        
        let trend = self.slope_to_trend(slope, significance, 0.01); // 1% threshold
        let current_efficiency = efficiency_values.last().copied().unwrap_or(0.0);
        
        // Predict efficiency in 1 hour (3600 seconds)
        let predicted_efficiency_1h = (current_efficiency + slope * 3600.0).max(0.0).min(1.0);
        
        EfficiencyTrend {
            trend,
            change_rate: slope,
            current_efficiency,
            predicted_efficiency_1h,
        }
    }
    
    /// Convert slope and significance to performance trend
    fn slope_to_trend(&self, slope: f32, significance: f32, threshold: f32) -> PerformanceTrend {
        if significance < 0.5 {
            return PerformanceTrend::Unknown;
        }
        
        if slope > threshold * self.config.change_sensitivity {
            if slope > threshold * self.config.change_sensitivity * 3.0 {
                PerformanceTrend::Critical
            } else {
                PerformanceTrend::Degrading
            }
        } else if slope < -threshold * self.config.change_sensitivity {
            PerformanceTrend::Improving
        } else {
            PerformanceTrend::Stable
        }
    }
    
    /// Calculate linear trend using least squares regression
    fn calculate_linear_trend(&self, values: &Vec<f32, 32>) -> Option<(f32, f32)> {
        if values.len() < 3 {
            return None;
        }
        
        let n = values.len() as f32;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let x = i as f32;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-6 {
            return None;
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        
        // Calculate R-squared for significance
        let mean_y = sum_y / n;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;
        
        for (i, &y) in values.iter().enumerate() {
            let x = i as f32;
            let y_pred = slope * x + (sum_y - slope * sum_x) / n;
            ss_res += (y - y_pred) * (y - y_pred);
            ss_tot += (y - mean_y) * (y - mean_y);
        }
        
        let r_squared = if ss_tot > 1e-6 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };
        
        Some((slope, r_squared.max(0.0)))
    }
    
    /// Determine overall performance trend
    fn determine_overall_trend(
        &self,
        timing_trends: &Vec<TimingTrend, 8>,
        memory_trends: &MemoryTrends,
        efficiency_trend: &EfficiencyTrend,
    ) -> PerformanceTrend {
        let mut critical_count = 0;
        let mut degrading_count = 0;
        let mut improving_count = 0;
        let mut stable_count = 0;
        
        // Count timing trends
        for trend in timing_trends {
            match trend.trend {
                PerformanceTrend::Critical => critical_count += 1,
                PerformanceTrend::Degrading => degrading_count += 1,
                PerformanceTrend::Improving => improving_count += 1,
                PerformanceTrend::Stable => stable_count += 1,
                _ => {}
            }
        }
        
        // Count memory trends
        match memory_trends.heap_trend {
            PerformanceTrend::Critical => critical_count += 1,
            PerformanceTrend::Degrading => degrading_count += 1,
            PerformanceTrend::Improving => improving_count += 1,
            PerformanceTrend::Stable => stable_count += 1,
            _ => {}
        }
        
        // Count efficiency trend
        match efficiency_trend.trend {
            PerformanceTrend::Critical => critical_count += 1,
            PerformanceTrend::Degrading => degrading_count += 1,
            PerformanceTrend::Improving => improving_count += 1,
            PerformanceTrend::Stable => stable_count += 1,
            _ => {}
        }
        
        // Determine overall trend
        if critical_count > 0 {
            PerformanceTrend::Critical
        } else if degrading_count > improving_count {
            PerformanceTrend::Degrading
        } else if improving_count > degrading_count {
            PerformanceTrend::Improving
        } else if stable_count > 0 {
            PerformanceTrend::Stable
        } else {
            PerformanceTrend::Unknown
        }
    }
    
    /// Calculate confidence in analysis results
    fn calculate_analysis_confidence(&self) -> f32 {
        if self.historical_data.len() < self.config.min_trend_points {
            return 0.0;
        }
        
        let data_quality = (self.historical_data.len() as f32 / 32.0).min(1.0);
        let time_span = self.calculate_analysis_period().as_secs() as f32;
        let time_quality = (time_span / self.config.trend_window_secs as f32).min(1.0);
        
        (data_quality * 0.6 + time_quality * 0.4).min(1.0)
    }
    
    /// Detect memory leak pattern
    fn detect_memory_leak_pattern(&self) -> Option<(f32, String<64>)> {
        if self.historical_data.len() < 5 {
            return None;
        }
        
        let heap_values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.heap_usage as f32)
            .collect();
        
        let (slope, significance) = self.calculate_linear_trend(&heap_values)?;
        
        // Memory leak if consistent upward trend
        if slope > 10.0 && significance > 0.7 {
            let confidence = significance * (slope / 50.0).min(1.0);
            let mut description = String::new();
            let _ = write!(description, "Heap growth: {:.1} bytes/sample", slope);
            Some((confidence, description))
        } else {
            None
        }
    }
    
    /// Detect gradual performance degradation
    fn detect_gradual_degradation(&self) -> Option<(f32, String<64>)> {
        if self.historical_data.len() < 5 {
            return None;
        }
        
        let sensor_values: Vec<f32, 32> = self.historical_data
            .iter()
            .map(|d| d.sensor_time_us as f32)
            .collect();
        
        let (slope, significance) = self.calculate_linear_trend(&sensor_values)?;
        
        // Degradation if sensor times increasing
        if slope > 5.0 && significance > 0.6 {
            let confidence = significance * (slope / 20.0).min(1.0);
            let mut description = String::new();
            let _ = write!(description, "Sensor time increase: {:.1} μs/sample", slope);
            Some((confidence, description))
        } else {
            None
        }
    }
    
    /// Detect periodic performance spikes
    fn detect_periodic_spikes(&self) -> Option<(f32, String<64>)> {
        if self.historical_data.len() < 8 {
            return None;
        }
        
        // Simple spike detection: look for values significantly above average
        let sensor_values: Vec<u32, 32> = self.historical_data
            .iter()
            .map(|d| d.sensor_time_us)
            .collect();
        
        let avg = sensor_values.iter().map(|&x| x as f32).sum::<f32>() / sensor_values.len() as f32;
        let spike_threshold = avg * 1.5;
        
        let spike_count = sensor_values.iter().filter(|&&x| x as f32 > spike_threshold).count();
        let spike_ratio = spike_count as f32 / sensor_values.len() as f32;
        
        if spike_ratio > 0.2 && spike_ratio < 0.8 {
            // Periodic if 20-80% of samples are spikes (indicating pattern)
            let confidence = if spike_ratio > 0.3 { 0.8 } else { 0.6 };
            let mut description = String::new();
            let _ = write!(description, "Spike ratio: {:.1}%", spike_ratio * 100.0);
            Some((confidence, description))
        } else {
            None
        }
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}