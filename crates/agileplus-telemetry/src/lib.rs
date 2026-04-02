//! AgilePlus telemetry — OpenTelemetry traces, metrics, and structured logs.
//!
//! # Quick-start
//!
//! ```no_run
//! use agileplus_telemetry::{init_telemetry, config::TelemetryConfig, trace_layer};
//! use tracing_subscriber::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cfg = TelemetryConfig::load().unwrap_or_default();
//!     let _guard = init_telemetry(cfg.clone()).expect("telemetry init");
//!     tracing_subscriber::registry().with(trace_layer()).init();
//! }
//! ```

pub mod adapter;
pub mod config;
pub mod logs;
pub mod metrics;
pub mod traces;

pub use adapter::{TelemetryAdapter, TelemetryError, TelemetryGuard};
pub use config::TelemetryConfig;
pub use metrics::{AgilePlusMetrics, MetricsRecorder};
pub use traces::{telemetry_layer, trace_layer};