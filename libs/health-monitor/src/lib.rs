//! Health monitoring system for AgilePlus services
//!
//! Provides health checking, status tracking, and self-healing capabilities.

pub mod check;
pub mod error;
pub mod monitor;
pub mod strategy;

pub use check::{HealthCheckResult, HealthStatus, RetryConfig};
pub use error::HealthMonitorError;
pub use monitor::{HealthEvent, HealthMonitor};
pub use strategy::{HealthAction, HealthStrategy, RetryStrategy, ThresholdStrategy};
