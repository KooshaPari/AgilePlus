//! Device node domain type — multi-device sync identity and vector clocks.
//!
//! Traceability: FR-051 / WP01-T005

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNode {
    pub id: Uuid,
    pub device_name: String,
    pub tailscale_ip: Option<String>,
    pub last_seen: DateTime<Utc>,
    pub sync_vector: HashMap<String, u64>,
    pub agileplus_version: String,
}

impl DeviceNode {
    pub fn new(
        device_name: impl Into<String>,
        agileplus_version: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            device_name: device_name.into(),
            tailscale_ip: None,
            last_seen: Utc::now(),
            sync_vector: HashMap::new(),
            agileplus_version: agileplus_version.into(),
        }
    }

    pub fn with_tailscale_ip(mut self, ip: impl Into<String>) -> Self {
        self.tailscale_ip = Some(ip.into());
        self
    }

    pub fn update_sync_vector(&mut self, entity_type: impl Into<String>, entity_id: impl Into<String>, sequence: u64) {
        let key = format!("{}:{}", entity_type.into(), entity_id.into());
        self.sync_vector.insert(key, sequence);
        self.last_seen = Utc::now();
    }

    pub fn get_sequence(&self, entity_type: &str, entity_id: &str) -> Option<u64> {
        let key = format!("{}:{}", entity_type, entity_id);
        self.sync_vector.get(&key).copied()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.device_name.is_empty() {
            return Err(ValidationError::empty_field("device_name"));
        }
        if self.agileplus_version.is_empty() {
            return Err(ValidationError::empty_field("agileplus_version"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_node() {
        let d = DeviceNode::new("macbook-pro".to_string(), "1.0.0".to_string());
        assert_eq!(d.device_name, "macbook-pro");
        assert!(d.tailscale_ip.is_none());
        assert!(d.sync_vector.is_empty());
    }

    #[test]
    fn device_node_with_tailscale() {
        let d = DeviceNode::new("macbook-pro".to_string(), "1.0.0".to_string())
            .with_tailscale_ip("100.64.0.1".to_string());
        assert_eq!(d.tailscale_ip.as_deref(), Some("100.64.0.1"));
    }

    #[test]
    fn sync_vector_update_and_get() {
        let mut d = DeviceNode::new("desktop".to_string(), "1.0.0".to_string());
        d.update_sync_vector("feature", "entity-1", 42);
        assert_eq!(d.get_sequence("feature", "entity-1"), Some(42));
        assert_eq!(d.get_sequence("feature", "entity-2"), None);
    }

    #[test]
    fn device_node_validation_empty_name() {
        let d = DeviceNode::new("".to_string(), "1.0.0".to_string());
        assert!(d.validate().is_err());
    }

    #[test]
    fn device_node_serde_roundtrip() {
        let mut d = DeviceNode::new("server".to_string(), "2.0.0".to_string())
            .with_tailscale_ip("100.64.0.2".to_string());
        d.update_sync_vector("wp", "entity-5", 100);
        let json = serde_json::to_string(&d).unwrap();
        let d2: DeviceNode = serde_json::from_str(&json).unwrap();
        assert_eq!(d2.device_name, "server");
        assert_eq!(d2.get_sequence("wp", "entity-5"), Some(100));
    }
}
