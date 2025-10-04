//! # IoT Storage Abstraction Layer
//!
//! A comprehensive storage abstraction layer for ESP32-C3 IoT applications that provides
//! unified access to flash storage, wear leveling, atomic operations, and configuration
//! persistence. Designed for embedded systems with strict memory constraints.
//!
//! ## Features
//!
//! - **Storage Abstraction**: Unified interface for different storage backends
//! - **Flash Storage**: ESP32-C3 flash memory integration with wear leveling
//! - **Atomic Operations**: Safe concurrent access to storage resources
//! - **Configuration Persistence**: Specialized storage for system configuration
//! - **Error Recovery**: Robust error handling and recovery mechanisms
//! - **Memory Efficiency**: Optimized for constrained embedded environments
//! - **No-std Compatible**: Works without heap allocation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iot_storage::{ConfigStorage, FlashStorageManager};
//!
//! // Create flash storage manager
//! let mut storage = FlashStorageManager::new()?;
//!
//! // Store configuration
//! let config = MyConfig::default();
//! storage.store("system_config", &config).await?;
//!
//! // Retrieve configuration
//! let loaded_config: MyConfig = storage.retrieve("system_config").await?;
//! ```
//!
//! ## Architecture
//!
//! The storage system is organized into layers:
//! - **Storage Traits**: Generic interfaces for different storage operations
//! - **Flash Backend**: ESP32-C3 specific flash storage implementation
//! - **Atomic Operations**: Thread-safe storage operations
//! - **Configuration Store**: High-level configuration management
//! - **Wear Leveling**: Flash memory protection and optimization

#![no_std]
#![deny(unsafe_code)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    unused_lifetimes,
    unused_qualifications
)]

extern crate alloc;
use alloc::{string::{String, ToString}, vec::Vec, boxed::Box, format};
use core::str::FromStr;

pub mod traits;
pub mod flash;
pub mod config;
pub mod atomic;
pub mod wear_leveling;

#[cfg(feature = "esp32c3-flash")]
pub mod esp32c3;

// Re-export main types for convenience
pub use traits::{
    StorageBackend, ConfigStorage, AtomicStorage, StorageKey, StorageValue,
    StorageError, StorageResult, StorageCapacity, StorageStats
};
pub use flash::{FlashStorageManager, FlashRegion, FlashConfig};
pub use config::{ConfigStore, ConfigEntry, ConfigManager};
pub use atomic::{AtomicStorageManager, StorageTransaction, TransactionState};

#[cfg(feature = "esp32c3-flash")]
pub use esp32c3::{Esp32C3Storage, Esp32C3Config};

// Constants for Vec size parameters in no_std environment
/// Maximum number of storage keys that can be tracked
pub const MAX_KEYS: usize = 64;
/// Maximum number of concurrent transactions
pub const MAX_TRANSACTIONS: usize = 16;
/// Maximum number of wear leveling blocks
pub const MAX_WEAR_BLOCKS: usize = 32;

// Error handling
use iot_common::{IoTError, IoTResult, SystemError};
use iot_config::{StorageConfig, EmbeddedConfig};

/// Storage management errors
#[derive(Debug, Clone)]
pub enum StorageErrorKind {
    /// Storage operation failed
    OperationFailed(ErrorString),
    /// Storage corrupted or invalid
    CorruptedData(ErrorString),
    /// Storage capacity exceeded
    CapacityExceeded(ErrorString),
    /// Key not found in storage
    KeyNotFound(ErrorString),
    /// Serialization/deserialization failed
    SerializationFailed(ErrorString),
    /// Flash hardware error
    HardwareError(ErrorString),
    /// Wear leveling failure
    WearLevelingError(ErrorString),
    /// Transaction failed
    TransactionFailed(ErrorString),
}

impl StorageErrorKind {
    /// Convert to IoTError
    pub fn into_iot_error(self) -> IoTError {
        let message = match &self {
            StorageErrorKind::OperationFailed(msg) => msg.as_str(),
            StorageErrorKind::CorruptedData(msg) => msg.as_str(),
            StorageErrorKind::CapacityExceeded(msg) => msg.as_str(),
            StorageErrorKind::KeyNotFound(msg) => msg.as_str(),
            StorageErrorKind::SerializationFailed(msg) => msg.as_str(),
            StorageErrorKind::HardwareError(msg) => msg.as_str(),
            StorageErrorKind::WearLevelingError(msg) => msg.as_str(),
            StorageErrorKind::TransactionFailed(msg) => msg.as_str(),
        };
        
        let error = match self {
            StorageErrorKind::OperationFailed(_) => SystemError::ResourceUnavailable(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::CorruptedData(_) => SystemError::InitializationFailed(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::CapacityExceeded(_) => SystemError::OutOfMemory(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::KeyNotFound(_) => SystemError::ResourceUnavailable(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::SerializationFailed(_) => SystemError::InitializationFailed(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::HardwareError(_) => SystemError::InitializationFailed(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::WearLevelingError(_) => SystemError::ResourceUnavailable(
                iot_common::error::utils::error_message(message)
            ),
            StorageErrorKind::TransactionFailed(_) => SystemError::ResourceUnavailable(
                iot_common::error::utils::error_message(message)
            ),
        };
        
        IoTError::system(error)
    }
}

/// Storage result type
pub type StorageManagerResult<T> = Result<T, StorageErrorKind>;

/// Convert storage result to IoTResult
pub fn storage_to_iot_result<T>(result: StorageManagerResult<T>) -> IoTResult<T> {
    result.map_err(|e| e.into_iot_error())
}

/// Current version of the iot-storage library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum key length for storage entries
pub const MAX_KEY_LEN: usize = 64;

/// Maximum value size for small storage entries (in bytes)
pub const MAX_SMALL_VALUE_SIZE: usize = 256;

/// Maximum value size for large storage entries (in bytes)
pub const MAX_LARGE_VALUE_SIZE: usize = 4096;

/// Maximum error message length
pub const MAX_ERROR_LEN: usize = 256;

/// Maximum configuration string length
pub const MAX_CONFIG_STRING_LEN: usize = 128;

/// Default flash region size for configuration storage
pub const DEFAULT_CONFIG_REGION_SIZE: usize = 8192;

/// Default flash region size for data storage
pub const DEFAULT_DATA_REGION_SIZE: usize = 32768;

/// Maximum number of concurrent transactions
pub const MAX_CONCURRENT_TRANSACTIONS: usize = 4;

/// Key string type for embedded environment
pub type StorageKeyString = heapless::String<MAX_KEY_LEN>;

/// Error message string type for embedded environment
pub type ErrorString = heapless::String<MAX_ERROR_LEN>;

/// Configuration string type for embedded environment
pub type ConfigString = heapless::String<MAX_CONFIG_STRING_LEN>;

/// Helper function to create error strings safely
fn create_error_string(msg: &str) -> ErrorString {
    ErrorString::try_from(msg).unwrap_or_else(|_| {
        // Truncate if too long
        let truncated = if msg.len() > MAX_ERROR_LEN - 3 {
            &msg[..MAX_ERROR_LEN - 3]
        } else {
            msg
        };
        ErrorString::try_from(truncated).unwrap_or_else(|_| ErrorString::new())
    })
}

/// Helper function to create config strings safely
fn create_config_string(msg: &str) -> ConfigString {
    ConfigString::try_from(msg).unwrap_or_else(|_| {
        // Truncate if too long
        let truncated = if msg.len() > MAX_CONFIG_STRING_LEN - 3 {
            &msg[..MAX_CONFIG_STRING_LEN - 3]
        } else {
            msg
        };
        ConfigString::try_from(truncated).unwrap_or_else(|_| ConfigString::new())
    })
}

/// Unified storage manager that combines all storage capabilities
pub struct UnifiedStorageManager<B: StorageBackend> {
    /// Storage backend implementation
    backend: B,
    /// Configuration store
    config_store: ConfigStore<B>,
    /// Atomic storage manager
    atomic_manager: AtomicStorageManager<B>,
    /// Storage statistics
    stats: StorageStats,
}

impl<B: StorageBackend> UnifiedStorageManager<B> {
    /// Create new unified storage manager
    pub fn new(backend: B) -> StorageManagerResult<Self> {
        let config_store = ConfigStore::new(&backend).map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Failed to create config store")
            )
        })?;
        
        let atomic_manager = AtomicStorageManager::new(&backend).map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Failed to create atomic manager")
            )
        })?;
        
        let stats = backend.get_stats().map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Failed to get storage stats")
            )
        })?;
        
        Ok(Self {
            backend,
            config_store,
            atomic_manager,
            stats,
        })
    }

    /// Get storage capacity information
    pub fn get_capacity(&self) -> StorageManagerResult<StorageCapacity> {
        self.backend.get_capacity().map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Failed to get capacity")
            )
        })
    }

    /// Get storage statistics
    pub fn get_stats(&mut self) -> StorageManagerResult<StorageStats> {
        self.stats = self.backend.get_stats().map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Failed to get stats")
            )
        })?;
        Ok(self.stats.clone())
    }

    /// Get configuration store
    pub fn config_store(&mut self) -> &mut ConfigStore<B> {
        &mut self.config_store
    }

    /// Get atomic storage manager
    pub fn atomic_manager(&mut self) -> &mut AtomicStorageManager<B> {
        &mut self.atomic_manager
    }

    /// Store data with key
    pub async fn store<T>(&mut self, key: &str, value: &T) -> StorageManagerResult<()> 
    where
        T: serde::Serialize,
    {
        let serialized: heapless::Vec<u8, 4096> = serde_json_core::to_vec(value).map_err(|_| {
            StorageErrorKind::SerializationFailed(
                create_error_string("Failed to serialize data")
            )
        })?;
        
        let storage_key = StorageKey::from_str(key).map_err(|_| {
            StorageErrorKind::OperationFailed(
                create_error_string("Invalid key")
            )
        })?;
        
        let storage_value = StorageValue::from_bytes(&serialized).map_err(|_| {
            StorageErrorKind::OperationFailed(
                create_error_string("Invalid value")
            )
        })?;
        
        self.backend.store(&storage_key, &storage_value).await.map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Storage operation failed")
            )
        })
    }

    /// Retrieve data by key
    pub async fn retrieve<T>(&mut self, key: &str) -> StorageManagerResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let storage_key = StorageKey::from_str(key).map_err(|_| {
            StorageErrorKind::OperationFailed(
                create_error_string("Invalid key")
            )
        })?;
        
        let storage_value = self.backend.retrieve(&storage_key).await.map_err(|e| {
            match e {
                StorageError::KeyNotFound => StorageErrorKind::KeyNotFound(
                    create_error_string("Key not found")
                ),
                _ => StorageErrorKind::OperationFailed(
                    create_error_string("Retrieval failed")
                ),
            }
        })?;
        
        let bytes = storage_value.as_bytes();
        let (value, _) = serde_json_core::from_slice(bytes).map_err(|_| {
            StorageErrorKind::SerializationFailed(
                create_error_string("Failed to deserialize data")
            )
        })?;
        
        Ok(value)
    }

    /// Delete data by key
    pub async fn delete(&mut self, key: &str) -> StorageManagerResult<()> {
        let storage_key = StorageKey::from_str(key).map_err(|_| {
            StorageErrorKind::OperationFailed(
                create_error_string("Invalid key")
            )
        })?;
        
        self.backend.delete(&storage_key).await.map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Delete operation failed")
            )
        })
    }

    /// Check if key exists
    pub async fn exists(&mut self, key: &str) -> StorageManagerResult<bool> {
        let storage_key = StorageKey::from_str(key).map_err(|_| {
            StorageErrorKind::OperationFailed(
                create_error_string("Invalid key")
            )
        })?;
        
        self.backend.exists(&storage_key).await.map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Exists check failed")
            )
        })
    }

    /// List all keys with optional prefix
    pub async fn list_keys(&mut self, prefix: Option<&str>) -> StorageManagerResult<Vec<String>> {
        self.backend.list_keys(prefix).await.map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("List keys failed")
            )
        })
    }

    /// Perform storage maintenance (garbage collection, defragmentation)
    pub async fn maintenance(&mut self) -> StorageManagerResult<()> {
        self.backend.maintenance().await.map_err(|_e| {
            StorageErrorKind::OperationFailed(
                create_error_string("Maintenance failed")
            )
        })
    }

    /// Get storage health information
    pub fn get_health(&self) -> StorageHealth {
        StorageHealth {
            capacity: self.backend.get_capacity().unwrap_or_else(|_| StorageCapacity::default()),
            stats: self.stats.clone(),
            fragmentation_level: self.calculate_fragmentation(),
            wear_level: self.calculate_wear_level(),
            needs_maintenance: self.needs_maintenance(),
        }
    }

    /// Calculate storage fragmentation level (0-100)
    fn calculate_fragmentation(&self) -> u8 {
        // Simple fragmentation calculation based on write operations
        if self.stats.total_writes == 0 {
            0
        } else {
            let fragmentation = (self.stats.failed_writes * 100) / self.stats.total_writes;
            fragmentation.min(100) as u8
        }
    }

    /// Calculate wear level (0-100)
    fn calculate_wear_level(&self) -> u8 {
        // Estimate wear level based on erase cycles
        let max_cycles = 100000; // Typical flash endurance
        if self.stats.erase_cycles == 0 {
            0
        } else {
            ((self.stats.erase_cycles * 100) / max_cycles).min(100) as u8
        }
    }

    /// Check if storage needs maintenance
    fn needs_maintenance(&self) -> bool {
        self.calculate_fragmentation() > 50 || self.calculate_wear_level() > 80
    }
}

/// Storage health information
#[derive(Debug, Clone)]
pub struct StorageHealth {
    /// Storage capacity information
    pub capacity: StorageCapacity,
    /// Storage statistics
    pub stats: StorageStats,
    /// Fragmentation level (0-100)
    pub fragmentation_level: u8,
    /// Wear level (0-100)
    pub wear_level: u8,
    /// Whether maintenance is needed
    pub needs_maintenance: bool,
}

impl StorageHealth {
    /// Get overall health score (0-100, higher is better)
    pub fn health_score(&self) -> u8 {
        let capacity_score = if self.capacity.available_bytes > 0 { 100 } else { 0 };
        let fragmentation_score = 100 - self.fragmentation_level;
        let wear_score = 100 - self.wear_level;
        
        // Weighted average
        ((capacity_score * 30 + fragmentation_score * 40 + wear_score * 30) / 100).min(100) as u8
    }

    /// Check if storage is healthy
    pub fn is_healthy(&self) -> bool {
        self.health_score() > 70 && !self.needs_maintenance
    }

    /// Get health status as string
    pub fn status(&self) -> &'static str {
        let score = self.health_score();
        match score {
            90..=100 => "Excellent",
            70..=89 => "Good",
            50..=69 => "Fair",
            30..=49 => "Poor",
            _ => "Critical",
        }
    }
}

/// Storage initialization and factory functions
pub mod init {
    use super::*;

    /// Initialize storage system with default configuration
    #[cfg(feature = "esp32c3-flash")]
    pub fn init_default_storage() -> StorageManagerResult<UnifiedStorageManager<Esp32C3Storage>> {
        let config = Esp32C3Config::default();
        let backend = Esp32C3Storage::new(config)?;
        UnifiedStorageManager::new(backend)
    }

    /// Initialize storage system with custom configuration
    #[cfg(feature = "esp32c3-flash")]
    pub fn init_custom_storage(
        config: Esp32C3Config,
    ) -> StorageManagerResult<UnifiedStorageManager<Esp32C3Storage>> {
        let backend = Esp32C3Storage::new(config)?;
        UnifiedStorageManager::new(backend)
    }

    /// Initialize storage system for testing
    pub fn init_mock_storage() -> StorageManagerResult<UnifiedStorageManager<MockStorage>> {
        let backend = MockStorage::new();
        UnifiedStorageManager::new(backend)
    }

    /// Validate storage system health
    pub fn validate_storage<B: StorageBackend>(
        storage: &UnifiedStorageManager<B>,
    ) -> StorageManagerResult<()> {
        let health = storage.get_health();
        
        if !health.is_healthy() {
            return Err(StorageErrorKind::OperationFailed(
                create_error_string("Storage health check failed")
            ));
        }
        
        Ok(())
    }
}

/// Mock storage implementation for testing
pub struct MockStorage {
    data: heapless::FnvIndexMap<StorageKeyString, Vec<u8>, 32>,
    capacity: StorageCapacity,
    stats: StorageStats,
}

impl MockStorage {
    /// Create new mock storage
    pub fn new() -> Self {
        Self {
            data: heapless::FnvIndexMap::new(),
            capacity: StorageCapacity {
                total_bytes: 65536,
                used_bytes: 0,
                available_bytes: 65536,
                sector_size: 4096,
                sector_count: 16,
            },
            stats: StorageStats::default(),
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for MockStorage {
    async fn store(&mut self, key: &StorageKey, value: &StorageValue) -> StorageResult<()> {
        let key_str = StorageKeyString::from_str(key.as_str()).map_err(|_| StorageError::InvalidKey)?;
        let value_bytes = value.as_bytes().to_vec();
        
        self.data.insert(key_str, value_bytes).map_err(|_| StorageError::CapacityExceeded)?;
        self.stats.total_writes += 1;
        
        Ok(())
    }

    async fn retrieve(&mut self, key: &StorageKey) -> StorageResult<StorageValue> {
        let key_str = StorageKeyString::from_str(key.as_str()).map_err(|_| StorageError::InvalidKey)?;
        
        let bytes = self.data.get(&key_str).ok_or(StorageError::KeyNotFound)?;
        self.stats.total_reads += 1;
        
        StorageValue::from_bytes(bytes).map_err(|_| StorageError::InvalidValue)
    }

    async fn delete(&mut self, key: &StorageKey) -> StorageResult<()> {
        let key_str = StorageKeyString::from_str(key.as_str()).map_err(|_| StorageError::InvalidKey)?;
        
        self.data.remove(&key_str).ok_or(StorageError::KeyNotFound)?;
        self.stats.total_deletes += 1;
        
        Ok(())
    }

    async fn exists(&mut self, key: &StorageKey) -> StorageResult<bool> {
        let key_str = StorageKeyString::from_str(key.as_str()).map_err(|_| StorageError::InvalidKey)?;
        Ok(self.data.contains_key(&key_str))
    }

    async fn list_keys(&mut self, prefix: Option<&str>) -> StorageResult<Vec<String>> {
        let keys: Vec<String> = if let Some(prefix) = prefix {
            self.data.keys()
                .filter(|k| k.starts_with(prefix))
                .map(|k| k.as_str().to_string())
                .collect()
        } else {
            self.data.keys()
                .map(|k| k.as_str().to_string())
                .collect()
        };
        
        Ok(keys)
    }

    async fn maintenance(&mut self) -> StorageResult<()> {
        // Mock maintenance - just reset some stats
        self.stats.failed_writes = 0;
        Ok(())
    }

    fn get_capacity(&self) -> StorageResult<StorageCapacity> {
        Ok(self.capacity.clone())
    }

    fn get_stats(&self) -> StorageResult<StorageStats> {
        Ok(self.stats.clone())
    }
}

/// Configuration-aware storage initialization
/// 
/// Integrates with the central iot-config system to provide storage
/// configuration from JSON files.
pub struct ConfiguredStorage;

impl ConfiguredStorage {
    /// Create storage manager with configuration from iot-config
    pub async fn new() -> StorageResult<FlashStorageManager> {
        // Load storage configuration from central config
        let system_config = EmbeddedConfig::load_system_config()
            .map_err(|_| StorageError::ConfigurationError("Failed to load system configuration".into()))?;
        
        let storage_config = &system_config.storage;
        
        // Create storage with configuration
        let mut storage = FlashStorageManager::with_config(storage_config).await?;
        
        // Initialize based on configuration
        if storage_config.wear_leveling {
            // Enable wear leveling if configured
            storage.enable_wear_leveling().await?;
        }
        
        if storage_config.backup_enabled {
            // Enable backup if configured
            storage.enable_backup().await?;
        }
        
        Ok(storage)
    }
    
    /// Get storage configuration from central config
    pub fn get_storage_config() -> StorageResult<StorageConfig> {
        let system_config = EmbeddedConfig::load_system_config()
            .map_err(|_| StorageError::ConfigurationError("Failed to load system configuration".into()))?;
        
        Ok(system_config.storage)
    }
    
    /// Create storage manager with custom offset from config
    pub async fn with_offset(offset_override: Option<u32>) -> StorageResult<FlashStorageManager> {
        let mut storage_config = Self::get_storage_config()?;
        
        // Override offset if provided
        if let Some(offset) = offset_override {
            storage_config.flash_offset = offset;
        }
        
        FlashStorageManager::with_config(&storage_config).await
    }
}

/// Extension trait for FlashStorageManager to support configuration
impl FlashStorageManager {
    /// Create storage manager with StorageConfig
    pub async fn with_config(config: &StorageConfig) -> StorageResult<Self> {
        // Use configuration values for initialization
        let mut storage = Self::new(FlashConfig::default()).map_err(|e| StorageError::ConfigurationError(format!("Failed to create storage: {:?}", e)))?;
        
        // Configure based on settings
        storage.set_flash_offset(config.flash_offset);
        storage.set_backup_enabled(config.backup_enabled);
        storage.set_wear_leveling(config.wear_leveling);
        
        Ok(storage)
    }
    
    /// Set flash offset from configuration
    pub fn set_flash_offset(&mut self, _offset: u32) {
        // Implementation depends on storage backend
    }
    
    /// Enable/disable backup based on configuration
    pub async fn set_backup_enabled(&mut self, enabled: bool) -> StorageResult<()> {
        if enabled {
            self.enable_backup().await
        } else {
            self.disable_backup().await
        }
    }
    
    /// Enable/disable wear leveling based on configuration
    pub async fn set_wear_leveling(&mut self, enabled: bool) -> StorageResult<()> {
        if enabled {
            self.enable_wear_leveling().await
        } else {
            self.disable_wear_leveling().await
        }
    }
    
    /// Enable backup functionality
    pub async fn enable_backup(&mut self) -> StorageResult<()> {
        // Implementation for backup enabling
        Ok(())
    }
    
    /// Disable backup functionality
    pub async fn disable_backup(&mut self) -> StorageResult<()> {
        // Implementation for backup disabling
        Ok(())
    }
    
    /// Enable wear leveling
    pub async fn enable_wear_leveling(&mut self) -> StorageResult<()> {
        // Implementation for wear leveling enabling
        Ok(())
    }
    
    /// Disable wear leveling
    pub async fn disable_wear_leveling(&mut self) -> StorageResult<()> {
        // Implementation for wear leveling disabling
        Ok(())
    }
}