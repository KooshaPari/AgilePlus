//! API key domain type — authentication for dashboard and API.
//!
//! Traceability: FR-028, FR-029 / WP01-T006

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

impl ApiKey {
    pub fn new(
        key_hash: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            key_hash: key_hash.into(),
            name: name.into(),
            created_at: Utc::now(),
            last_used_at: None,
            permissions: Vec::new(),
            is_active: true,
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn touch(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    pub fn revoke(&mut self) {
        self.is_active = false;
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.key_hash.is_empty() {
            return Err(ValidationError::empty_field("key_hash"));
        }
        if self.name.is_empty() {
            return Err(ValidationError::empty_field("name"));
        }
        if self.permissions.is_empty() {
            return Err(ValidationError::empty_field("permissions"));
        }
        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.is_active
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ApiKey({}, name={}, active={})",
            self.id, self.name, self.is_active
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_api_key() {
        let k = ApiKey::new("sha256_hash_value".to_string(), "default".to_string());
        assert_eq!(k.name, "default");
        assert!(k.is_active);
        assert!(k.last_used_at.is_none());
    }

    #[test]
    fn api_key_with_permissions() {
        let k = ApiKey::new("hash".to_string(), "admin".to_string())
            .with_permissions(vec!["read".to_string(), "write".to_string()]);
        assert_eq!(k.permissions.len(), 2);
    }

    #[test]
    fn api_key_lifecycle() {
        let mut k = ApiKey::new("hash".to_string(), "cli".to_string());
        assert!(k.is_valid());

        k.touch();
        assert!(k.last_used_at.is_some());

        k.revoke();
        assert!(!k.is_valid());
    }

    #[test]
    fn api_key_validation_empty_name() {
        let k = ApiKey::new("hash".to_string(), "".to_string());
        assert!(k.validate().is_err());
    }

    #[test]
    fn api_key_validation_empty_permissions() {
        let k = ApiKey::new("hash".to_string(), "test".to_string());
        assert!(k.validate().is_err());
    }

    #[test]
    fn api_key_serde_roundtrip() {
        let k = ApiKey::new("key_hash".to_string(), "test".to_string())
            .with_permissions(vec!["read".to_string()]);
        let json = serde_json::to_string(&k).unwrap();
        let k2: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(k2.name, "test");
        assert!(k2.is_active);
    }

    #[test]
    fn api_key_display() {
        let k = ApiKey::new("hash".to_string(), "my-key".to_string());
        assert!(k.to_string().contains("my-key"));
    }
}
