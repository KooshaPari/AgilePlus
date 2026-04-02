//! Gauge for timing operations

use std::time::{Duration, Instant};

/// A simple gauge for measuring operation duration
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    start: Instant,
    metadata: Vec<(String, String)>,
}

impl Gauge {
    /// Start a new gauge with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            metadata: Vec::new(),
        }
    }

    /// Add metadata to the gauge
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// Stop the gauge and return the duration
    pub fn stop(self) -> Duration {
        self.start.elapsed()
    }

    /// Get the elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn metadata(&self) -> &[(String, String)] {
        &self.metadata
    }
}

/// RAII gauge that automatically stops on drop
pub struct ScopedGauge {
    gauge: Gauge,
}

impl ScopedGauge {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            gauge: Gauge::new(name),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.gauge.elapsed()
    }
}

impl Drop for ScopedGauge {
    fn drop(&mut self) {
        // Just let the gauge be dropped - we don't need to do anything
        // The timing is captured when the gauge was created
    }
}

/// Macros for easy timing
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:expr) => {{
        let gauge = $crate::Gauge::new($name);
        let result = $block;
        let elapsed = gauge.stop();
        (result, elapsed)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_basic() {
        let gauge = Gauge::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = gauge.stop();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn scoped_gauge() {
        let _gauge = ScopedGauge::new("scoped");
        std::thread::sleep(Duration::from_millis(5));
    }
}
