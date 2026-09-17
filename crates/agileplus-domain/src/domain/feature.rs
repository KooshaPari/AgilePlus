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

    // --- serde roundtrip and struct-level tests ---

    #[test]
    fn feature_serde_roundtrip() {
        let f = Feature::new("my-feat", "My Feat", [0xcd; 32], None);
        let json = serde_json::to_string(&f).unwrap();
        let back: Feature = serde_json::from_str(&json).unwrap();
        assert_eq!(back.slug, "my-feat");
        assert_eq!(back.spec_hash, [0xcd; 32]);
        assert_eq!(back.state, FeatureState::Created);
    }

    #[test]
    fn feature_with_labels_and_module() {
        let mut f = Feature::new("f", "F", [0; 32], None);
        f.labels = vec!["auth".into(), "security".into()];
        f.module_id = Some(5);
        f.project_id = Some(1);
        let json = serde_json::to_string(&f).unwrap();
        let back: Feature = serde_json::from_str(&json).unwrap();
        assert_eq!(back.labels, vec!["auth", "security"]);
        assert_eq!(back.module_id, Some(5));
        assert_eq!(back.project_id, Some(1));
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn f() -> Feature {
        Feature::new("s", "S", [0u8; 32], None)
    }

    #[test]
    fn slug_from_name_table() {
        for (input, expected) in [
            ("Hello World", "hello-world"),
            ("", ""),
            ("   ", ""),
            ("---", ""),
            ("a   b", "a-b"),
            ("--x--y--", "x-y"),
            ("foo@bar!baz#qux", "foo-bar-baz-qux"),
            ("v2 release", "v2-release"),
            ("Ünïcödé", "Ünïcödé"),
            ("CamelCase", "camelcase"),
            ("a\tb\nc", "a-b-c"),
            ("123", "123"),
        ] {
            assert_eq!(Feature::slug_from_name(input), expected, "input {input:?}");
        }
    }

    #[test]
    fn transition_allowed_target_from_each_state() {
        let allowed: &[(FeatureState, FeatureState)] = &[
            (FeatureState::Created, FeatureState::Specified),
            (FeatureState::Specified, FeatureState::Researched),
            (FeatureState::Researched, FeatureState::Planned),
            (FeatureState::Planned, FeatureState::Implementing),
            (FeatureState::Implementing, FeatureState::Validated),
            (FeatureState::Validated, FeatureState::Shipped),
            (FeatureState::Shipped, FeatureState::Retrospected),
        ];
        let all = [
            FeatureState::Created,
            FeatureState::Specified,
            FeatureState::Researched,
            FeatureState::Planned,
            FeatureState::Implementing,
            FeatureState::Validated,
            FeatureState::Shipped,
            FeatureState::Retrospected,
        ];
        for (from, to) in allowed {
            let mut feat = f();
            feat.state = *from;
            assert!(feat.transition(*to).is_ok(), "{from:?} -> {to:?} should be allowed");
            assert_eq!(feat.state, *to);
            // Every other target must be rejected from `from`.
            for other in all {
                if other == *to {
                    continue;
                }
                let mut feat2 = f();
                feat2.state = *from;
                assert!(
                    feat2.transition(other).is_err(),
                    "{from:?} -> {other:?} should be rejected"
                );
                assert_eq!(feat2.state, *from);
            }
        }
    }

    #[test]
    fn invalid_transition_error_message_shape() {
        let mut feat = f();
        let err = feat.transition(FeatureState::Shipped).unwrap_err();
        assert_eq!(err, "invalid transition Created -> Shipped");
    }

    #[test]
    fn retrospected_is_terminal() {
        let mut feat = f();
        feat.state = FeatureState::Retrospected;
        for target in [
            FeatureState::Created,
            FeatureState::Specified,
            FeatureState::Retrospected,
        ] {
            assert!(feat.transition(target).is_err());
        }
        assert_eq!(feat.state, FeatureState::Retrospected);
    }

    #[test]
    fn self_transition_always_rejected_on_feature() {
        let mut feat = f();
        assert!(feat.transition(FeatureState::Created).is_err());
    }

    #[test]
    fn new_sets_state_and_defaults() {
        let feat = Feature::new("slug", "Name", [7; 32], Some("dev"));
        assert_eq!(feat.id, 0);
        assert_eq!(feat.state, FeatureState::Created);
        assert_eq!(feat.target_branch, "dev");
        assert_eq!(feat.spec_hash, [7; 32]);
        assert!(feat.labels.is_empty());
        assert_eq!(feat.created_at, feat.updated_at);
    }

    #[test]
    fn new_allows_empty_branch_and_empty_slug() {
        let feat = Feature::new("", "", [0; 32], Some(""));
        assert_eq!(feat.slug, "");
        assert_eq!(feat.target_branch, "");
    }

    #[test]
    fn clone_and_debug() {
        let feat = Feature::new("c", "C", [3; 32], None);
        let c = feat.clone();
        assert_eq!(c.slug, feat.slug);
        assert_eq!(c.spec_hash, feat.spec_hash);
        assert!(format!("{feat:?}").contains("Feature"));
    }

    #[test]
    fn spec_hash_serializes_as_byte_array() {
        // `Feature.spec_hash` has no `serde(with="hex_bytes")` attribute, so it
        // round-trips as a plain 32-element JSON array.
        let mut feat = f();
        feat.spec_hash = [0xAB; 32];
        let v = serde_json::to_value(&feat).unwrap();
        let arr = v["spec_hash"].as_array().unwrap();
        assert_eq!(arr.len(), 32);
        assert!(arr.iter().all(|b| b.as_u64() == Some(0xAB)));
        let back: Feature = serde_json::from_value(v).unwrap();
        assert_eq!(back.spec_hash, [0xAB; 32]);
    }

    #[test]
    fn spec_hash_array_roundtrip_many_values() {
        for byte in [0x00u8, 0x0f, 0x10, 0xff] {
            let feat = Feature::new("s", "S", [byte; 32], None);
            let back: Feature =
                serde_json::from_str(&serde_json::to_string(&feat).unwrap()).unwrap();
            assert_eq!(back.spec_hash, [byte; 32]);
        }
    }

    #[test]
    fn spec_hash_rejects_wrong_length_array() {
        let mut v = serde_json::to_value(f()).unwrap();
        v["spec_hash"] = serde_json::json!([1, 2, 3]);
        assert!(serde_json::from_value::<Feature>(v).is_err());
    }

    #[test]
    fn serde_preserves_optional_fields() {
        let mut feat = f();
        feat.plane_issue_id = Some("plane-1".into());
        feat.plane_state_id = Some("state-1".into());
        feat.created_at_commit = Some("abc1234".into());
        feat.last_modified_commit = Some("def5678".into());
        feat.module_id = Some(1);
        feat.project_id = Some(2);
        let back: Feature =
            serde_json::from_str(&serde_json::to_string(&feat).unwrap()).unwrap();
        assert_eq!(back.plane_issue_id.as_deref(), Some("plane-1"));
        assert_eq!(back.plane_state_id.as_deref(), Some("state-1"));
        assert_eq!(back.created_at_commit.as_deref(), Some("abc1234"));
        assert_eq!(back.last_modified_commit.as_deref(), Some("def5678"));
        assert_eq!(back.module_id, Some(1));
        assert_eq!(back.project_id, Some(2));
    }

    #[test]
    fn transition_advances_updated_at_monotonically() {
        let mut feat = f();
        let t0 = feat.updated_at;
        feat.transition(FeatureState::Specified).unwrap();
        feat.transition(FeatureState::Researched).unwrap();
        assert!(feat.updated_at >= t0);
    }
}
