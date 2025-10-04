//! Standardized Timing Configuration
//! 
//! This module provides unified timing intervals that ALL ESP32-C3 IoT applications
//! must use to achieve synchronized behavior across different architectures.

#[cfg(feature = "embassy")]
use embassy_time::Duration;

/// Standard timing configuration that all applications must conform to
/// 
/// This ensures synchronized behavior across:
/// - main-nodeps (synchronous blocking)
/// - main-min (async minimal)
/// - main-app (async full-featured)
#[derive(Debug, Clone, Copy)]
pub struct StandardTimingConfig {
    /// Sensor reading interval in seconds (UNIFIED: 30 seconds)
    pub sensor_reading_interval_secs: u32,
    
    /// Heartbeat interval in seconds (UNIFIED: 60 seconds)
    pub heartbeat_interval_secs: u32,
    
    /// Status reporting interval in seconds (UNIFIED: 120 seconds)
    pub status_interval_secs: u32,
    
    /// Main loop delay for synchronous architecture (50ms)
    pub sync_loop_delay_ms: u32,
    
    /// Cycle duration for async architecture (10 seconds)
    pub async_cycle_duration_secs: u32,
    
    /// Timing offsets to prevent conflicts
    pub sensor_offset_ms: u32,     // 0ms - immediate start
    /// Heartbeat offset timing - 5000ms (5s after sensors)
    pub heartbeat_offset_ms: u32,
    /// Status offset timing - 10000ms (10s after sensors)
    pub status_offset_ms: u32,
}

impl Default for StandardTimingConfig {
    fn default() -> Self {
        Self {
            sensor_reading_interval_secs: 30,  // Every 30 seconds
            heartbeat_interval_secs: 60,       // Every 60 seconds
            status_interval_secs: 120,         // Every 2 minutes
            sync_loop_delay_ms: 50,            // 50ms main loop
            async_cycle_duration_secs: 10,     // 10s async cycles
            sensor_offset_ms: 0,               // Sensors start immediately
            heartbeat_offset_ms: 5000,         // Heartbeats offset by 5s
            status_offset_ms: 10000,           // Status offset by 10s
        }
    }
}

/// Timing configuration for synchronous (main-nodeps) architecture
#[derive(Debug, Clone, Copy)]
pub struct SyncTimingCycles {
    /// Number of main loop cycles between sensor readings
    pub sensor_interval_cycles: u32,
    
    /// Number of main loop cycles between heartbeats
    pub heartbeat_interval_cycles: u32,
    
    /// Number of main loop cycles between status reports
    pub status_interval_cycles: u32,
    
    /// Cycle offset for sensor readings
    pub sensor_offset_cycles: u32,
    
    /// Cycle offset for heartbeats
    pub heartbeat_offset_cycles: u32,
    
    /// Cycle offset for status reports
    pub status_offset_cycles: u32,
}

/// Timing configuration for asynchronous architectures
#[cfg(feature = "embassy")]
#[derive(Debug, Clone, Copy)]
pub struct AsyncTimingDurations {
    /// Duration between sensor readings
    pub sensor_interval: Duration,
    
    /// Duration between heartbeats
    pub heartbeat_interval: Duration,
    
    /// Duration between status reports
    pub status_interval: Duration,
    
    /// Main cycle duration for async loops
    pub cycle_duration: Duration,
    
    /// Offset for sensor readings
    pub sensor_offset: Duration,
    
    /// Offset for heartbeats
    pub heartbeat_offset: Duration,
    
    /// Offset for status reports
    pub status_offset: Duration,
}

impl StandardTimingConfig {
    /// Convert to synchronous cycle-based timing (for main-nodeps)
    pub fn to_sync_cycles(&self) -> SyncTimingCycles {
        let cycles_per_second = 1000 / self.sync_loop_delay_ms;
        
        SyncTimingCycles {
            sensor_interval_cycles: self.sensor_reading_interval_secs * cycles_per_second,
            heartbeat_interval_cycles: self.heartbeat_interval_secs * cycles_per_second,
            status_interval_cycles: self.status_interval_secs * cycles_per_second,
            sensor_offset_cycles: self.sensor_offset_ms / self.sync_loop_delay_ms,
            heartbeat_offset_cycles: self.heartbeat_offset_ms / self.sync_loop_delay_ms,
            status_offset_cycles: self.status_offset_ms / self.sync_loop_delay_ms,
        }
    }
    
    /// Convert to asynchronous duration-based timing (for main-min/main-app)
    #[cfg(feature = "embassy")]
    pub fn to_async_durations(&self) -> AsyncTimingDurations {
        AsyncTimingDurations {
            sensor_interval: Duration::from_secs(self.sensor_reading_interval_secs as u64),
            heartbeat_interval: Duration::from_secs(self.heartbeat_interval_secs as u64),
            status_interval: Duration::from_secs(self.status_interval_secs as u64),
            cycle_duration: Duration::from_secs(self.async_cycle_duration_secs as u64),
            sensor_offset: Duration::from_millis(self.sensor_offset_ms as u64),
            heartbeat_offset: Duration::from_millis(self.heartbeat_offset_ms as u64),
            status_offset: Duration::from_millis(self.status_offset_ms as u64),
        }
    }
    
    /// Get cycle count for specific event in synchronous mode
    pub fn get_sync_cycle_count(&self, event: TimingEvent, current_cycle: u32) -> bool {
        let cycles = self.to_sync_cycles();
        
        match event {
            TimingEvent::SensorReading => {
                current_cycle > 0 && 
                (current_cycle % cycles.sensor_interval_cycles) == cycles.sensor_offset_cycles
            }
            TimingEvent::Heartbeat => {
                current_cycle > 0 && 
                (current_cycle % cycles.heartbeat_interval_cycles) == cycles.heartbeat_offset_cycles
            }
            TimingEvent::StatusReport => {
                current_cycle > 0 && 
                (current_cycle % cycles.status_interval_cycles) == cycles.status_offset_cycles
            }
        }
    }
    
    /// Get cycle count for specific event in async mode
    #[cfg(feature = "embassy")]
    pub fn get_async_cycle_count(&self, event: TimingEvent, cycle_counter: u32) -> bool {
        let cycles_per_interval = match event {
            TimingEvent::SensorReading => {
                (self.sensor_reading_interval_secs + self.async_cycle_duration_secs - 1) 
                / self.async_cycle_duration_secs
            }
            TimingEvent::Heartbeat => {
                (self.heartbeat_interval_secs + self.async_cycle_duration_secs - 1) 
                / self.async_cycle_duration_secs
            }
            TimingEvent::StatusReport => {
                (self.status_interval_secs + self.async_cycle_duration_secs - 1) 
                / self.async_cycle_duration_secs
            }
        };
        
        cycle_counter > 0 && (cycle_counter % cycles_per_interval) == 0
    }
    
    /// Create configuration for specific architecture
    pub fn for_architecture(arch: crate::standard_messages::IoTArchitecture) -> Self {
        let mut config = Self::default();
        
        // Architecture-specific adjustments if needed
        match arch {
            crate::standard_messages::IoTArchitecture::Synchronous => {
                // main-nodeps optimizations
                config.sync_loop_delay_ms = 50;
            }
            crate::standard_messages::IoTArchitecture::AsyncMinimal => {
                // main-min optimizations
                config.async_cycle_duration_secs = 10;
            }
            crate::standard_messages::IoTArchitecture::AsyncFull => {
                // main-app can handle more frequent cycles
                config.async_cycle_duration_secs = 10;
            }
        }
        
        config
    }
    
    /// Validate timing configuration
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.sensor_reading_interval_secs == 0 {
            return Err("Sensor interval must be > 0");
        }
        
        if self.heartbeat_interval_secs == 0 {
            return Err("Heartbeat interval must be > 0");
        }
        
        if self.status_interval_secs == 0 {
            return Err("Status interval must be > 0");
        }
        
        if self.sync_loop_delay_ms == 0 {
            return Err("Sync loop delay must be > 0");
        }
        
        if self.async_cycle_duration_secs == 0 {
            return Err("Async cycle duration must be > 0");
        }
        
        // Ensure reasonable timing relationships
        if self.heartbeat_interval_secs < self.sensor_reading_interval_secs {
            return Err("Heartbeat interval should be >= sensor interval");
        }
        
        if self.status_interval_secs < self.heartbeat_interval_secs {
            return Err("Status interval should be >= heartbeat interval");
        }
        
        Ok(())
    }
}

/// Timing events that can be scheduled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingEvent {
    /// Sensor reading event
    SensorReading,
    /// Heartbeat publishing event
    Heartbeat,
    /// Status report publishing event
    StatusReport,
}

/// Timer manager for tracking timing events
pub struct TimingManager {
    config: StandardTimingConfig,
    cycle_counter: u32,
    architecture: crate::standard_messages::IoTArchitecture,
}

impl TimingManager {
    /// Create new timing manager for specific architecture
    pub fn new(architecture: crate::standard_messages::IoTArchitecture) -> Self {
        Self {
            config: StandardTimingConfig::for_architecture(architecture),
            cycle_counter: 0,
            architecture,
        }
    }
    
    /// Increment cycle counter (call once per main loop/cycle)
    pub fn increment_cycle(&mut self) {
        self.cycle_counter = self.cycle_counter.wrapping_add(1);
    }
    
    /// Check if it's time for a specific event
    pub fn is_time_for_event(&self, event: TimingEvent) -> bool {
        match self.architecture {
            crate::standard_messages::IoTArchitecture::Synchronous => {
                self.config.get_sync_cycle_count(event, self.cycle_counter)
            }
            _ => {
                #[cfg(feature = "embassy")]
                {
                    self.config.get_async_cycle_count(event, self.cycle_counter)
                }
                #[cfg(not(feature = "embassy"))]
                {
                    false
                }
            }
        }
    }
    
    /// Get current cycle count
    pub fn get_cycle_count(&self) -> u32 {
        self.cycle_counter
    }
    
    /// Get timing configuration
    pub fn get_config(&self) -> StandardTimingConfig {
        self.config
    }
    
    /// Reset cycle counter
    pub fn reset(&mut self) {
        self.cycle_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard_messages::IoTArchitecture;
    
    #[test]
    fn test_default_timing_config() {
        let config = StandardTimingConfig::default();
        assert_eq!(config.sensor_reading_interval_secs, 30);
        assert_eq!(config.heartbeat_interval_secs, 60);
        assert_eq!(config.status_interval_secs, 120);
        assert_eq!(config.sync_loop_delay_ms, 50);
    }
    
    #[test]
    fn test_sync_timing_conversion() {
        let config = StandardTimingConfig::default();
        let cycles = config.to_sync_cycles();
        
        // With 50ms loop delay: 20 cycles per second
        assert_eq!(cycles.sensor_interval_cycles, 30 * 20); // 600 cycles for 30s
        assert_eq!(cycles.heartbeat_interval_cycles, 60 * 20); // 1200 cycles for 60s
        assert_eq!(cycles.status_interval_cycles, 120 * 20); // 2400 cycles for 120s
    }
    
    #[test]
    fn test_timing_validation() {
        let config = StandardTimingConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config;
        invalid_config.sensor_reading_interval_secs = 0;
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_timing_manager() {
        let mut manager = TimingManager::new(IoTArchitecture::Synchronous);
        assert_eq!(manager.get_cycle_count(), 0);
        
        // Test sensor timing (should trigger at cycle 1, then every 600 cycles)
        manager.increment_cycle(); // cycle 1
        assert!(manager.is_time_for_event(TimingEvent::SensorReading));
        assert!(!manager.is_time_for_event(TimingEvent::Heartbeat));
        
        // Advance to heartbeat time
        for _ in 2..=100 { // cycle 100 = 5000ms offset
            manager.increment_cycle();
        }
        assert!(manager.is_time_for_event(TimingEvent::Heartbeat));
    }
    
    #[test]
    fn test_architecture_specific_config() {
        let sync_config = StandardTimingConfig::for_architecture(IoTArchitecture::Synchronous);
        let async_config = StandardTimingConfig::for_architecture(IoTArchitecture::AsyncMinimal);
        
        assert_eq!(sync_config.sync_loop_delay_ms, 50);
        assert_eq!(async_config.async_cycle_duration_secs, 10);
    }
}