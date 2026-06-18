//! Metrics collection for AgilePlus
//!
//! Provides a trait-based metrics abstraction with counter, gauge, and histogram support.

pub mod error;

pub use error::{MetricsError, Result};

/// Counter metric type - monotonically increasing values.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Counter {
    value: f64,
}

impl Counter {
    /// Create a new counter with initial value of 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter by a value.
    pub fn inc(&mut self, value: f64) {
        self.value += value;
    }

    /// Get the current value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Reset the counter to zero.
    pub fn reset(&mut self) {
        self.value = 0.0;
    }
}

/// Gauge metric type - can go up or down.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Gauge {
    value: f64,
}

impl Gauge {
    /// Create a new gauge with initial value of 0.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gauge to a specific value.
    pub fn set(&mut self, value: f64) {
        self.value = value;
    }

    /// Increment the gauge by a value.
    pub fn inc(&mut self, value: f64) {
        self.value += value;
    }

    /// Decrement the gauge by a value.
    pub fn dec(&mut self, value: f64) {
        self.value -= value;
    }

    /// Get the current value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Histogram metric type - tracks distributions.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Histogram {
    values: Vec<f64>,
    count: u64,
    sum: f64,
}

impl Histogram {
    /// Create a new empty histogram.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a value in the histogram.
    pub fn record(&mut self, value: f64) {
        self.values.push(value);
        self.sum += value;
        self.count += 1;
    }

    /// Get the number of recorded values.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Get the sum of all recorded values.
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// Get the average of recorded values.
    pub fn average(&self) -> Option<f64> {
        if self.count == 0 {
            None
        } else {
            Some(self.sum / self.count as f64)
        }
    }
}

/// Metrics trait for collecting application metrics.
pub trait Metrics: Send + Sync {
    /// Increment a counter metric.
    fn increment(&self, name: &str, value: f64) -> Result<()>;

    /// Set a gauge metric to a specific value.
    fn set_gauge(&self, name: &str, value: f64) -> Result<()>;

    /// Record a timing/duration value in a histogram.
    fn record_timing(&self, name: &str, duration_ms: f64) -> Result<()>;

    /// Get the current value of a counter.
    fn get_counter(&self, name: &str) -> Result<f64>;

    /// Get the current value of a gauge.
    fn get_gauge(&self, name: &str) -> Result<f64>;

    /// Get histogram statistics.
    fn get_histogram_stats(&self, name: &str) -> Result<HistogramStats>;
}

/// Histogram statistics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HistogramStats {
    pub count: u64,
    pub sum: f64,
    pub average: Option<f64>,
}

impl HistogramStats {
    /// Create from a histogram.
    pub fn from_histogram(h: &Histogram) -> Self {
        Self {
            count: h.count(),
            sum: h.sum(),
            average: h.average(),
        }
    }
}

/// In-memory metrics implementation.
#[derive(Debug, Default)]
pub struct InMemoryMetrics {
    counters: parking_lot::RwLock<std::collections::HashMap<String, Counter>>,
    gauges: parking_lot::RwLock<std::collections::HashMap<String, Gauge>>,
    histograms: parking_lot::RwLock<std::collections::HashMap<String, Histogram>>,
}

impl InMemoryMetrics {
    /// Create a new in-memory metrics collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all metrics.
    pub fn clear(&self) {
        self.counters.write().clear();
        self.gauges.write().clear();
        self.histograms.write().clear();
    }
}

impl Metrics for InMemoryMetrics {
    fn increment(&self, name: &str, value: f64) -> Result<()> {
        let mut counters = self.counters.write();
        counters.entry(name.to_string()).or_default().inc(value);
        Ok(())
    }

    fn set_gauge(&self, name: &str, value: f64) -> Result<()> {
        let mut gauges = self.gauges.write();
        gauges.entry(name.to_string()).or_default().set(value);
        Ok(())
    }

    fn record_timing(&self, name: &str, duration_ms: f64) -> Result<()> {
        let mut histograms = self.histograms.write();
        histograms
            .entry(name.to_string())
            .or_default()
            .record(duration_ms);
        Ok(())
    }

    fn get_counter(&self, name: &str) -> Result<f64> {
        let counters = self.counters.read();
        counters
            .get(name)
            .map(|c| c.value())
            .ok_or_else(|| MetricsError::NotFound(name.to_string()))
    }

    fn get_gauge(&self, name: &str) -> Result<f64> {
        let gauges = self.gauges.read();
        gauges
            .get(name)
            .map(|g| g.value())
            .ok_or_else(|| MetricsError::NotFound(name.to_string()))
    }

    fn get_histogram_stats(&self, name: &str) -> Result<HistogramStats> {
        let histograms = self.histograms.read();
        histograms
            .get(name)
            .map(HistogramStats::from_histogram)
            .ok_or_else(|| MetricsError::NotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_increment() {
        let mut counter = Counter::new();
        counter.inc(5.0);
        counter.inc(3.0);
        assert_eq!(counter.value(), 8.0);
    }

    #[test]
    fn test_gauge_operations() {
        let mut gauge = Gauge::new();
        gauge.set(10.0);
        assert_eq!(gauge.value(), 10.0);
        gauge.inc(5.0);
        assert_eq!(gauge.value(), 15.0);
        gauge.dec(3.0);
        assert_eq!(gauge.value(), 12.0);
    }

    #[test]
    fn test_histogram_stats() {
        let mut hist = Histogram::new();
        hist.record(10.0);
        hist.record(20.0);
        hist.record(30.0);
        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 60.0);
        assert_eq!(hist.average(), Some(20.0));
    }

    #[test]
    fn test_in_memory_metrics() {
        let metrics = InMemoryMetrics::new();
        metrics.increment("requests", 1.0).unwrap();
        metrics.increment("requests", 2.0).unwrap();
        metrics.set_gauge("connections", 5.0).unwrap();
        metrics.record_timing("latency", 100.0).unwrap();

        assert_eq!(metrics.get_counter("requests").unwrap(), 3.0);
        assert_eq!(metrics.get_gauge("connections").unwrap(), 5.0);
        let stats = metrics.get_histogram_stats("latency").unwrap();
        assert_eq!(stats.count, 1);
        assert_eq!(stats.sum, 100.0);
    }

    #[test]
    fn test_metric_not_found() {
        let metrics = InMemoryMetrics::new();
        assert!(metrics.get_counter("nonexistent").is_err());
        assert!(metrics.get_gauge("nonexistent").is_err());
        assert!(metrics.get_histogram_stats("nonexistent").is_err());
    }
}
