//! Criterion benchmarks for phenotype-cache-adapter crate.
//!
//! Benchmarks two-tier cache operations: L1 (LRU) and L2 (Moka) tiers.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark cache get operations - L1 hit vs L2 hit vs miss
fn bench_cache_get(c: &mut Criterion) {
    use phenotype_cache_adapter::TwoTierCache;

    let mut group = c.benchmark_group("cache_get");

    // L1 hit - fastest path
    group.bench_function("l1_hit", |b| {
        let cache = TwoTierCache::new(1000, 10000);
        cache.put("key1", "value1".to_string());

        b.iter(|| {
            let result = cache.get(&"key1");
            black_box(result);
        })
    });

    // L2 hit - requires L2 lookup and L1 promotion
    group.bench_function("l2_hit", |b| {
        let cache = TwoTierCache::new(100, 10000);
        // Fill L1 to capacity
        for i in 0..100 {
            cache.put(format!("l1-key-{}", i), format!("value-{}", i));
        }
        // This key will be in L2 only
        cache.put("l2-key", "l2-value".to_string());

        b.iter(|| {
            let result = cache.get(&"l2-key");
            black_box(result);
        })
    });

    // Cache miss - slowest path
    group.bench_function("cache_miss", |b| {
        let cache = TwoTierCache::new(1000, 10000);

        b.iter(|| {
            let result = cache.get(&"nonexistent");
            black_box(result);
        })
    });

    group.finish();
}

/// Benchmark cache put operations
fn bench_cache_put(c: &mut Criterion) {
    use phenotype_cache_adapter::TwoTierCache;

    let mut group = c.benchmark_group("cache_put");

    group.bench_function("put_small_value", |b| {
        let cache = TwoTierCache::new(1000, 10000);
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            cache.put(counter, "small value".to_string());
        })
    });

    group.bench_function("put_large_value", |b| {
        let cache = TwoTierCache::new(1000, 10000);
        let large_value = "x".repeat(10000);
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            cache.put(counter, large_value.clone());
        })
    });

    group.finish();
}

/// Benchmark cache with varying sizes
fn bench_cache_scaling(c: &mut Criterion) {
    use phenotype_cache_adapter::TwoTierCache;

    let mut group = c.benchmark_group("cache_scaling");

    for l1_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*l1_size as u64));
        group.bench_with_input(
            BenchmarkId::new("sequential_access", l1_size),
            l1_size,
            |b, &l1_size| {
                let cache = TwoTierCache::new(l1_size, l1_size as u64 * 10);

                // Populate cache
                for i in 0..l1_size {
                    cache.put(i, format!("value-{}", i));
                }

                let mut idx = 0usize;
                b.iter(|| {
                    idx = (idx + 1) % l1_size;
                    let result = cache.get(&idx);
                    black_box(result);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent cache access (simulating real-world usage)
fn bench_concurrent_cache(c: &mut Criterion) {
    use phenotype_cache_adapter::TwoTierCache;
    use std::sync::Arc;
    use std::thread;

    c.bench_function("concurrent_reads", |b| {
        let cache = Arc::new(TwoTierCache::new(1000, 10000));

        // Pre-populate
        for i in 0..1000 {
            cache.put(i, format!("value-{}", i));
        }

        b.iter(|| {
            let cache_clone = cache.clone();
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    let cache = cache_clone.clone();
                    thread::spawn(move || {
                        for i in 0..100 {
                            let key = (thread_id * 100 + i) % 1000;
                            black_box(cache.get(&key));
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        })
    });
}

/// Benchmark cache under churn (high insert rate)
fn bench_cache_churn(c: &mut Criterion) {
    use phenotype_cache_adapter::TwoTierCache;

    c.bench_function("high_churn", |b| {
        let cache = TwoTierCache::new(100, 1000);
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            cache.put(counter, format!("value-{}", counter));
            // Read a recent entry
            black_box(cache.get(&(counter.saturating_sub(10))));
        })
    });
}

criterion_group!(
    cache_adapter_benches,
    bench_cache_get,
    bench_cache_put,
    bench_cache_scaling,
    bench_concurrent_cache,
    bench_cache_churn
);
criterion_main!(cache_adapter_benches);
