//! Service types for the intent registry

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a registered service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetadata {
    /// Human-readable name
    pub name: String,

    /// Version of the service
    pub version: String,

    /// Optional description
    #[serde(default)]
    pub description: Option<String>,

    /// Health check endpoint
    #[serde(default)]
    pub health_endpoint: Option<String>,

    /// Additional key-value metadata
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl ServiceMetadata {
    /// Create new metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            health_endpoint: None,
            labels: HashMap::new(),
        }
    }

    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a health endpoint
    pub fn with_health_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.health_endpoint = Some(endpoint.into());
        self
    }

    /// Add a label
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// A registered service with its metadata and intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Unique identifier
    pub id: String,

    /// Service metadata
    pub metadata: ServiceMetadata,

    /// Base URL or connection string
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Whether the service is currently available
    #[serde(default)]
    pub available: bool,
}

impl RegisteredService {
    /// Create a new registered service
    pub fn new(id: impl Into<String>, metadata: ServiceMetadata) -> Self {
        Self {
            id: id.into(),
            metadata,
            endpoint: None,
            available: true,
        }
    }

    /// Set the endpoint
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Check if service is available
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Mark service as unavailable
    pub fn mark_unavailable(&mut self) {
        self.available = false;
    }

    /// Mark service as available
    pub fn mark_available(&mut self) {
        self.available = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_metadata() {
        let meta = ServiceMetadata::new("git-service", "1.0.0")
            .with_description("Git operations service")
            .with_label("env", "production");

        assert_eq!(meta.name, "git-service");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.labels.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_registered_service() {
        let meta = ServiceMetadata::new("storage", "2.0.0");
        let mut service =
            RegisteredService::new("storage-service", meta).with_endpoint("http://localhost:8080");

        assert_eq!(service.id, "storage-service");
        assert!(service.is_available());
        service.mark_unavailable();
        assert!(!service.is_available());
    }
}
