//! Performance Baseline Management - Phase 2 target validation
//!
//! Defines and validates performance baselines established for the ESP32-C3 IoT system.
//! Provides comparison capabilities and regression detection for architectural improvements.

use embassy_time::Duration;
use heapless::Vec;

use crate::timing::{TimingStatistics, TimingCategory};
use crate::memory::MemorySnapshot;

/// Performance baseline definition for comparison and validation
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// Baseline timing requirements by category
    pub timing_baselines: Vec<TimingBaseline, 10>,
    
    /// Memory usage baselines
    pub memory_baseline: MemoryBaseline,
    
    /// System-level performance requirements
    pub system_baseline: SystemBaseline,
    
    /// Baseline version identifier
    pub version: &'static str,
    
    /// Description of this baseline
    pub description: &'static str,
}

/// Timing performance baseline for a specific category
#[derive(Debug, Clone, Copy)]
pub struct TimingBaseline {
    /// Operation category
    pub category: TimingCategory,
    
    /// Target average time
    pub target_average: Duration,
    
    /// Maximum acceptable time
    pub max_acceptable: Duration,
    
    /// Warning threshold (percentage of max_acceptable)
    pub warning_threshold: f32,
    
    /// Whether this timing is critical for system operation
    pub is_critical: bool,
}

/// Memory usage baseline requirements
#[derive(Debug, Clone, Copy)]
pub struct MemoryBaseline {
    /// Target heap usage in bytes
    pub target_heap_bytes: usize,
    
    /// Maximum acceptable heap usage
    pub max_heap_bytes: usize,
    
    /// Target stack usage in bytes
    pub target_stack_bytes: usize,
    
    /// Maximum acceptable stack usage
    pub max_stack_bytes: usize,
    
    /// Flash usage target
    pub target_flash_bytes: usize,
    
    /// Maximum flash usage
    pub max_flash_bytes: usize,
}

/// System-level baseline requirements
#[derive(Debug, Clone, Copy)]
pub struct SystemBaseline {
    /// Boot time requirement
    pub max_boot_time: Duration,
    
    /// Maximum acceptable system cycle time
    pub max_cycle_time: Duration,
    
    /// Target system efficiency (0.0 to 1.0)
    pub target_efficiency: f32,
    
    /// Maximum number of performance alerts per hour
    pub max_alerts_per_hour: u32,
}

/// Result of comparing current performance against baseline
#[derive(Debug, Clone)]
pub struct BaselineComparison {
    /// Overall comparison status
    pub status: BaselineStatus,
    
    /// Timing comparison results
    pub timing_results: Vec<TimingComparisonResult, 10>,
    
    /// Memory comparison result
    pub memory_result: MemoryComparisonResult,
    
    /// System comparison result
    pub system_result: SystemComparisonResult,
    
    /// Performance grade (A+ to F)
    pub performance_grade: PerformanceGrade,
    
    /// Overall compliance percentage (0.0 to 100.0)
    pub compliance_percentage: f32,
}

/// Status of baseline comparison
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BaselineStatus {
    /// All metrics meet or exceed baseline requirements
    Excellent,
    
    /// Most metrics meet baseline, minor issues
    Good,
    
    /// Some metrics fail baseline requirements
    Acceptable,
    
    /// Many metrics fail baseline requirements
    Poor,
    
    /// Critical baseline failures
    Failed,
}

/// Result of timing comparison for a specific category
#[derive(Debug, Clone, Copy)]
pub struct TimingComparisonResult {
    /// Category being compared
    pub category: TimingCategory,
    
    /// Current measured average time
    pub measured_average: Duration,
    
    /// Baseline target time
    pub baseline_target: Duration,
    
    /// Comparison status
    pub status: ComparisonStatus,
    
    /// Performance ratio (measured / target)
    pub performance_ratio: f32,
    
    /// Whether this result affects overall grade
    pub affects_grade: bool,
}

/// Result of memory usage comparison
#[derive(Debug, Clone, Copy)]
pub struct MemoryComparisonResult {
    /// Heap usage comparison
    pub heap_status: ComparisonStatus,
    
    /// Stack usage comparison
    pub stack_status: ComparisonStatus,
    
    /// Flash usage comparison
    pub flash_status: ComparisonStatus,
    
    /// Overall memory status
    pub overall_status: ComparisonStatus,
    
    /// Heap usage ratio (measured / target)
    pub heap_ratio: f32,
    
    /// Stack usage ratio (measured / target)
    pub stack_ratio: f32,
}

/// Result of system-level comparison
#[derive(Debug, Clone, Copy)]
pub struct SystemComparisonResult {
    /// Boot time comparison
    pub boot_time_status: ComparisonStatus,
    
    /// Cycle time comparison
    pub cycle_time_status: ComparisonStatus,
    
    /// Efficiency comparison
    pub efficiency_status: ComparisonStatus,
    
    /// Overall system status
    pub overall_status: ComparisonStatus,
}

/// Status of individual metric comparison
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ComparisonStatus {
    /// Significantly better than baseline
    Excellent,
    
    /// Meets baseline requirements
    Pass,
    
    /// Approaching baseline limit
    Warning,
    
    /// Exceeds baseline limit
    Fail,
    
    /// No data available for comparison
    NoData,
}

/// Performance grade assignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceGrade {
    /// Exceptional performance (95%+ compliance)
    APlus,
    
    /// Excellent performance (90-94% compliance)
    A,
    
    /// Good performance (80-89% compliance)
    B,
    
    /// Acceptable performance (70-79% compliance)
    C,
    
    /// Poor performance (60-69% compliance)
    D,
    
    /// Failed performance (<60% compliance)
    F,
}

impl PerformanceBaseline {
    /// Create Phase 2 target baseline for ESP32-C3 IoT system
    pub fn phase_2_targets() -> Self {
        let mut timing_baselines = Vec::new();
        
        // Critical timing baselines
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::SensorReading,
            target_average: Duration::from_micros(400),
            max_acceptable: Duration::from_micros(500),
            warning_threshold: 0.8,
            is_critical: true,
        });
        
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::MqttPublish,
            target_average: Duration::from_millis(300),
            max_acceptable: Duration::from_millis(500),
            warning_threshold: 0.8,
            is_critical: true,
        });
        
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::NetworkOperation,
            target_average: Duration::from_millis(3000),
            max_acceptable: Duration::from_millis(5000),
            warning_threshold: 0.7,
            is_critical: false,
        });
        
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::SystemBoot,
            target_average: Duration::from_millis(2000),
            max_acceptable: Duration::from_millis(2500),
            warning_threshold: 0.8,
            is_critical: true,
        });
        
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::ConsoleCommand,
            target_average: Duration::from_millis(50),
            max_acceptable: Duration::from_millis(100),
            warning_threshold: 0.8,
            is_critical: false,
        });
        
        let memory_baseline = MemoryBaseline {
            target_heap_bytes: 48 * 1024,     // 48KB target
            max_heap_bytes: 52 * 1024,        // 52KB maximum
            target_stack_bytes: 4 * 1024,     // 4KB target
            max_stack_bytes: 8 * 1024,        // 8KB maximum
            target_flash_bytes: 256 * 1024,   // 256KB target
            max_flash_bytes: 512 * 1024,      // 512KB maximum
        };
        
        let system_baseline = SystemBaseline {
            max_boot_time: Duration::from_millis(2500),
            max_cycle_time: Duration::from_secs(35), // 30s sensor interval + 5s buffer
            target_efficiency: 0.85,
            max_alerts_per_hour: 5,
        };
        
        Self {
            timing_baselines,
            memory_baseline,
            system_baseline,
            version: "Phase2-v1.0",
            description: "ESP32-C3 IoT Environmental Monitoring System Phase 2 Performance Targets",
        }
    }
    
    /// Create baseline for Phase 0 (original system) for regression testing
    pub fn phase_0_baseline() -> Self {
        let mut timing_baselines = Vec::new();
        
        // Phase 0 measured baselines
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::SensorReading,
            target_average: Duration::from_micros(450),
            max_acceptable: Duration::from_micros(600),
            warning_threshold: 0.8,
            is_critical: true,
        });
        
        let _ = timing_baselines.push(TimingBaseline {
            category: TimingCategory::SystemBoot,
            target_average: Duration::from_millis(2300),
            max_acceptable: Duration::from_millis(3000),
            warning_threshold: 0.8,
            is_critical: true,
        });
        
        let memory_baseline = MemoryBaseline {
            target_heap_bytes: 48 * 1024,
            max_heap_bytes: 64 * 1024,
            target_stack_bytes: 4 * 1024,
            max_stack_bytes: 12 * 1024,
            target_flash_bytes: 200 * 1024,
            max_flash_bytes: 400 * 1024,
        };
        
        let system_baseline = SystemBaseline {
            max_boot_time: Duration::from_millis(3000),
            max_cycle_time: Duration::from_secs(35),
            target_efficiency: 0.75,
            max_alerts_per_hour: 10,
        };
        
        Self {
            timing_baselines,
            memory_baseline,
            system_baseline,
            version: "Phase0-Baseline",
            description: "Original ESP32-C3 IoT System Performance Baseline",
        }
    }
    
    /// Compare current performance against this baseline
    pub fn compare_current_performance(
        &self,
        timing_stats: &TimingStatistics,
        memory_snapshot: &MemorySnapshot,
    ) -> BaselineComparison {
        let timing_results = self.compare_timing_performance(timing_stats);
        let memory_result = self.compare_memory_performance(memory_snapshot);
        let system_result = self.compare_system_performance(timing_stats, memory_snapshot);
        
        let compliance_percentage = self.calculate_compliance_percentage(
            &timing_results,
            &memory_result,
            &system_result,
        );
        
        let performance_grade = Self::calculate_performance_grade(compliance_percentage);
        let status = Self::determine_overall_status(&timing_results, &memory_result, &system_result);
        
        BaselineComparison {
            status,
            timing_results,
            memory_result,
            system_result,
            performance_grade,
            compliance_percentage,
        }
    }
    
    /// Compare timing performance against baseline
    fn compare_timing_performance(&self, timing_stats: &TimingStatistics) -> Vec<TimingComparisonResult, 10> {
        let mut results = Vec::new();
        
        for baseline in &self.timing_baselines {
            if let Some(measured_average) = timing_stats.get_average_time(baseline.category) {
                let performance_ratio = measured_average.as_micros() as f32 / 
                                      baseline.target_average.as_micros() as f32;
                
                let status = if measured_average <= baseline.target_average {
                    ComparisonStatus::Excellent
                } else if measured_average <= Duration::from_micros(
                    (baseline.max_acceptable.as_micros() as f32 * baseline.warning_threshold) as u64
                ) {
                    ComparisonStatus::Pass
                } else if measured_average <= baseline.max_acceptable {
                    ComparisonStatus::Warning
                } else {
                    ComparisonStatus::Fail
                };
                
                let _ = results.push(TimingComparisonResult {
                    category: baseline.category,
                    measured_average,
                    baseline_target: baseline.target_average,
                    status,
                    performance_ratio,
                    affects_grade: baseline.is_critical,
                });
            } else {
                let _ = results.push(TimingComparisonResult {
                    category: baseline.category,
                    measured_average: Duration::from_millis(0),
                    baseline_target: baseline.target_average,
                    status: ComparisonStatus::NoData,
                    performance_ratio: 0.0,
                    affects_grade: baseline.is_critical,
                });
            }
        }
        
        results
    }
    
    /// Compare memory performance against baseline
    fn compare_memory_performance(&self, memory_snapshot: &MemorySnapshot) -> MemoryComparisonResult {
        let heap_ratio = memory_snapshot.heap_used as f32 / self.memory_baseline.target_heap_bytes as f32;
        let stack_ratio = memory_snapshot.stack_used as f32 / self.memory_baseline.target_stack_bytes as f32;
        
        let heap_status = if memory_snapshot.heap_used <= self.memory_baseline.target_heap_bytes {
            ComparisonStatus::Excellent
        } else if memory_snapshot.heap_used <= self.memory_baseline.max_heap_bytes {
            ComparisonStatus::Warning
        } else {
            ComparisonStatus::Fail
        };
        
        let stack_status = if memory_snapshot.stack_used <= self.memory_baseline.target_stack_bytes {
            ComparisonStatus::Excellent
        } else if memory_snapshot.stack_used <= self.memory_baseline.max_stack_bytes {
            ComparisonStatus::Warning
        } else {
            ComparisonStatus::Fail
        };
        
        let flash_status = if memory_snapshot.flash_used <= self.memory_baseline.target_flash_bytes {
            ComparisonStatus::Excellent
        } else if memory_snapshot.flash_used <= self.memory_baseline.max_flash_bytes {
            ComparisonStatus::Warning
        } else {
            ComparisonStatus::Fail
        };
        
        let overall_status = match (heap_status, stack_status, flash_status) {
            (ComparisonStatus::Excellent, ComparisonStatus::Excellent, ComparisonStatus::Excellent) => ComparisonStatus::Excellent,
            (s1, s2, s3) if [s1, s2, s3].iter().any(|&s| s == ComparisonStatus::Fail) => ComparisonStatus::Fail,
            (s1, s2, s3) if [s1, s2, s3].iter().any(|&s| s == ComparisonStatus::Warning) => ComparisonStatus::Warning,
            _ => ComparisonStatus::Pass,
        };
        
        MemoryComparisonResult {
            heap_status,
            stack_status,
            flash_status,
            overall_status,
            heap_ratio,
            stack_ratio,
        }
    }
    
    /// Compare system-level performance against baseline
    fn compare_system_performance(
        &self,
        timing_stats: &TimingStatistics,
        _memory_snapshot: &MemorySnapshot,
    ) -> SystemComparisonResult {
        // Boot time comparison
        let boot_time_status = if let Some(boot_time) = timing_stats.get_average_time(TimingCategory::SystemBoot) {
            if boot_time <= self.system_baseline.max_boot_time {
                ComparisonStatus::Pass
            } else {
                ComparisonStatus::Fail
            }
        } else {
            ComparisonStatus::NoData
        };
        
        // Cycle time comparison (use sensor reading as proxy)
        let cycle_time_status = if let Some(cycle_time) = timing_stats.get_average_time(TimingCategory::SystemCycle) {
            if cycle_time <= self.system_baseline.max_cycle_time {
                ComparisonStatus::Pass
            } else {
                ComparisonStatus::Fail
            }
        } else {
            ComparisonStatus::NoData
        };
        
        // Efficiency comparison (simplified)
        let efficiency_status = if timing_stats.get_overall_stats().efficiency_ratio >= self.system_baseline.target_efficiency {
            ComparisonStatus::Pass
        } else {
            ComparisonStatus::Warning
        };
        
        let overall_status = match (boot_time_status, cycle_time_status, efficiency_status) {
            (ComparisonStatus::Pass, ComparisonStatus::Pass, ComparisonStatus::Pass) => ComparisonStatus::Pass,
            (s1, s2, s3) if [s1, s2, s3].iter().any(|&s| s == ComparisonStatus::Fail) => ComparisonStatus::Fail,
            _ => ComparisonStatus::Warning,
        };
        
        SystemComparisonResult {
            boot_time_status,
            cycle_time_status,
            efficiency_status,
            overall_status,
        }
    }
    
    /// Calculate overall compliance percentage
    fn calculate_compliance_percentage(
        &self,
        timing_results: &Vec<TimingComparisonResult, 10>,
        memory_result: &MemoryComparisonResult,
        system_result: &SystemComparisonResult,
    ) -> f32 {
        let mut total_score = 0.0;
        let mut max_score = 0.0;
        
        // Weight timing results
        for result in timing_results {
            let weight = if result.affects_grade { 2.0 } else { 1.0 };
            max_score += weight;
            
            let score = match result.status {
                ComparisonStatus::Excellent => weight,
                ComparisonStatus::Pass => weight * 0.9,
                ComparisonStatus::Warning => weight * 0.7,
                ComparisonStatus::Fail => 0.0,
                ComparisonStatus::NoData => weight * 0.5, // Partial credit
            };
            total_score += score;
        }
        
        // Weight memory results
        max_score += 3.0; // heap, stack, flash
        total_score += match memory_result.overall_status {
            ComparisonStatus::Excellent => 3.0,
            ComparisonStatus::Pass => 2.7,
            ComparisonStatus::Warning => 2.1,
            ComparisonStatus::Fail => 0.0,
            ComparisonStatus::NoData => 1.5,
        };
        
        // Weight system results
        max_score += 2.0;
        total_score += match system_result.overall_status {
            ComparisonStatus::Pass => 2.0,
            ComparisonStatus::Warning => 1.4,
            ComparisonStatus::Fail => 0.0,
            ComparisonStatus::NoData => 1.0,
            _ => 1.8,
        };
        
        if max_score > 0.0 {
            let percentage = (total_score / max_score * 100.0) as f32;
            percentage.min(100.0)
        } else {
            0.0
        }
    }
    
    /// Calculate performance grade from compliance percentage
    fn calculate_performance_grade(compliance_percentage: f32) -> PerformanceGrade {
        match compliance_percentage {
            p if p >= 95.0 => PerformanceGrade::APlus,
            p if p >= 90.0 => PerformanceGrade::A,
            p if p >= 80.0 => PerformanceGrade::B,
            p if p >= 70.0 => PerformanceGrade::C,
            p if p >= 60.0 => PerformanceGrade::D,
            _ => PerformanceGrade::F,
        }
    }
    
    /// Determine overall baseline status
    fn determine_overall_status(
        timing_results: &Vec<TimingComparisonResult, 10>,
        memory_result: &MemoryComparisonResult,
        system_result: &SystemComparisonResult,
    ) -> BaselineStatus {
        let has_critical_failures = timing_results.iter().any(|r| 
            r.affects_grade && r.status == ComparisonStatus::Fail
        ) || memory_result.overall_status == ComparisonStatus::Fail
          || system_result.overall_status == ComparisonStatus::Fail;
        
        if has_critical_failures {
            return BaselineStatus::Failed;
        }
        
        let warning_count = timing_results.iter().filter(|r| 
            r.status == ComparisonStatus::Warning
        ).count() + 
        if memory_result.overall_status == ComparisonStatus::Warning { 1 } else { 0 } +
        if system_result.overall_status == ComparisonStatus::Warning { 1 } else { 0 };
        
        let excellent_count = timing_results.iter().filter(|r| 
            r.status == ComparisonStatus::Excellent
        ).count() + 
        if memory_result.overall_status == ComparisonStatus::Excellent { 1 } else { 0 };
        
        match (warning_count, excellent_count) {
            (0, n) if n >= 5 => BaselineStatus::Excellent,
            (w, _) if w <= 1 => BaselineStatus::Good,
            (w, _) if w <= 3 => BaselineStatus::Acceptable,
            _ => BaselineStatus::Poor,
        }
    }
}