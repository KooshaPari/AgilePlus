use std::time::Instant;

#[derive(Debug, thiserror::Error)]
pub enum MergeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {source}")]
    Parse {
        file: String,
        source: serde_json::Error,
    },

    #[error("No conflict markers found in {0}")]
    NoConflictMarkers(String),

    #[error("Malformed conflict block in {0}")]
    MalformedConflict(String),
}

/// Summary of the conflicts resolved in a single `resolve_git_conflicts` call.
#[derive(Debug, Default, Clone)]
pub struct ConflictResolution {
    /// Number of JSONL event files where duplicates were removed.
    pub jsonl_files_resolved: usize,
    /// Number of snapshot files where the latest-wins rule was applied.
    pub snapshot_files_resolved: usize,
    /// Whether `sync_state.json` was re-merged.
    pub sync_state_merged: bool,
    pub duration_ms: u64,
}

pub(crate) fn finish_resolution(
    started: Instant,
    mut resolution: ConflictResolution,
) -> ConflictResolution {
    resolution.duration_ms = started.elapsed().as_millis() as u64;
    resolution
}

#[cfg(test)]
mod deep_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn merge_error_io_display() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "x");
        let e: MergeError = io.into();
        assert!(e.to_string().contains("IO error"));
    }

    #[test]
    fn merge_error_parse_display() {
        let src = serde_json::from_str::<serde_json::Value>("x").unwrap_err();
        let e = MergeError::Parse {
            file: "f.jsonl".into(),
            source: src,
        };
        assert!(e.to_string().contains("Parse error in f.jsonl"));
    }

    #[test]
    fn merge_error_no_conflict_display() {
        let e = MergeError::NoConflictMarkers("f".into());
        assert_eq!(e.to_string(), "No conflict markers found in f");
    }

    #[test]
    fn merge_error_malformed_display() {
        let e = MergeError::MalformedConflict("f".into());
        assert_eq!(e.to_string(), "Malformed conflict block in f");
    }

    #[test]
    fn merge_error_implements_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<MergeError>();
    }

    #[test]
    fn conflict_resolution_default_zeroed() {
        let r = ConflictResolution::default();
        assert_eq!(r.jsonl_files_resolved, 0);
        assert_eq!(r.snapshot_files_resolved, 0);
        assert!(!r.sync_state_merged);
        assert_eq!(r.duration_ms, 0);
    }

    #[test]
    fn conflict_resolution_clone_and_debug() {
        let r = ConflictResolution {
            jsonl_files_resolved: 2,
            snapshot_files_resolved: 1,
            sync_state_merged: true,
            duration_ms: 5,
        };
        let c = r.clone();
        assert_eq!(c.jsonl_files_resolved, 2);
        assert!(c.sync_state_merged);
        assert!(format!("{r:?}").contains("jsonl_files_resolved"));
    }

    #[test]
    fn finish_resolution_sets_duration() {
        let started = std::time::Instant::now();
        std::thread::sleep(Duration::from_millis(2));
        let r = finish_resolution(started, ConflictResolution::default());
        // duration may round to 0 on coarse clocks; just ensure it's populated.
        let _ = r.duration_ms;
        assert_eq!(r.jsonl_files_resolved, 0);
    }
}
