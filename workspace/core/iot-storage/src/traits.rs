//! # Storage Abstraction Traits
//!
//! This module defines the core traits and types for the storage abstraction layer.
//! These traits provide a unified interface for different storage backends while
//! maintaining efficiency and type safety in embedded environments.

use heapless::{String, Vec};
use serde::{Deserialize, Serialize};
use alloc::{boxed::Box, string::String as AllocString};
use crate::MAX_KEY_LEN;

/// Maximum value size for storage operations
const MAX_VALUE_SIZE: usize = 4096;

/// Storage key type for embedded environment
pub type StorageKeyString = String<MAX_KEY_LEN>;

/// Storage operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    /// Key not found in storage
    KeyNotFound,
    /// Invalid key format or length
    InvalidKey,
    /// Invalid value format or size
    InvalidValue,
    /// Storage capacity exceeded
    CapacityExceeded,
    /// Hardware or I/O error
    HardwareError,
    /// Data corruption detected
    CorruptedData,
    /// Operation timed out
    Timeout,
    /// Storage is in read-only mode
    ReadOnly,
    /// Transaction conflict
    TransactionConflict,
    /// Wear leveling failure
    WearLevelingError,
    /// Configuration error
    ConfigurationError(AllocString),
    /// Unknown error
    Unknown,
}

/// Storage operation result type
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage key wrapper with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StorageKey {
    key: StorageKeyString,
}

impl StorageKey {
    /// Create new storage key from string
    pub fn new(key: &str) -> StorageResult<Self> {
        if key.is_empty() {
            return Err(StorageError::InvalidKey);
        }
        
        if key.len() > MAX_KEY_LEN {
            return Err(StorageError::InvalidKey);
        }
        
        // Validate key characters (alphanumeric, underscore, dash, dot)
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
            return Err(StorageError::InvalidKey);
        }
        
        let key_string = StorageKeyString::try_from(key).map_err(|_| StorageError::InvalidKey)?;
        
        Ok(Self { key: key_string })
    }

    /// Create storage key from string (fallible constructor)
    pub fn from_str(key: &str) -> StorageResult<Self> {
        Self::new(key)
    }

    /// Get key as string slice
    pub fn as_str(&self) -> &str {
        &self.key
    }

    /// Get key length
    pub fn len(&self) -> usize {
        self.key.len()
    }

    /// Check if key is empty
    pub fn is_empty(&self) -> bool {
        self.key.is_empty()
    }

    /// Check if key starts with prefix
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.key.starts_with(prefix)
    }

    /// Check if key ends with suffix
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.key.ends_with(suffix)
    }
}

/// Storage value wrapper with size validation
#[derive(Debug, Clone)]
pub struct StorageValue {
    data: Vec<u8, MAX_VALUE_SIZE>,
}

impl StorageValue {
    /// Create new storage value from bytes
    pub fn new(data: &[u8]) -> StorageResult<Self> {
        if data.len() > MAX_VALUE_SIZE {
            return Err(StorageError::InvalidValue);
        }
        
        let mut value_data = Vec::new();
        for &byte in data {
            value_data.push(byte).map_err(|_| StorageError::InvalidValue)?;
        }
        
        Ok(Self { data: value_data })
    }

    /// Create storage value from bytes (fallible constructor)
    pub fn from_bytes(data: &[u8]) -> StorageResult<Self> {
        Self::new(data)
    }

    /// Create storage value from serializable object
    pub fn from_serializable<T: Serialize>(value: &T) -> StorageResult<Self> {
        let serialized: Vec<u8, 4096> = serde_json_core::to_vec(value).map_err(|_| StorageError::InvalidValue)?;
        Self::from_bytes(&serialized)
    }

    /// Get value as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get value size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if value is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Deserialize value as specific type
    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> StorageResult<T> {
        let (value, _) = serde_json_core::from_slice(&self.data)
            .map_err(|_| StorageError::CorruptedData)?;
        Ok(value)
    }
}

/// Storage capacity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCapacity {
    /// Total storage capacity in bytes
    pub total_bytes: usize,
    /// Currently used storage in bytes
    pub used_bytes: usize,
    /// Available storage in bytes
    pub available_bytes: usize,
    /// Storage sector size in bytes
    pub sector_size: usize,
    /// Number of storage sectors
    pub sector_count: usize,
}

impl StorageCapacity {
    /// Create new storage capacity info
    pub fn new(total: usize, used: usize, sector_size: usize) -> Self {
        let sector_count = total / sector_size;
        Self {
            total_bytes: total,
            used_bytes: used,
            available_bytes: total.saturating_sub(used),
            sector_size,
            sector_count,
        }
    }

    /// Get usage percentage (0-100)
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

    /// Check if storage is critically full (>95%)
    pub fn is_critically_full(&self) -> bool {
        self.usage_percentage() > 95
    }

    /// Get free space in bytes
    pub fn free_bytes(&self) -> usize {
        self.available_bytes
    }

    /// Get free space percentage
    pub fn free_percentage(&self) -> u8 {
        100 - self.usage_percentage()
    }
}

impl Default for StorageCapacity {
    fn default() -> Self {
        Self::new(65536, 0, 4096) // Default 64KB with 4KB sectors
    }
}

/// Storage operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of read operations
    pub total_reads: u64,
    /// Total number of write operations
    pub total_writes: u64,
    /// Total number of delete operations
    pub total_deletes: u64,
    /// Number of failed read operations
    pub failed_reads: u64,
    /// Number of failed write operations
    pub failed_writes: u64,
    /// Number of failed delete operations
    pub failed_deletes: u64,
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Number of erase cycles performed
    pub erase_cycles: u64,
    /// Last operation timestamp (implementation-defined)
    pub last_operation_time: u64,
}

impl StorageStats {
    /// Create new storage statistics
    pub fn new() -> Self {
        Self {
            total_reads: 0,
            total_writes: 0,
            total_deletes: 0,
            failed_reads: 0,
            failed_writes: 0,
            failed_deletes: 0,
            bytes_read: 0,
            bytes_written: 0,
            erase_cycles: 0,
            last_operation_time: 0,
        }
    }

    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.total_reads + self.total_writes + self.total_deletes
    }

    /// Get total failed operations count
    pub fn total_failures(&self) -> u64 {
        self.failed_reads + self.failed_writes + self.failed_deletes
    }

    /// Get failure rate percentage
    pub fn failure_rate_percentage(&self) -> f32 {
        let total = self.total_operations();
        if total == 0 {
            0.0
        } else {
            (self.total_failures() as f32 / total as f32) * 100.0
        }
    }

    /// Get read success rate percentage
    pub fn read_success_rate(&self) -> f32 {
        if self.total_reads == 0 {
            100.0
        } else {
            ((self.total_reads - self.failed_reads) as f32 / self.total_reads as f32) * 100.0
        }
    }

    /// Get write success rate percentage
    pub fn write_success_rate(&self) -> f32 {
        if self.total_writes == 0 {
            100.0
        } else {
            ((self.total_writes - self.failed_writes) as f32 / self.total_writes as f32) * 100.0
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for StorageStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Main storage backend trait
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a key-value pair
    async fn store(&mut self, key: &StorageKey, value: &StorageValue) -> StorageResult<()>;

    /// Retrieve value by key
    async fn retrieve(&mut self, key: &StorageKey) -> StorageResult<StorageValue>;

    /// Delete key-value pair
    async fn delete(&mut self, key: &StorageKey) -> StorageResult<()>;

    /// Check if key exists
    async fn exists(&mut self, key: &StorageKey) -> StorageResult<bool>;

    /// List all keys, optionally filtered by prefix
    async fn list_keys(&mut self, prefix: Option<&str>) -> StorageResult<alloc::vec::Vec<alloc::string::String>>;

    /// Perform maintenance operations (garbage collection, defragmentation)
    async fn maintenance(&mut self) -> StorageResult<()>;

    /// Get storage capacity information
    fn get_capacity(&self) -> StorageResult<StorageCapacity>;

    /// Get storage operation statistics
    fn get_stats(&self) -> StorageResult<StorageStats>;
}

/// Configuration-specific storage trait
#[async_trait::async_trait]
pub trait ConfigStorage: Send + Sync {
    /// Store configuration by name
    async fn store_config<T>(&mut self, name: &str, config: &T) -> StorageResult<()>
    where
        T: Serialize + Send + Sync;

    /// Retrieve configuration by name
    async fn retrieve_config<T>(&mut self, name: &str) -> StorageResult<T>
    where
        T: serde::de::DeserializeOwned + Send + Sync;

    /// Delete configuration by name
    async fn delete_config(&mut self, name: &str) -> StorageResult<()>;

    /// Check if configuration exists
    async fn config_exists(&mut self, name: &str) -> StorageResult<bool>;

    /// List all configuration names
    async fn list_configs(&mut self) -> StorageResult<alloc::vec::Vec<alloc::string::String>>;

    /// Backup configuration to secondary location
    async fn backup_config(&mut self, name: &str) -> StorageResult<()>;

    /// Restore configuration from backup
    async fn restore_config(&mut self, name: &str) -> StorageResult<()>;
}

/// Atomic storage operations trait
#[async_trait::async_trait]
pub trait AtomicStorage: Send + Sync {
    /// Begin atomic transaction
    async fn begin_transaction(&mut self) -> StorageResult<TransactionId>;

    /// Commit transaction
    async fn commit_transaction(&mut self, transaction_id: TransactionId) -> StorageResult<()>;

    /// Rollback transaction
    async fn rollback_transaction(&mut self, transaction_id: TransactionId) -> StorageResult<()>;

    /// Store value within transaction
    async fn atomic_store(
        &mut self,
        transaction_id: TransactionId,
        key: &StorageKey,
        value: &StorageValue,
    ) -> StorageResult<()>;

    /// Retrieve value within transaction
    async fn atomic_retrieve(
        &mut self,
        transaction_id: TransactionId,
        key: &StorageKey,
    ) -> StorageResult<StorageValue>;

    /// Delete value within transaction
    async fn atomic_delete(
        &mut self,
        transaction_id: TransactionId,
        key: &StorageKey,
    ) -> StorageResult<()>;
}

/// Transaction identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(pub u32);

impl TransactionId {
    /// Create new transaction ID
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get transaction ID value
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Storage maintenance operations trait
pub trait StorageMaintenance {
    /// Defragment storage to reclaim space
    fn defragment(&mut self) -> StorageResult<usize>;

    /// Garbage collect deleted entries
    fn garbage_collect(&mut self) -> StorageResult<usize>;

    /// Verify data integrity
    fn verify_integrity(&mut self) -> StorageResult<Vec<StorageKey, {crate::MAX_KEYS}>>;

    /// Repair corrupted data
    fn repair_data(&mut self, keys: &[StorageKey]) -> StorageResult<usize>;

    /// Get fragmentation level (0-100)
    fn get_fragmentation_level(&self) -> u8;

    /// Check if maintenance is needed
    fn needs_maintenance(&self) -> bool;
}

/// Wear leveling operations trait
pub trait WearLeveling {
    /// Get wear level for specific region (0-100)
    fn get_wear_level(&self, region: usize) -> u8;

    /// Get average wear level across all regions
    fn get_average_wear_level(&self) -> u8;

    /// Perform wear leveling operation
    fn level_wear(&mut self) -> StorageResult<()>;

    /// Get bad block count
    fn get_bad_block_count(&self) -> usize;

    /// Mark block as bad
    fn mark_bad_block(&mut self, block: usize) -> StorageResult<()>;

    /// Get estimated remaining lifetime (0-100)
    fn get_remaining_lifetime(&self) -> u8;
}

/// Storage encryption trait (optional feature)
#[cfg(feature = "encryption")]
pub trait StorageEncryption {
    /// Set encryption key
    fn set_encryption_key(&mut self, key: &[u8; 32]) -> StorageResult<()>;

    /// Enable/disable encryption
    fn set_encryption_enabled(&mut self, enabled: bool);

    /// Check if encryption is enabled
    fn is_encryption_enabled(&self) -> bool;

    /// Rotate encryption key
    fn rotate_encryption_key(&mut self, new_key: &[u8; 32]) -> StorageResult<()>;
}

/// Storage compression trait (optional feature)
#[cfg(feature = "compression")]
pub trait StorageCompression {
    /// Set compression algorithm
    fn set_compression_algorithm(&mut self, algorithm: CompressionAlgorithm);

    /// Enable/disable compression
    fn set_compression_enabled(&mut self, enabled: bool);

    /// Check if compression is enabled
    fn is_compression_enabled(&self) -> bool;

    /// Get compression ratio
    fn get_compression_ratio(&self) -> f32;
}

/// Compression algorithms
#[cfg(feature = "compression")]
#[derive(Debug, Clone, Copy)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 compression (fast)
    Lz4,
    /// DEFLATE compression (balanced)
    Deflate,
    /// LZMA compression (high ratio)
    Lzma,
}

/// Storage event listener trait
pub trait StorageEventListener {
    /// Called when a key is stored
    fn on_store(&mut self, key: &StorageKey, size: usize);

    /// Called when a key is retrieved
    fn on_retrieve(&mut self, key: &StorageKey, size: usize);

    /// Called when a key is deleted
    fn on_delete(&mut self, key: &StorageKey);

    /// Called when an error occurs
    fn on_error(&mut self, error: &StorageError, operation: &str);

    /// Called when maintenance is performed
    fn on_maintenance(&mut self, operation: &str, recovered_bytes: usize);
}

/// Helper macros for storage operations
#[macro_export]
macro_rules! storage_key {
    ($key:expr) => {
        $crate::traits::StorageKey::new($key).expect("Invalid storage key")
    };
}

#[macro_export]
macro_rules! storage_value {
    ($data:expr) => {
        $crate::traits::StorageValue::from_bytes($data).expect("Invalid storage value")
    };
}

/// Storage utilities
pub mod utils {
    use super::*;
    use alloc::format;

    /// Validate key format
    pub fn validate_key(key: &str) -> bool {
        !key.is_empty() 
            && key.len() <= MAX_KEY_LEN
            && key.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    }

    /// Sanitize key by removing invalid characters
    pub fn sanitize_key(key: &str) -> String<MAX_KEY_LEN> {
        let sanitized: String<MAX_KEY_LEN> = key
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .take(MAX_KEY_LEN)
            .collect();
        
        if sanitized.is_empty() {
            String::try_from("default").unwrap_or_default()
        } else {
            sanitized
        }
    }

    /// Calculate checksum for data integrity
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        let mut checksum = 0u32;
        for &byte in data {
            checksum = checksum.wrapping_add(byte as u32);
            checksum = checksum.wrapping_mul(1103515245);
            checksum = checksum.wrapping_add(12345);
        }
        checksum
    }

    /// Verify data integrity using checksum
    pub fn verify_checksum(data: &[u8], expected: u32) -> bool {
        calculate_checksum(data) == expected
    }

    /// Format bytes as human-readable string
    pub fn format_bytes(bytes: usize) -> String<16> {
        if bytes < 1024 {
            String::try_from(format!("{}B", bytes).as_str()).unwrap_or_default()
        } else if bytes < 1024 * 1024 {
            String::try_from(format!("{}KB", bytes / 1024).as_str()).unwrap_or_default()
        } else {
            String::try_from(format!("{}MB", bytes / (1024 * 1024)).as_str()).unwrap_or_default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String as AllocString;

    #[test]
    fn test_storage_key_creation() {
        // Valid keys
        assert!(StorageKey::new("valid_key").is_ok());
        assert!(StorageKey::new("valid-key").is_ok());
        assert!(StorageKey::new("valid.key").is_ok());
        assert!(StorageKey::new("ValidKey123").is_ok());
        assert!(StorageKey::new("a").is_ok());
        
        // Invalid keys
        assert!(StorageKey::new("").is_err());
        assert!(StorageKey::new("invalid key").is_err());
        assert!(StorageKey::new("invalid@key").is_err());
        assert!(StorageKey::new("invalid#key").is_err());
        assert!(StorageKey::new(&"a".repeat(MAX_KEY_LEN + 1)).is_err());
    }

    #[test]
    fn test_storage_key_properties() {
        let key = StorageKey::new("test_key").unwrap();
        assert_eq!(key.as_str(), "test_key");
        assert_eq!(key.len(), 8);
        assert!(!key.is_empty());
        assert!(key.starts_with("test"));
        assert!(key.ends_with("key"));
    }

    #[test]
    fn test_storage_value_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let value = StorageValue::new(&data).unwrap();
        assert_eq!(value.as_bytes(), &data);
        assert_eq!(value.len(), 5);
        assert!(!value.is_empty());
        
        // Test large value
        let large_data = vec![0u8; MAX_VALUE_SIZE];
        assert!(StorageValue::new(&large_data).is_ok());
        
        // Test oversized value
        let oversized_data = vec![0u8; MAX_VALUE_SIZE + 1];
        assert!(StorageValue::new(&oversized_data).is_err());
    }

    #[test]
    fn test_storage_value_serialization() {
        use serde::{Deserialize, Serialize};
        
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            id: u32,
            name: AllocString,
        }
        
        let test_data = TestData {
            id: 42,
            name: AllocString::from("test"),
        };
        
        let value = StorageValue::from_serializable(&test_data).unwrap();
        let deserialized: TestData = value.deserialize().unwrap();
        
        assert_eq!(test_data, deserialized);
    }

    #[test]
    fn test_storage_capacity() {
        let capacity = StorageCapacity::new(65536, 16384, 4096);
        
        assert_eq!(capacity.total_bytes, 65536);
        assert_eq!(capacity.used_bytes, 16384);
        assert_eq!(capacity.available_bytes, 49152);
        assert_eq!(capacity.sector_size, 4096);
        assert_eq!(capacity.sector_count, 16);
        assert_eq!(capacity.usage_percentage(), 25);
        assert_eq!(capacity.free_percentage(), 75);
        assert!(!capacity.is_nearly_full());
        assert!(!capacity.is_critically_full());
    }

    #[test]
    fn test_storage_capacity_edge_cases() {
        // Full storage
        let full_capacity = StorageCapacity::new(1000, 1000, 100);
        assert_eq!(full_capacity.usage_percentage(), 100);
        assert_eq!(full_capacity.free_percentage(), 0);
        assert!(full_capacity.is_nearly_full());
        assert!(full_capacity.is_critically_full());
        
        // Empty storage
        let empty_capacity = StorageCapacity::new(1000, 0, 100);
        assert_eq!(empty_capacity.usage_percentage(), 0);
        assert_eq!(empty_capacity.free_percentage(), 100);
        assert!(!empty_capacity.is_nearly_full());
        assert!(!empty_capacity.is_critically_full());
        
        // Zero total capacity
        let zero_capacity = StorageCapacity::new(0, 0, 100);
        assert_eq!(zero_capacity.usage_percentage(), 0);
    }

    #[test]
    fn test_storage_stats() {
        let mut stats = StorageStats::new();
        
        assert_eq!(stats.total_operations(), 0);
        assert_eq!(stats.total_failures(), 0);
        assert_eq!(stats.failure_rate_percentage(), 0.0);
        assert_eq!(stats.read_success_rate(), 100.0);
        assert_eq!(stats.write_success_rate(), 100.0);
        
        // Simulate some operations
        stats.total_reads = 100;
        stats.total_writes = 50;
        stats.total_deletes = 25;
        stats.failed_reads = 5;
        stats.failed_writes = 2;
        stats.failed_deletes = 1;
        
        assert_eq!(stats.total_operations(), 175);
        assert_eq!(stats.total_failures(), 8);
        assert!((stats.failure_rate_percentage() - 4.57).abs() < 0.1);
        assert_eq!(stats.read_success_rate(), 95.0);
        assert_eq!(stats.write_success_rate(), 96.0);
        
        // Test reset
        stats.reset();
        assert_eq!(stats.total_operations(), 0);
    }

    #[test]
    fn test_transaction_id() {
        let tx_id = TransactionId::new(42);
        assert_eq!(tx_id.value(), 42);
        
        let tx_id2 = TransactionId::new(42);
        assert_eq!(tx_id, tx_id2);
    }

    #[test]
    fn test_utils_validate_key() {
        assert!(utils::validate_key("valid_key"));
        assert!(utils::validate_key("valid-key"));
        assert!(utils::validate_key("valid.key"));
        assert!(utils::validate_key("ValidKey123"));
        
        assert!(!utils::validate_key(""));
        assert!(!utils::validate_key("invalid key"));
        assert!(!utils::validate_key("invalid@key"));
        assert!(!utils::validate_key(&"a".repeat(MAX_KEY_LEN + 1)));
    }

    #[test]
    fn test_utils_sanitize_key() {
        assert_eq!(utils::sanitize_key("valid_key").as_str(), "valid_key");
        assert_eq!(utils::sanitize_key("invalid key").as_str(), "invalidkey");
        assert_eq!(utils::sanitize_key("invalid@#$key").as_str(), "invalidkey");
        assert_eq!(utils::sanitize_key("").as_str(), "default");
        
        let long_key = "a".repeat(MAX_KEY_LEN + 10);
        let sanitized = utils::sanitize_key(&long_key);
        assert_eq!(sanitized.len(), MAX_KEY_LEN);
    }

    #[test]
    fn test_utils_checksum() {
        let data = b"test data";
        let checksum = utils::calculate_checksum(data);
        
        assert!(utils::verify_checksum(data, checksum));
        assert!(!utils::verify_checksum(data, checksum + 1));
        
        // Different data should have different checksums (usually)
        let other_data = b"other data";
        let other_checksum = utils::calculate_checksum(other_data);
        assert_ne!(checksum, other_checksum);
    }

    #[test]
    fn test_utils_format_bytes() {
        assert_eq!(utils::format_bytes(500).as_str(), "500B");
        assert_eq!(utils::format_bytes(1024).as_str(), "1KB");
        assert_eq!(utils::format_bytes(2048).as_str(), "2KB");
        assert_eq!(utils::format_bytes(1024 * 1024).as_str(), "1MB");
        assert_eq!(utils::format_bytes(2 * 1024 * 1024).as_str(), "2MB");
    }

    #[test]
    fn test_storage_error_types() {
        let errors = vec![
            StorageError::KeyNotFound,
            StorageError::InvalidKey,
            StorageError::InvalidValue,
            StorageError::CapacityExceeded,
            StorageError::HardwareError,
            StorageError::CorruptedData,
            StorageError::Timeout,
            StorageError::ReadOnly,
            StorageError::TransactionConflict,
            StorageError::WearLevelingError,
            StorageError::Unknown,
        ];
        
        // Test that all error types are distinct
        for (i, error1) in errors.iter().enumerate() {
            for (j, error2) in errors.iter().enumerate() {
                if i != j {
                    assert_ne!(error1, error2);
                }
            }
        }
    }

    #[test]
    fn test_storage_macros() {
        let key = storage_key!("test_key");
        assert_eq!(key.as_str(), "test_key");
        
        let data = b"test data";
        let value = storage_value!(data);
        assert_eq!(value.as_bytes(), data);
    }
}