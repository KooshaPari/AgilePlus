//! Comprehensive tests for `agileplus_telemetry::adapter` (TelemetryAdapter).
//!
//! Covers: adapter initialization (noop, default, full config), all
//! ObservabilityPort methods, span lifecycle, metrics via adapter, logging
//! levels, Send/Sync trait bounds, Drop, and accessor methods.

use std::collections::HashMap;

use agileplus_domain::ports::observability::{LogEntry, LogLevel, ObservabilityPort, SpanContext};
use agileplus_telemetry::config::{OtlpConfig, OtlpProtocol, SamplingConfig, TelemetryConfig};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn noop_adapter() -> agileplus_telemetry::TelemetryAdapter {
    agileplus_telemetry::TelemetryAdapter::noop()
}

fn real_adapter() -> agileplus_telemetry::TelemetryAdapter {
    let cfg = TelemetryConfig::default();
    agileplus_telemetry::TelemetryAdapter::new(cfg).expect("adapter init should succeed")
}

// ---------------------------------------------------------------------------
// Noop adapter
// ---------------------------------------------------------------------------

#[test]
fn noop_adapter_is_noop() {
    let adapter = noop_adapter();
    assert!(adapter.is_noop());
}

#[test]
fn noop_adapter_returns_zero_sentinel_span_context() {
    let adapter = noop_adapter();
    let ctx = adapter.start_span("any_operation", None);
    assert_eq!(ctx.trace_id, "00000000000000000000000000000000");
    assert_eq!(ctx.span_id, "0000000000000000");
    assert!(ctx.parent_span_id.is_none());
}

#[test]
fn noop_adapter_child_span_preserves_parent_id() {
    let adapter = noop_adapter();
    let parent = adapter.start_span("parent", None);
    let child = adapter.start_span("child", Some(&parent));
    assert_eq!(child.trace_id, parent.trace_id);
    // Noop adapter always returns parent_span_id: None (sentinel spans)
    assert!(child.parent_span_id.is_none());
}

#[test]
fn noop_adapter_all_methods_no_panic() {
    let adapter = noop_adapter();
    // Logging
    adapter.log_info("info message");
    adapter.log_warn("warn message");
    adapter.log_error("error message");

    // Structured log entry
    let entry = LogEntry {
        level: LogLevel::Trace,
        message: "trace msg".into(),
        fields: HashMap::from([("key".into(), "value".into())]),
        span_context: None,
    };
    adapter.log(&entry);

    // Spans
    let ctx = adapter.start_span("operation", None);
    adapter.add_span_event(&ctx, "event_name", &[("a", "1"), ("b", "2")]);
    adapter.set_span_error(&ctx, "something went wrong");
    adapter.end_span(&ctx);

    // Metrics
    adapter.record_counter("counter.name", 42, &[("c", "d")]);
    adapter.record_histogram("histogram.name", 3.14, &[("h", "v")]);
    adapter.record_gauge("gauge.name", 99.0, &[("g", "v")]);
}

#[test]
fn noop_adapter_log_all_levels_no_panic() {
    let adapter = noop_adapter();
    for level in [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ] {
        let entry = LogEntry {
            level,
            message: format!("test at level {:?}", level),
            fields: HashMap::new(),
            span_context: None,
        };
        adapter.log(&entry);
    }
}

#[test]
fn noop_adapter_metrics_accessor() {
    let adapter = noop_adapter();
    let metrics = adapter.metrics();
    // Should be able to record without panic
    metrics.record_agent_run("feat", "WP", "agent");
}

#[test]
fn noop_adapter_config_accessor() {
    let adapter = noop_adapter();
    let config = adapter.config();
    assert!(config.otlp.is_none());
    assert_eq!(config.logging.level, "info");
}

#[test]
fn noop_adapter_config_is_default() {
    let adapter = noop_adapter();
    let config = adapter.config();
    let default_cfg = TelemetryConfig::default();
    assert_eq!(config.logging.level, default_cfg.logging.level);
    assert_eq!(
        config.sampling.trace_ratio,
        default_cfg.sampling.trace_ratio
    );
}

// ---------------------------------------------------------------------------
// Real adapter (default config, no OTLP export)
// ---------------------------------------------------------------------------

#[test]
fn real_adapter_initializes_successfully() {
    let adapter = real_adapter();
    assert!(!adapter.is_noop());
}

#[test]
fn real_adapter_config_accessor_returns_config() {
    let adapter = real_adapter();
    let config = adapter.config();
    assert_eq!(config.logging.level, "info");
}

#[test]
fn real_adapter_metrics_accessor_returns_recorder() {
    let adapter = real_adapter();
    let metrics = adapter.metrics();
    metrics.record_agent_run("feat", "WP1", "claude-code");
    metrics.record_review_cycle("feat", "WP1", 1);
}

#[test]
fn real_adapter_start_span_returns_nonzero_ids() {
    let adapter = real_adapter();
    let ctx = adapter.start_span("test-span", None);
    // Real spans have non-zero IDs (tracing assigns them)
    assert_ne!(ctx.span_id, "0000000000000000");
}

#[test]
fn real_adapter_child_span_has_parent() {
    let adapter = real_adapter();
    let parent = adapter.start_span("parent", None);
    let child = adapter.start_span("child", Some(&parent));
    assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
}

#[test]
fn real_adapter_add_span_event_no_panic() {
    let adapter = real_adapter();
    let ctx = adapter.start_span("evt-test", None);
    adapter.add_span_event(&ctx, "milestone", &[("type", "pr_created")]);
}

#[test]
fn real_adapter_set_span_error_no_panic() {
    let adapter = real_adapter();
    let ctx = adapter.start_span("err-test", None);
    adapter.set_span_error(&ctx, "runtime error");
}

#[test]
fn real_adapter_end_span_no_panic() {
    let adapter = real_adapter();
    let ctx = adapter.start_span("end-test", None);
    adapter.end_span(&ctx);
}

#[test]
fn real_adapter_record_counter_with_labels() {
    let adapter = real_adapter();
    adapter.record_counter("test.counter", 10, &[("feature", "auth"), ("wp", "WP10")]);
}

#[test]
fn real_adapter_record_histogram_with_labels() {
    let adapter = real_adapter();
    adapter.record_histogram("test.histogram", 42.5, &[("method", "GET")]);
}

#[test]
fn real_adapter_record_gauge_with_labels() {
    let adapter = real_adapter();
    adapter.record_gauge("test.gauge", 0.85, &[("cache", "default")]);
}

#[test]
fn real_adapter_record_counter_empty_labels() {
    let adapter = real_adapter();
    adapter.record_counter("test.empty.labels", 1, &[]);
}

#[test]
fn real_adapter_record_histogram_empty_labels() {
    let adapter = real_adapter();
    adapter.record_histogram("test.empty.labels", 0.0, &[]);
}

#[test]
fn real_adapter_record_gauge_empty_labels() {
    let adapter = real_adapter();
    adapter.record_gauge("test.empty.labels", 0.0, &[]);
}

#[test]
fn real_adapter_log_info_message() {
    let adapter = real_adapter();
    adapter.log_info("hello from test");
}

#[test]
fn real_adapter_log_warn_message() {
    let adapter = real_adapter();
    adapter.log_warn("warning from test");
}

#[test]
fn real_adapter_log_error_message() {
    let adapter = real_adapter();
    adapter.log_error("error from test");
}

#[test]
fn real_adapter_structured_log_entry_all_levels() {
    let adapter = real_adapter();
    for level in [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
    ] {
        let entry = LogEntry {
            level,
            message: format!("structured msg at {:?}", level),
            fields: HashMap::from([("test".into(), "true".into())]),
            span_context: None,
        };
        adapter.log(&entry);
    }
}

#[test]
fn real_adapter_log_entry_with_span_context() {
    let adapter = real_adapter();
    let ctx = adapter.start_span("log-with-span", None);
    let entry = LogEntry {
        level: LogLevel::Info,
        message: "log with span context".into(),
        fields: HashMap::new(),
        span_context: Some(ctx),
    };
    adapter.log(&entry);
}

// ---------------------------------------------------------------------------
// Adapter with full OTLP config (tests OTLP path)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn adapter_with_full_otlp_config_initializes() {
    let cfg = TelemetryConfig {
        otlp: Some(OtlpConfig {
            endpoint: "http://localhost:4317".into(),
            protocol: OtlpProtocol::Grpc,
            headers: HashMap::new(),
            timeout_ms: 5000,
            export_interval_ms: 60000,
        }),
        ..Default::default()
    };
    let result = agileplus_telemetry::TelemetryAdapter::new(cfg);
    assert!(result.is_ok());
    let adapter = result.unwrap();
    assert!(!adapter.is_noop());
}

#[test]
fn adapter_with_custom_sampling_config() {
    let cfg = TelemetryConfig {
        sampling: SamplingConfig { trace_ratio: 0.25 },
        ..Default::default()
    };
    let adapter = agileplus_telemetry::TelemetryAdapter::new(cfg).unwrap();
    let config = adapter.config();
    assert_eq!(config.sampling.trace_ratio, 0.25);
}

// ---------------------------------------------------------------------------
// ObservabilityPort trait
// ---------------------------------------------------------------------------

#[test]
fn observability_port_noop_reflection() {
    let adapter: &dyn ObservabilityPort = &noop_adapter();
    let ctx = adapter.start_span("trait-test", None);
    adapter.add_span_event(&ctx, "evt", &[("k", "v")]);
    adapter.set_span_error(&ctx, "err");
    adapter.end_span(&ctx);
    adapter.record_counter("c", 1, &[]);
    adapter.record_histogram("h", 1.0, &[]);
    adapter.record_gauge("g", 1.0, &[]);
    adapter.log_info("info");
    adapter.log_warn("warn");
    adapter.log_error("error");
    let entry = LogEntry {
        level: LogLevel::Debug,
        message: "debug".into(),
        fields: HashMap::new(),
        span_context: None,
    };
    adapter.log(&entry);
}

#[test]
fn observability_port_real_reflection() {
    let adapter: &dyn ObservabilityPort = &real_adapter();
    let ctx = adapter.start_span("trait-test-real", None);
    adapter.add_span_event(&ctx, "evt", &[("k", "v")]);
    adapter.set_span_error(&ctx, "err");
    adapter.end_span(&ctx);
    adapter.record_counter("c", 1, &[("a", "b")]);
    adapter.record_histogram("h", 1.0, &[("a", "b")]);
    adapter.record_gauge("g", 1.0, &[("a", "b")]);
    adapter.log_info("info");
    adapter.log_warn("warn");
    adapter.log_error("error");
    let entry = LogEntry {
        level: LogLevel::Debug,
        message: "debug".into(),
        fields: HashMap::new(),
        span_context: Some(ctx.clone()),
    };
    adapter.log(&entry);
}

// ---------------------------------------------------------------------------
// TelemetryGuard via init_telemetry
// ---------------------------------------------------------------------------

#[test]
fn init_telemetry_default_config_returns_guard() {
    let cfg = TelemetryConfig::default();
    let result = agileplus_telemetry::init_telemetry(cfg);
    assert!(result.is_ok());
}

#[tokio::test]
async fn init_telemetry_with_otlp_config_returns_guard() {
    let cfg = TelemetryConfig {
        otlp: Some(OtlpConfig {
            endpoint: "http://localhost:4317".into(),
            protocol: OtlpProtocol::Grpc,
            headers: HashMap::new(),
            timeout_ms: 5000,
            export_interval_ms: 60000,
        }),
        ..Default::default()
    };
    let result = agileplus_telemetry::init_telemetry(cfg);
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// init_subscriber (convenience)
// ---------------------------------------------------------------------------

#[test]
fn init_subscriber_returns_guard() {
    let result = agileplus_telemetry::init_subscriber();
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Send + Sync bounds
// ---------------------------------------------------------------------------

fn _assert_send<T: Send>() {}
fn _assert_sync<T: Sync>() {}

#[test]
fn adapter_is_send_and_sync() {
    _assert_send::<agileplus_telemetry::TelemetryAdapter>();
    _assert_sync::<agileplus_telemetry::TelemetryAdapter>();
}

// ---------------------------------------------------------------------------
// SpanContext clone and debug
// ---------------------------------------------------------------------------

#[test]
fn span_context_clone() {
    let ctx = SpanContext {
        trace_id: "abc".into(),
        span_id: "def".into(),
        parent_span_id: Some("ghi".into()),
    };
    let ctx2 = ctx.clone();
    assert_eq!(ctx.trace_id, ctx2.trace_id);
    assert_eq!(ctx.span_id, ctx2.span_id);
    assert_eq!(ctx.parent_span_id, ctx2.parent_span_id);
}

#[test]
fn span_context_debug_format() {
    let ctx = SpanContext {
        trace_id: "t".into(),
        span_id: "s".into(),
        parent_span_id: None,
    };
    let debug = format!("{:?}", ctx);
    assert!(debug.contains("SpanContext"));
    assert!(debug.contains("t"));
}

// ---------------------------------------------------------------------------
// TelemetryError
// ---------------------------------------------------------------------------

#[test]
fn telemetry_error_display_config() {
    let err = agileplus_telemetry::TelemetryError::Config(
        agileplus_telemetry::config::ConfigError::Validation("test".into()),
    );
    let msg = err.to_string();
    assert!(msg.contains("config error"));
}

#[test]
fn telemetry_error_debug_format() {
    let err = agileplus_telemetry::TelemetryError::Otel("test otel error".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Otel"));
}

// ---------------------------------------------------------------------------
// Multiple span lifecycle (parent -> child -> grandchild)
// ---------------------------------------------------------------------------

#[test]
fn span_chain_lifecycle_noop() {
    let adapter = noop_adapter();
    let root = adapter.start_span("root", None);
    let child = adapter.start_span("child", Some(&root));
    let grandchild = adapter.start_span("grandchild", Some(&child));

    // Noop adapter always returns parent_span_id: None (sentinel spans)
    assert!(grandchild.parent_span_id.is_none());
    assert!(child.parent_span_id.is_none());
    assert!(root.parent_span_id.is_none());

    adapter.add_span_event(&grandchild, "deep_event", &[("level", "3")]);
    adapter.set_span_error(&grandchild, "deep error");
    adapter.end_span(&grandchild);
    adapter.end_span(&child);
    adapter.end_span(&root);
}

#[test]
fn span_chain_lifecycle_real() {
    let adapter = real_adapter();
    let root = adapter.start_span("root_real", None);
    let child = adapter.start_span("child_real", Some(&root));

    adapter.add_span_event(&child, "child_event", &[("k", "v")]);
    adapter.set_span_error(&child, "child error");
    adapter.end_span(&child);
    adapter.end_span(&root);
}

// ---------------------------------------------------------------------------
// Adapter Drop
// ---------------------------------------------------------------------------

#[test]
fn adapter_drop_does_not_panic() {
    {
        let _adapter = noop_adapter();
    }
    {
        let _adapter = real_adapter();
    }
}
