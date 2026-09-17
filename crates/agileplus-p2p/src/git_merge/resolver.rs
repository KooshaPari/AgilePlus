use std::path::Path;
use std::time::Instant;

use tracing::debug;

use super::jsonl::resolve_jsonl_conflict;
use super::snapshot::resolve_snapshot_conflict;
use super::sync_state::resolve_sync_state_conflict;
use super::types::{ConflictResolution, MergeError, finish_resolution};

pub(crate) fn resolve_git_conflicts(repo_dir: &Path) -> Result<ConflictResolution, MergeError> {
    let started = Instant::now();
    let mut resolution = ConflictResolution::default();

    let sync_dir = repo_dir.join(".agileplus").join("sync");

    if !sync_dir.exists() {
        debug!("No .agileplus/sync directory found — nothing to resolve");
        return Ok(finish_resolution(started, resolution));
    }

    let events_dir = sync_dir.join("events");
    if events_dir.exists() {
        for entry in walkdir(&events_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("jsonl")
                && resolve_jsonl_conflict(&entry)?
            {
                resolution.jsonl_files_resolved += 1;
            }
        }
    }

    let snapshots_dir = sync_dir.join("snapshots");
    if snapshots_dir.exists() {
        for entry in walkdir(&snapshots_dir)? {
            if entry.extension().and_then(|e| e.to_str()) == Some("json")
                && resolve_snapshot_conflict(&entry)?
            {
                resolution.snapshot_files_resolved += 1;
            }
        }
    }

    let sync_state_path = sync_dir.join("sync_state.json");
    if sync_state_path.exists() && resolve_sync_state_conflict(&sync_state_path)? {
        resolution.sync_state_merged = true;
    }

    Ok(finish_resolution(started, resolution))
}

/// Simple recursive file walker (avoids pulling in the `walkdir` crate).
fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>, MergeError> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            result.extend(walkdir(&p)?);
        } else {
            result.push(p);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn walkdir_missing_dir_errors() {
        let err = walkdir(&PathBuf::from("/definitely/not/here")).unwrap_err();
        assert!(matches!(err, MergeError::Io(_)));
    }

    #[test]
    fn walkdir_empty_dir_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(walkdir(tmp.path()).unwrap().is_empty());
    }

    #[test]
    fn walkdir_recurses_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("f.txt"), "x").unwrap();
        std::fs::write(tmp.path().join("top.txt"), "y").unwrap();
        let files = walkdir(tmp.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn resolve_no_sync_dir_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 0);
        assert_eq!(r.snapshot_files_resolved, 0);
        assert!(!r.sync_state_merged);
    }

    #[test]
    fn resolve_sync_dir_exists_but_empty() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".agileplus/sync")).unwrap();
        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 0);
        assert_eq!(r.snapshot_files_resolved, 0);
    }

    #[test]
    fn resolve_ignores_clean_files() {
        let tmp = tempfile::tempdir().unwrap();
        let events = tmp.path().join(".agileplus/sync/events/Feature");
        std::fs::create_dir_all(&events).unwrap();
        std::fs::write(events.join("1.jsonl"), "no conflict here\n").unwrap();
        let snaps = tmp.path().join(".agileplus/sync/snapshots/Feature");
        std::fs::create_dir_all(&snaps).unwrap();
        std::fs::write(snaps.join("1.json"), "{}").unwrap();
        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 0);
        assert_eq!(r.snapshot_files_resolved, 0);
    }

    #[test]
    fn resolve_nested_entity_subdirs() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".agileplus/sync/events/Feature/sub");
        std::fs::create_dir_all(&dir).unwrap();
        let mut e1 = agileplus_domain::domain::event::Event::new(
            "Feature",
            1,
            "created",
            serde_json::json!({}),
            "t",
        );
        e1.sequence = 1;
        e1.hash[0] = 1;
        let mut e2 = e1.clone();
        e2.sequence = 2;
        e2.hash[0] = 2;
        let content = format!(
            "<<<<<<< HEAD\n{}\n{}\n=======\n{}\n>>>>>>> x\n",
            serde_json::to_string(&e1).unwrap(),
            serde_json::to_string(&e2).unwrap(),
            serde_json::to_string(&e1).unwrap(),
        );
        std::fs::write(dir.join("1.jsonl"), content).unwrap();
        let r = resolve_git_conflicts(tmp.path()).unwrap();
        assert_eq!(r.jsonl_files_resolved, 1);
    }
}
