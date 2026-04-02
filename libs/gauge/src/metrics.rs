//! Metrics collection

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// A metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MetricValue {
    Counter {
        value: u64,
    },
    Gauge {
        value: f64,
    },
    Histogram {
        samples: Vec<f64>,
        count: u64,
        sum: f64,
        min: f64,
        max: f64,
    },
    Timing {
        duration_ms: f64,
    },
}

impl MetricValue {
    pub fn counter(value: u64) -> Self {
        Self::Counter { value }
    }
    pub fn gauge(value: f64) -> Self {
        Self::Gauge { value }
    }
    pub fn timing(duration: Duration) -> Self {
        Self::Timing {
            duration_ms: duration.as_secs_f64() * 1000.0,
        }
    }
}

/// A snapshot of all metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp: i64,
    pub metrics: HashMap<String, MetricValue>,
}

impl MetricSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: utc_timestamp(),
            metrics: HashMap::new(),
        }
    }

    pub fn with_metric(mut self, name: &str, value: MetricValue) -> Self {
        self.metrics.insert(name.to_string(), value);
        self
    }
}

impl Default for MetricSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

fn utc_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Thread-safe metrics collector
#[derive(Default)]
pub struct MetricsCollector {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, f64>>>,
    timings: Arc<RwLock<Vec<(String, Duration)>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self, name: &str, value: u64) {
        *self.counters.write().entry(name.to_string()).or_insert(0) += value;
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        self.gauges.write().insert(name.to_string(), value);
    }

    pub fn record_timing(&self, name: &str, duration: Duration) {
        self.timings.write().push((name.to_string(), duration));
    }

    pub fn snapshot(&self) -> MetricSnapshot {
        let mut snapshot = MetricSnapshot::new();

        for (name, &value) in self.counters.read().iter() {
            snapshot
                .metrics
                .insert(name.clone(), MetricValue::counter(value));
        }

        for (name, &value) in self.gauges.read().iter() {
            snapshot
                .metrics
                .insert(name.clone(), MetricValue::gauge(value));
        }

        for (name, duration) in self.timings.read().iter() {
            snapshot
                .metrics
                .insert(name.clone(), MetricValue::timing(*duration));
        }

        snapshot
    }

    pub fn reset(&self) {
        self.counters.write().clear();
        self.gauges.write().clear();
        self.timings.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_collector_increment() {
        let collector = MetricsCollector::new();
        collector.increment("requests", 1);
        collector.increment("requests", 1);
        let snap = collector.snapshot();
        if let Some(MetricValue::Counter { value }) = snap.metrics.get("requests") {
            assert_eq!(*value, 2);
        } else {
            panic!("Expected counter");
        }
    }

    #[test]
    fn metrics_collector_gauge() {
        let collector = MetricsCollector::new();
        collector.set_gauge("memory_mb", 42.5);
        let snap = collector.snapshot();
        if let Some(MetricValue::Gauge { value }) = snap.metrics.get("memory_mb") {
            assert_eq!(*value, 42.5);
        }
    }
}
