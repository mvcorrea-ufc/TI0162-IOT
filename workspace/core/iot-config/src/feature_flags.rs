//! Feature Flag System
//! 
//! Compile-time and runtime feature management for optional components.

extern crate alloc;
use alloc::{string::String, format, string::ToString};

use serde::{Deserialize, Serialize};
use crate::{ConfigResult, ConfigError};

/// Runtime feature flags that can be toggled based on deployment needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFeatures {
    pub performance_monitoring: bool,
    pub debug_logging: bool,
    pub console_enabled: bool,
    pub status_led: bool,
    pub sensor_calibration: bool,
    pub auto_reconnect: bool,
    pub data_persistence: bool,
}

impl Default for RuntimeFeatures {
    fn default() -> Self {
        Self {
            performance_monitoring: true,
            debug_logging: cfg!(debug_assertions),
            console_enabled: true,
            status_led: true,
            sensor_calibration: true,
            auto_reconnect: true,
            data_persistence: cfg!(feature = "storage"),
        }
    }
}

/// Compile-time feature detection and validation
pub struct CompileTimeFeatures;

impl CompileTimeFeatures {
    /// Check if WiFi support is compiled in
    pub const fn has_wifi() -> bool {
        cfg!(feature = "wifi")
    }

    /// Check if MQTT support is compiled in
    pub const fn has_mqtt() -> bool {
        cfg!(feature = "mqtt")
    }

    /// Check if console support is compiled in
    pub const fn has_console() -> bool {
        cfg!(feature = "console")
    }

    /// Check if performance monitoring is compiled in
    pub const fn has_performance() -> bool {
        cfg!(feature = "performance")
    }

    /// Check if dependency injection container is compiled in
    pub const fn has_container() -> bool {
        cfg!(feature = "container")
    }

    /// Check if storage support is compiled in
    pub const fn has_storage() -> bool {
        cfg!(feature = "storage")
    }

    /// Check if this is a minimal build
    pub const fn is_minimal() -> bool {
        !Self::has_wifi() && !Self::has_mqtt() && !Self::has_console()
    }

    /// Check if this is a full-featured build
    pub const fn is_full() -> bool {
        Self::has_wifi() && Self::has_mqtt() && Self::has_console() && Self::has_performance()
    }

    /// Get build configuration name
    pub fn build_config_name() -> &'static str {
        if Self::is_minimal() {
            "minimal"
        } else if Self::is_full() {
            "full"
        } else if Self::has_wifi() && Self::has_mqtt() {
            "standard"
        } else if Self::has_wifi() {
            "wifi-only"
        } else {
            "custom"
        }
    }

    /// Validate feature combination
    pub fn validate_features() -> ConfigResult<()> {
        // MQTT requires WiFi
        if Self::has_mqtt() && !Self::has_wifi() {
            return Err(ConfigError::FeatureNotEnabled(
                "MQTT feature requires WiFi feature to be enabled".to_string()
            ));
        }

        // Container feature implies storage
        if Self::has_container() && !Self::has_storage() {
            return Err(ConfigError::FeatureNotEnabled(
                "Container feature works best with storage feature enabled".to_string()
            ));
        }

        Ok(())
    }

    /// Get estimated flash usage based on enabled features
    pub fn estimated_flash_usage() -> usize {
        let mut size = 45_000; // Base size (core + sensor)

        if Self::has_wifi() {
            size += 25_000; // WiFi stack
        }

        if Self::has_mqtt() {
            size += 15_000; // MQTT protocol
        }

        if Self::has_console() {
            size += 10_000; // Console interface
        }

        if Self::has_performance() {
            size += 8_000; // Performance monitoring
        }

        if Self::has_container() {
            size += 12_000; // Dependency injection
        }

        if Self::has_storage() {
            size += 6_000; // Storage abstraction
        }

        size
    }

    /// Get list of enabled features
    pub fn enabled_features() -> heapless::Vec<&'static str, 8> {
        let mut features = heapless::Vec::new();

        if Self::has_wifi() {
            features.push("wifi").ok();
        }
        if Self::has_mqtt() {
            features.push("mqtt").ok();
        }
        if Self::has_console() {
            features.push("console").ok();
        }
        if Self::has_performance() {
            features.push("performance").ok();
        }
        if Self::has_container() {
            features.push("container").ok();
        }
        if Self::has_storage() {
            features.push("storage").ok();
        }

        features
    }
}

/// Feature-gated configuration loading
impl RuntimeFeatures {
    /// Load features from environment or use defaults
    /// Note: Runtime environment loading is limited in no_std
    pub fn from_environment() -> Self {
        let mut features = Self::default();

        // Runtime feature detection based on compile-time features
        features.performance_monitoring = CompileTimeFeatures::has_performance();
        features.console_enabled = CompileTimeFeatures::has_console();
        features.debug_logging = cfg!(debug_assertions);
        features.status_led = true; // Default enabled
        features.auto_reconnect = true; // Default enabled
        features.sensor_calibration = true; // Default enabled
        features.data_persistence = CompileTimeFeatures::has_storage();

        features
    }

    /// Validate runtime features against compile-time features
    pub fn validate_against_compile_time(&self) -> ConfigResult<()> {
        if self.console_enabled && !CompileTimeFeatures::has_console() {
            return Err(ConfigError::FeatureNotEnabled(
                "Console feature not compiled in".to_string()
            ));
        }

        if self.performance_monitoring && !CompileTimeFeatures::has_performance() {
            return Err(ConfigError::FeatureNotEnabled(
                "Performance monitoring feature not compiled in".to_string()
            ));
        }

        if self.data_persistence && !CompileTimeFeatures::has_storage() {
            return Err(ConfigError::FeatureNotEnabled(
                "Storage feature not compiled in".to_string()
            ));
        }

        Ok(())
    }

    /// Get feature summary for debugging
    pub fn summary(&self) -> String {
        format!(
            "Runtime Features: perf={}, debug={}, console={}, led={}, cal={}, reconnect={}, persist={}",
            self.performance_monitoring,
            self.debug_logging,
            self.console_enabled,
            self.status_led,
            self.sensor_calibration,
            self.auto_reconnect,
            self.data_persistence
        )
    }
}

/// Macro for conditional compilation based on features
#[macro_export]
macro_rules! feature_enabled {
    ($feature:literal) => {
        cfg!(feature = $feature)
    };
}

/// Macro for conditional code execution at runtime
#[macro_export]
macro_rules! if_feature_enabled {
    ($features:expr, $feature:ident, $code:block) => {
        if $features.$feature {
            $code
        }
    };
}

/// Build information structure
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub config_name: &'static str,
    pub enabled_features: heapless::Vec<&'static str, 8>,
    pub estimated_flash_size: usize,
    pub build_timestamp: &'static str,
    pub is_debug: bool,
}

impl BuildInfo {
    /// Get comprehensive build information
    pub fn current() -> Self {
        Self {
            config_name: CompileTimeFeatures::build_config_name(),
            enabled_features: CompileTimeFeatures::enabled_features(),
            estimated_flash_size: CompileTimeFeatures::estimated_flash_usage(),
            build_timestamp: "compile-time",
            is_debug: cfg!(debug_assertions),
        }
    }

    /// Get build summary string
    pub fn summary(&self) -> String {
        format!(
            "Build: {} ({} features, ~{}KB flash, debug={})",
            self.config_name,
            self.enabled_features.len(),
            self.estimated_flash_size / 1024,
            self.is_debug
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_features() {
        // These tests will vary based on enabled features
        let _has_wifi = CompileTimeFeatures::has_wifi();
        let _has_mqtt = CompileTimeFeatures::has_mqtt();
        
        // Validate feature combinations
        assert!(CompileTimeFeatures::validate_features().is_ok());
    }

    #[test]
    fn test_runtime_features() {
        let features = RuntimeFeatures::default();
        assert!(features.validate_against_compile_time().is_ok());
    }

    #[test]
    fn test_build_info() {
        let build_info = BuildInfo::current();
        assert!(!build_info.config_name.is_empty());
        assert!(build_info.estimated_flash_size > 40_000);
    }

    #[test]
    fn test_feature_macro() {
        // Test compile-time feature detection
        let _wifi_enabled = feature_enabled!("wifi");
        let _mqtt_enabled = feature_enabled!("mqtt");
    }

    #[test]
    fn test_estimated_flash_usage() {
        let size = CompileTimeFeatures::estimated_flash_usage();
        // Should be reasonable for ESP32-C3
        assert!(size >= 45_000);
        assert!(size <= 300_000);
    }
}