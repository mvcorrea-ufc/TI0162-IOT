//! # Flash Storage Integration
//!
//! This module provides integration with the flash storage system for persistent
//! configuration management. It handles serialization, storage, and retrieval
//! of configuration data from ESP32-C3 flash memory.

use heapless::Vec;
use serde::{Deserialize, Serialize};
use alloc::vec;
use crate::{IoTSystemConfig, ConfigResult, ConfigErrorKind, ConfigString};

/// Flash storage configuration constants
pub const FLASH_CONFIG_OFFSET: u32 = 0x310000; // Flash offset for configuration storage
pub const FLASH_CONFIG_SIZE: usize = 1024;     // Size allocated for configuration
pub const CONFIG_MAGIC_BYTES: [u8; 4] = [0xCA, 0xFE, 0xC0, 0xDE]; // Magic bytes for validation
pub const CONFIG_VERSION: u8 = 1;              // Configuration format version

/// Configuration storage trait for different storage backends
pub trait ConfigStorage {
    /// Error type for storage operations
    type Error;

    /// Save configuration to storage
    fn save_config(&mut self, config: &IoTSystemConfig) -> Result<(), Self::Error>;

    /// Load configuration from storage
    fn load_config(&mut self) -> Result<IoTSystemConfig, Self::Error>;

    /// Check if configuration exists in storage
    fn config_exists(&mut self) -> Result<bool, Self::Error>;

    /// Clear configuration from storage
    fn clear_config(&mut self) -> Result<(), Self::Error>;

    /// Get storage capacity information
    fn get_capacity(&self) -> StorageCapacity;
}

/// Storage capacity information
#[derive(Debug, Clone)]
pub struct StorageCapacity {
    /// Total storage size in bytes
    pub total_bytes: usize,
    /// Used storage size in bytes
    pub used_bytes: usize,
    /// Available storage size in bytes
    pub available_bytes: usize,
}

impl StorageCapacity {
    /// Create new storage capacity info
    pub fn new(total: usize, used: usize) -> Self {
        Self {
            total_bytes: total,
            used_bytes: used,
            available_bytes: total.saturating_sub(used),
        }
    }

    /// Get usage percentage
    pub fn usage_percentage(&self) -> u8 {
        if self.total_bytes == 0 {
            0
        } else {
            ((self.used_bytes * 100) / self.total_bytes) as u8
        }
    }

    /// Check if storage is nearly full (>90%)
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percentage() > 90
    }
}

/// Flash configuration manager for ESP32-C3
#[cfg(feature = "flash-storage")]
pub struct FlashConfigManager {
    /// Flash storage instance
    storage: esp_storage::FlashStorage,
}

#[cfg(feature = "flash-storage")]
impl FlashConfigManager {
    /// Create new flash configuration manager
    pub fn new() -> Self {
        Self {
            storage: esp_storage::FlashStorage::new(),
        }
    }

    /// Serialize configuration to binary format
    fn serialize_config(&self, config: &IoTSystemConfig) -> ConfigResult<Vec<u8, FLASH_CONFIG_SIZE>> {
        // Create header with magic bytes and version
        let mut buffer = Vec::new();
        
        // Add magic bytes
        for &byte in &CONFIG_MAGIC_BYTES {
            buffer.push(byte).map_err(|_| {
                ConfigErrorKind::SerializationFailed(
                    ConfigString::from("Buffer overflow").unwrap_or_default()
                )
            })?;
        }
        
        // Add version
        buffer.push(CONFIG_VERSION).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("Buffer overflow").unwrap_or_default()
            )
        })?;
        
        // Reserve space for data length (2 bytes)
        buffer.push(0).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("Buffer overflow").unwrap_or_default()
            )
        })?;
        buffer.push(0).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("Buffer overflow").unwrap_or_default()
            )
        })?;
        
        // Serialize configuration data
        let serialized = serde_json_core::to_vec(config).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("JSON serialization failed").unwrap_or_default()
            )
        })?;
        
        // Check if serialized data fits
        if serialized.len() > FLASH_CONFIG_SIZE - 8 { // Reserve space for header and checksum
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Configuration too large").unwrap_or_default()
            ));
        }
        
        // Update data length in header
        let len_bytes = (serialized.len() as u16).to_le_bytes();
        buffer[5] = len_bytes[0];
        buffer[6] = len_bytes[1];
        
        // Add serialized data
        for &byte in serialized.iter() {
            buffer.push(byte).map_err(|_| {
                ConfigErrorKind::SerializationFailed(
                    ConfigString::from("Buffer overflow").unwrap_or_default()
                )
            })?;
        }
        
        // Add checksum (simple XOR checksum for now)
        let checksum = self.calculate_checksum(&buffer[7..]);
        buffer.push(checksum).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("Buffer overflow").unwrap_or_default()
            )
        })?;
        
        Ok(buffer)
    }

    /// Deserialize configuration from binary format
    fn deserialize_config(&self, data: &[u8]) -> ConfigResult<IoTSystemConfig> {
        // Check minimum size
        if data.len() < 8 {
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Data too short").unwrap_or_default()
            ));
        }
        
        // Check magic bytes
        if data[0..4] != CONFIG_MAGIC_BYTES {
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Invalid magic bytes").unwrap_or_default()
            ));
        }
        
        // Check version
        if data[4] != CONFIG_VERSION {
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Unsupported version").unwrap_or_default()
            ));
        }
        
        // Get data length
        let data_len = u16::from_le_bytes([data[5], data[6]]) as usize;
        
        // Validate data length
        if data_len > data.len() - 8 {
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Invalid data length").unwrap_or_default()
            ));
        }
        
        // Extract and verify checksum
        let config_data = &data[7..7 + data_len];
        let stored_checksum = data[7 + data_len];
        let calculated_checksum = self.calculate_checksum(config_data);
        
        if stored_checksum != calculated_checksum {
            return Err(ConfigErrorKind::SerializationFailed(
                ConfigString::from("Checksum mismatch").unwrap_or_default()
            ));
        }
        
        // Deserialize configuration
        serde_json_core::from_slice(config_data).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("JSON deserialization failed").unwrap_or_default()
            )
        }).map(|(config, _)| config)
    }

    /// Calculate simple XOR checksum
    fn calculate_checksum(&self, data: &[u8]) -> u8 {
        data.iter().fold(0, |acc, &byte| acc ^ byte)
    }

    /// Check if flash region appears to be erased
    fn is_flash_erased(&mut self, offset: u32, size: usize) -> Result<bool, esp_storage::FlashStorageError> {
        let mut buffer = vec![0u8; size.min(256)]; // Check in chunks
        let mut chunks_checked = 0;
        let max_chunks = (size + buffer.len() - 1) / buffer.len();
        
        for chunk_offset in (0..size).step_by(buffer.len()) {
            let chunk_size = (size - chunk_offset).min(buffer.len());
            buffer.resize(chunk_size, 0);
            
            self.storage.read(offset + chunk_offset as u32, &mut buffer)?;
            
            // Check if all bytes are 0xFF (erased state)
            if buffer.iter().all(|&b| b == 0xFF) {
                chunks_checked += 1;
                if chunks_checked > max_chunks / 4 { // Check at least 25% of the area
                    return Ok(true);
                }
            } else {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

#[cfg(feature = "flash-storage")]
impl ConfigStorage for FlashConfigManager {
    type Error = ConfigErrorKind;

    fn save_config(&mut self, config: &IoTSystemConfig) -> Result<(), Self::Error> {
        // Serialize configuration
        let serialized = self.serialize_config(config)?;
        
        // Create buffer for flash write (must be aligned and sized correctly)
        let mut flash_buffer = [0xFFu8; FLASH_CONFIG_SIZE];
        
        // Copy serialized data to flash buffer
        let data_len = serialized.len().min(FLASH_CONFIG_SIZE);
        flash_buffer[..data_len].copy_from_slice(&serialized[..data_len]);
        
        // Write to flash
        self.storage.write(FLASH_CONFIG_OFFSET, &flash_buffer).map_err(|e| {
            ConfigErrorKind::StorageFailed(
                ConfigString::from(match e {
                    esp_storage::FlashStorageError::OutOfBounds => "Flash out of bounds",
                    esp_storage::FlashStorageError::Alignment => "Flash alignment error",
                    esp_storage::FlashStorageError::Other => "Flash write error",
                }).unwrap_or_default()
            )
        })?;
        
        Ok(())
    }

    fn load_config(&mut self) -> Result<IoTSystemConfig, Self::Error> {
        // Read from flash
        let mut buffer = [0u8; FLASH_CONFIG_SIZE];
        self.storage.read(FLASH_CONFIG_OFFSET, &mut buffer).map_err(|e| {
            ConfigErrorKind::StorageFailed(
                ConfigString::from(match e {
                    esp_storage::FlashStorageError::OutOfBounds => "Flash out of bounds",
                    esp_storage::FlashStorageError::Alignment => "Flash alignment error", 
                    esp_storage::FlashStorageError::Other => "Flash read error",
                }).unwrap_or_default()
            )
        })?;
        
        // Find actual data length by looking for non-0xFF bytes
        let mut actual_len = FLASH_CONFIG_SIZE;
        for (i, &byte) in buffer.iter().enumerate().rev() {
            if byte != 0xFF {
                actual_len = i + 1;
                break;
            }
        }
        
        // Deserialize configuration
        self.deserialize_config(&buffer[..actual_len])
    }

    fn config_exists(&mut self) -> Result<bool, Self::Error> {
        // Read just the header to check for magic bytes
        let mut header = [0u8; 8];
        self.storage.read(FLASH_CONFIG_OFFSET, &mut header).map_err(|_| {
            ConfigErrorKind::StorageFailed(
                ConfigString::from("Flash read error").unwrap_or_default()
            )
        })?;
        
        // Check for magic bytes
        Ok(header[0..4] == CONFIG_MAGIC_BYTES)
    }

    fn clear_config(&mut self) -> Result<(), Self::Error> {
        // Erase flash region by writing all 0xFF
        let erase_buffer = [0xFFu8; FLASH_CONFIG_SIZE];
        self.storage.write(FLASH_CONFIG_OFFSET, &erase_buffer).map_err(|_| {
            ConfigErrorKind::StorageFailed(
                ConfigString::from("Flash erase error").unwrap_or_default()
            )
        })?;
        
        Ok(())
    }

    fn get_capacity(&self) -> StorageCapacity {
        StorageCapacity::new(FLASH_CONFIG_SIZE, 0) // Used size would need to be calculated
    }
}

/// Mock storage implementation for testing
pub struct MockStorage {
    config: Option<IoTSystemConfig>,
    capacity: StorageCapacity,
}

impl MockStorage {
    /// Create new mock storage
    pub fn new() -> Self {
        Self {
            config: None,
            capacity: StorageCapacity::new(FLASH_CONFIG_SIZE, 0),
        }
    }

    /// Set stored configuration (for testing)
    pub fn set_config(&mut self, config: IoTSystemConfig) {
        self.config = Some(config);
        self.capacity.used_bytes = 256; // Simulate some usage
        self.capacity.available_bytes = self.capacity.total_bytes - self.capacity.used_bytes;
    }
}

impl ConfigStorage for MockStorage {
    type Error = ConfigErrorKind;

    fn save_config(&mut self, config: &IoTSystemConfig) -> Result<(), Self::Error> {
        self.config = Some(config.clone());
        self.capacity.used_bytes = 256; // Simulate storage usage
        self.capacity.available_bytes = self.capacity.total_bytes - self.capacity.used_bytes;
        Ok(())
    }

    fn load_config(&mut self) -> Result<IoTSystemConfig, Self::Error> {
        self.config.clone().ok_or_else(|| {
            ConfigErrorKind::StorageFailed(
                ConfigString::from("No config found").unwrap_or_default()
            )
        })
    }

    fn config_exists(&mut self) -> Result<bool, Self::Error> {
        Ok(self.config.is_some())
    }

    fn clear_config(&mut self) -> Result<(), Self::Error> {
        self.config = None;
        self.capacity.used_bytes = 0;
        self.capacity.available_bytes = self.capacity.total_bytes;
        Ok(())
    }

    fn get_capacity(&self) -> StorageCapacity {
        self.capacity.clone()
    }
}

/// Configuration persistence manager
pub struct ConfigPersistenceManager<S: ConfigStorage> {
    storage: S,
    cached_config: Option<IoTSystemConfig>,
    dirty: bool,
}

impl<S: ConfigStorage> ConfigPersistenceManager<S> {
    /// Create new persistence manager
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            cached_config: None,
            dirty: false,
        }
    }

    /// Load configuration (uses cache if available)
    pub fn load(&mut self) -> Result<IoTSystemConfig, S::Error> {
        if let Some(ref config) = self.cached_config {
            if !self.dirty {
                return Ok(config.clone());
            }
        }

        let config = self.storage.load_config()?;
        self.cached_config = Some(config.clone());
        self.dirty = false;
        Ok(config)
    }

    /// Save configuration
    pub fn save(&mut self, config: &IoTSystemConfig) -> Result<(), S::Error> {
        self.storage.save_config(config)?;
        self.cached_config = Some(config.clone());
        self.dirty = false;
        Ok(())
    }

    /// Update cached configuration (marks as dirty)
    pub fn update_cache(&mut self, config: IoTSystemConfig) {
        self.cached_config = Some(config);
        self.dirty = true;
    }

    /// Check if configuration exists
    pub fn exists(&mut self) -> Result<bool, S::Error> {
        self.storage.config_exists()
    }

    /// Clear configuration
    pub fn clear(&mut self) -> Result<(), S::Error> {
        self.storage.clear_config()?;
        self.cached_config = None;
        self.dirty = false;
        Ok(())
    }

    /// Force reload from storage (bypasses cache)
    pub fn reload(&mut self) -> Result<IoTSystemConfig, S::Error> {
        let config = self.storage.load_config()?;
        self.cached_config = Some(config.clone());
        self.dirty = false;
        Ok(config)
    }

    /// Get storage capacity
    pub fn get_capacity(&self) -> StorageCapacity {
        self.storage.get_capacity()
    }

    /// Check if cached config is dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Get cached configuration if available
    pub fn get_cached(&self) -> Option<&IoTSystemConfig> {
        self.cached_config.as_ref()
    }
}

/// Storage utilities
pub mod storage_utils {
    use super::*;

    /// Create flash storage manager (only available with flash-storage feature)
    #[cfg(feature = "flash-storage")]
    pub fn create_flash_manager() -> FlashConfigManager {
        FlashConfigManager::new()
    }

    /// Create mock storage manager for testing
    pub fn create_mock_manager() -> MockStorage {
        MockStorage::new()
    }

    /// Backup configuration to secondary storage
    pub fn backup_config<S1, S2>(
        primary: &mut S1,
        backup: &mut S2,
    ) -> Result<(), S1::Error>
    where
        S1: ConfigStorage,
        S2: ConfigStorage<Error = S1::Error>,
    {
        let config = primary.load_config()?;
        backup.save_config(&config)?;
        Ok(())
    }

    /// Validate flash region before writing
    #[cfg(feature = "flash-storage")]
    pub fn validate_flash_region() -> Result<bool, esp_storage::FlashStorageError> {
        let mut storage = esp_storage::FlashStorage::new();
        let mut test_buffer = [0u8; 64];
        
        // Try to read from the configuration offset
        match storage.read(FLASH_CONFIG_OFFSET, &mut test_buffer) {
            Ok(()) => Ok(true),
            Err(e) => {
                // Check if it's just an out of bounds error vs hardware issue
                match e {
                    esp_storage::FlashStorageError::OutOfBounds => Ok(false),
                    _ => Err(e),
                }
            }
        }
    }

    /// Estimate configuration size in bytes
    pub fn estimate_config_size(config: &IoTSystemConfig) -> Result<usize, ConfigErrorKind> {
        let serialized = serde_json_core::to_vec(config).map_err(|_| {
            ConfigErrorKind::SerializationFailed(
                ConfigString::from("Size estimation failed").unwrap_or_default()
            )
        })?;
        
        // Add overhead for header, checksum, alignment
        Ok(serialized.len() + 16)
    }
}