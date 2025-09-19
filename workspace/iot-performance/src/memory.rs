//! Memory Usage Analysis - Comprehensive memory tracking and optimization
//!
//! Provides detailed memory usage tracking including heap, stack, and flash
//! analysis specifically designed for ESP32-C3 embedded environments.

use embassy_time::Instant;
use heapless::Vec;
use core::ptr;

/// Memory regions that can be monitored
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    /// Heap memory allocation tracking
    Heap,
    
    /// Stack usage monitoring
    Stack,
    
    /// Flash memory usage
    Flash,
    
    /// Static memory allocations
    Static,
    
    /// DMA buffer usage
    DmaBuffers,
    
    /// Embassy task stacks
    TaskStacks,
}

/// Snapshot of memory usage at a specific point in time
#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    /// Heap memory currently in use (bytes)
    pub heap_used: usize,
    
    /// Peak heap usage since last reset (bytes)
    pub heap_peak: usize,
    
    /// Current stack usage (bytes)
    pub stack_used: usize,
    
    /// Peak stack usage since last reset (bytes)
    pub stack_peak: usize,
    
    /// Flash memory used by application (bytes)
    pub flash_used: usize,
    
    /// Total available flash memory (bytes)
    pub flash_total: usize,
    
    /// Static memory allocations (bytes)
    pub static_used: usize,
    
    /// DMA buffer allocations (bytes)
    pub dma_buffers: usize,
    
    /// Free heap memory (bytes)
    pub heap_free: usize,
    
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
}

/// Memory usage tracker with historical analysis
pub struct MemoryTracker {
    /// Historical snapshots for trend analysis
    snapshots: Vec<MemorySnapshot, 16>,
    
    /// Current memory state
    current_snapshot: MemorySnapshot,
    
    /// Peak values since tracker creation
    all_time_peaks: MemoryPeaks,
    
    /// Memory allocation tracking (if enabled)
    allocation_tracking: bool,
    
    /// Fragmentation analysis data
    fragmentation_data: FragmentationData,
}

/// Peak memory usage values
#[derive(Debug, Clone, Copy)]
pub struct MemoryPeaks {
    /// Peak heap usage ever recorded
    pub heap_peak: usize,
    
    /// Peak stack usage ever recorded
    pub stack_peak: usize,
    
    /// Maximum fragmentation percentage
    pub max_fragmentation: f32,
    
    /// Timestamp of peak heap usage
    pub heap_peak_time: Instant,
    
    /// Timestamp of peak stack usage
    pub stack_peak_time: Instant,
}

/// Memory fragmentation analysis
#[derive(Debug, Clone, Copy)]
pub struct FragmentationData {
    /// Largest contiguous free block size
    pub largest_free_block: usize,
    
    /// Number of free blocks
    pub free_block_count: u32,
    
    /// Fragmentation percentage (0.0 to 100.0)
    pub fragmentation_percent: f32,
    
    /// Allocation failure count
    pub allocation_failures: u32,
}

/// Comprehensive memory analysis results
#[derive(Debug, Clone)]
pub struct MemoryAnalysis {
    /// Current memory usage summary
    pub current_usage: MemorySnapshot,
    
    /// Memory usage trends
    pub trends: MemoryTrends,
    
    /// Fragmentation analysis
    pub fragmentation: FragmentationData,
    
    /// Performance impact assessment
    pub performance_impact: MemoryPerformanceImpact,
    
    /// Optimization recommendations
    pub recommendations: Vec<MemoryRecommendation, 8>,
}

/// Memory usage trend analysis
#[derive(Debug, Clone, Copy)]
pub struct MemoryTrends {
    /// Heap usage trend (bytes per second)
    pub heap_trend_bps: f32,
    
    /// Stack usage trend (bytes per second)
    pub stack_trend_bps: f32,
    
    /// Allocation frequency trend (allocations per second)
    pub allocation_frequency: f32,
    
    /// Memory efficiency trend (0.0 to 1.0)
    pub efficiency_trend: f32,
}

/// Assessment of memory usage impact on system performance
#[derive(Debug, Clone, Copy)]
pub struct MemoryPerformanceImpact {
    /// Estimated allocation overhead (microseconds per operation)
    pub allocation_overhead_us: u32,
    
    /// Fragmentation impact on allocation speed (0.0 to 1.0)
    pub fragmentation_impact: f32,
    
    /// Memory pressure level (0.0 to 1.0)
    pub memory_pressure: f32,
    
    /// Risk of out-of-memory condition (0.0 to 1.0)
    pub oom_risk: f32,
}

/// Memory optimization recommendations
#[derive(Debug, Clone, Copy)]
pub enum MemoryRecommendation {
    /// Reduce heap allocations in hot paths
    ReduceHeapAllocations,
    
    /// Implement memory pooling for frequent allocations
    UseMemoryPools,
    
    /// Optimize stack usage in recursive functions
    OptimizeStackUsage,
    
    /// Defragment heap memory
    DefragmentHeap,
    
    /// Increase heap size allocation
    IncreaseHeapSize,
    
    /// Use static allocations instead of dynamic
    PreferStaticAllocation,
    
    /// Implement circular buffers for streaming data
    UseCircularBuffers,
    
    /// Review and optimize large data structures
    OptimizeDataStructures,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        let initial_snapshot = Self::take_memory_snapshot();
        
        Self {
            snapshots: Vec::new(),
            current_snapshot: initial_snapshot,
            all_time_peaks: MemoryPeaks {
                heap_peak: initial_snapshot.heap_used,
                stack_peak: initial_snapshot.stack_used,
                max_fragmentation: 0.0,
                heap_peak_time: initial_snapshot.timestamp,
                stack_peak_time: initial_snapshot.timestamp,
            },
            allocation_tracking: true,
            fragmentation_data: FragmentationData {
                largest_free_block: 0,
                free_block_count: 0,
                fragmentation_percent: 0.0,
                allocation_failures: 0,
            },
        }
    }
    
    /// Record a memory snapshot for a specific region
    pub fn record_snapshot(&mut self, region: MemoryRegion, usage: usize) -> Result<(), &'static str> {
        let snapshot = Self::take_memory_snapshot();
        
        // Update current snapshot based on region
        match region {
            MemoryRegion::Heap => {
                self.current_snapshot.heap_used = usage;
                if usage > self.current_snapshot.heap_peak {
                    self.current_snapshot.heap_peak = usage;
                }
                if usage > self.all_time_peaks.heap_peak {
                    self.all_time_peaks.heap_peak = usage;
                    self.all_time_peaks.heap_peak_time = snapshot.timestamp;
                }
            }
            MemoryRegion::Stack => {
                self.current_snapshot.stack_used = usage;
                if usage > self.current_snapshot.stack_peak {
                    self.current_snapshot.stack_peak = usage;
                }
                if usage > self.all_time_peaks.stack_peak {
                    self.all_time_peaks.stack_peak = usage;
                    self.all_time_peaks.stack_peak_time = snapshot.timestamp;
                }
            }
            MemoryRegion::Flash => self.current_snapshot.flash_used = usage,
            MemoryRegion::Static => self.current_snapshot.static_used = usage,
            MemoryRegion::DmaBuffers => self.current_snapshot.dma_buffers = usage,
            MemoryRegion::TaskStacks => {
                // Task stacks contribute to overall stack usage
                self.current_snapshot.stack_used = self.current_snapshot.stack_used.saturating_add(usage);
            }
        }
        
        self.current_snapshot.timestamp = snapshot.timestamp;
        
        // Add to historical snapshots
        if self.snapshots.is_full() {
            self.snapshots.remove(0);
        }
        self.snapshots.push(self.current_snapshot).map_err(|_| "Failed to record snapshot")?;
        
        Ok(())
    }
    
    /// Take a comprehensive memory snapshot of current system state
    fn take_memory_snapshot() -> MemorySnapshot {
        let timestamp = Instant::now();
        
        // For ESP32-C3, we can use esp_system functions to get memory info
        // This is a simplified implementation - in real usage, would use ESP-IDF functions
        let (heap_used, heap_free) = Self::get_heap_info();
        let stack_used = Self::estimate_stack_usage();
        let flash_info = Self::get_flash_info();
        
        MemorySnapshot {
            heap_used,
            heap_peak: heap_used, // Will be updated by tracker
            stack_used,
            stack_peak: stack_used, // Will be updated by tracker
            flash_used: flash_info.0,
            flash_total: flash_info.1,
            static_used: Self::estimate_static_usage(),
            dma_buffers: 0, // Would be tracked separately in real implementation
            heap_free,
            timestamp,
        }
    }
    
    /// Get current heap memory information
    fn get_heap_info() -> (usize, usize) {
        // In a real implementation, this would use:
        // - esp_get_free_heap_size()
        // - esp_get_minimum_free_heap_size()
        // For now, return estimated values
        let heap_total = 32 * 1024; // 32KB heap allocation
        let heap_free = 16 * 1024;  // Estimated free
        let heap_used = heap_total - heap_free;
        (heap_used, heap_free)
    }
    
    /// Estimate current stack usage
    fn estimate_stack_usage() -> usize {
        // In a real implementation, this would:
        // 1. Check stack pointer vs stack base
        // 2. Use stack canaries for overflow detection
        // 3. Analyze stack watermark
        // For now, return estimated value
        4 * 1024 // Estimated 4KB stack usage
    }
    
    /// Get flash memory usage information
    fn get_flash_info() -> (usize, usize) {
        // In a real implementation, this would use:
        // - esp_partition_get_used_bytes()
        // - esp_partition_get_total_bytes()
        // For now, return estimated values
        let flash_used = 256 * 1024; // Estimated 256KB used
        let flash_total = 4 * 1024 * 1024; // 4MB total flash
        (flash_used, flash_total)
    }
    
    /// Estimate static memory allocation usage
    fn estimate_static_usage() -> usize {
        // In a real implementation, this would analyze:
        // - .data section size
        // - .bss section size
        // - Static allocations
        // For now, return estimated value
        8 * 1024 // Estimated 8KB static usage
    }
    
    /// Perform comprehensive memory analysis
    pub fn analyze_usage_patterns(&self) -> MemoryAnalysis {
        let trends = self.calculate_memory_trends();
        let fragmentation = self.analyze_fragmentation();
        let performance_impact = self.assess_performance_impact(&fragmentation);
        let recommendations = self.generate_recommendations(&trends, &fragmentation, &performance_impact);
        
        MemoryAnalysis {
            current_usage: self.current_snapshot,
            trends,
            fragmentation,
            performance_impact,
            recommendations,
        }
    }
    
    /// Calculate memory usage trends from historical data
    fn calculate_memory_trends(&self) -> MemoryTrends {
        if self.snapshots.len() < 3 {
            return MemoryTrends {
                heap_trend_bps: 0.0,
                stack_trend_bps: 0.0,
                allocation_frequency: 0.0,
                efficiency_trend: 1.0,
            };
        }
        
        let first = &self.snapshots[0];
        let last = &self.snapshots[self.snapshots.len() - 1];
        let time_diff = last.timestamp.duration_since(first.timestamp).as_secs() as f32;
        
        if time_diff <= 0.0 {
            return MemoryTrends {
                heap_trend_bps: 0.0,
                stack_trend_bps: 0.0,
                allocation_frequency: 0.0,
                efficiency_trend: 1.0,
            };
        }
        
        let heap_change = last.heap_used as i32 - first.heap_used as i32;
        let stack_change = last.stack_used as i32 - first.stack_used as i32;
        
        let heap_trend_bps = heap_change as f32 / time_diff;
        let stack_trend_bps = stack_change as f32 / time_diff;
        
        // Calculate allocation frequency based on heap usage variance
        let allocation_frequency = self.estimate_allocation_frequency();
        
        // Calculate efficiency trend (how well memory is being utilized)
        let efficiency_trend = self.calculate_efficiency_trend();
        
        MemoryTrends {
            heap_trend_bps,
            stack_trend_bps,
            allocation_frequency,
            efficiency_trend,
        }
    }
    
    /// Estimate allocation frequency from usage patterns
    fn estimate_allocation_frequency(&self) -> f32 {
        if self.snapshots.len() < 2 {
            return 0.0;
        }
        
        let mut allocations = 0;
        for i in 1..self.snapshots.len() {
            if self.snapshots[i].heap_used > self.snapshots[i - 1].heap_used {
                allocations += 1;
            }
        }
        
        let time_span = self.snapshots.last().unwrap().timestamp
            .duration_since(self.snapshots[0].timestamp)
            .as_secs() as f32;
        
        if time_span > 0.0 {
            allocations as f32 / time_span
        } else {
            0.0
        }
    }
    
    /// Calculate memory usage efficiency trend
    fn calculate_efficiency_trend(&self) -> f32 {
        // Efficiency is defined as (used memory / (used + free)) ratio
        let total_heap = self.current_snapshot.heap_used + self.current_snapshot.heap_free;
        if total_heap > 0 {
            self.current_snapshot.heap_used as f32 / total_heap as f32
        } else {
            0.0
        }
    }
    
    /// Analyze memory fragmentation
    fn analyze_fragmentation(&self) -> FragmentationData {
        // In a real implementation, this would:
        // 1. Walk the heap free list
        // 2. Analyze free block sizes and distribution
        // 3. Calculate fragmentation metrics
        
        // For now, provide estimated fragmentation analysis
        let heap_total = self.current_snapshot.heap_used + self.current_snapshot.heap_free;
        let fragmentation_percent = if heap_total > 0 {
            // Estimate fragmentation based on allocation patterns
            let efficiency = self.current_snapshot.heap_used as f32 / heap_total as f32;
            ((1.0 - efficiency) * 100.0).min(100.0)
        } else {
            0.0
        };
        
        FragmentationData {
            largest_free_block: self.current_snapshot.heap_free / 2, // Estimate
            free_block_count: 4, // Estimated number of fragments
            fragmentation_percent,
            allocation_failures: 0, // Would be tracked in real implementation
        }
    }
    
    /// Assess performance impact of current memory usage
    fn assess_performance_impact(&self, fragmentation: &FragmentationData) -> MemoryPerformanceImpact {
        // Calculate allocation overhead based on fragmentation
        let allocation_overhead_us = if fragmentation.fragmentation_percent > 20.0 {
            (fragmentation.fragmentation_percent * 2.0) as u32
        } else {
            10 // Base overhead
        };
        
        let fragmentation_impact = fragmentation.fragmentation_percent / 100.0;
        
        // Calculate memory pressure
        let heap_total = self.current_snapshot.heap_used + self.current_snapshot.heap_free;
        let memory_pressure = if heap_total > 0 {
            (self.current_snapshot.heap_used as f32 / heap_total as f32).min(1.0)
        } else {
            0.0
        };
        
        // Calculate OOM risk
        let oom_risk = if memory_pressure > 0.9 {
            1.0
        } else if memory_pressure > 0.8 {
            0.5
        } else {
            0.0
        };
        
        MemoryPerformanceImpact {
            allocation_overhead_us,
            fragmentation_impact,
            memory_pressure,
            oom_risk,
        }
    }
    
    /// Generate optimization recommendations
    fn generate_recommendations(
        &self,
        trends: &MemoryTrends,
        fragmentation: &FragmentationData,
        impact: &MemoryPerformanceImpact,
    ) -> Vec<MemoryRecommendation, 8> {
        let mut recommendations = Vec::new();
        
        // High memory pressure
        if impact.memory_pressure > 0.8 {
            let _ = recommendations.push(MemoryRecommendation::ReduceHeapAllocations);
            let _ = recommendations.push(MemoryRecommendation::PreferStaticAllocation);
        }
        
        // High fragmentation
        if fragmentation.fragmentation_percent > 20.0 {
            let _ = recommendations.push(MemoryRecommendation::UseMemoryPools);
            let _ = recommendations.push(MemoryRecommendation::DefragmentHeap);
        }
        
        // High allocation frequency
        if trends.allocation_frequency > 1.0 {
            let _ = recommendations.push(MemoryRecommendation::UseCircularBuffers);
        }
        
        // Growing heap usage trend
        if trends.heap_trend_bps > 100.0 {
            let _ = recommendations.push(MemoryRecommendation::OptimizeDataStructures);
        }
        
        // High stack usage
        if self.current_snapshot.stack_used > 6 * 1024 {
            let _ = recommendations.push(MemoryRecommendation::OptimizeStackUsage);
        }
        
        recommendations
    }
    
    /// Get current memory snapshot
    pub fn get_current_snapshot(&self) -> MemorySnapshot {
        self.current_snapshot
    }
    
    /// Get all-time peak memory usage
    pub fn get_all_time_peaks(&self) -> MemoryPeaks {
        self.all_time_peaks
    }
    
    /// Reset tracking data and start fresh
    pub fn reset(&mut self) {
        self.snapshots.clear();
        self.current_snapshot = Self::take_memory_snapshot();
        self.all_time_peaks = MemoryPeaks {
            heap_peak: self.current_snapshot.heap_used,
            stack_peak: self.current_snapshot.stack_used,
            max_fragmentation: 0.0,
            heap_peak_time: self.current_snapshot.timestamp,
            stack_peak_time: self.current_snapshot.timestamp,
        };
        self.fragmentation_data = FragmentationData {
            largest_free_block: 0,
            free_block_count: 0,
            fragmentation_percent: 0.0,
            allocation_failures: 0,
        };
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}