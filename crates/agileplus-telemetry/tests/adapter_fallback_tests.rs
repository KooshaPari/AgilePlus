//! Failure-path, span-identity, and thread-safety tests for
//! `agileplus_telemetry::adapter`.
//!
//! The crate's existing adapter tests cover happy paths and method smoke calls.
//! These tests cover what happens when the configured OTLP endpoint cannot be
//! turned into an exporter, plus the span identity rules and the `Send`/`Sync`
//! contract. Nothing here opens a network connection: exporter construction is
//! lazy, and unparseable endpoints fail before any socket is created.

use std::{collections::HashMap, sync::Arc, time::Duration};

use agileplus_domain::ports::observability::ObservabilityPort;
use agileplus_telemetry::TelemetryAdapter;
use agileplus_telemetry::config::{OtlpConfig, OtlpProtocol, SamplingConfig, TelemetryConfig};

fn config_with_otlp(endpoint: &str, timeout_ms: u64) -> TelemetryConfig {
    TelemetryConfig {
        otlp: Some(OtlpConfig {
            endpoint: endpoint.to_string(),
            protocol: OtlpProtocol::Grpc,
            headers: HashMap::new(),
            timeout_ms,
            export_interval_ms: 60_000,
        }),
        ..TelemetryConfig::default()
    }
}

// ---------------------------------------------------------------------------
// Degraded OTLP export
// ---------------------------------------------------------------------------

#[test]
fn adapter_with_unparseable_endpoint_still_initialises() {
    let config = config_with_otlp("definitely not a url", 1_000);

    let adapter = TelemetryAdapter::new(config).expect("adapter init must not depend on OTLP");

    assert!(
        !adapter.is_noop(),
        "the adapter stays live; only trace export falls back to a no-op provider"
    );
    assert_eq!(
        adapter
            .config()
            .otlp
            .as_ref()
            .expect("otlp block is kept for diagnostics")
            .endpoint,
        "definitely not a url"
    );
}

#[test]
fn init_telemetry_with_unparseable_endpoint_returns_a_guard() {
    let guard = agileplus_telemetry::init_telemetry(config_with_otlp("not a url", 1_000));

    assert!(
        guard.is_ok(),
        "startup must degrade instead of failing when the collector endpoint is malformed"
    );
}

#[tokio::test]
async fn adapter_accepts_a_zero_timeout_otlp_config() {
    // `TelemetryConfig::validate` rejects timeout_ms == 0 when loading config, but
    // a directly constructed config must still produce a working adapter.
    let adapter = TelemetryAdapter::new(config_with_otlp("http://127.0.0.1:4317", 0))
        .expect("zero timeout must not panic the adapter");

    assert!(!adapter.is_noop());
    assert_eq!(
        adapter
            .config()
            .otlp
            .as_ref()
            .expect("otlp block")
            .timeout_ms,
        0
    );
}

#[tokio::test]
async fn adapter_keeps_the_full_supplied_otlp_configuration() {
    let mut config = config_with_otlp("http://127.0.0.1:1", 5);
    if let Some(otlp) = config.otlp.as_mut() {
        otlp.protocol = OtlpProtocol::Http;
        otlp.headers.insert("x-tenant".into(), "acme".into());
    }
    config.sampling = SamplingConfig { trace_ratio: 0.25 };

    let adapter = TelemetryAdapter::new(config).expect("adapter init");

    let kept = adapter.config();
    assert_eq!(kept.sampling.trace_ratio, 0.25);
    let otlp = kept.otlp.as_ref().expect("otlp block");
    assert_eq!(otlp.endpoint, "http://127.0.0.1:1");
    assert_eq!(otlp.protocol, OtlpProtocol::Http);
    assert_eq!(
        otlp.headers.get("x-tenant").map(String::as_str),
        Some("acme")
    );
    assert_eq!(otlp.timeout_ms, 5);
}

// ---------------------------------------------------------------------------
// Span identity
// ---------------------------------------------------------------------------

#[test]
fn real_adapter_root_span_seeds_its_trace_id_from_its_span_id() {
    let adapter = TelemetryAdapter::new(TelemetryConfig::default()).expect("adapter init");

    let root = adapter.start_span("root", None);

    assert!(root.parent_span_id.is_none(), "a root span has no parent");
    assert_eq!(
        root.trace_id, root.span_id,
        "without an incoming context the adapter reuses the span id as the trace id"
    );
    let span_id: u64 = root
        .span_id
        .parse()
        .expect("the adapter formats span ids as decimal u64 values");
    assert_ne!(span_id, 0, "real spans must have a nonzero id");
}

#[test]
fn real_adapter_child_span_inherits_the_parent_trace_and_span_ids() {
    let adapter = TelemetryAdapter::new(TelemetryConfig::default()).expect("adapter init");
    let root = adapter.start_span("root", None);

    let child = adapter.start_span("child", Some(&root));

    assert_eq!(child.trace_id, root.trace_id);
    assert_eq!(child.parent_span_id.as_deref(), Some(root.span_id.as_str()));
    assert_ne!(
        child.span_id, root.span_id,
        "each span must get its own span id"
    );
    let child_id: u64 = child.span_id.parse().expect("decimal u64 span id");
    assert_ne!(child_id, 0);
}

#[test]
fn noop_adapter_span_ids_are_the_zero_sentinel() {
    let adapter = TelemetryAdapter::noop();

    let context = adapter.start_span("operation", None);

    assert_eq!(
        context.span_id.parse::<u64>().expect("decimal span id"),
        0,
        "the noop span id is the zero sentinel"
    );
    assert_eq!(context.trace_id, "0".repeat(32));
}

// ---------------------------------------------------------------------------
// Metrics accessor wiring
// ---------------------------------------------------------------------------

#[test]
fn adapter_metrics_accessor_tracks_recorded_runs_in_snapshots() {
    let adapter = TelemetryAdapter::new(TelemetryConfig::default()).expect("adapter init");
    let metrics = adapter.metrics();

    metrics.record_agent_run("001-sde", "WP10", "claude-code");
    metrics.record_agent_run("001-sde", "WP11", "codex");
    metrics.record_review_cycle("001-sde", "WP10", 1);

    let snapshot = metrics.collect_snapshot("implement", Duration::from_millis(250));
    assert_eq!(snapshot.command, "implement");
    assert_eq!(snapshot.duration_ms, 250);
    assert_eq!(snapshot.agent_runs, 2);
    assert_eq!(snapshot.review_cycles, 1);

    let second = metrics.collect_snapshot("review", Duration::from_millis(10));
    assert_eq!(second.agent_runs, 0, "snapshots report deltas");
    assert_eq!(second.review_cycles, 0);
}

// ---------------------------------------------------------------------------
// Send/Sync contract
// ---------------------------------------------------------------------------

#[test]
fn adapter_is_usable_from_several_threads() {
    let adapter =
        Arc::new(TelemetryAdapter::new(TelemetryConfig::default()).expect("adapter init"));

    let workers: Vec<_> = (0..4)
        .map(|worker| {
            let adapter = Arc::clone(&adapter);
            std::thread::spawn(move || {
                adapter.record_counter("agileplus.thread.counter", 1, &[("worker", "shared")]);
                adapter.record_histogram("agileplus.thread.histogram", 1.0, &[]);
                adapter.record_gauge("agileplus.thread.gauge", 1.0, &[]);
                let context = adapter.start_span("worker", None);
                adapter.add_span_event(&context, "step", &[("worker", "shared")]);
                adapter.set_span_error(&context, "ignored-by-noop-subscriber");
                adapter.end_span(&context);
                adapter.log_info("worker finished");
                worker
            })
        })
        .collect();

    let mut finished: Vec<usize> = workers
        .into_iter()
        .map(|handle| handle.join().expect("worker thread must not panic"))
        .collect();
    finished.sort_unstable();
    assert_eq!(finished, vec![0, 1, 2, 3]);

    // The shared adapter is still usable after the workers joined.
    adapter.record_counter("agileplus.thread.counter", 1, &[]);
}
