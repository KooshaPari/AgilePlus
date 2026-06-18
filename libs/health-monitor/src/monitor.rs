//! Main health monitor implementation

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::check::{HealthCheckResult, HealthStatus};
use crate::strategy::{HealthAction, HealthStrategy, ThresholdStrategy};

/// Health monitor for tracking service health
pub struct HealthMonitor {
    /// Current health status for each target
    statuses: Arc<RwLock<HashMap<String, HealthStatus>>>,
    /// Failure counts for each target
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// Health strategy to use
    strategy: Arc<dyn HealthStrategy>,
    /// Event channel for health events
    event_tx: broadcast::Sender<HealthEvent>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(strategy: Option<Arc<dyn HealthStrategy>>) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            statuses: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            strategy: strategy.unwrap_or_else(|| Arc::new(ThresholdStrategy::default())),
            event_tx,
        }
    }

    /// Get current status for a target
    pub fn get_status(&self, target: &str) -> Option<HealthStatus> {
        let statuses = self.statuses.read();
        statuses.get(target).copied()
    }

    /// Get all current statuses
    pub fn get_all_statuses(&self) -> HashMap<String, HealthStatus> {
        let statuses = self.statuses.read();
        statuses.clone()
    }

    /// Record a health check result
    pub fn record(&self, result: HealthCheckResult) {
        let action = self.strategy.evaluate(&result);

        // Update status
        {
            let mut statuses = self.statuses.write();
            statuses.insert(result.target.clone(), result.status);
        }

        // Update failure count
        {
            let mut failures = self.failure_counts.write();
            if result.status == HealthStatus::Unhealthy {
                let count = failures.entry(result.target.clone()).or_insert(0);
                *count += 1;
            } else {
                failures.insert(result.target.clone(), 0);
            }
        }

        // Send event
        let event = HealthEvent {
            target: result.target.clone(),
            status: result.status,
            action,
            message: result.message.clone(),
        };
        let _ = self.event_tx.send(event);
    }

    /// Subscribe to health events
    pub fn subscribe(&self) -> broadcast::Receiver<HealthEvent> {
        self.event_tx.subscribe()
    }
}

/// Health event for broadcasting
#[derive(Debug, Clone)]
pub struct HealthEvent {
    pub target: String,
    pub status: HealthStatus,
    pub action: HealthAction,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_healthy() {
        let monitor = HealthMonitor::new(None);
        let result = HealthCheckResult::healthy("test-service");
        monitor.record(result);
        assert_eq!(
            monitor.get_status("test-service"),
            Some(HealthStatus::Healthy)
        );
    }

    #[test]
    fn test_record_unhealthy() {
        let monitor = HealthMonitor::new(None);
        let result = HealthCheckResult::unhealthy("test-service", "failed");
        monitor.record(result);
        assert_eq!(
            monitor.get_status("test-service"),
            Some(HealthStatus::Unhealthy)
        );
    }

    #[test]
    fn test_get_all_statuses() {
        let monitor = HealthMonitor::new(None);
        monitor.record(HealthCheckResult::healthy("svc1"));
        monitor.record(HealthCheckResult::healthy("svc2"));
        let all = monitor.get_all_statuses();
        assert_eq!(all.len(), 2);
    }
}
