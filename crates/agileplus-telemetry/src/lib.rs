//! AgilePlus OpenTelemetry telemetry crate.
//!
//! Provides a thin hexagonal telemetry init module with:
//! - [`init_subscriber`] — configures the global `tracing` subscriber with an
//!   optional OTLP export layer.  No-op (stdout-only) when
//!   `OTEL_EXPORTER_OTLP_ENDPOINT` is unset.
//! - [`TelemetryAdapter`] — implements `ObservabilityPort` using the global
//!   OTel tracer + `tracing`.
//! - [`TelemetryGuard`] — RAII guard; flush happens on drop.
//!
//! The OTLP exporter is gated entirely on the presence of the env-var so local
//! runs and unit tests never require a live collector.

pub mod adapter;
pub mod config;
pub mod logs;
pub mod metrics;
pub mod traces;

pub use adapter::{init_telemetry, TelemetryAdapter, TelemetryError, TelemetryGuard};
pub use config::TelemetryConfig;

use tracing_subscriber::prelude::*;

/// Result of [`init_subscriber`].
pub struct SubscriberGuard {
    _log_guard: Option<tracing_appender::non_blocking::WorkerGuard>,
    _tracer_provider: Option<opentelemetry_sdk::trace::TracerProvider>,
}

/// Initialise the global `tracing` subscriber.
///
/// No-op when `OTEL_EXPORTER_OTLP_ENDPOINT` is unset: no network connection
/// is attempted and the function succeeds.
pub fn init_subscriber() -> Result<SubscriberGuard, String> {
    let level_filter = std::env::var("AGILEPLUS_LOG_LEVEL")
        .or_else(|_| std::env::var("RUST_LOG"))
        .unwrap_or_else(|_| "info".into());

    let env_filter = tracing_subscriber::EnvFilter::try_new(&level_filter)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let (stdout_writer, log_guard) = tracing_appender::non_blocking(std::io::stdout());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(stdout_writer)
        .with_target(true);

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    let (tracer_provider, otel_layer) = if let Some(ref ep) = endpoint {
        use opentelemetry::trace::TracerProvider as _;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::trace::TracerProvider;

        let build_result = (|| -> Result<TracerProvider, String> {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_endpoint(ep)
                .build()
                .map_err(|e| e.to_string())?;
            Ok(TracerProvider::builder()
                .with_simple_exporter(exporter)
                .build())
        })();

        match build_result {
            Ok(provider) => {
                opentelemetry::global::set_tracer_provider(provider.clone());
                let tracer = provider.tracer("agileplus");
                let layer = tracing_opentelemetry::layer().with_tracer(tracer);
                (Some(provider), Some(layer))
            }
            Err(e) => {
                eprintln!(
                    "[agileplus-telemetry] OTLP exporter init failed: {e}; continuing without OTLP export"
                );
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init()
        .map_err(|e| e.to_string())?;

    Ok(SubscriberGuard {
        _log_guard: Some(log_guard),
        _tracer_provider: tracer_provider,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify init_subscriber succeeds (or "already set") when no OTLP
    /// endpoint is configured — no network connection must occur.
    #[test]
    fn init_subscriber_no_endpoint_is_noop() {
        // SAFETY: single-threaded test binary; no concurrent env-var readers.
        unsafe { std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT") };
        let result = init_subscriber();
        match result {
            Ok(guard) => {
                assert!(
                    guard._tracer_provider.is_none(),
                    "no OTel provider expected when endpoint is absent"
                );
            }
            Err(e) => {
                // Another test already set a global subscriber — acceptable.
                assert!(
                    e.contains("already") || e.contains("set"),
                    "unexpected error: {e}"
                );
            }
        }
    }

    /// Verify TelemetryAdapter::noop() works without a collector.
    #[test]
    fn noop_adapter_works_without_collector() {
        use agileplus_domain::ports::observability::ObservabilityPort;
        let adapter = TelemetryAdapter::noop();
        assert!(adapter.is_noop());
        adapter.log_info("hello");
        adapter.log_warn("warn");
        adapter.log_error("error");
        let ctx = adapter.start_span("test-span", None);
        adapter.add_span_event(&ctx, "evt", &[("k", "v")]);
        adapter.set_span_error(&ctx, "oops");
        adapter.end_span(&ctx);
    }

    /// Verify init_telemetry with default (no OTLP) config returns a guard.
    #[test]
    fn init_telemetry_no_otlp_returns_guard() {
        let config = TelemetryConfig::default();
        assert!(
            config.otlp.is_none(),
            "default config must have no OTLP endpoint"
        );
        let result = init_telemetry(config);
        assert!(result.is_ok(), "init_telemetry failed: {:?}", result.err());
    }
}
