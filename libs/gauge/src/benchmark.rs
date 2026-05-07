//! Benchmarking utilities

use std::time::{Duration, Instant};

use crate::gauge::Gauge;

/// Benchmark result
#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u32,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub ops_per_second: f64,
}

impl BenchmarkResult {
    pub fn new(name: String) -> Self {
        Self {
            name,
            iterations: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            ops_per_second: 0.0,
        }
    }
}

/// Run a benchmark
pub fn benchmark<F>(name: &str, iterations: u32, mut f: F) -> BenchmarkResult
where
    F: FnMut(),
{
    let mut result = BenchmarkResult::new(name.to_string());
    result.iterations = iterations;

    let start = Instant::now();
    for _ in 0..iterations {
        let gauge = Gauge::new("benchmark_iteration");
        f();
        let elapsed = gauge.stop();

        result.min_duration = result.min_duration.min(elapsed);
        result.max_duration = result.max_duration.max(elapsed);
    }
    result.total_duration = start.elapsed();
    result.avg_duration = result.total_duration / iterations;

    if result.total_duration > Duration::ZERO {
        result.ops_per_second = iterations as f64 / result.total_duration.as_secs_f64();
    }

    result
}

/// Async benchmark
pub async fn benchmark_async<F, Fut>(name: &str, iterations: u32, mut f: F) -> BenchmarkResult
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut result = BenchmarkResult::new(name.to_string());
    result.iterations = iterations;

    let start = Instant::now();
    for _ in 0..iterations {
        let before = Instant::now();
        f().await;
        let elapsed = before.elapsed();

        result.min_duration = result.min_duration.min(elapsed);
        result.max_duration = result.max_duration.max(elapsed);
    }
    result.total_duration = start.elapsed();
    result.avg_duration = result.total_duration / iterations;

    if result.total_duration > Duration::ZERO {
        result.ops_per_second = iterations as f64 / result.total_duration.as_secs_f64();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_basic() {
        let result = benchmark("sleep_test", 10, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(result.name, "sleep_test");
        assert_eq!(result.iterations, 10);
        assert!(result.avg_duration >= Duration::from_millis(1));
    }
}
