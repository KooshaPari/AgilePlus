//! Metric-emission tests for `agileplus_telemetry::metrics`.
//!
//! The crate's inline tests only assert that recording calls do not panic. These
//! tests drive the real OpenTelemetry metric pipeline with a capturing
//! [`PushMetricExporter`] and assert on what actually reaches an exporter:
//! instrument names, descriptions, aggregation kind, aggregated values, label
//! sets, histogram bucket boundaries, and histogram bucket counts.

use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use agileplus_telemetry::metrics::{AgilePlusMetrics, MetricsRecorder};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::{KeyValue, metrics::Meter};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::metrics::data::{AggregatedMetrics, MetricData, ResourceMetrics};
use opentelemetry_sdk::metrics::exporter::PushMetricExporter;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider, Temporality};

// ---------------------------------------------------------------------------
// Capturing exporter
// ---------------------------------------------------------------------------

/// Which OpenTelemetry aggregation produced a captured metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Sum,
    Gauge,
    Histogram,
}

/// One exported metric data point, flattened into assertion-friendly values.
#[derive(Debug, Clone, PartialEq)]
struct Point {
    attributes: Vec<(String, String)>,
    u64_value: Option<u64>,
    f64_value: Option<f64>,
    count: Option<u64>,
    min: Option<f64>,
    max: Option<f64>,
}

/// One exported instrument, with its aggregation metadata and data points.
#[derive(Debug, Clone)]
struct CapturedMetric {
    name: String,
    description: String,
    kind: Kind,
    monotonic: Option<bool>,
    bounds: Vec<f64>,
    bucket_counts: Vec<u64>,
    points: Vec<Point>,
}

fn labels<'a>(attributes: impl Iterator<Item = &'a KeyValue>) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = attributes
        .map(|kv| (kv.key.as_str().to_string(), kv.value.as_str().into_owned()))
        .collect();
    // OpenTelemetry does not guarantee attribute ordering; sort for stable
    // comparisons.
    pairs.sort();
    pairs
}

fn capture(metrics: &ResourceMetrics) -> Vec<CapturedMetric> {
    let mut captured = Vec::new();

    for scope in metrics.scope_metrics() {
        for metric in scope.metrics() {
            let mut bounds = Vec::new();
            let mut bucket_counts = Vec::new();

            let (kind, monotonic, points) = match metric.data() {
                AggregatedMetrics::U64(data) => match data {
                    MetricData::Sum(sum) => (
                        Kind::Sum,
                        Some(sum.is_monotonic()),
                        sum.data_points()
                            .map(|p| Point {
                                attributes: labels(p.attributes()),
                                u64_value: Some(p.value()),
                                f64_value: None,
                                count: None,
                                min: None,
                                max: None,
                            })
                            .collect(),
                    ),
                    MetricData::Gauge(gauge) => (
                        Kind::Gauge,
                        None,
                        gauge
                            .data_points()
                            .map(|p| Point {
                                attributes: labels(p.attributes()),
                                u64_value: Some(p.value()),
                                f64_value: None,
                                count: None,
                                min: None,
                                max: None,
                            })
                            .collect(),
                    ),
                    MetricData::Histogram(histogram) => {
                        bounds = histogram
                            .data_points()
                            .flat_map(|p| p.bounds().collect::<Vec<f64>>())
                            .collect();
                        bucket_counts = histogram
                            .data_points()
                            .flat_map(|p| p.bucket_counts().collect::<Vec<u64>>())
                            .collect();
                        (
                            Kind::Histogram,
                            None,
                            histogram
                                .data_points()
                                .map(|p| Point {
                                    attributes: labels(p.attributes()),
                                    u64_value: Some(p.sum()),
                                    f64_value: None,
                                    count: Some(p.count()),
                                    min: p.min().map(|v| v as f64),
                                    max: p.max().map(|v| v as f64),
                                })
                                .collect(),
                        )
                    }
                    MetricData::ExponentialHistogram(_) => (Kind::Histogram, None, Vec::new()),
                },
                AggregatedMetrics::F64(data) => match data {
                    MetricData::Sum(sum) => (
                        Kind::Sum,
                        Some(sum.is_monotonic()),
                        sum.data_points()
                            .map(|p| Point {
                                attributes: labels(p.attributes()),
                                u64_value: None,
                                f64_value: Some(p.value()),
                                count: None,
                                min: None,
                                max: None,
                            })
                            .collect(),
                    ),
                    MetricData::Gauge(gauge) => (
                        Kind::Gauge,
                        None,
                        gauge
                            .data_points()
                            .map(|p| Point {
                                attributes: labels(p.attributes()),
                                u64_value: None,
                                f64_value: Some(p.value()),
                                count: None,
                                min: None,
                                max: None,
                            })
                            .collect(),
                    ),
                    MetricData::Histogram(histogram) => {
                        bounds = histogram
                            .data_points()
                            .flat_map(|p| p.bounds().collect::<Vec<f64>>())
                            .collect();
                        bucket_counts = histogram
                            .data_points()
                            .flat_map(|p| p.bucket_counts().collect::<Vec<u64>>())
                            .collect();
                        (
                            Kind::Histogram,
                            None,
                            histogram
                                .data_points()
                                .map(|p| Point {
                                    attributes: labels(p.attributes()),
                                    u64_value: None,
                                    f64_value: Some(p.sum()),
                                    count: Some(p.count()),
                                    min: p.min(),
                                    max: p.max(),
                                })
                                .collect(),
                        )
                    }
                    MetricData::ExponentialHistogram(_) => (Kind::Histogram, None, Vec::new()),
                },
                AggregatedMetrics::I64(_) => (Kind::Sum, None, Vec::new()),
            };

            captured.push(CapturedMetric {
                name: metric.name().to_string(),
                description: metric.description().to_string(),
                kind,
                monotonic,
                bounds,
                bucket_counts,
                points,
            });
        }
    }

    captured
}

/// A push exporter that records everything the SDK hands it.
#[derive(Debug, Clone)]
struct CaptureExporter {
    store: Arc<Mutex<Vec<CapturedMetric>>>,
}

impl PushMetricExporter for CaptureExporter {
    fn export(&self, metrics: &ResourceMetrics) -> impl Future<Output = OTelSdkResult> + Send {
        let batch = capture(metrics);
        self.store
            .lock()
            .expect("capture store poisoned")
            .extend(batch);
        std::future::ready(Ok(()))
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }

    fn temporality(&self) -> Temporality {
        Temporality::Cumulative
    }
}

// ---------------------------------------------------------------------------
// Test pipeline
// ---------------------------------------------------------------------------

struct TestPipeline {
    provider: SdkMeterProvider,
    store: Arc<Mutex<Vec<CapturedMetric>>>,
}

impl TestPipeline {
    fn new() -> Self {
        let store = Arc::new(Mutex::new(Vec::new()));
        let reader = PeriodicReader::builder(CaptureExporter {
            store: Arc::clone(&store),
        })
        .build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        Self { provider, store }
    }

    fn meter(&self) -> Meter {
        self.provider.meter("agileplus-test")
    }

    fn recorder(&self) -> MetricsRecorder {
        MetricsRecorder::new(&self.meter())
    }

    /// Flush the pipeline and return everything exported so far.
    fn exported(&self) -> Vec<CapturedMetric> {
        self.provider.force_flush().expect("force_flush");
        self.store.lock().expect("capture store poisoned").clone()
    }
}

fn metric<'a>(captured: &'a [CapturedMetric], name: &str) -> &'a CapturedMetric {
    captured.iter().find(|m| m.name == name).unwrap_or_else(|| {
        panic!(
            "metric {name:?} was not exported; exported names: {:?}",
            captured.iter().map(|m| m.name.as_str()).collect::<Vec<_>>()
        )
    })
}

fn point<'a>(metric: &'a CapturedMetric, attributes: &[(&str, &str)]) -> &'a Point {
    let mut wanted: Vec<(String, String)> = attributes
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    wanted.sort();
    metric
        .points
        .iter()
        .find(|p| p.attributes == wanted)
        .unwrap_or_else(|| {
            panic!(
                "no data point with attributes {wanted:?} on metric {:?}; points: {:?}",
                metric.name, metric.points
            )
        })
}

// ---------------------------------------------------------------------------
// Counter emission
// ---------------------------------------------------------------------------

#[test]
fn agent_run_counter_exports_name_description_labels_and_aggregated_value() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_agent_run("001-sde", "WP10", "claude-code");
    recorder.record_agent_run("001-sde", "WP10", "claude-code");

    let exported = pipeline.exported();
    let metric = metric(&exported, "agileplus.agent.runs");

    assert_eq!(metric.description, "Number of agent invocations");
    assert_eq!(metric.kind, Kind::Sum);
    assert_eq!(
        metric.monotonic,
        Some(true),
        "agent runs is a monotonic counter"
    );
    assert_eq!(
        metric.points.len(),
        1,
        "identical label sets must aggregate into a single data point"
    );
    let data_point = point(
        metric,
        &[
            ("feature_slug", "001-sde"),
            ("wp_id", "WP10"),
            ("agent_type", "claude-code"),
        ],
    );
    assert_eq!(data_point.u64_value, Some(2));
}

#[test]
fn review_cycle_counter_exports_cycle_number_as_a_label() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_review_cycle("001-sde", "WP10", 3);

    let exported = pipeline.exported();
    let metric = metric(&exported, "agileplus.review.cycles");

    assert_eq!(metric.description, "Number of review-fix loop iterations");
    assert_eq!(metric.kind, Kind::Sum);
    let data_point = point(
        metric,
        &[
            ("feature_slug", "001-sde"),
            ("wp_id", "WP10"),
            ("cycle", "3"),
        ],
    );
    assert_eq!(data_point.u64_value, Some(1));
}

#[test]
fn events_processed_counter_separates_empty_and_populated_label_sets() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_event_processed(&[KeyValue::new("source", "git")]);
    recorder.record_event_processed(&[]);

    let exported = pipeline.exported();
    let metric = metric(&exported, "events_processed");

    assert_eq!(
        metric.description,
        "Total events appended to the event store"
    );
    assert_eq!(metric.kind, Kind::Sum);
    assert_eq!(metric.points.len(), 2);
    assert_eq!(point(metric, &[("source", "git")]).u64_value, Some(1));

    let unlabelled = metric
        .points
        .iter()
        .find(|p| p.attributes.is_empty())
        .expect("an unlabelled data point must be exported");
    assert_eq!(unlabelled.u64_value, Some(1));
}

// ---------------------------------------------------------------------------
// Histogram emission
// ---------------------------------------------------------------------------

#[test]
fn command_duration_histogram_aggregates_count_sum_min_max_and_buckets() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_command_duration("implement", Some("001-sde"), Duration::from_millis(1500));
    recorder.record_command_duration("implement", Some("001-sde"), Duration::from_millis(60));

    let exported = pipeline.exported();
    let metric = metric(&exported, "agileplus.command.duration_ms");

    assert_eq!(
        metric.description,
        "CLI command execution duration in milliseconds"
    );
    assert_eq!(metric.kind, Kind::Histogram);
    assert_eq!(
        metric.bounds,
        vec![
            10.0, 50.0, 100.0, 500.0, 1_000.0, 5_000.0, 30_000.0, 60_000.0
        ],
        "the command histogram must keep its configured bucket boundaries"
    );

    let data_point = point(
        metric,
        &[("command", "implement"), ("feature_slug", "001-sde")],
    );
    assert_eq!(data_point.count, Some(2));
    assert_eq!(data_point.f64_value, Some(1_560.0));
    assert_eq!(data_point.min, Some(60.0));
    assert_eq!(data_point.max, Some(1_500.0));

    // 60 ms lands in the (50, 100] bucket; 1500 ms lands in the (1000, 5000] bucket.
    assert_eq!(metric.bucket_counts, vec![0, 0, 1, 0, 0, 1, 0, 0, 0]);
}

#[test]
fn command_duration_without_feature_slug_omits_that_label() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_command_duration("list", None, Duration::from_secs(1));

    let exported = pipeline.exported();
    let metric = metric(&exported, "agileplus.command.duration_ms");

    assert_eq!(metric.points.len(), 1);
    let data_point = point(metric, &[("command", "list")]);
    assert_eq!(
        data_point.attributes,
        vec![("command".to_string(), "list".to_string())]
    );
    assert_eq!(data_point.f64_value, Some(1_000.0));
}

#[test]
fn command_duration_keeps_sub_millisecond_precision() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_command_duration("fast", None, Duration::from_micros(250));

    let exported = pipeline.exported();
    let metric = metric(&exported, "agileplus.command.duration_ms");

    assert_eq!(point(metric, &[("command", "fast")]).f64_value, Some(0.25));
}

#[test]
fn sync_and_api_histograms_use_their_configured_boundaries() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.record_sync_duration(120.0, &[KeyValue::new("sync_type", "full")]);
    recorder.record_api_request_duration(7.5, &[KeyValue::new("endpoint", "/v1/specs")]);

    let exported = pipeline.exported();

    let sync = metric(&exported, "sync_duration_ms");
    assert_eq!(
        sync.description,
        "Duration in milliseconds for sync operations"
    );
    assert_eq!(
        sync.bounds,
        vec![1.0, 5.0, 10.0, 50.0, 100.0, 500.0, 1_000.0, 5_000.0]
    );
    assert_eq!(point(sync, &[("sync_type", "full")]).f64_value, Some(120.0));

    let api = metric(&exported, "api_request_duration_ms");
    assert_eq!(api.description, "API request latency in milliseconds");
    assert_eq!(
        api.bounds,
        vec![1.0, 5.0, 10.0, 50.0, 100.0, 250.0, 500.0, 1_000.0]
    );
    assert_eq!(
        point(api, &[("endpoint", "/v1/specs")]).f64_value,
        Some(7.5)
    );
}

// ---------------------------------------------------------------------------
// Gauge emission
// ---------------------------------------------------------------------------

#[test]
fn cache_hit_rate_gauge_reports_the_latest_value_per_label_set() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.set_cache_hit_rate(0.25, &[KeyValue::new("cache", "default")]);
    recorder.set_cache_hit_rate(0.75, &[KeyValue::new("cache", "default")]);

    let exported = pipeline.exported();
    let metric = metric(&exported, "cache_hit_rate");

    assert_eq!(metric.description, "Cache hit/miss ratio (0.0 to 1.0)");
    assert_eq!(metric.kind, Kind::Gauge);
    assert_eq!(metric.points.len(), 1);
    assert_eq!(
        point(metric, &[("cache", "default")]).f64_value,
        Some(0.75),
        "a gauge keeps the most recent measurement"
    );
}

#[test]
fn active_features_gauge_reports_the_recorded_count() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();
    recorder.set_active_features(7, &[]);

    let exported = pipeline.exported();
    let metric = metric(&exported, "active_features");

    assert_eq!(metric.description, "Count of non-terminal features");
    assert_eq!(metric.kind, Kind::Gauge);
    assert_eq!(metric.points.len(), 1);
    assert!(metric.points[0].attributes.is_empty());
    assert_eq!(metric.points[0].u64_value, Some(7));
}

// ---------------------------------------------------------------------------
// AgilePlusMetrics facade
// ---------------------------------------------------------------------------

#[test]
fn agileplus_metrics_maps_str_tuple_labels_onto_instruments() {
    let pipeline = TestPipeline::new();
    let metrics = AgilePlusMetrics::new(&pipeline.meter());
    metrics.events_processed(&[("source", "git"), ("kind", "commit")]);

    let exported = pipeline.exported();
    let metric = metric(&exported, "events_processed");

    assert_eq!(metric.points.len(), 1);
    assert_eq!(
        point(metric, &[("source", "git"), ("kind", "commit")]).u64_value,
        Some(1)
    );
}

#[test]
fn agileplus_metrics_clone_shares_the_same_recorder() {
    let pipeline = TestPipeline::new();
    let metrics = AgilePlusMetrics::new(&pipeline.meter());
    let cloned = metrics.clone();

    metrics.events_processed(&[("source", "git")]);
    cloned.events_processed(&[("source", "git")]);

    let exported = pipeline.exported();
    let metric = metric(&exported, "events_processed");

    assert_eq!(
        metric.points.len(),
        1,
        "a clone must share the recorder so equal label sets aggregate together"
    );
    assert_eq!(point(metric, &[("source", "git")]).u64_value, Some(2));
}

#[test]
fn agileplus_metrics_facade_methods_reach_the_pipeline() {
    let pipeline = TestPipeline::new();
    let metrics = AgilePlusMetrics::new(&pipeline.meter());
    metrics.record_sync_duration(12.5, &[("sync_type", "full")]);
    metrics.set_cache_hit_rate(0.5, &[("cache", "default")]);
    metrics.record_api_request_duration(3.5, &[("endpoint", "/x")]);

    let exported = pipeline.exported();

    assert_eq!(
        point(
            metric(&exported, "sync_duration_ms"),
            &[("sync_type", "full")]
        )
        .f64_value,
        Some(12.5)
    );
    assert_eq!(
        point(metric(&exported, "cache_hit_rate"), &[("cache", "default")]).f64_value,
        Some(0.5)
    );
    assert_eq!(
        point(
            metric(&exported, "api_request_duration_ms"),
            &[("endpoint", "/x")]
        )
        .f64_value,
        Some(3.5)
    );
}

// ---------------------------------------------------------------------------
// Snapshot boundaries
// ---------------------------------------------------------------------------

#[test]
fn snapshot_duration_truncates_sub_millisecond_remainders() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();

    assert_eq!(
        recorder
            .collect_snapshot("sub-ms", Duration::from_micros(999))
            .duration_ms,
        0
    );
    assert_eq!(
        recorder
            .collect_snapshot("one-and-a-half", Duration::from_micros(1_500))
            .duration_ms,
        1
    );
    assert_eq!(
        recorder
            .collect_snapshot("ninety-seconds", Duration::from_secs(90))
            .duration_ms,
        90_000
    );
    assert_eq!(
        recorder
            .collect_snapshot("large", Duration::from_millis(4_294_967_296))
            .duration_ms,
        4_294_967_296
    );
}

#[test]
fn reset_discards_recorded_counts_for_later_snapshots() {
    let pipeline = TestPipeline::new();
    let recorder = pipeline.recorder();

    recorder.record_agent_run("f", "WP1", "codex");
    recorder.reset();
    recorder.record_agent_run("f", "WP2", "codex");

    let snapshot = recorder.collect_snapshot("after-reset", Duration::from_millis(2));
    assert_eq!(snapshot.command, "after-reset");
    assert_eq!(snapshot.duration_ms, 2);
    assert_eq!(snapshot.agent_runs, 1);
    assert_eq!(snapshot.review_cycles, 0);
}
