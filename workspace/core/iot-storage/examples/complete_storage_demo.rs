//! Complete IoT Storage Module Demonstration
//!
//! This example demonstrates all major features of the iot-storage module:
//! - Basic storage operations (store, retrieve, delete)
//! - Configuration management
//! - Atomic transactions
//! - Storage health monitoring
//! - Maintenance operations
//! - Error handling patterns

#![no_std]
#![no_main]

extern crate alloc;
use alloc::{string::String, vec::Vec, format};

use iot_storage::{
    init::{init_mock_storage, validate_storage},
    traits::{StorageKey, StorageValue, StorageError},
    UnifiedStorageManager, MockStorage,
    storage_to_iot_result,
};
use serde::{Deserialize, Serialize};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use rtt_target::{rprintln, rtt_init_print};

/// IoT device configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct IoTDeviceConfig {
    pub device_id: String,
    pub wifi_ssid: String,
    pub mqtt_broker: String,
    pub sensor_interval_ms: u32,
    pub enable_encryption: bool,
    pub max_retries: u8,
}

impl Default for IoTDeviceConfig {
    fn default() -> Self {
        Self {
            device_id: String::from("esp32c3_001"),
            wifi_ssid: String::from("IoT_Network"),
            mqtt_broker: String::from("192.168.1.100:1883"),
            sensor_interval_ms: 30000,
            enable_encryption: true,
            max_retries: 3,
        }
    }
}

/// Sensor data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorReading {
    pub timestamp: u64,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub device_id: String,
}

impl SensorReading {
    fn new(timestamp: u64, temp: f32, humidity: f32, pressure: f32, device_id: String) -> Self {
        Self {
            timestamp,
            temperature: temp,
            humidity,
            pressure,
            device_id,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkConfig {
    pub ssid: String,
    pub password: String,
    pub timeout_ms: u32,
    pub auto_reconnect: bool,
}

/// Main storage demo task
#[embassy_executor::task]
async fn storage_demo_task() -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("🚀 IoT Storage Module Complete Demo Starting...");
    
    // Initialize storage system
    rprintln!("📦 Initializing storage system...");
    let mut storage = init_mock_storage()
        .map_err(|e| {
            rprintln!("❌ Failed to initialize storage: {:?}", e);
            e
        })?;
    
    // Validate storage health
    rprintln!("🏥 Validating storage health...");
    validate_storage(&storage)
        .map_err(|e| {
            rprintln!("❌ Storage validation failed: {:?}", e);
            e
        })?;
    
    rprintln!("✅ Storage system initialized and validated");
    
    // Demo 1: Basic Configuration Management
    rprintln!("\n📋 Demo 1: Basic Configuration Management");
    demo_basic_config_management(&mut storage).await?;
    
    // Demo 2: Sensor Data Storage
    rprintln!("\n📊 Demo 2: Sensor Data Storage");
    demo_sensor_data_storage(&mut storage).await?;
    
    // Demo 3: Atomic Transactions
    rprintln!("\n⚛️ Demo 3: Atomic Transactions");
    demo_atomic_transactions(&mut storage).await?;
    
    // Demo 4: Storage Health Monitoring
    rprintln!("\n🏥 Demo 4: Storage Health Monitoring");
    demo_health_monitoring(&storage).await?;
    
    // Demo 5: Batch Operations
    rprintln!("\n📦 Demo 5: Batch Operations");
    demo_batch_operations(&mut storage).await?;
    
    // Demo 6: Error Handling
    rprintln!("\n⚠️ Demo 6: Error Handling");
    demo_error_handling(&mut storage).await?;
    
    // Demo 7: Maintenance Operations
    rprintln!("\n🔧 Demo 7: Maintenance Operations");
    demo_maintenance_operations(&mut storage).await?;
    
    // Demo 8: Performance Monitoring
    rprintln!("\n⚡ Demo 8: Performance Monitoring");
    demo_performance_monitoring(&mut storage).await?;
    
    rprintln!("\n🎉 All demos completed successfully!");
    
    Ok(())
}

/// Demonstrate basic configuration management
async fn demo_basic_config_management(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  📝 Storing device configuration...");
    
    let device_config = IoTDeviceConfig::default();
    storage.store("device_config", &device_config).await?;
    rprintln!("  ✅ Device configuration stored");
    
    rprintln!("  📖 Retrieving device configuration...");
    let retrieved_config: IoTDeviceConfig = storage.retrieve("device_config").await?;
    rprintln!("  📋 Retrieved config: device_id = {}", retrieved_config.device_id);
    rprintln!("  📋 WiFi SSID: {}", retrieved_config.wifi_ssid);
    rprintln!("  📋 Sensor interval: {}ms", retrieved_config.sensor_interval_ms);
    
    // Verify configuration
    if device_config == retrieved_config {
        rprintln!("  ✅ Configuration verification successful");
    } else {
        rprintln!("  ❌ Configuration verification failed");
        return Err(iot_storage::StorageErrorKind::CorruptedData(
            heapless::String::from("Config mismatch").unwrap_or_default()
        ));
    }
    
    // Test configuration updates
    rprintln!("  🔄 Updating configuration...");
    let mut updated_config = retrieved_config;
    updated_config.sensor_interval_ms = 15000;
    updated_config.max_retries = 5;
    
    storage.store("device_config", &updated_config).await?;
    
    let final_config: IoTDeviceConfig = storage.retrieve("device_config").await?;
    rprintln!("  📋 Updated sensor interval: {}ms", final_config.sensor_interval_ms);
    rprintln!("  📋 Updated max retries: {}", final_config.max_retries);
    
    Ok(())
}

/// Demonstrate sensor data storage
async fn demo_sensor_data_storage(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  🌡️ Storing sensor readings...");
    
    // Simulate sensor readings over time
    let readings = vec![
        SensorReading::new(1000, 23.5, 45.2, 1013.25, String::from("esp32c3_001")),
        SensorReading::new(2000, 23.8, 44.8, 1013.30, String::from("esp32c3_001")),
        SensorReading::new(3000, 24.1, 44.5, 1013.35, String::from("esp32c3_001")),
        SensorReading::new(4000, 24.3, 44.1, 1013.28, String::from("esp32c3_001")),
    ];
    
    for (i, reading) in readings.iter().enumerate() {
        let key = format!("sensor_reading_{}", i);
        storage.store(&key, reading).await?;
        rprintln!("  📊 Stored reading {}: {}°C, {}%RH, {}hPa", 
                  i, reading.temperature, reading.humidity, reading.pressure);
    }
    
    // Retrieve and verify sensor data
    rprintln!("  📖 Retrieving sensor readings...");
    for i in 0..readings.len() {
        let key = format!("sensor_reading_{}", i);
        let retrieved: SensorReading = storage.retrieve(&key).await?;
        rprintln!("  📊 Reading {}: {}°C at timestamp {}", 
                  i, retrieved.temperature, retrieved.timestamp);
    }
    
    // List all sensor readings
    rprintln!("  📋 Listing all sensor reading keys...");
    let keys = storage.list_keys(Some("sensor_reading")).await?;
    rprintln!("  📋 Found {} sensor reading keys", keys.len());
    
    Ok(())
}

/// Demonstrate atomic transactions
async fn demo_atomic_transactions(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  🔄 Starting atomic transaction demo...");
    
    let atomic_manager = storage.atomic_manager();
    
    // Successful transaction
    rprintln!("  ✅ Testing successful transaction...");
    let tx_id = atomic_manager.begin_transaction().await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Failed to begin transaction").unwrap_or_default()
        ))?;
    
    rprintln!("  📝 Transaction {} started", tx_id.value());
    
    // Store multiple configurations atomically
    let network_config = NetworkConfig {
        ssid: String::from("IoT_Secure"),
        password: String::from("secure_pass_123"),
        timeout_ms: 5000,
        auto_reconnect: true,
    };
    
    let key = StorageKey::new("network_config")
        .map_err(|_| iot_storage::StorageErrorKind::OperationFailed(
            heapless::String::from("Invalid key").unwrap_or_default()
        ))?;
    
    let value = StorageValue::from_serializable(&network_config)
        .map_err(|_| iot_storage::StorageErrorKind::SerializationFailed(
            heapless::String::from("Serialization failed").unwrap_or_default()
        ))?;
    
    atomic_manager.atomic_store(tx_id, &key, &value).await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Atomic store failed").unwrap_or_default()
        ))?;
    
    rprintln!("  📝 Network config stored in transaction");
    
    // Verify data exists in transaction
    let retrieved_value = atomic_manager.atomic_retrieve(tx_id, &key).await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Atomic retrieve failed").unwrap_or_default()
        ))?;
    
    let retrieved_config: NetworkConfig = retrieved_value.deserialize()
        .map_err(|_| iot_storage::StorageErrorKind::SerializationFailed(
            heapless::String::from("Deserialization failed").unwrap_or_default()
        ))?;
    
    rprintln!("  📖 Retrieved SSID in transaction: {}", retrieved_config.ssid);
    
    // Commit transaction
    atomic_manager.commit_transaction(tx_id).await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Commit failed").unwrap_or_default()
        ))?;
    
    rprintln!("  ✅ Transaction committed successfully");
    
    // Verify data persisted
    let persisted_config: NetworkConfig = storage.retrieve("network_config").await?;
    rprintln!("  📋 Persisted SSID: {}", persisted_config.ssid);
    
    // Rollback transaction demo
    rprintln!("  🔄 Testing transaction rollback...");
    let rollback_tx = atomic_manager.begin_transaction().await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Failed to begin rollback transaction").unwrap_or_default()
        ))?;
    
    // Store test data
    let test_key = StorageKey::new("rollback_test")
        .map_err(|_| iot_storage::StorageErrorKind::OperationFailed(
            heapless::String::from("Invalid rollback key").unwrap_or_default()
        ))?;
    
    let test_config = IoTDeviceConfig {
        device_id: String::from("rollback_device"),
        ..Default::default()
    };
    
    let test_value = StorageValue::from_serializable(&test_config)
        .map_err(|_| iot_storage::StorageErrorKind::SerializationFailed(
            heapless::String::from("Rollback serialization failed").unwrap_or_default()
        ))?;
    
    atomic_manager.atomic_store(rollback_tx, &test_key, &test_value).await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Rollback store failed").unwrap_or_default()
        ))?;
    
    rprintln!("  📝 Test data stored in rollback transaction");
    
    // Rollback instead of commit
    atomic_manager.rollback_transaction(rollback_tx).await
        .map_err(|_| iot_storage::StorageErrorKind::TransactionFailed(
            heapless::String::from("Rollback failed").unwrap_or_default()
        ))?;
    
    rprintln!("  🔄 Transaction rolled back");
    
    // Verify data was not persisted
    let exists = storage.exists("rollback_test").await?;
    if !exists {
        rprintln!("  ✅ Rollback successful - data not persisted");
    } else {
        rprintln!("  ❌ Rollback failed - data was persisted");
    }
    
    Ok(())
}

/// Demonstrate storage health monitoring
async fn demo_health_monitoring(
    storage: &UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  🏥 Checking storage health...");
    
    let health = storage.get_health();
    
    rprintln!("  📊 Health Score: {}/100", health.health_score());
    rprintln!("  📊 Fragmentation Level: {}%", health.fragmentation_level);
    rprintln!("  📊 Wear Level: {}%", health.wear_level);
    rprintln!("  📊 Health Status: {}", health.status());
    rprintln!("  📊 Needs Maintenance: {}", health.needs_maintenance);
    
    // Storage capacity information
    rprintln!("  💾 Storage Capacity:");
    rprintln!("    Total: {} bytes", health.capacity.total_bytes);
    rprintln!("    Used: {} bytes", health.capacity.used_bytes);
    rprintln!("    Available: {} bytes", health.capacity.available_bytes);
    rprintln!("    Usage: {}%", health.capacity.usage_percentage());
    
    // Storage statistics
    rprintln!("  📈 Storage Statistics:");
    rprintln!("    Total Reads: {}", health.stats.total_reads);
    rprintln!("    Total Writes: {}", health.stats.total_writes);
    rprintln!("    Total Operations: {}", health.stats.total_operations());
    rprintln!("    Failure Rate: {:.2}%", health.stats.failure_rate_percentage());
    
    if health.is_healthy() {
        rprintln!("  ✅ Storage is healthy");
    } else {
        rprintln!("  ⚠️ Storage needs attention");
    }
    
    Ok(())
}

/// Demonstrate batch operations
async fn demo_batch_operations(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  📦 Performing batch operations...");
    
    // Store multiple configurations
    let configs = vec![
        ("wifi_config", NetworkConfig {
            ssid: String::from("WiFi_Network_1"),
            password: String::from("password1"),
            timeout_ms: 5000,
            auto_reconnect: true,
        }),
        ("backup_wifi", NetworkConfig {
            ssid: String::from("WiFi_Network_2"),
            password: String::from("password2"),
            timeout_ms: 10000,
            auto_reconnect: false,
        }),
        ("guest_wifi", NetworkConfig {
            ssid: String::from("Guest_Network"),
            password: String::from("guest123"),
            timeout_ms: 3000,
            auto_reconnect: true,
        }),
    ];
    
    rprintln!("  📝 Storing {} network configurations...", configs.len());
    for (key, config) in &configs {
        storage.store(key, config).await?;
        rprintln!("    ✅ Stored {}: {}", key, config.ssid);
    }
    
    // Retrieve all configurations
    rprintln!("  📖 Retrieving all network configurations...");
    for (key, original_config) in &configs {
        let retrieved: NetworkConfig = storage.retrieve(key).await?;
        rprintln!("    📋 Retrieved {}: {} (timeout: {}ms)", 
                  key, retrieved.ssid, retrieved.timeout_ms);
        
        if retrieved.ssid == original_config.ssid {
            rprintln!("    ✅ Configuration verified");
        } else {
            rprintln!("    ❌ Configuration mismatch");
        }
    }
    
    // List all configurations
    rprintln!("  📋 Listing all stored keys...");
    let all_keys = storage.list_keys(None).await?;
    rprintln!("    📋 Total keys stored: {}", all_keys.len());
    for key in &all_keys {
        rprintln!("      - {}", key);
    }
    
    Ok(())
}

/// Demonstrate error handling patterns
async fn demo_error_handling(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  ⚠️ Testing error handling patterns...");
    
    // Test 1: Retrieve non-existent key
    rprintln!("  🔍 Testing retrieval of non-existent key...");
    match storage.retrieve::<IoTDeviceConfig>("nonexistent_key").await {
        Ok(_) => rprintln!("    ❌ Unexpected success"),
        Err(e) => rprintln!("    ✅ Expected error: {:?}", e),
    }
    
    // Test 2: Delete non-existent key
    rprintln!("  🗑️ Testing deletion of non-existent key...");
    match storage.delete("nonexistent_key").await {
        Ok(_) => rprintln!("    ❌ Unexpected success"),
        Err(e) => rprintln!("    ✅ Expected error: {:?}", e),
    }
    
    // Test 3: Invalid key creation
    rprintln!("  🔑 Testing invalid key creation...");
    match StorageKey::new("invalid key with spaces") {
        Ok(_) => rprintln!("    ❌ Unexpected success"),
        Err(e) => rprintln!("    ✅ Expected error: {:?}", e),
    }
    
    // Test 4: Large value handling
    rprintln!("  📏 Testing large value handling...");
    let large_data = vec![0u8; 10000]; // Larger than MAX_VALUE_SIZE
    match StorageValue::new(&large_data) {
        Ok(_) => rprintln!("    ❌ Unexpected success"),
        Err(e) => rprintln!("    ✅ Expected error: {:?}", e),
    }
    
    rprintln!("  ✅ Error handling tests completed");
    
    Ok(())
}

/// Demonstrate maintenance operations
async fn demo_maintenance_operations(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  🔧 Performing maintenance operations...");
    
    // Add some data first
    rprintln!("  📝 Adding test data for maintenance...");
    for i in 0..10 {
        let sensor_data = SensorReading::new(
            i * 1000,
            20.0 + i as f32 * 0.5,
            50.0 - i as f32 * 0.3,
            1013.0 + i as f32 * 0.1,
            String::from("maintenance_test"),
        );
        storage.store(&format!("maintenance_data_{}", i), &sensor_data).await?;
    }
    
    rprintln!("  📊 Storage stats before maintenance:");
    let stats_before = storage.get_stats()?;
    rprintln!("    Writes: {}", stats_before.total_writes);
    rprintln!("    Reads: {}", stats_before.total_reads);
    
    // Perform maintenance
    rprintln!("  🔧 Running storage maintenance...");
    storage.maintenance().await?;
    rprintln!("  ✅ Maintenance completed");
    
    // Check stats after maintenance
    rprintln!("  📊 Storage stats after maintenance:");
    let stats_after = storage.get_stats()?;
    rprintln!("    Writes: {}", stats_after.total_writes);
    rprintln!("    Reads: {}", stats_after.total_reads);
    
    // Verify data integrity after maintenance
    rprintln!("  🔍 Verifying data integrity after maintenance...");
    for i in 0..5 {
        let key = format!("maintenance_data_{}", i);
        match storage.retrieve::<SensorReading>(&key).await {
            Ok(data) => rprintln!("    ✅ Data {} intact: {}°C", i, data.temperature),
            Err(e) => rprintln!("    ❌ Data {} corrupted: {:?}", i, e),
        }
    }
    
    // Clean up maintenance data
    rprintln!("  🧹 Cleaning up maintenance test data...");
    for i in 0..10 {
        let key = format!("maintenance_data_{}", i);
        storage.delete(&key).await?;
    }
    
    rprintln!("  ✅ Maintenance operations completed");
    
    Ok(())
}

/// Demonstrate performance monitoring
async fn demo_performance_monitoring(
    storage: &mut UnifiedStorageManager<MockStorage>
) -> Result<(), iot_storage::StorageErrorKind> {
    rprintln!("  ⚡ Performance monitoring demo...");
    
    // Simulate high-frequency sensor data
    rprintln!("  📊 Simulating high-frequency operations...");
    
    let start_timestamp = 0u64; // In real code, use embassy_time::Instant::now()
    
    // Rapid data storage
    for i in 0..50 {
        let sensor_data = SensorReading::new(
            start_timestamp + i * 100, // 100ms intervals
            25.0 + (i as f32 * 0.1) % 5.0, // Temperature variation
            45.0 + (i as f32 * 0.2) % 10.0, // Humidity variation
            1013.0 + (i as f32 * 0.05) % 2.0, // Pressure variation
            String::from("perf_test"),
        );
        
        let key = format!("perf_sensor_{:03}", i);
        storage.store(&key, &sensor_data).await?;
        
        if i % 10 == 0 {
            rprintln!("    📊 Stored {} readings", i + 1);
        }
    }
    
    let end_timestamp = start_timestamp + 5000; // Simulate 5 seconds later
    
    // Performance analysis
    rprintln!("  📈 Performance Analysis:");
    let final_stats = storage.get_stats()?;
    rprintln!("    Total Operations: {}", final_stats.total_operations());
    rprintln!("    Success Rate: {:.2}%", 
              100.0 - final_stats.failure_rate_percentage());
    rprintln!("    Read Success Rate: {:.2}%", final_stats.read_success_rate());
    rprintln!("    Write Success Rate: {:.2}%", final_stats.write_success_rate());
    
    // Storage efficiency
    let capacity = storage.get_capacity()?;
    rprintln!("  💾 Storage Efficiency:");
    rprintln!("    Space Utilization: {}%", capacity.usage_percentage());
    rprintln!("    Available Space: {} bytes", capacity.available_bytes);
    
    // Health assessment
    let health = storage.get_health();
    rprintln!("  🏥 Health Assessment:");
    rprintln!("    Overall Score: {}/100", health.health_score());
    rprintln!("    Status: {}", health.status());
    
    if health.health_score() > 80 {
        rprintln!("  ✅ Performance is excellent");
    } else if health.health_score() > 60 {
        rprintln!("  ⚠️ Performance is acceptable");
    } else {
        rprintln!("  ❌ Performance needs improvement");
    }
    
    // Clean up performance test data
    rprintln!("  🧹 Cleaning up performance test data...");
    for i in 0..50 {
        let key = format!("perf_sensor_{:03}", i);
        storage.delete(&key).await?;
    }
    
    rprintln!("  ✅ Performance monitoring completed");
    
    Ok(())
}

/// Error handling task
#[embassy_executor::task]
async fn error_handler_task() {
    loop {
        Timer::after(Duration::from_secs(10)).await;
        // In a real application, this would handle storage errors
        // and perform recovery operations
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize RTT for logging
    rtt_init_print!();
    
    rprintln!("🎯 IoT Storage Module Complete Demo");
    rprintln!("====================================");
    
    // Spawn error handler task
    spawner.spawn(error_handler_task()).unwrap();
    
    // Run the main demo
    match spawner.spawn(storage_demo_task()) {
        Ok(_) => {
            rprintln!("✅ Demo task spawned successfully");
        }
        Err(e) => {
            rprintln!("❌ Failed to spawn demo task: {:?}", e);
        }
    }
    
    // Keep the demo running
    loop {
        Timer::after(Duration::from_secs(60)).await;
        rprintln!("💤 Demo still running... (press reset to restart)");
    }
}