//! Service health domain types — platform service monitoring.
//!
//! Traceability: FR-016 / WP01-T004

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health status of a platform service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Health information for a single platform service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub connection_info: String,
    pub metadata: serde_json::Value,
}

impl ServiceHealth {
    pub fn new(service_name: impl Into<String>, connection_info: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            status: HealthStatus::Unavailable,
            last_check: Utc::now(),
            uptime_seconds: 0,
            connection_info: connection_info.into(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn mark_healthy(&mut self, uptime_seconds: u64) {
        self.status = HealthStatus::Healthy;
        self.uptime_seconds = uptime_seconds;
        self.last_check = Utc::now();
    }

    pub fn mark_degraded(&mut self, reason: &str) {
        self.status = HealthStatus::Degraded;
        self.last_check = Utc::now();
        self.metadata["degraded_reason"] = serde_json::Value::String(reason.to_string());
    }

    pub fn mark_unavailable(&mut self) {
        self.status = HealthStatus::Unavailable;
        self.last_check = Utc::now();
    }
}

/// Aggregated platform health across all services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub services: Vec<ServiceHealth>,
    pub overall: HealthStatus,
    pub checked_at: DateTime<Utc>,
}

impl PlatformStatus {
    pub fn from_services(services: Vec<ServiceHealth>) -> Self {
        let overall = if services.iter().all(|s| s.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if services
            .iter()
            .any(|s| s.status == HealthStatus::Unavailable)
        {
            HealthStatus::Unavailable
        } else {
            HealthStatus::Degraded
        };
        Self {
            services,
            overall,
            checked_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unavailable.to_string(), "unavailable");
    }

    #[test]
    fn service_health_transitions() {
        let mut h = ServiceHealth::new("nats", "localhost:4222");
        assert_eq!(h.status, HealthStatus::Unavailable);

        h.mark_healthy(120);
        assert_eq!(h.status, HealthStatus::Healthy);
        assert_eq!(h.uptime_seconds, 120);

        h.mark_degraded("high latency");
        assert_eq!(h.status, HealthStatus::Degraded);
    }

    #[test]
    fn platform_status_aggregation() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let mut dragonfly = ServiceHealth::new("dragonfly", "localhost:6379");
        dragonfly.mark_healthy(100);

        let status = PlatformStatus::from_services(vec![nats, dragonfly]);
        assert_eq!(status.overall, HealthStatus::Healthy);
    }

    #[test]
    fn platform_status_degraded() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let mut dragonfly = ServiceHealth::new("dragonfly", "localhost:6379");
        dragonfly.mark_degraded("slow");

        let status = PlatformStatus::from_services(vec![nats, dragonfly]);
        assert_eq!(status.overall, HealthStatus::Degraded);
    }

    #[test]
    fn platform_status_unavailable() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let neo4j = ServiceHealth::new("neo4j", "localhost:7687");

        let status = PlatformStatus::from_services(vec![nats, neo4j]);
        assert_eq!(status.overall, HealthStatus::Unavailable);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn new_service_defaults_to_unavailable() {
        let h = ServiceHealth::new("nats", "localhost:4222");
        assert_eq!(h.service_name, "nats");
        assert_eq!(h.connection_info, "localhost:4222");
        assert_eq!(h.status, HealthStatus::Unavailable);
        assert_eq!(h.uptime_seconds, 0);
        assert!(h.metadata.is_object());
        assert_eq!(h.metadata.as_object().unwrap().len(), 0);
    }

    #[test]
    fn mark_healthy_sets_uptime_and_clears_nothing() {
        let mut h = ServiceHealth::new("svc", "addr");
        h.mark_degraded("slow");
        h.mark_healthy(55);
        assert_eq!(h.status, HealthStatus::Healthy);
        assert_eq!(h.uptime_seconds, 55);
        // degraded_reason is left in metadata (documents current behavior)
        assert_eq!(h.metadata["degraded_reason"], "slow");
    }

    #[test]
    fn mark_degraded_records_reason() {
        let mut h = ServiceHealth::new("svc", "addr");
        h.mark_degraded("latency spike");
        assert_eq!(h.status, HealthStatus::Degraded);
        assert_eq!(h.metadata["degraded_reason"], "latency spike");
    }

    #[test]
    fn mark_unavailable_resets_status() {
        let mut h = ServiceHealth::new("svc", "addr");
        h.mark_healthy(10);
        h.mark_unavailable();
        assert_eq!(h.status, HealthStatus::Unavailable);
        // uptime is not reset by mark_unavailable
        assert_eq!(h.uptime_seconds, 10);
    }

    #[test]
    fn platform_status_empty_is_healthy_vacuously() {
        let s = PlatformStatus::from_services(vec![]);
        assert_eq!(s.overall, HealthStatus::Healthy);
        assert!(s.services.is_empty());
    }

    #[test]
    fn platform_status_all_healthy() {
        let mut a = ServiceHealth::new("a", "x");
        a.mark_healthy(1);
        let mut b = ServiceHealth::new("b", "y");
        b.mark_healthy(2);
        assert_eq!(
            PlatformStatus::from_services(vec![a, b]).overall,
            HealthStatus::Healthy
        );
    }

    #[test]
    fn platform_status_any_unavailable_wins_over_degraded() {
        let mut a = ServiceHealth::new("a", "x");
        a.mark_degraded("slow");
        let b = ServiceHealth::new("b", "y"); // Unavailable
        assert_eq!(
            PlatformStatus::from_services(vec![a, b]).overall,
            HealthStatus::Unavailable
        );
    }

    #[test]
    fn platform_status_single_degraded() {
        let mut a = ServiceHealth::new("a", "x");
        a.mark_degraded("slow");
        assert_eq!(
            PlatformStatus::from_services(vec![a]).overall,
            HealthStatus::Degraded
        );
    }

    #[test]
    fn platform_status_single_unavailable() {
        let a = ServiceHealth::new("a", "x");
        assert_eq!(
            PlatformStatus::from_services(vec![a]).overall,
            HealthStatus::Unavailable
        );
    }

    #[test]
    fn health_status_display_and_hash() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unavailable.to_string(), "unavailable");
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(HealthStatus::Healthy);
        set.insert(HealthStatus::Healthy);
        set.insert(HealthStatus::Degraded);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn health_status_serde_roundtrip() {
        for s in [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unavailable,
        ] {
            let json = serde_json::to_string(&s).unwrap();
            let back: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn service_health_serde_roundtrip() {
        let mut h = ServiceHealth::new("svc", "addr");
        h.mark_healthy(9);
        let back: ServiceHealth =
            serde_json::from_str(&serde_json::to_string(&h).unwrap()).unwrap();
        assert_eq!(back.service_name, "svc");
        assert_eq!(back.status, HealthStatus::Healthy);
        assert_eq!(back.uptime_seconds, 9);
    }

    #[test]
    fn platform_status_serde_roundtrip() {
        let mut a = ServiceHealth::new("a", "x");
        a.mark_healthy(3);
        let ps = PlatformStatus::from_services(vec![a]);
        let back: PlatformStatus =
            serde_json::from_str(&serde_json::to_string(&ps).unwrap()).unwrap();
        assert_eq!(back.overall, HealthStatus::Healthy);
        assert_eq!(back.services.len(), 1);
    }

    #[test]
    fn service_health_clone_and_debug() {
        let h = ServiceHealth::new("n", "c");
        let c = h.clone();
        assert_eq!(c.service_name, h.service_name);
        assert!(format!("{h:?}").contains("ServiceHealth"));
    }
}
