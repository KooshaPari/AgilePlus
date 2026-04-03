//! Criterion benchmarks for phenotype-error-core crate.
//!
//! Benchmarks error creation, conversion, and serialization performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark error type creation
fn bench_error_creation(c: &mut Criterion) {
    use phenotype_error_core::{ApiError, ConfigError, DomainError, RepositoryError, StorageError};

    let mut group = c.benchmark_group("error_creation");

    // API errors
    group.bench_function("api_not_found", |b| {
        b.iter(|| {
            let err = ApiError::NotFound {
                resource: "user".into(),
                id: "123".into(),
            };
            black_box(err);
        })
    });

    group.bench_function("api_bad_request", |b| {
        b.iter(|| {
            let err = ApiError::BadRequest("invalid input".into());
            black_box(err);
        })
    });

    // Domain errors
    group.bench_function("domain_validation", |b| {
        b.iter(|| {
            let err = DomainError::Validation("field required".into());
            black_box(err);
        })
    });

    group.bench_function("domain_not_found", |b| {
        b.iter(|| {
            let err = DomainError::NotFound {
                entity: "order".into(),
                id: "456".into(),
            };
            black_box(err);
        })
    });

    // Repository errors
    group.bench_function("repo_not_found", |b| {
        b.iter(|| {
            let err = RepositoryError::NotFound {
                entity: "record".into(),
                id: "789".into(),
            };
            black_box(err);
        })
    });

    group.bench_function("repo_connection", |b| {
        b.iter(|| {
            let err = RepositoryError::Connection("database offline".into());
            black_box(err);
        })
    });

    // Config errors
    group.bench_function("config_file_not_found", |b| {
        use std::path::PathBuf;

        b.iter(|| {
            let err = ConfigError::FileNotFound {
                path: PathBuf::from("/etc/config.toml"),
            };
            black_box(err);
        })
    });

    // Storage errors
    group.bench_function("storage_io", |b| {
        b.iter(|| {
            let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
            let err = StorageError::Io(io_err);
            black_box(err);
        })
    });

    group.finish();
}

/// Benchmark error conversions
fn bench_error_conversions(c: &mut Criterion) {
    use phenotype_error_core::{ApiError, DomainError, RepositoryError, StorageError};

    let mut group = c.benchmark_group("error_conversions");

    // Domain -> API
    group.bench_function("domain_to_api", |b| {
        let domain_err = DomainError::validation("test");

        b.iter(|| {
            let api_err: ApiError = domain_err.clone().into();
            black_box(api_err);
        })
    });

    // Repository -> API
    group.bench_function("repo_to_api", |b| {
        let repo_err = RepositoryError::NotFound {
            entity: "user".into(),
            id: "123".into(),
        };

        b.iter(|| {
            let api_err: ApiError = repo_err.clone().into();
            black_box(api_err);
        })
    });

    // Storage -> Repository
    group.bench_function("storage_to_repo", |b| {
        let storage_err = StorageError::NotFound("file.dat".into());

        b.iter(|| {
            let repo_err: RepositoryError = storage_err.clone().into();
            black_box(repo_err);
        })
    });

    group.finish();
}

/// Benchmark error operations (status_code, is_retryable, etc.)
fn bench_error_operations(c: &mut Criterion) {
    use phenotype_error_core::ApiError;

    let mut group = c.benchmark_group("error_operations");

    group.bench_function("status_code", |b| {
        let err = ApiError::NotFound {
            resource: "user".into(),
            id: "123".into(),
        };

        b.iter(|| {
            let code = err.status_code();
            black_box(code);
        })
    });

    group.bench_function("is_retryable", |b| {
        let err = ApiError::RateLimited;

        b.iter(|| {
            let retryable = err.is_retryable();
            black_box(retryable);
        })
    });

    group.bench_function("error_to_string", |b| {
        let err = ApiError::Internal("something went wrong".into());

        b.iter(|| {
            let msg = err.to_string();
            black_box(msg);
        })
    });

    group.finish();
}

/// Benchmark error envelope creation and serialization
fn bench_error_envelope(c: &mut Criterion) {
    use phenotype_error_core::{ApiError, ErrorEnvelope};

    let mut group = c.benchmark_group("error_envelope");

    group.bench_function("envelope_from_api", |b| {
        let err = ApiError::NotFound {
            resource: "project".into(),
            id: "42".into(),
        };

        b.iter(|| {
            let envelope = ErrorEnvelope::from(&err);
            black_box(envelope);
        })
    });

    group.bench_function("envelope_serialize", |b| {
        let err = ApiError::BadRequest("invalid input".into());
        let envelope = ErrorEnvelope::from(&err);

        b.iter(|| {
            let json = serde_json::to_string(&envelope).unwrap();
            black_box(json);
        })
    });

    group.bench_function("envelope_deserialize", |b| {
        let json = r#"{"code":"ERR_404","message":"not found"}"#;

        b.iter(|| {
            let envelope: ErrorEnvelope = serde_json::from_str(json).unwrap();
            black_box(envelope);
        })
    });

    group.finish();
}

/// Benchmark anyhow interop
fn bench_anyhow_interop(c: &mut Criterion) {
    use phenotype_error_core::DomainError;

    c.bench_function("to_anyhow", |b| {
        let err = DomainError::Validation("test error".into());

        b.iter(|| {
            let anyhow_err: anyhow::Error = err.clone().into();
            black_box(anyhow_err);
        })
    });
}

/// Benchmark context helper
fn bench_context_helper(c: &mut Criterion) {
    use phenotype_error_core::ErrorContext;

    c.bench_function("context_ok", |b| {
        let result: Result<(), &str> = Ok(());

        b.iter(|| {
            let _ = result.context("operation").is_ok();
        })
    });

    c.bench_function("context_err", |b| {
        let result: Result<(), &str> = Err("failure");

        b.iter(|| {
            let ctx = result.context("loading config");
            black_box(ctx);
        })
    });
}

criterion_group!(
    error_core_benches,
    bench_error_creation,
    bench_error_conversions,
    bench_error_operations,
    bench_error_envelope,
    bench_anyhow_interop,
    bench_context_helper
);
criterion_main!(error_core_benches);
