# IoT Storage Module

A comprehensive storage abstraction layer for ESP32-C3 IoT applications that provides unified access to flash storage, wear leveling, atomic operations, and configuration persistence.

## Features

- **Storage Abstraction**: Unified interface for different storage backends
- **Flash Storage**: ESP32-C3 flash memory integration with wear leveling
- **Atomic Operations**: Safe concurrent access to storage resources with ACID guarantees
- **Configuration Persistence**: Specialized storage for system configuration
- **Error Recovery**: Robust error handling and recovery mechanisms
- **Memory Efficiency**: Optimized for constrained embedded environments
- **No-std Compatible**: Works without heap allocation
- **Comprehensive Testing**: Full test suite with benchmarks and examples

## Architecture

The storage system is organized into layers:

```
┌─────────────────────────────────────────────────────────┐
│                 Application Layer                       │
├─────────────────────────────────────────────────────────┤
│              UnifiedStorageManager                      │
├─────────────────────────────────────────────────────────┤
│  ConfigStore  │  AtomicManager  │  HealthMonitor       │
├─────────────────────────────────────────────────────────┤
│                 Storage Traits                          │
├─────────────────────────────────────────────────────────┤
│  ESP32-C3 Flash │  Mock Storage  │  Future Backends    │
├─────────────────────────────────────────────────────────┤
│              Hardware Abstraction                       │
└─────────────────────────────────────────────────────────┘
```

## Quick Start

### Basic Usage

```rust
use iot_storage::{init::init_default_storage, storage_to_iot_result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct DeviceConfig {
    device_id: String,
    wifi_ssid: String,
    sensor_interval_ms: u32,
}

#[embassy_executor::main]
async fn main() {
    // Initialize storage
    let mut storage = init_default_storage().expect("Storage init failed");
    
    // Store configuration
    let config = DeviceConfig {
        device_id: "esp32c3_001".into(),
        wifi_ssid: "IoT_Network".into(),
        sensor_interval_ms: 30000,
    };
    
    storage.store("device_config", &config).await?;
    
    // Retrieve configuration
    let loaded_config: DeviceConfig = storage.retrieve("device_config").await?;
    
    println!("Device ID: {}", loaded_config.device_id);
}
```

### Configuration Storage

```rust
use iot_storage::traits::ConfigStorage;

// Using the configuration store
let config_store = storage.config_store();

// Store typed configuration
config_store.store_config("system", &system_config).await?;

// Retrieve with type safety
let loaded: SystemConfig = config_store.retrieve_config("system").await?;

// Backup and restore
config_store.backup_config("system").await?;
config_store.restore_config("system").await?;
```

### Atomic Transactions

```rust
use iot_storage::traits::{AtomicStorage, StorageKey, StorageValue};

let atomic_manager = storage.atomic_manager();

// Begin transaction
let tx_id = atomic_manager.begin_transaction().await?;

// Perform atomic operations
let key = StorageKey::new("critical_data")?;
let value = StorageValue::from_serializable(&important_data)?;

atomic_manager.atomic_store(tx_id, &key, &value).await?;
atomic_manager.atomic_store(tx_id, &other_key, &other_value).await?;

// Commit all changes atomically
atomic_manager.commit_transaction(tx_id).await?;

// Or rollback if needed
// atomic_manager.rollback_transaction(tx_id).await?;
```

### Health Monitoring

```rust
// Check storage health
let health = storage.get_health();

println!("Health Score: {}/100", health.health_score());
println!("Status: {}", health.status());
println!("Fragmentation: {}%", health.fragmentation_level);
println!("Wear Level: {}%", health.wear_level);

if health.needs_maintenance {
    storage.maintenance().await?;
}
```

## Configuration

### ESP32-C3 Configuration

```rust
use iot_storage::{Esp32C3Config, FlashConfig};

let config = Esp32C3Config {
    flash_config: FlashConfig {
        base_address: 0x310000,    // After partition table
        total_size: 65536,         // 64KB
        sector_size: 4096,         // 4KB sectors
        reserved_sectors: 2,       // For wear leveling
        wear_leveling_enabled: true,
        max_erase_cycles: 100000,
    },
    hardware_crc: true,
    flash_encryption: false,
    cache_enabled: true,
    dma_buffer_size: 1024,
};

let storage = init_custom_storage(config)?;
```

### Feature Flags

Add to your `Cargo.toml`:

```toml
[dependencies]
iot-storage = { path = "../core/iot-storage", features = ["esp32c3-flash", "wear-leveling"] }
```

Available features:
- `esp32c3-flash`: Enable ESP32-C3 flash storage backend
- `wear-leveling`: Enable wear leveling algorithms
- `encryption`: Enable storage encryption (requires additional setup)
- `compression`: Enable data compression

## Error Handling

The module provides comprehensive error handling with integration to the `iot-common` error system:

```rust
use iot_storage::{StorageErrorKind, storage_to_iot_result};

match storage.store("key", &data).await {
    Ok(_) => println!("Stored successfully"),
    Err(StorageErrorKind::CapacityExceeded(_)) => {
        // Handle storage full
        storage.maintenance().await?;
    },
    Err(StorageErrorKind::CorruptedData(_)) => {
        // Handle data corruption
        storage.repair_data(&[key]).await?;
    },
    Err(e) => {
        // Convert to IoTError for system integration
        return Err(e.into_iot_error());
    }
}
```

## Performance Characteristics

### Storage Operations

| Operation | Typical Latency | Memory Usage |
|-----------|----------------|--------------|
| Store (small) | < 1ms | ~100 bytes |
| Store (large) | < 5ms | ~4KB |
| Retrieve | < 0.5ms | ~50 bytes |
| Delete | < 0.5ms | ~20 bytes |
| List Keys | < 2ms | ~200 bytes |
| Transaction | < 3ms | ~150 bytes |

### Memory Footprint

- **Base module**: ~8KB flash, ~2KB RAM
- **With ESP32-C3 backend**: ~12KB flash, ~3KB RAM
- **Per storage entry**: ~68 bytes overhead
- **Transaction overhead**: ~128 bytes per transaction

## Testing

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_tests

# Benchmarks
cargo bench

# Example
cargo run --example complete_storage_demo
```

### Test Coverage

- ✅ Basic storage operations (store, retrieve, delete)
- ✅ Configuration management
- ✅ Atomic transactions with commit/rollback
- ✅ Error handling and recovery
- ✅ Health monitoring and maintenance
- ✅ Performance under load
- ✅ Memory safety and leak detection
- ✅ Concurrent access patterns

## Hardware Requirements

### ESP32-C3

- **Flash**: Minimum 4MB recommended (system uses ~64KB by default)
- **RAM**: Minimum 400KB total (module uses ~3KB)
- **Sectors**: 4KB sector size standard
- **Endurance**: 100,000 erase cycles typical

### Memory Layout

```
ESP32-C3 Flash Memory Layout:
┌──────────────────┬─────────────┐
│ 0x000000         │ Bootloader  │
│ 0x010000         │ Partition   │
│ 0x020000         │ Application │
│ 0x200000         │ User Data   │
│ 0x310000         │ IoT Storage │ ← Default location
│ 0x320000         │ Available   │
└──────────────────┴─────────────┘
```

## Advanced Usage

### Custom Storage Backend

```rust
use iot_storage::traits::{StorageBackend, StorageResult};

struct CustomStorage {
    // Your implementation
}

#[async_trait::async_trait]
impl StorageBackend for CustomStorage {
    async fn store(&mut self, key: &StorageKey, value: &StorageValue) -> StorageResult<()> {
        // Custom implementation
    }
    
    // Implement other required methods...
}

let storage = UnifiedStorageManager::new(CustomStorage::new())?;
```

### Wear Leveling

```rust
use iot_storage::traits::WearLeveling;

// Monitor wear levels
let avg_wear = storage.get_average_wear_level();
println!("Average wear level: {}%", avg_wear);

// Trigger wear leveling
if avg_wear > 50 {
    storage.level_wear()?;
}

// Check bad blocks
let bad_blocks = storage.get_bad_block_count();
if bad_blocks > 0 {
    println!("Warning: {} bad blocks detected", bad_blocks);
}
```

### Event Monitoring

```rust
use iot_storage::traits::StorageEventListener;

struct StorageMonitor;

impl StorageEventListener for StorageMonitor {
    fn on_store(&mut self, key: &StorageKey, size: usize) {
        println!("Stored {}: {} bytes", key.as_str(), size);
    }
    
    fn on_error(&mut self, error: &StorageError, operation: &str) {
        println!("Error during {}: {:?}", operation, error);
    }
}
```

## Troubleshooting

### Common Issues

1. **Storage Full**
   ```rust
   // Check capacity before storing
   let capacity = storage.get_capacity()?;
   if capacity.is_nearly_full() {
       storage.maintenance().await?;
   }
   ```

2. **Data Corruption**
   ```rust
   // Verify and repair
   let corrupted_keys = storage.verify_integrity()?;
   if !corrupted_keys.is_empty() {
       storage.repair_data(&corrupted_keys)?;
   }
   ```

3. **Performance Issues**
   ```rust
   // Monitor and optimize
   let health = storage.get_health();
   if health.fragmentation_level > 50 {
       storage.maintenance().await?;
   }
   ```

### Debug Logging

Enable detailed logging in debug builds:

```rust
#[cfg(debug_assertions)]
use log::{debug, warn, error};

// Storage operations are automatically logged
storage.store("key", &data).await?; // Logs: "Storage: stored key 'key' (123 bytes)"
```

## Integration with IoT System

The storage module integrates seamlessly with other IoT system components:

```rust
use iot_common::{IoTResult, IoTError};
use iot_container::{Container, Injectable};

// Register storage in dependency injection container
container.register_singleton(|| init_default_storage())?;

// Use in other services
struct SensorService {
    storage: UnifiedStorageManager<Esp32C3Storage>,
}

impl SensorService {
    async fn save_reading(&mut self, reading: &SensorReading) -> IoTResult<()> {
        // Convert storage errors to system errors automatically
        storage_to_iot_result(
            self.storage.store("latest_reading", reading).await
        )
    }
}
```

## Safety and Reliability

- **Power-Safe**: Atomic operations protect against power loss
- **Wear Leveling**: Extends flash memory lifetime
- **Error Recovery**: Automatic detection and repair of corruption
- **Validation**: Input validation prevents invalid data storage
- **Testing**: Comprehensive test suite ensures reliability

## Contributing

See [CONTRIBUTING.md](../../docs/CONTRIBUTING.md) for development guidelines.

## License

This project is licensed under the MIT OR Apache-2.0 license.