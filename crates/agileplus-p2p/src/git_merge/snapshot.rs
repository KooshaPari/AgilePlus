use std::path::Path;

use agileplus_domain::domain::snapshot::Snapshot;
use tracing::{info, warn};

use super::parser::parse_conflict_blocks;
use super::types::MergeError;

/// Resolve a conflicted snapshot JSON file.
///
/// Parses both sides and keeps the snapshot with the higher `event_sequence`.
pub(crate) fn resolve_snapshot_conflict(path: &Path) -> Result<bool, MergeError> {
    let content = std::fs::read_to_string(path)?;

    if !content.contains("<<<<<<<") {
        return Ok(false);
    }

    let blocks = parse_conflict_blocks(&content);
    let mut winner: Option<Snapshot> = None;

    for block in &blocks {
        for side in [&block.ours, &block.theirs] {
            let text = side.trim();
            if text.is_empty() {
                continue;
            }
            match serde_json::from_str::<Snapshot>(text) {
                Ok(snap) => {
                    let replace = match &winner {
                        None => true,
                        Some(w) => snap.event_sequence > w.event_sequence,
                    };
                    if replace {
                        winner = Some(snap);
                    }
                }
                Err(e) => {
                    warn!("Skipping unparsable snapshot side in {}: {e}", path.display());
                }
            }
        }
    }

    let resolved = match winner {
        Some(snap) => {
            let json = serde_json::to_string_pretty(&snap).map_err(|e| MergeError::Parse {
                file: path.display().to_string(),
                source: e,
            })?;
            std::fs::write(path, json.as_bytes())?;
            true
        }
        None => {
            warn!("No parseable snapshot sides found in {}", path.display());
            false
        }
    };

    if resolved {
        info!("Resolved snapshot conflict in {}", path.display());
    }
    Ok(resolved)
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    fn conflict(ours: &str, theirs: &str) -> String {
        format!("<<<<<<< HEAD\n{ours}\n=======\n{theirs}\n>>>>>>> x\n")
    }

    #[test]
    fn clean_file_returns_false() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.json");
        std::fs::write(&path, "{}").unwrap();
        assert!(!resolve_snapshot_conflict(&path).unwrap());
    }

    #[test]
    fn missing_file_errors() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(resolve_snapshot_conflict(&tmp.path().join("nope.json")).is_err());
    }

    #[test]
    fn higher_sequence_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.json");
        let low = serde_json::to_string_pretty(&Snapshot::new("F", 1, serde_json::json!({"v": 1}), 1)).unwrap();
        let high = serde_json::to_string_pretty(&Snapshot::new("F", 1, serde_json::json!({"v": 9}), 9)).unwrap();
        std::fs::write(&path, conflict(&low, &high)).unwrap();
        assert!(resolve_snapshot_conflict(&path).unwrap());
        let back: Snapshot = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.event_sequence, 9);
    }

    #[test]
    fn ours_wins_when_higher() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.json");
        let high = serde_json::to_string_pretty(&Snapshot::new("F", 1, serde_json::json!({}), 7)).unwrap();
        let low = serde_json::to_string_pretty(&Snapshot::new("F", 1, serde_json::json!({}), 2)).unwrap();
        std::fs::write(&path, conflict(&high, &low)).unwrap();
        resolve_snapshot_conflict(&path).unwrap();
        let back: Snapshot = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.event_sequence, 7);
    }

    #[test]
    fn both_unparsable_returns_false() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.json");
        std::fs::write(&path, conflict("{bad", "also bad")).unwrap();
        assert!(!resolve_snapshot_conflict(&path).unwrap());
    }

    #[test]
    fn one_side_empty_uses_other() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("1.json");
        let good = serde_json::to_string_pretty(&Snapshot::new("F", 1, serde_json::json!({}), 4)).unwrap();
        std::fs::write(&path, conflict("", &good)).unwrap();
        assert!(resolve_snapshot_conflict(&path).unwrap());
        let back: Snapshot = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(back.event_sequence, 4);
    }
}
