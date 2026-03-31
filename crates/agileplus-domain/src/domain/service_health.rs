//! Service health domain types — platform service monitoring.
//!
//! Traceability: FR-016 / WP01-T004

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: HealthStatus,
    pub last_checked: DateTime<Utc>,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

impl ServiceHealth {
    pub fn new(
        service_name: impl Into<String>,
        status: HealthStatus,
        last_checked: DateTime<Utc>,
    ) -> Self {
        Self {
            service_name: service_name.into(),
            status,
            last_checked,
            message: None,
            latency_ms: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.service_name.is_empty() {
            return Err(ValidationError::empty_field("service_name"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_service_health() {
        let h = ServiceHealth::new(
            "nats".to_string(),
            HealthStatus::Healthy,
            Utc::now(),
        );
        assert_eq!(h.service_name, "nats");
        assert_eq!(h.status, HealthStatus::Healthy);
        assert!(h.message.is_none());
        assert!(h.latency_ms.is_none());
    }

    #[test]
    fn service_health_with_message_and_latency() {
        let h = ServiceHealth::new(
            "database".to_string(),
            HealthStatus::Degraded,
            Utc::now(),
        )
        .with_message("High connection latency")
        .with_latency_ms(250);
        assert_eq!(h.status, HealthStatus::Degraded);
        assert_eq!(h.message.as_deref(), Some("High connection latency"));
        assert_eq!(h.latency_ms, Some(250));
    }

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
    }

    #[test]
    fn service_health_validation_empty_name() {
        let h = ServiceHealth::new(
            "".to_string(),
            HealthStatus::Healthy,
            Utc::now(),
        );
        assert!(h.validate().is_err());
    }

    #[test]
    fn service_health_serde_roundtrip() {
        let h = ServiceHealth::new(
            "redis".to_string(),
            HealthStatus::Healthy,
            Utc::now(),
        )
        .with_latency_ms(5);
        let json = serde_json::to_string(&h).unwrap();
        let h2: ServiceHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(h2.service_name, "redis");
        assert_eq!(h2.latency_ms, Some(5));
    }
}
