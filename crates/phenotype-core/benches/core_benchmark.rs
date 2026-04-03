//! Criterion benchmarks for phenotype-core crate.
//!
//! Benchmarks the umbrella crate's re-export aggregation overhead
//! and core type operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark error creation - the most common operation
fn bench_error_creation(c: &mut Criterion) {
    c.bench_function("error_api_not_found", |b| {
        b.iter(|| {
            let err = phenotype_core::error::ApiError::NotFound {
                resource: "test-resource".into(),
                id: "123".into(),
            };
            black_box(err);
        })
    });

    c.bench_function("error_domain_validation", |b| {
        b.iter(|| {
            let err = phenotype_core::error::DomainError::Validation("field required".into());
            black_box(err);
        })
    });

    c.bench_function("error_repo_not_found", |b| {
        b.iter(|| {
            let err = phenotype_core::error::RepositoryError::NotFound {
                entity: "user".into(),
                id: "456".into(),
            };
            black_box(err);
        })
    });
}

/// Benchmark type conversions and result types
fn bench_type_operations(c: &mut Criterion) {
    c.bench_function("domain_result_ok", |b| {
        b.iter(|| {
            let result: phenotype_core::types::DomainResult<String> = Ok("success".to_string());
            black_box(result.is_ok());
        })
    });

    c.bench_function("domain_result_err", |b| {
        b.iter(|| {
            let result: phenotype_core::types::DomainResult<String> =
                Err(phenotype_core::error::DomainError::NotFound {
                    entity: "test".into(),
                    id: "1".into(),
                });
            black_box(result.is_err());
        })
    });
}

/// Benchmark Event envelope creation
fn bench_event_operations(c: &mut Criterion) {
    c.bench_function("event_id_new", |b| {
        b.iter(|| {
            let id = phenotype_core::external::Ulid::new();
            black_box(id);
        })
    });
}

/// Benchmark DashMap operations (common concurrent collection)
fn bench_concurrent_collections(c: &mut Criterion) {
    use phenotype_core::external::DashMap;

    c.bench_function("dashmap_insert", |b| {
        let map = DashMap::new();
        let mut counter = 0u64;
        b.iter(|| {
            counter += 1;
            map.insert(counter, counter * 2);
        })
    });

    c.bench_function("dashmap_get", |b| {
        let map = DashMap::new();
        for i in 0..1000 {
            map.insert(i, i * 2);
        }
        let mut counter = 0u64;
        b.iter(|| {
            counter = (counter + 1) % 1000;
            black_box(map.get(&counter));
        })
    });
}

criterion_group!(
    core_benches,
    bench_error_creation,
    bench_type_operations,
    bench_event_operations,
    bench_concurrent_collections
);
criterion_main!(core_benches);
