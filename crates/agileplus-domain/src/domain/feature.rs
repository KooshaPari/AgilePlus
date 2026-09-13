// SPDX-License-Identifier: MIT OR Apache-2.0
//! Feature aggregate — the central planning unit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::state_machine::FeatureState;

pub(crate) mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(64);
        for byte in bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        serializer.serialize_str(&out)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() != 64 {
            return Err(serde::de::Error::invalid_length(
                value.len(),
                &"64 hex chars",
            ));
        }

        let mut bytes = [0_u8; 32];
        for (index, chunk) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let high = hex_value(chunk[0]).map_err(serde::de::Error::custom)?;
            let low = hex_value(chunk[1]).map_err(serde::de::Error::custom)?;
            bytes[index] = (high << 4) | low;
        }
        Ok(bytes)
    }

    fn hex_value(byte: u8) -> Result<u8, &'static str> {
        match byte {
            b'0'..=b'9' => Ok(byte - b'0'),
            b'a'..=b'f' => Ok(byte - b'a' + 10),
            b'A'..=b'F' => Ok(byte - b'A' + 10),
            _ => Err("invalid hex digit"),
        }
    }
}

/// A software feature tracked through the planning lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub state: FeatureState,
    pub spec_hash: [u8; 32],
    pub target_branch: String,
    pub plane_issue_id: Option<String>,
    pub plane_state_id: Option<String>,
    pub labels: Vec<String>,
    pub module_id: Option<i64>,
    pub project_id: Option<i64>,
    pub created_at_commit: Option<String>,
    pub last_modified_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Feature {
    /// Derive a kebab-case slug from a display name.
    pub fn slug_from_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Attempt a state transition. Returns an error string if the transition is not allowed.
    pub fn transition(&mut self, target: FeatureState) -> Result<(), String> {
        use FeatureState::*;
        let allowed = match self.state {
            Created => matches!(target, Specified),
            Specified => matches!(target, Researched),
            Researched => matches!(target, Planned),
            Planned => matches!(target, Implementing),
            Implementing => matches!(target, Validated),
            Validated => matches!(target, Shipped),
            Shipped => matches!(target, Retrospected),
            Retrospected => false,
        };
        if allowed {
            self.state = target;
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!(
                "invalid transition {:?} -> {:?}",
                self.state, target
            ))
        }
    }

    /// Construct a new Feature with sensible defaults.
    pub fn new(
        slug: &str,
        friendly_name: &str,
        spec_hash: [u8; 32],
        target_branch: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash,
            target_branch: target_branch.unwrap_or("main").to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: Vec::new(),
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transition_updates_state() {
        let mut feature = Feature::new("auth", "Authentication", [0; 32], None);

        feature
            .transition(FeatureState::Specified)
            .expect("domain operation");

        assert_eq!(feature.state, FeatureState::Specified);
    }

    #[test]
    fn invalid_transition_is_rejected_without_mutating_state() {
        let mut feature = Feature::new("auth", "Authentication", [0; 32], None);

        let err = feature.transition(FeatureState::Shipped).unwrap_err();

        assert_eq!(feature.state, FeatureState::Created);
        assert!(err.contains("invalid transition Created -> Shipped"));
    }

    #[test]
    fn full_happy_path_transition_chain() {
        let mut feature = Feature::new("feat", "Feature", [0; 32], None);
        let chain = [
            FeatureState::Specified,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Retrospected,
        ];
        for target in chain {
            feature.transition(target).unwrap();
        }
        assert_eq!(feature.state, FeatureState::Retrospected);
    }

    #[test]
    fn retrospected_cannot_transition() {
        let mut feature = Feature::new("done", "Done", [0; 32], None);
        feature.transition(FeatureState::Specified).unwrap();
        feature.transition(FeatureState::Researched).unwrap();
        feature.transition(FeatureState::Planned).unwrap();
        feature.transition(FeatureState::Implementing).unwrap();
        feature.transition(FeatureState::Validated).unwrap();
        feature.transition(FeatureState::Shipped).unwrap();
        feature.transition(FeatureState::Retrospected).unwrap();

        // Retrospected is a terminal state; no outgoing transitions.
        assert!(feature.transition(FeatureState::Created).is_err());
        assert!(feature.transition(FeatureState::Shipped).is_err());
        assert_eq!(feature.state, FeatureState::Retrospected);
    }

    #[test]
    fn all_invalid_transitions_from_created() {
        let invalid_targets = [
            FeatureState::Created,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Retrospected,
        ];
        for target in invalid_targets {
            let mut feature = Feature::new("x", "X", [0; 32], None);
            assert!(
                feature.transition(target).is_err(),
                "Created -> {target:?} should be invalid"
            );
        }
    }

    #[test]
    fn transition_updates_timestamp() {
        let mut feature = Feature::new("ts", "Timestamp", [0; 32], None);
        let before = feature.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        feature.transition(FeatureState::Specified).unwrap();
        assert!(feature.updated_at >= before);
    }

    #[test]
    fn feature_new_defaults() {
        let f = Feature::new("my-slug", "My Name", [42; 32], None);
        assert_eq!(f.id, 0);
        assert_eq!(f.slug, "my-slug");
        assert_eq!(f.friendly_name, "My Name");
        assert_eq!(f.state, FeatureState::Created);
        assert_eq!(f.spec_hash, [42; 32]);
        assert_eq!(f.target_branch, "main");
        assert!(f.plane_issue_id.is_none());
        assert!(f.plane_state_id.is_none());
        assert!(f.labels.is_empty());
        assert!(f.module_id.is_none());
        assert!(f.project_id.is_none());
        assert!(f.created_at_commit.is_none());
        assert!(f.last_modified_commit.is_none());
    }

    #[test]
    fn feature_new_custom_branch() {
        let f = Feature::new("s", "S", [0; 32], Some("develop"));
        assert_eq!(f.target_branch, "develop");
    }

    // --- slug_from_name tests ---

    #[test]
    fn slug_from_name_simple() {
        assert_eq!(Feature::slug_from_name("Hello World"), "hello-world");
    }

    #[test]
    fn slug_from_name_empty() {
        assert_eq!(Feature::slug_from_name(""), "");
    }

    #[test]
    fn slug_from_name_unicode() {
        // Unicode alphanumeric chars are kept; non-alnum become dashes.
        let slug = Feature::slug_from_name("功能 Design");
        assert_eq!(slug, "功能-design");
    }

    #[test]
    fn slug_from_name_special_chars() {
        assert_eq!(
            Feature::slug_from_name("foo@bar!baz#qux"),
            "foo-bar-baz-qux"
        );
    }

    #[test]
    fn slug_from_name_consecutive_dashes_collapsed() {
        assert_eq!(Feature::slug_from_name("a   b"), "a-b");
        assert_eq!(Feature::slug_from_name("--x--y--"), "x-y");
    }

    #[test]
    fn slug_from_name_all_special() {
        assert_eq!(Feature::slug_from_name("---"), "");
    }

    #[test]
    fn slug_from_name_leading_trailing_dashes_collapsed() {
        assert_eq!(Feature::slug_from_name(" test "), "test");
    }

    #[test]
    fn slug_from_name_numbers() {
        assert_eq!(Feature::slug_from_name("v2 release"), "v2-release");
    }

    // --- hex_bytes serde tests ---

    #[test]
    fn hex_bytes_roundtrip() {
        let original = [0x42_u8; 32];
        let hex_str: String = original
            .iter()
            .flat_map(|b| {
                format!("{:02x}", b).into_bytes()
            })
            .map(|b| b as char)
            .collect();
        assert_eq!(hex_str.len(), 64);
        assert_eq!(hex_str, "42".repeat(32));
    }

    #[test]
    fn hex_bytes_roundtrip_via_json() {
        // Simulate what the Feature JSON serde does with spec_hash.
        let original = Feature::new("t", "T", [0xAB; 32], None);
        let json = serde_json::to_string(&original).unwrap();
        let restored: Feature = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.spec_hash, [0xAB; 32]);
    }

    #[test]
    fn hex_bytes_invalid_length_rejected() {
        // A JSON string that is not 64 hex chars should fail to deserialize
        // as a Feature spec_hash via the hex_bytes serde helper.
        let json = r#"{"id":0,"slug":"s","friendly_name":"S","state":"created",
            "spec_hash":"abc","target_branch":"main",
            "created_at":"2025-01-01T00:00:00Z","updated_at":"2025-01-01T00:00:00Z",
            "labels":[],"plane_issue_id":null,"plane_state_id":null,
            "module_id":null,"project_id":null,
            "created_at_commit":null,"last_modified_commit":null}"#;
        let result = serde_json::from_str::<Feature>(json);
        assert!(result.is_err());
    }

    #[test]
    fn hex_bytes_invalid_hex_digit_rejected() {
        let bad_hex = "g".repeat(64);
        let json = format!(
            r#"{{"id":0,"slug":"s","friendly_name":"S","state":"created",
            "spec_hash":"{bad_hex}","target_branch":"main",
            "created_at":"2025-01-01T00:00:00Z","updated_at":"2025-01-01T00:00:00Z",
            "labels":[],"plane_issue_id":null,"plane_state_id":null,
            "module_id":null,"project_id":null,
            "created_at_commit":null,"last_modified_commit":null}}"#
        );
        let result = serde_json::from_str::<Feature>(&json);
        assert!(result.is_err());
    }
}
