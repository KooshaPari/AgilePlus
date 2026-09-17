use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use agileplus_domain::domain::sync_mapping::SyncMapping;
use tracing::info;

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Merge two `sync_state.json` values.
///
/// Strategy:
/// - `sync_mappings`: union by `(entity_type, entity_id)`, keeping the entry
///   with the highest `conflict_count` (most battle-tested).
/// - `sync_vector.entries`: per-key maximum sequence.
pub(crate) fn merge_sync_state(
    ours: &serde_json::Value,
    theirs: &serde_json::Value,
) -> serde_json::Value {
    let merge_mappings = |a: &serde_json::Value, b: &serde_json::Value| -> serde_json::Value {
        let a_vec: Vec<SyncMapping> = serde_json::from_value(a.clone()).unwrap_or_default();
        let b_vec: Vec<SyncMapping> = serde_json::from_value(b.clone()).unwrap_or_default();

        let mut map: BTreeMap<(String, i64), SyncMapping> = BTreeMap::new();
        for m in a_vec.into_iter().chain(b_vec) {
            let key = (m.entity_type.clone(), m.entity_id);
            let replace = match map.get(&key) {
                None => true,
                Some(existing) => m.conflict_count > existing.conflict_count,
            };
            if replace {
                map.insert(key, m);
            }
        }

        serde_json::to_value(map.into_values().collect::<Vec<_>>())
            .unwrap_or(serde_json::Value::Array(vec![]))
    };

    let merge_vectors = |a: &serde_json::Value, b: &serde_json::Value| -> serde_json::Value {
        let a_entries: HashMap<String, u64> =
            serde_json::from_value(a.get("entries").cloned().unwrap_or_default())
                .unwrap_or_default();
        let b_entries: HashMap<String, u64> =
            serde_json::from_value(b.get("entries").cloned().unwrap_or_default())
                .unwrap_or_default();

        let mut merged: BTreeMap<String, u64> = BTreeMap::new();
        for (k, v) in a_entries.into_iter().chain(b_entries) {
            let entry = merged.entry(k).or_insert(0);
            if v > *entry {
                *entry = v;
            }
        }

        let device_id = a
            .get("device_id")
            .or_else(|| b.get("device_id"))
            .cloned()
            .unwrap_or_default();

        serde_json::json!({
            "device_id": device_id,
            "entries": merged,
        })
    };

    let ours_mappings = ours.get("sync_mappings").cloned().unwrap_or_default();
    let theirs_mappings = theirs.get("sync_mappings").cloned().unwrap_or_default();
    let ours_vector = ours.get("sync_vector").cloned().unwrap_or_default();
    let theirs_vector = theirs.get("sync_vector").cloned().unwrap_or_default();

    serde_json::json!({
        "sync_mappings": merge_mappings(&ours_mappings, &theirs_mappings),
        "sync_vector": merge_vectors(&ours_vector, &theirs_vector),
    })
}

/// Resolve a conflicted `sync_state.json` file.
pub(crate) fn resolve_sync_state_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut merged: Option<serde_json::Value> = None;

    for block in &blocks {
        let parse_side = |text: &str| -> Option<serde_json::Value> {
            let t = text.trim();
            if t.is_empty() {
                None
            } else {
                serde_json::from_str(t).ok()
            }
        };

        match (parse_side(&block.ours), parse_side(&block.theirs)) {
            (Some(ours), Some(theirs)) => {
                let partial = merge_sync_state(&ours, &theirs);
                merged = Some(match merged {
                    None => partial,
                    Some(prev) => merge_sync_state(&prev, &partial),
                });
            }
            (Some(v), None) | (None, Some(v)) => {
                merged = Some(match merged {
                    None => v.clone(),
                    Some(prev) => merge_sync_state(&prev, &v),
                });
            }
            (None, None) => {}
        }
    }

    let result = match merged {
        Some(v) => v,
        None => {
            return Err(MergeError::MalformedConflict(path.display().to_string()));
        }
    };

    let json = serde_json::to_string_pretty(&result).map_err(|e| MergeError::Parse {
        file: path.display().to_string(),
        source: e,
    })?;
    std::fs::write(path, json.as_bytes())?;

    info!("Resolved sync_state.json conflict at {}", path.display());
    Ok(true)
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    fn mapping(entity_id: i64, conflicts: i32) -> serde_json::Value {
        let mut m = SyncMapping::new("Feature", entity_id, format!("p{entity_id}"), "h");
        m.id = entity_id;
        m.conflict_count = conflicts;
        serde_json::to_value(m).unwrap()
    }

    #[test]
    fn merge_unions_disjoint_mappings() {
        let ours = serde_json::json!({
            "sync_mappings": [mapping(1, 0)],
            "sync_vector": {"device_id": "d", "entries": {}}
        });
        let theirs = serde_json::json!({
            "sync_mappings": [mapping(2, 0)],
            "sync_vector": {"device_id": "d", "entries": {}}
        });
        let merged = merge_sync_state(&ours, &theirs);
        let maps: Vec<SyncMapping> =
            serde_json::from_value(merged["sync_mappings"].clone()).unwrap();
        assert_eq!(maps.len(), 2);
    }

    #[test]
    fn merge_keeps_higher_conflict_count() {
        let ours = serde_json::json!({
            "sync_mappings": [mapping(1, 5)],
            "sync_vector": {}
        });
        let theirs = serde_json::json!({
            "sync_mappings": [mapping(1, 1)],
            "sync_vector": {}
        });
        let merged = merge_sync_state(&ours, &theirs);
        let maps: Vec<SyncMapping> =
            serde_json::from_value(merged["sync_mappings"].clone()).unwrap();
        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].conflict_count, 5);
    }

    #[test]
    fn merge_handles_missing_sections() {
        let merged = merge_sync_state(&serde_json::json!({}), &serde_json::json!({}));
        assert!(merged.get("sync_mappings").is_some());
        assert!(merged.get("sync_vector").is_some());
    }

    #[test]
    fn merge_vector_takes_max_per_key_across_sides() {
        let ours = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"device_id": "d1", "entries": {"A/1": 3, "A/2": 9}}
        });
        let theirs = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"device_id": "d2", "entries": {"A/1": 7}}
        });
        let merged = merge_sync_state(&ours, &theirs);
        let entries = &merged["sync_vector"]["entries"];
        assert_eq!(entries["A/1"].as_u64(), Some(7));
        assert_eq!(entries["A/2"].as_u64(), Some(9));
    }

    #[test]
    fn merge_vector_prefers_first_non_empty_device_id() {
        let ours = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"device_id": "ours-dev", "entries": {}}
        });
        let theirs = serde_json::json!({
            "sync_mappings": [],
            "sync_vector": {"entries": {}}
        });
        let merged = merge_sync_state(&ours, &theirs);
        assert_eq!(merged["sync_vector"]["device_id"], "ours-dev");
    }

    fn conflict(ours: &serde_json::Value, theirs: &serde_json::Value) -> String {
        format!(
            "<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> x\n",
            serde_json::to_string_pretty(ours).unwrap(),
            serde_json::to_string_pretty(theirs).unwrap()
        )
    }

    #[test]
    fn resolve_clean_file_returns_false() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sync_state.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(!resolve_sync_state_conflict(&path).unwrap());
    }

    #[test]
    fn resolve_missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(resolve_sync_state_conflict(&tmp.path().join("nope.json")).is_err());
    }

    #[test]
    fn resolve_both_sides_unparsable_is_malformed_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sync_state.json");
        // Both sides are genuinely invalid JSON (bare words / unbalanced brace).
        std::fs::write(&path, "<<<<<<< HEAD\n{bad\n=======\nnot json\n>>>>>>> b\n").unwrap();
        let err = resolve_sync_state_conflict(&path).unwrap_err();
        assert!(matches!(err, MergeError::MalformedConflict(_)));
    }

    #[test]
    fn resolve_merges_and_writes_valid_json() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sync_state.json");
        let ours = serde_json::json!({
            "sync_mappings": [mapping(1, 0)],
            "sync_vector": {"device_id": "d", "entries": {"A/1": 2}}
        });
        let theirs = serde_json::json!({
            "sync_mappings": [mapping(1, 3)],
            "sync_vector": {"device_id": "d", "entries": {"A/1": 8}}
        });
        std::fs::write(&path, conflict(&ours, &theirs)).unwrap();
        assert!(resolve_sync_state_conflict(&path).unwrap());
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["sync_vector"]["entries"]["A/1"].as_u64(), Some(8));
        let maps: Vec<SyncMapping> =
            serde_json::from_value(value["sync_mappings"].clone()).unwrap();
        assert_eq!(maps[0].conflict_count, 3);
    }

    #[test]
    fn resolve_multiple_blocks_folds_together() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sync_state.json");
        let a = serde_json::json!({"sync_mappings": [mapping(1, 0)], "sync_vector": {"device_id":"d","entries":{"A/1":1}}});
        let b = serde_json::json!({"sync_mappings": [mapping(2, 0)], "sync_vector": {"device_id":"d","entries":{"A/2":2}}});
        let content = format!("{}{}", conflict(&a, &a), conflict(&b, &b));
        std::fs::write(&path, content).unwrap();
        assert!(resolve_sync_state_conflict(&path).unwrap());
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let maps: Vec<SyncMapping> =
            serde_json::from_value(value["sync_mappings"].clone()).unwrap();
        assert_eq!(maps.len(), 2);
    }
}
