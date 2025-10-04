//! Configuration Management - Single file module

use iot_config::{EmbeddedConfig, IoTSystemConfig};

pub struct MinimalConfig {
    #[allow(dead_code)]  // Reserved for future config expansion
    pub system: IoTSystemConfig,
}

impl MinimalConfig {
    pub fn load() -> Self {
        let system = match EmbeddedConfig::load_system_config() {
            Ok(config) => {
                rtt_target::rprintln!("Configuration loaded");
                config
            }
            Err(_) => {
                rtt_target::rprintln!("Using defaults");
                IoTSystemConfig::default_embedded()
            }
        };

        Self { system }
    }

    pub fn sensor_interval_secs(&self) -> u32 {
        // UNIFIED TIMING STANDARD - enforce 30-second intervals across all apps
        30 // Override config to match unified timing standards
    }
}