//! Criterion benchmarks for phenotype-event-sourcing crate.
//!
//! Benchmarks event creation, hash computation, and chain verification.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark event hash computation
fn bench_hash_computation(c: &mut Criterion) {
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    c.bench_function("hash_compute_small_payload", |b| {
        let id = Uuid::new_v4();
        let ts = Utc::now();
        let payload = json!({"type": "UserCreated", "id": "123"});

        b.iter(|| {
            let hash = phenotype_event_sourcing::hash::compute_hash(
                &id,
                ts,
                "users",
                "user-123",
                &payload,
                "system",
                phenotype_event_sourcing::hash::ZERO_HASH,
            );
            black_box(hash);
        })
    });

    c.bench_function("hash_compute_large_payload", |b| {
        let id = Uuid::new_v4();
        let ts = Utc::now();
        let payload = json!({
            "type": "LargeEvent",
            "data": (0..1000).map(|i| format!("field_{}", i)).collect::<Vec<_>>(),
            "nested": {
                "level1": {
                    "level2": {
                        "values": (0..100).collect::<Vec<_>>()
                    }
                }
            }
        });

        b.iter(|| {
            let hash = phenotype_event_sourcing::hash::compute_hash(
                &id,
                ts,
                "events",
                "entity-456",
                &payload,
                "actor",
                phenotype_event_sourcing::hash::ZERO_HASH,
            );
            black_box(hash);
        })
    });
}

/// Benchmark event envelope creation
fn bench_event_envelope_creation(c: &mut Criterion) {
    c.bench_function("event_envelope_small", |b| {
        b.iter(|| {
            let event = phenotype_event_sourcing::EventEnvelope::new(
                "users",
                "user-123",
                "UserCreated",
                "system",
            );
            black_box(event);
        })
    });

    c.bench_function("event_envelope_with_struct", |b| {
        #[derive(Clone, serde::Serialize)]
        struct UserEvent {
            user_id: String,
            email: String,
            name: String,
        }

        let payload = UserEvent {
            user_id: "user-456".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
        };

        b.iter(|| {
            let event = phenotype_event_sourcing::EventEnvelope::new(
                "users",
                "user-456",
                payload.clone(),
                "api",
            );
            black_box(event);
        })
    });
}

/// Benchmark chain verification
fn bench_chain_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("chain_verify");

    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            // Build a chain of events
            let mut chain: Vec<(String, String)> = Vec::with_capacity(size);
            let zero = phenotype_event_sourcing::hash::ZERO_HASH.to_string();
            let mut prev_hash = zero;

            for i in 0..size {
                let hash = format!("{:064x}", i);
                chain.push((hash.clone(), prev_hash));
                prev_hash = hash;
            }

            b.iter(|| {
                let result = phenotype_event_sourcing::hash::verify_chain(black_box(&chain));
                black_box(result);
            })
        });
    }

    group.finish();
}

/// Benchmark memory store operations
fn bench_memory_store(c: &mut Criterion) {
    use phenotype_event_sourcing::memory::InMemoryEventStore;

    c.bench_function("memory_store_append", |b| {
        let store = InMemoryEventStore::new();
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            let event = phenotype_event_sourcing::EventEnvelope::new(
                "test",
                format!("entity-{}", counter),
                format!("payload-{}", counter),
                "system",
            );
            store.append(event).unwrap();
        })
    });

    c.bench_function("memory_store_get_stream", |b| {
        let store = InMemoryEventStore::new();

        // Pre-populate with events
        for i in 0..100 {
            let event = phenotype_event_sourcing::EventEnvelope::new(
                "orders",
                "order-123",
                format!("event-{}", i),
                "system",
            );
            store.append(event).unwrap();
        }

        b.iter(|| {
            let stream = store.get_stream("orders", "order-123").unwrap();
            black_box(stream.len());
        })
    });
}

criterion_group!(
    event_sourcing_benches,
    bench_hash_computation,
    bench_event_envelope_creation,
    bench_chain_verification,
    bench_memory_store
);
criterion_main!(event_sourcing_benches);
