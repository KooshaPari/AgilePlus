//! API key domain type — authentication for dashboard and API.
//!
//! Traceability: FR-028, FR-029 / WP01-T006

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::feature::hex_bytes;

/// An API key for authenticating requests to the AgilePlus API and dashboard.
///
/// The plaintext key is never stored — only its SHA-256 hash.
/// The plaintext is shown to the user once on generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    #[serde(with = "hex_bytes")]
    pub key_hash: [u8; 32],
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

impl ApiKey {
    pub fn new(key_hash: [u8; 32], name: impl Into<String>) -> Self {
        Self {
            id: 0,
            key_hash,
            name: name.into(),
            created_at: Utc::now(),
            last_used_at: None,
            revoked: false,
        }
    }

    /// Check if this key is valid (not revoked).
    pub fn is_valid(&self) -> bool {
        !self.revoked
    }

    /// Mark this key as used (update last_used_at).
    pub fn touch(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    /// Revoke this key (soft-delete).
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ApiKey({}, name={}, revoked={})",
            self.id, self.name, self.revoked
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_api_key() {
        let k = ApiKey::new([0xab; 32], "default");
        assert_eq!(k.name, "default");
        assert!(k.is_valid());
        assert!(k.last_used_at.is_none());
    }

    #[test]
    fn api_key_lifecycle() {
        let mut k = ApiKey::new([0xff; 32], "cli");
        assert!(k.is_valid());

        k.touch();
        assert!(k.last_used_at.is_some());

        k.revoke();
        assert!(!k.is_valid());
    }

    #[test]
    fn api_key_serde_roundtrip() {
        let k = ApiKey::new([0xcd; 32], "test");
        let json = serde_json::to_string(&k).unwrap();
        let k2: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(k2.key_hash, [0xcd; 32]);
        assert_eq!(k2.name, "test");
    }

    #[test]
    fn api_key_display() {
        let k = ApiKey::new([0; 32], "my-key");
        assert!(k.to_string().contains("my-key"));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn key_json(hash: &str) -> String {
        format!(
            r#"{{"id":1,"key_hash":"{hash}","name":"k","created_at":"2025-01-01T00:00:00Z","last_used_at":null,"revoked":false}}"#
        )
    }

    #[test]
    fn new_defaults() {
        let k = ApiKey::new([0x01; 32], "n");
        assert_eq!(k.id, 0);
        assert_eq!(k.name, "n");
        assert_eq!(k.key_hash, [0x01; 32]);
        assert!(k.last_used_at.is_none());
        assert!(!k.revoked);
        assert!(k.is_valid());
    }

    #[test]
    fn touch_sets_last_used_and_keeps_valid() {
        let mut k = ApiKey::new([0; 32], "n");
        let before = Utc::now();
        k.touch();
        assert!(k.last_used_at.unwrap() >= before);
        assert!(k.is_valid());
    }

    #[test]
    fn touch_twice_advances_or_equals() {
        let mut k = ApiKey::new([0; 32], "n");
        k.touch();
        let first = k.last_used_at.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        k.touch();
        assert!(k.last_used_at.unwrap() >= first);
    }

    #[test]
    fn revoke_is_idempotent_and_persistent() {
        let mut k = ApiKey::new([0; 32], "n");
        k.revoke();
        assert!(!k.is_valid());
        k.revoke();
        assert!(!k.is_valid());
        assert!(k.revoked);
    }

    #[test]
    fn display_contains_fields() {
        let mut k = ApiKey::new([0; 32], "my-key");
        k.id = 5;
        assert_eq!(k.to_string(), "ApiKey(5, name=my-key, revoked=false)");
        k.revoke();
        assert!(k.to_string().contains("revoked=true"));
    }

    #[test]
    fn serde_wire_format_is_hex_string() {
        let k = ApiKey::new([0xAB; 32], "n");
        let v = serde_json::to_value(&k).unwrap();
        let hash = v["key_hash"].as_str().unwrap();
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, "ab".repeat(32));
    }

    #[test]
    fn serde_accepts_uppercase_hex() {
        let json = key_json(&"AB".repeat(32));
        let k: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(k.key_hash, [0xAB; 32]);
    }

    #[test]
    fn serde_rejects_wrong_length_hex() {
        assert!(serde_json::from_str::<ApiKey>(&key_json("abc")).is_err());
        assert!(serde_json::from_str::<ApiKey>(&key_json(&"a".repeat(63))).is_err());
        assert!(serde_json::from_str::<ApiKey>(&key_json(&"a".repeat(65))).is_err());
    }

    #[test]
    fn serde_rejects_non_hex_digits() {
        assert!(serde_json::from_str::<ApiKey>(&key_json(&"z".repeat(64))).is_err());
        assert!(serde_json::from_str::<ApiKey>(&key_json(&"!".repeat(64))).is_err());
    }

    #[test]
    fn serde_roundtrip_all_byte_values() {
        for byte in [0x00u8, 0x0f, 0x10, 0xff] {
            let mut k = ApiKey::new([byte; 32], "n");
            k.touch();
            let back: ApiKey =
                serde_json::from_str(&serde_json::to_string(&k).unwrap()).unwrap();
            assert_eq!(back.key_hash, [byte; 32]);
            assert!(back.last_used_at.is_some());
        }
    }

    #[test]
    fn revoked_flag_survives_serde() {
        let mut k = ApiKey::new([1; 32], "n");
        k.revoke();
        let back: ApiKey = serde_json::from_str(&serde_json::to_string(&k).unwrap()).unwrap();
        assert!(back.revoked);
        assert!(!back.is_valid());
    }

    #[test]
    fn clone_and_debug() {
        let k = ApiKey::new([2; 32], "n");
        let c = k.clone();
        assert_eq!(c.name, k.name);
        assert!(format!("{k:?}").contains("ApiKey"));
    }
}
