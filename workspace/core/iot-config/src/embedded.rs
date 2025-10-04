//! Embedded JSON Configuration
//! 
//! Loads configuration from JSON files embedded in the binary at compile time.

extern crate alloc;
#[allow(unused_imports)]
use alloc::string::String;

use crate::{ConfigResult, ConfigError, IoTSystemConfig};

/// Embedded JSON configuration loader
pub struct EmbeddedConfig;

impl EmbeddedConfig {
    /// Default configuration JSON embedded in binary
    pub const DEFAULT_CONFIG_JSON: &'static str = include_str!("../config/default.json");
    
    /// Development configuration (enabled with --features development)
    #[cfg(feature = "development")]
    pub const CONFIG_JSON: &'static str = include_str!("../config/development.json");
    
    /// Production configuration (enabled with --features production)
    #[cfg(feature = "production")]
    pub const CONFIG_JSON: &'static str = include_str!("../config/production.json");
    
    /// Default to default.json if no specific environment is selected
    #[cfg(not(any(feature = "development", feature = "production")))]
    pub const CONFIG_JSON: &'static str = Self::DEFAULT_CONFIG_JSON;
    
    /// Load system configuration from embedded JSON
    pub fn load_system_config() -> ConfigResult<IoTSystemConfig> {
        Self::load_from_json_str(Self::CONFIG_JSON)
    }
    
    /// Load configuration from specific JSON string
    pub fn load_from_json_str(json_str: &str) -> ConfigResult<IoTSystemConfig> {
        IoTSystemConfig::from_json_bytes(json_str.as_bytes())
    }
    
    /// Get the current configuration profile name
    pub fn get_profile_name() -> &'static str {
        #[cfg(feature = "development")]
        return "development";
        
        #[cfg(feature = "production")]
        return "production";
        
        #[cfg(not(any(feature = "development", feature = "production")))]
        return "default";
    }
    
    /// Get configuration JSON as string for inspection
    pub fn get_config_json() -> &'static str {
        Self::CONFIG_JSON
    }
    
    /// Validate embedded configuration at compile time
    pub fn validate_embedded_config() -> ConfigResult<()> {
        let config = Self::load_system_config()?;
        let validation_report = crate::validation::ConfigValidator::validate(&config);
        if !validation_report.is_valid {
            return Err(ConfigError::ValidationFailed("Embedded configuration validation failed".into()));
        }
        Ok(())
    }
}

/// Runtime configuration management with fallback hierarchy
pub struct RuntimeConfigManager;

impl RuntimeConfigManager {
    /// Load configuration with fallback hierarchy:
    /// 1. Flash storage (user updates)
    /// 2. Embedded JSON (compile-time default)
    /// 3. Struct defaults (last resort)
    pub async fn load_config() -> IoTSystemConfig {
        // TODO: Try loading from flash storage first
        // if let Ok(config) = Self::load_from_flash().await {
        //     return config;
        // }
        
        // Fall back to embedded JSON
        if let Ok(config) = EmbeddedConfig::load_system_config() {
            return config;
        }
        
        // Final fallback to struct defaults
        // Note: In no_std environment, logging may not be available
        IoTSystemConfig::default()
    }
    
    /// Load configuration from flash storage (integration with iot-storage)
    #[cfg(feature = "storage")]
    pub async fn load_from_flash() -> ConfigResult<IoTSystemConfig> {
        use iot_storage::{FlashStorage, ConfigStorage};
        
        let mut storage = FlashStorage::new().await
            .map_err(|_| ConfigError::StorageError("Failed to initialize flash storage".into()))?;
        
        storage.retrieve_config::<IoTSystemConfig>("system_config").await
            .map_err(|_| ConfigError::StorageError("Failed to load config from flash".into()))
    }
    
    /// Store configuration to flash storage
    #[cfg(feature = "storage")]
    pub async fn store_to_flash(config: &IoTSystemConfig) -> ConfigResult<()> {
        use iot_storage::{FlashStorage, ConfigStorage};
        
        let mut storage = FlashStorage::new().await
            .map_err(|_| ConfigError::StorageError("Failed to initialize flash storage".into()))?;
        
        storage.store_config("system_config", config).await
            .map_err(|_| ConfigError::StorageError("Failed to store config to flash".into()))
    }
    
    /// Update configuration from JSON string and persist
    pub async fn update_config_from_json(json_str: &str) -> ConfigResult<IoTSystemConfig> {
        // Parse and validate new configuration
        let new_config = EmbeddedConfig::load_from_json_str(json_str)?;
        
        // Store to flash if storage is available
        #[cfg(feature = "storage")]
        Self::store_to_flash(&new_config).await?;
        
        Ok(new_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embedded_config_loading() {
        let config = EmbeddedConfig::load_system_config();
        assert!(config.is_ok(), "Embedded configuration should load successfully");
        
        let config = config.unwrap();
        assert!(!config.wifi.ssid.is_empty(), "WiFi SSID should not be empty");
        assert_ne!(config.mqtt.broker_port, 0, "MQTT port should be set");
    }
    
    #[test]
    fn test_profile_detection() {
        let profile = EmbeddedConfig::get_profile_name();
        assert!(
            profile == "development" || profile == "production" || profile == "default",
            "Profile should be one of the known types"
        );
    }
    
    #[test]
    fn test_json_string_access() {
        let json = EmbeddedConfig::get_config_json();
        assert!(!json.is_empty(), "Configuration JSON should not be empty");
        assert!(json.contains("wifi"), "JSON should contain wifi section");
        assert!(json.contains("mqtt"), "JSON should contain mqtt section");
    }
    
    #[test]
    fn test_embedded_config_validation() {
        let result = EmbeddedConfig::validate_embedded_config();
        assert!(result.is_ok(), "Embedded configuration should be valid");
    }
}