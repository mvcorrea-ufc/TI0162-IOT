//! Performance benchmarks for iot-storage module
//!
//! Comprehensive benchmarks covering storage operations,
//! memory usage, and platform-specific optimizations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use iot_storage::{
    init::init_mock_storage,
    traits::{StorageKey, StorageValue},
    UnifiedStorageManager,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Test configuration for benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchConfig {
    pub id: u32,
    pub name: String,
    pub data: Vec<u8>,
    pub enabled: bool,
}

impl BenchConfig {
    fn new(id: u32, size: usize) -> Self {
        Self {
            id,
            name: format!("config_{}", id),
            data: vec![id as u8; size],
            enabled: id % 2 == 0,
        }
    }
}

/// Runtime for async benchmarks
fn get_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

/// Benchmark basic storage operations
fn benchmark_basic_operations(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("basic_operations");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark store operation
    group.bench_function("store_small_config", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            let config = BenchConfig::new(1, 100);
            storage.store(black_box("bench_config"), black_box(&config)).await.unwrap();
        });
    });
    
    // Benchmark retrieve operation
    group.bench_function("retrieve_small_config", |b| {
        let mut storage = runtime.block_on(async {
            let mut storage = init_mock_storage().unwrap();
            let config = BenchConfig::new(1, 100);
            storage.store("bench_config", &config).await.unwrap();
            storage
        });
        
        b.to_async(&runtime).iter(|| async {
            let _config: BenchConfig = storage.retrieve(black_box("bench_config")).await.unwrap();
        });
    });
    
    // Benchmark delete operation
    group.bench_function("delete_config", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            let config = BenchConfig::new(1, 100);
            storage.store("bench_config", &config).await.unwrap();
            storage.delete(black_box("bench_config")).await.unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark operations with different data sizes
fn benchmark_data_sizes(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("data_sizes");
    group.measurement_time(Duration::from_secs(15));
    
    let sizes = vec![10, 100, 1000, 4000];
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("store", size),
            &size,
            |b, &size| {
                b.to_async(&runtime).iter(|| async {
                    let mut storage = init_mock_storage().unwrap();
                    let config = BenchConfig::new(1, size);
                    storage.store(black_box("bench_config"), black_box(&config)).await.unwrap();
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("retrieve", size),
            &size,
            |b, &size| {
                let mut storage = runtime.block_on(async {
                    let mut storage = init_mock_storage().unwrap();
                    let config = BenchConfig::new(1, size);
                    storage.store("bench_config", &config).await.unwrap();
                    storage
                });
                
                b.to_async(&runtime).iter(|| async {
                    let _config: BenchConfig = storage.retrieve(black_box("bench_config")).await.unwrap();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch operations
fn benchmark_batch_operations(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("batch_operations");
    group.measurement_time(Duration::from_secs(20));
    
    let batch_sizes = vec![10, 50, 100];
    
    for batch_size in batch_sizes {
        group.bench_with_input(
            BenchmarkId::new("batch_store", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.to_async(&runtime).iter(|| async {
                    let mut storage = init_mock_storage().unwrap();
                    
                    for i in 0..batch_size {
                        let config = BenchConfig::new(i, 100);
                        let key = format!("config_{}", i);
                        storage.store(black_box(&key), black_box(&config)).await.unwrap();
                    }
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("batch_retrieve", batch_size),
            &batch_size,
            |b, &batch_size| {
                let mut storage = runtime.block_on(async {
                    let mut storage = init_mock_storage().unwrap();
                    
                    for i in 0..batch_size {
                        let config = BenchConfig::new(i, 100);
                        let key = format!("config_{}", i);
                        storage.store(&key, &config).await.unwrap();
                    }
                    
                    storage
                });
                
                b.to_async(&runtime).iter(|| async {
                    for i in 0..batch_size {
                        let key = format!("config_{}", i);
                        let _config: BenchConfig = storage.retrieve(black_box(&key)).await.unwrap();
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark key operations
fn benchmark_key_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_operations");
    
    // Benchmark key creation
    group.bench_function("key_creation", |b| {
        b.iter(|| {
            let key = StorageKey::new(black_box("benchmark_key")).unwrap();
            black_box(key);
        });
    });
    
    // Benchmark key validation
    group.bench_function("key_validation", |b| {
        b.iter(|| {
            let result = StorageKey::new(black_box("valid_key_12345"));
            black_box(result);
        });
    });
    
    // Benchmark key operations
    group.bench_function("key_operations", |b| {
        let key = StorageKey::new("benchmark_key").unwrap();
        b.iter(|| {
            black_box(key.as_str());
            black_box(key.len());
            black_box(key.starts_with("bench"));
            black_box(key.ends_with("key"));
        });
    });
    
    group.finish();
}

/// Benchmark value operations
fn benchmark_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_operations");
    
    let data_sizes = vec![100, 1000, 4000];
    
    for size in data_sizes {
        group.bench_with_input(
            BenchmarkId::new("value_creation", size),
            &size,
            |b, &size| {
                let data = vec![42u8; size];
                b.iter(|| {
                    let value = StorageValue::new(black_box(&data)).unwrap();
                    black_box(value);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("value_serialization", size),
            &size,
            |b, &size| {
                let config = BenchConfig::new(1, size);
                b.iter(|| {
                    let value = StorageValue::from_serializable(black_box(&config)).unwrap();
                    black_box(value);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("value_deserialization", size),
            &size,
            |b, &size| {
                let config = BenchConfig::new(1, size);
                let value = StorageValue::from_serializable(&config).unwrap();
                
                b.iter(|| {
                    let deserialized: BenchConfig = value.deserialize().unwrap();
                    black_box(deserialized);
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark concurrent-like access patterns
fn benchmark_concurrent_patterns(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("concurrent_patterns");
    group.measurement_time(Duration::from_secs(15));
    
    // Benchmark interleaved operations
    group.bench_function("interleaved_operations", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            
            // Simulate concurrent access pattern
            for i in 0..20 {
                let config = BenchConfig::new(i, 100);
                let key = format!("config_{}", i);
                
                // Store
                storage.store(black_box(&key), black_box(&config)).await.unwrap();
                
                // Immediately read back
                let _retrieved: BenchConfig = storage.retrieve(black_box(&key)).await.unwrap();
                
                // Check existence
                let _exists = storage.exists(black_box(&key)).await.unwrap();
                
                // Delete every other entry
                if i % 2 == 0 {
                    storage.delete(black_box(&key)).await.unwrap();
                }
            }
        });
    });
    
    group.finish();
}

/// Benchmark memory overhead
fn benchmark_memory_overhead(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("memory_overhead");
    
    // Benchmark storage initialization
    group.bench_function("storage_initialization", |b| {
        b.iter(|| {
            let storage = init_mock_storage().unwrap();
            black_box(storage);
        });
    });
    
    // Benchmark storage with many entries
    group.bench_function("storage_many_entries", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            
            for i in 0..100 {
                let config = BenchConfig::new(i, 50);
                let key = format!("config_{}", i);
                storage.store(black_box(&key), black_box(&config)).await.unwrap();
            }
            
            black_box(storage);
        });
    });
    
    group.finish();
}

/// Benchmark atomic operations
fn benchmark_atomic_operations(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("atomic_operations");
    group.measurement_time(Duration::from_secs(15));
    
    // Benchmark transaction lifecycle
    group.bench_function("transaction_lifecycle", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            let atomic_manager = storage.atomic_manager();
            
            let tx_id = atomic_manager.begin_transaction().await.unwrap();
            
            let key = StorageKey::new("atomic_test").unwrap();
            let config = BenchConfig::new(1, 100);
            let value = StorageValue::from_serializable(&config).unwrap();
            
            atomic_manager.atomic_store(black_box(tx_id), black_box(&key), black_box(&value)).await.unwrap();
            let _retrieved = atomic_manager.atomic_retrieve(black_box(tx_id), black_box(&key)).await.unwrap();
            
            atomic_manager.commit_transaction(black_box(tx_id)).await.unwrap();
        });
    });
    
    // Benchmark transaction rollback
    group.bench_function("transaction_rollback", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            let atomic_manager = storage.atomic_manager();
            
            let tx_id = atomic_manager.begin_transaction().await.unwrap();
            
            let key = StorageKey::new("rollback_test").unwrap();
            let config = BenchConfig::new(1, 100);
            let value = StorageValue::from_serializable(&config).unwrap();
            
            atomic_manager.atomic_store(black_box(tx_id), black_box(&key), black_box(&value)).await.unwrap();
            atomic_manager.rollback_transaction(black_box(tx_id)).await.unwrap();
        });
    });
    
    group.finish();
}

/// Benchmark maintenance operations
fn benchmark_maintenance(c: &mut Criterion) {
    let runtime = get_runtime();
    
    let mut group = c.benchmark_group("maintenance");
    
    // Benchmark storage maintenance
    group.bench_function("storage_maintenance", |b| {
        b.to_async(&runtime).iter(|| async {
            let mut storage = init_mock_storage().unwrap();
            
            // Add some data first
            for i in 0..50 {
                let config = BenchConfig::new(i, 100);
                storage.store(&format!("config_{}", i), &config).await.unwrap();
            }
            
            // Perform maintenance
            storage.maintenance().await.unwrap();
        });
    });
    
    // Benchmark health check
    group.bench_function("health_check", |b| {
        let storage = runtime.block_on(async {
            let mut storage = init_mock_storage().unwrap();
            
            // Add some data
            for i in 0..20 {
                let config = BenchConfig::new(i, 100);
                storage.store(&format!("config_{}", i), &config).await.unwrap();
            }
            
            storage
        });
        
        b.iter(|| {
            let health = storage.get_health();
            black_box(health);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_basic_operations,
    benchmark_data_sizes,
    benchmark_batch_operations,
    benchmark_key_operations,
    benchmark_value_operations,
    benchmark_concurrent_patterns,
    benchmark_memory_overhead,
    benchmark_atomic_operations,
    benchmark_maintenance
);

criterion_main!(benches);