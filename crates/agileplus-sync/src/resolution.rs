//! Conflict resolution strategies.
//!
//! Traceability: FR-SYNC-RESOLUTION / WP09-T055

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::conflict::{SyncConflict, hash_value};
use crate::error::SyncError;

/// Specifies which side wins for a particular field in field-level resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldSource {
    Local,
    Remote,
}

/// Strategy to apply when resolving a sync conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum ResolutionStrategy {
    /// Accept local version as the resolved value.
    LocalWins,
    /// Accept remote version as the resolved value.
    RemoteWins,
    /// Accept a user-provided merged value.
    Manual(Value),
    /// Pick each field from local or remote independently.
    FieldLevel(HashMap<String, FieldSource>),
}

/// The outcome of applying a resolution strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionResult {
    /// The resolved entity value.
    pub resolved_value: Value,
    /// SHA-256 hash of the resolved value.
    pub resolved_hash: String,
    /// Human-readable label for the strategy that was applied.
    pub strategy_label: String,
}

/// Apply a `ResolutionStrategy` to a `SyncConflict`, returning the resolved value.
///
/// For `FieldLevel`, unknown fields fall back to the remote value.
pub fn apply_resolution(
    conflict: &SyncConflict,
    strategy: &ResolutionStrategy,
) -> Result<ResolutionResult, SyncError> {
    let (resolved_value, strategy_label) = match strategy {
        ResolutionStrategy::LocalWins => (conflict.local_version.clone(), "local_wins".to_string()),
        ResolutionStrategy::RemoteWins => {
            (conflict.remote_version.clone(), "remote_wins".to_string())
        }
        ResolutionStrategy::Manual(v) => (v.clone(), "manual".to_string()),
        ResolutionStrategy::FieldLevel(field_map) => {
            let local_obj = conflict.local_version.as_object().ok_or_else(|| {
                SyncError::ResolutionFailed("local version is not an object".into())
            })?;
            let remote_obj = conflict.remote_version.as_object().ok_or_else(|| {
                SyncError::ResolutionFailed("remote version is not an object".into())
            })?;

            let mut merged = serde_json::Map::new();
            // Collect all field names from both sides.
            let all_keys: std::collections::HashSet<&String> =
                local_obj.keys().chain(remote_obj.keys()).collect();

            for key in all_keys {
                let source = field_map.get(key).unwrap_or(&FieldSource::Remote);
                let value = match source {
                    FieldSource::Local => local_obj
                        .get(key)
                        .or_else(|| remote_obj.get(key))
                        .cloned()
                        .unwrap_or(Value::Null),
                    FieldSource::Remote => remote_obj
                        .get(key)
                        .or_else(|| local_obj.get(key))
                        .cloned()
                        .unwrap_or(Value::Null),
                };
                merged.insert(key.clone(), value);
            }

            (Value::Object(merged), "field_level".to_string())
        }
    };

    let resolved_hash = hash_value(&resolved_value);
    Ok(ResolutionResult {
        resolved_value,
        resolved_hash,
        strategy_label,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflict::SyncConflict;
    use serde_json::json;

    fn make_conflict() -> SyncConflict {
        SyncConflict::new(
            "feature",
            1,
            json!({"title": "local title", "status": "open"}),
            json!({"title": "remote title", "status": "closed"}),
        )
    }

    #[test]
    fn local_wins() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::LocalWins).unwrap();
        assert_eq!(result.resolved_value["title"], "local title");
        assert_eq!(result.strategy_label, "local_wins");
    }

    #[test]
    fn remote_wins() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::RemoteWins).unwrap();
        assert_eq!(result.resolved_value["title"], "remote title");
        assert_eq!(result.strategy_label, "remote_wins");
    }

    #[test]
    fn manual_resolution() {
        let c = make_conflict();
        let merged = json!({"title": "manual title", "status": "open"});
        let result = apply_resolution(&c, &ResolutionStrategy::Manual(merged.clone())).unwrap();
        assert_eq!(result.resolved_value, merged);
        assert_eq!(result.strategy_label, "manual");
    }

    #[test]
    fn field_level_resolution() {
        let c = make_conflict();
        let mut field_map = HashMap::new();
        field_map.insert("title".to_string(), FieldSource::Local);
        field_map.insert("status".to_string(), FieldSource::Remote);
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(field_map)).unwrap();
        assert_eq!(result.resolved_value["title"], "local title");
        assert_eq!(result.resolved_value["status"], "closed");
        assert_eq!(result.strategy_label, "field_level");
    }

    #[test]
    fn resolved_hash_is_hash_of_resolved_value() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::LocalWins).unwrap();
        let expected = hash_value(&result.resolved_value);
        assert_eq!(result.resolved_hash, expected);
    }

    #[test]
    fn field_level_unknown_field_falls_back_to_remote() {
        let c = SyncConflict::new(
            "feature",
            1,
            json!({"title": "local", "extra": "local_extra"}),
            json!({"title": "remote", "extra": "remote_extra", "status": "open"}),
        );
        let mut field_map = HashMap::new();
        field_map.insert("title".to_string(), FieldSource::Local);
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(field_map)).unwrap();
        assert_eq!(result.resolved_value["title"], "local");
        assert_eq!(result.resolved_value["extra"], "remote_extra");
        assert_eq!(result.resolved_value["status"], "open");
    }

    #[test]
    fn field_level_with_empty_map_uses_remote() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(HashMap::new())).unwrap();
        assert_eq!(result.resolved_value["title"], "remote title");
        assert_eq!(result.resolved_value["status"], "closed");
    }

    #[test]
    fn field_level_merges_all_keys_from_both_sides() {
        let c = SyncConflict::new(
            "test",
            1,
            json!({"local_only": 1}),
            json!({"remote_only": 2}),
        );
        let field_map = HashMap::new();
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(field_map)).unwrap();
        assert!(result.resolved_value.get("local_only").is_some());
        assert!(result.resolved_value.get("remote_only").is_some());
    }

    #[test]
    fn manual_resolution_with_null_value() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::Manual(serde_json::Value::Null)).unwrap();
        assert_eq!(result.resolved_value, serde_json::Value::Null);
        assert_eq!(result.strategy_label, "manual");
    }

    #[test]
    fn manual_resolution_with_complex_value() {
        let c = make_conflict();
        let merged = json!({"nested": {"deep": [1, 2, 3]}, "scalar": "ok"});
        let result = apply_resolution(&c, &ResolutionStrategy::Manual(merged.clone())).unwrap();
        assert_eq!(result.resolved_value, merged);
    }

    #[test]
    fn remote_wins_resolution_label() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::RemoteWins).unwrap();
        assert_eq!(result.strategy_label, "remote_wins");
    }

    #[test]
    fn local_wins_resolution_label() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::LocalWins).unwrap();
        assert_eq!(result.strategy_label, "local_wins");
    }

    #[test]
    fn field_level_resolution_label() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(HashMap::new())).unwrap();
        assert_eq!(result.strategy_label, "field_level");
    }

    #[test]
    fn field_level_non_object_local_returns_error() {
        let c = SyncConflict::new("test", 1, serde_json::Value::String("not an object".into()), json!({"a": 1}));
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(HashMap::new()));
        assert!(result.is_err());
    }

    #[test]
    fn field_level_non_object_remote_returns_error() {
        let c = SyncConflict::new("test", 1, json!({"a": 1}), serde_json::Value::Number(42.into()));
        let result = apply_resolution(&c, &ResolutionStrategy::FieldLevel(HashMap::new()));
        assert!(result.is_err());
    }

    #[test]
    fn resolution_result_fields_are_populated() {
        let c = make_conflict();
        let result = apply_resolution(&c, &ResolutionStrategy::LocalWins).unwrap();
        assert!(!result.resolved_hash.is_empty());
        assert!(!result.strategy_label.is_empty());
    }

    proptest::proptest! {
        #[test]
        fn local_wins_always_returns_local_version(local: serde_json::Value, remote: serde_json::Value) {
            let c = SyncConflict::new("test", 1, local.clone(), remote.clone());
            let result = apply_resolution(&c, &ResolutionStrategy::LocalWins).unwrap();
            prop_assert_eq!(result.resolved_value, local);
        }

        #[test]
        fn remote_wins_always_returns_remote_version(local: serde_json::Value, remote: serde_json::Value) {
            let c = SyncConflict::new("test", 1, local.clone(), remote.clone());
            let result = apply_resolution(&c, &ResolutionStrategy::RemoteWins).unwrap();
            prop_assert_eq!(result.resolved_value, remote);
        }

        #[test]
        fn manual_resolution_returns_provided_value(provided: serde_json::Value) {
            let c = make_conflict();
            let result = apply_resolution(&c, &ResolutionStrategy::Manual(provided.clone())).unwrap();
            prop_assert_eq!(result.resolved_value, provided);
        }

        #[test]
        fn resolution_never_panics(local: serde_json::Value, remote: serde_json::Value, field_map: HashMap<String, FieldSource>) {
            let c = SyncConflict::new("test", 1, local, remote.clone());
            let _ = apply_resolution(&c, &ResolutionStrategy::FieldLevel(field_map));
        }
    }
}
