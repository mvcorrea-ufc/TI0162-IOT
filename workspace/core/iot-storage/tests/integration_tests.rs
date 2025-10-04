//! Integration tests for iot-storage module
//!
//! Comprehensive test suite covering all storage operations,
//! error handling, and platform-specific functionality.

#![no_std]

extern crate alloc;
use alloc::{string::String, vec::Vec};

use iot_storage::{
    traits::{StorageBackend, StorageKey, StorageValue, StorageError},
    init::init_mock_storage,
    UnifiedStorageManager, MockStorage,
};
use serde::{Deserialize, Serialize};

/// Test configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    pub name: String,
    pub value: u32,
    pub enabled: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            name: String::from("test"),
            value: 42,
            enabled: true,
        }
    }
}

/// Test basic storage operations
#[tokio::test]
async fn test_basic_storage_operations() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Test store and retrieve
    let test_data = TestConfig::default();
    storage.store("test_config", &test_data).await
        .expect("Failed to store test config");
    
    let retrieved: TestConfig = storage.retrieve("test_config").await
        .expect("Failed to retrieve test config");
    
    assert_eq!(test_data, retrieved);
    
    // Test exists
    let exists = storage.exists("test_config").await
        .expect("Failed to check existence");
    assert!(exists);
    
    let not_exists = storage.exists("nonexistent").await
        .expect("Failed to check non-existence");
    assert!(!not_exists);
    
    // Test delete
    storage.delete("test_config").await
        .expect("Failed to delete test config");
    
    let exists_after_delete = storage.exists("test_config").await
        .expect("Failed to check existence after delete");
    assert!(!exists_after_delete);
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Test retrieving non-existent key
    let result: Result<TestConfig, _> = storage.retrieve("nonexistent").await;
    assert!(result.is_err());
    
    // Test deleting non-existent key
    let result = storage.delete("nonexistent").await;
    assert!(result.is_err());
}

/// Test storage capacity and statistics
#[tokio::test]
async fn test_capacity_and_stats() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Get initial capacity
    let capacity = storage.get_capacity()
        .expect("Failed to get capacity");
    assert!(capacity.total_bytes > 0);
    assert!(capacity.available_bytes > 0);
    
    // Get initial stats
    let stats = storage.get_stats()
        .expect("Failed to get stats");
    assert_eq!(stats.total_reads, 0);
    assert_eq!(stats.total_writes, 0);
    
    // Perform some operations and check stats
    let test_data = TestConfig::default();
    storage.store("test", &test_data).await
        .expect("Failed to store data");
    
    let updated_stats = storage.get_stats()
        .expect("Failed to get updated stats");
    assert_eq!(updated_stats.total_writes, 1);
    
    let _retrieved: TestConfig = storage.retrieve("test").await
        .expect("Failed to retrieve data");
    
    let final_stats = storage.get_stats()
        .expect("Failed to get final stats");
    assert_eq!(final_stats.total_reads, 1);
    assert_eq!(final_stats.total_writes, 1);
}

/// Test list operations
#[tokio::test]
async fn test_list_operations() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Store multiple configs
    for i in 0..5 {
        let config = TestConfig {
            name: format!("config_{}", i),
            value: i,
            enabled: i % 2 == 0,
        };
        storage.store(&format!("test_config_{}", i), &config).await
            .expect("Failed to store config");
    }
    
    // List all keys
    let all_keys = storage.list_keys(None).await
        .expect("Failed to list all keys");
    assert_eq!(all_keys.len(), 5);
    
    // List keys with prefix
    let prefixed_keys = storage.list_keys(Some("test_config")).await
        .expect("Failed to list prefixed keys");
    assert_eq!(prefixed_keys.len(), 5);
    
    // List keys with non-matching prefix
    let no_match_keys = storage.list_keys(Some("nonexistent")).await
        .expect("Failed to list non-matching keys");
    assert_eq!(no_match_keys.len(), 0);
}

/// Test config store operations
#[tokio::test]
async fn test_config_store() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    let config_store = storage.config_store();
    
    // Store configuration
    let test_config = TestConfig {
        name: String::from("test_system"),
        value: 100,
        enabled: true,
    };
    
    config_store.store_config("system", &test_config).await
        .expect("Failed to store config");
    
    // Retrieve configuration
    let retrieved: TestConfig = config_store.retrieve_config("system").await
        .expect("Failed to retrieve config");
    
    assert_eq!(test_config, retrieved);
    
    // Check config exists
    let exists = config_store.config_exists("system").await
        .expect("Failed to check config existence");
    assert!(exists);
    
    // List configurations
    let configs = config_store.list_configs().await
        .expect("Failed to list configs");
    assert!(configs.contains(&String::from("system")));
    
    // Test backup and restore
    config_store.backup_config("system").await
        .expect("Failed to backup config");
    
    config_store.restore_config("system").await
        .expect("Failed to restore config");
    
    // Delete configuration
    config_store.delete_config("system").await
        .expect("Failed to delete config");
    
    let exists_after_delete = config_store.config_exists("system").await
        .expect("Failed to check existence after delete");
    assert!(!exists_after_delete);
}

/// Test atomic operations
#[tokio::test]
async fn test_atomic_operations() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    let atomic_manager = storage.atomic_manager();
    
    // Begin transaction
    let transaction_id = atomic_manager.begin_transaction().await
        .expect("Failed to begin transaction");
    
    // Perform atomic operations
    let key = StorageKey::new("atomic_test").expect("Invalid key");
    let value = StorageValue::from_serializable(&TestConfig::default())
        .expect("Failed to serialize value");
    
    atomic_manager.atomic_store(transaction_id, &key, &value).await
        .expect("Failed to atomic store");
    
    let retrieved = atomic_manager.atomic_retrieve(transaction_id, &key).await
        .expect("Failed to atomic retrieve");
    
    assert_eq!(value.as_bytes(), retrieved.as_bytes());
    
    // Commit transaction
    atomic_manager.commit_transaction(transaction_id).await
        .expect("Failed to commit transaction");
    
    // Verify data persisted after commit
    let persisted = storage.retrieve::<TestConfig>("atomic_test").await
        .expect("Failed to retrieve after commit");
    
    assert_eq!(persisted, TestConfig::default());
}

/// Test atomic transaction rollback
#[tokio::test]
async fn test_atomic_rollback() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    let atomic_manager = storage.atomic_manager();
    
    // Store initial data
    storage.store("rollback_test", &TestConfig::default()).await
        .expect("Failed to store initial data");
    
    // Begin transaction
    let transaction_id = atomic_manager.begin_transaction().await
        .expect("Failed to begin transaction");
    
    // Modify data in transaction
    let key = StorageKey::new("rollback_test").expect("Invalid key");
    let modified_config = TestConfig {
        name: String::from("modified"),
        value: 999,
        enabled: false,
    };
    let modified_value = StorageValue::from_serializable(&modified_config)
        .expect("Failed to serialize modified value");
    
    atomic_manager.atomic_store(transaction_id, &key, &modified_value).await
        .expect("Failed to atomic store modified data");
    
    // Rollback transaction
    atomic_manager.rollback_transaction(transaction_id).await
        .expect("Failed to rollback transaction");
    
    // Verify original data is still there
    let original: TestConfig = storage.retrieve("rollback_test").await
        .expect("Failed to retrieve after rollback");
    
    assert_eq!(original, TestConfig::default());
}

/// Test storage health monitoring
#[tokio::test]
async fn test_storage_health() {
    let storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    let health = storage.get_health();
    
    // Check health metrics are reasonable
    assert!(health.health_score() <= 100);
    assert!(health.fragmentation_level <= 100);
    assert!(health.wear_level <= 100);
    
    // New storage should be healthy
    assert!(health.is_healthy());
    assert_eq!(health.status(), "Excellent");
}

/// Test storage maintenance
#[tokio::test]
async fn test_storage_maintenance() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Perform maintenance
    storage.maintenance().await
        .expect("Failed to perform maintenance");
    
    // Verify storage is still functional after maintenance
    let test_data = TestConfig::default();
    storage.store("maintenance_test", &test_data).await
        .expect("Failed to store after maintenance");
    
    let retrieved: TestConfig = storage.retrieve("maintenance_test").await
        .expect("Failed to retrieve after maintenance");
    
    assert_eq!(test_data, retrieved);
}

/// Test key validation
#[test]
fn test_key_validation() {
    // Valid keys
    assert!(StorageKey::new("valid_key").is_ok());
    assert!(StorageKey::new("valid-key").is_ok());
    assert!(StorageKey::new("valid.key").is_ok());
    assert!(StorageKey::new("ValidKey123").is_ok());
    
    // Invalid keys
    assert!(StorageKey::new("").is_err());  // Empty
    assert!(StorageKey::new("invalid key").is_err());  // Space
    assert!(StorageKey::new("invalid@key").is_err());  // Special char
    assert!(StorageKey::new(&"a".repeat(100)).is_err());  // Too long
}

/// Test value validation
#[test]
fn test_value_validation() {
    // Valid values
    let small_data = vec![1, 2, 3, 4];
    assert!(StorageValue::new(&small_data).is_ok());
    
    let medium_data = vec![0u8; 1000];
    assert!(StorageValue::new(&medium_data).is_ok());
    
    // Large value (should still work within limits)
    let large_data = vec![0u8; 4000];
    assert!(StorageValue::new(&large_data).is_ok());
    
    // Too large value
    let too_large_data = vec![0u8; 10000];
    assert!(StorageValue::new(&too_large_data).is_err());
}

/// Test serialization and deserialization
#[test]
fn test_serialization() {
    let original = TestConfig {
        name: String::from("serialization_test"),
        value: 42,
        enabled: true,
    };
    
    // Test through StorageValue
    let storage_value = StorageValue::from_serializable(&original)
        .expect("Failed to serialize");
    
    let deserialized: TestConfig = storage_value.deserialize()
        .expect("Failed to deserialize");
    
    assert_eq!(original, deserialized);
}

/// Performance test for storage operations
#[tokio::test]
async fn test_storage_performance() {
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    let start_time = std::time::Instant::now();
    
    // Perform multiple operations
    for i in 0..100 {
        let config = TestConfig {
            name: format!("perf_test_{}", i),
            value: i,
            enabled: i % 2 == 0,
        };
        
        storage.store(&format!("perf_{}", i), &config).await
            .expect("Failed to store in performance test");
    }
    
    let store_duration = start_time.elapsed();
    
    // Read back all data
    let read_start = std::time::Instant::now();
    for i in 0..100 {
        let _config: TestConfig = storage.retrieve(&format!("perf_{}", i)).await
            .expect("Failed to retrieve in performance test");
    }
    let read_duration = read_start.elapsed();
    
    // Basic performance assertions (adjust thresholds as needed)
    assert!(store_duration.as_millis() < 1000, "Store operations too slow");
    assert!(read_duration.as_millis() < 500, "Read operations too slow");
    
    // Clean up
    for i in 0..100 {
        storage.delete(&format!("perf_{}", i)).await
            .expect("Failed to delete in performance test");
    }
}

/// Test concurrent access patterns
#[tokio::test]
async fn test_concurrent_patterns() {
    use alloc::boxed::Box;
    use core::future::Future;
    use core::pin::Pin;
    
    let mut storage = init_mock_storage().expect("Failed to initialize mock storage");
    
    // Simulate concurrent-like access by interleaving operations
    let configs: Vec<TestConfig> = (0..10).map(|i| TestConfig {
        name: format!("concurrent_{}", i),
        value: i,
        enabled: i % 2 == 0,
    }).collect();
    
    // Store all configs
    for (i, config) in configs.iter().enumerate() {
        storage.store(&format!("concurrent_{}", i), config).await
            .expect("Failed to store in concurrent test");
    }
    
    // Retrieve all configs in different order
    for i in (0..10).rev() {
        let retrieved: TestConfig = storage.retrieve(&format!("concurrent_{}", i)).await
            .expect("Failed to retrieve in concurrent test");
        assert_eq!(retrieved, configs[i]);
    }
    
    // Mixed operations
    for i in 0..5 {
        // Delete even indices
        if i % 2 == 0 {
            storage.delete(&format!("concurrent_{}", i)).await
                .expect("Failed to delete in concurrent test");
        }
        // Update odd indices
        else {
            let updated_config = TestConfig {
                name: format!("updated_{}", i),
                value: i + 1000,
                enabled: false,
            };
            storage.store(&format!("concurrent_{}", i), &updated_config).await
                .expect("Failed to update in concurrent test");
        }
    }
    
    // Verify final state
    for i in 0..5 {
        let exists = storage.exists(&format!("concurrent_{}", i)).await
            .expect("Failed to check existence in concurrent test");
        
        if i % 2 == 0 {
            assert!(!exists, "Even index should be deleted");
        } else {
            assert!(exists, "Odd index should exist");
            
            let retrieved: TestConfig = storage.retrieve(&format!("concurrent_{}", i)).await
                .expect("Failed to retrieve updated config");
            assert_eq!(retrieved.name, format!("updated_{}", i));
            assert_eq!(retrieved.value, i + 1000);
            assert!(!retrieved.enabled);
        }
    }
}