//! Performance measurement library for AgilePlus
//!
//! Provides utilities for measuring:
//! - Operation latency
//! - Throughput (requests/second)
//! - Memory usage
//! - Benchmarking

pub mod benchmark;
pub mod gauge;
pub mod metrics;

pub use benchmark::{BenchmarkResult, benchmark};
pub use gauge::{Gauge, ScopedGauge};
pub use metrics::{MetricSnapshot, MetricValue, MetricsCollector};
