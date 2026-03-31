//! SyncReport — audit summary for a single sync run.
//!
//! Traceability: FR-SYNC-REPORT / WP09-T057

use std::fmt;
use std::time::Duration;

use crate::conflict::SyncConflict;
use crate::error::SyncError;

/// Summary of a completed sync operation, suitable for CLI output and audit logging.
#[derive(Debug, Default)]
pub struct SyncReport {
    /// Entities created during this sync run: `(entity_type, entity_id)`.
    pub created: Vec<(String, i64)>,
    /// Entities updated during this sync run.
    pub updated: Vec<(String, i64)>,
    /// Entities skipped (no changes detected).
    pub skipped: Vec<(String, i64)>,
    /// Conflicts detected but not yet resolved.
    pub conflicts: Vec<SyncConflict>,
    /// Errors encountered during sync.
    pub errors: Vec<SyncError>,
    /// Wall-clock duration of the sync run.
    pub duration: Duration,
}

impl SyncReport {
    /// Create an empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of entities processed (created + updated + skipped + conflicts + errors).
    pub fn total_processed(&self) -> usize {
        self.created.len()
            + self.updated.len()
            + self.skipped.len()
            + self.conflicts.len()
            + self.errors.len()
    }

    /// Returns `true` when no conflicts or errors were recorded.
    pub fn is_clean(&self) -> bool {
        self.conflicts.is_empty() && self.errors.is_empty()
    }
}

impl fmt::Display for SyncReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sep = "─".repeat(52);
        writeln!(f, "┌{sep}┐")?;
        writeln!(f, "│  AgilePlus Sync Report{:>29}│", "")?;
        writeln!(f, "├{sep}┤")?;
        writeln!(f, "│  {:<20} {:>28}│", "Created", self.created.len())?;
        writeln!(f, "│  {:<20} {:>28}│", "Updated", self.updated.len())?;
        writeln!(f, "│  {:<20} {:>28}│", "Skipped", self.skipped.len())?;
        writeln!(f, "│  {:<20} {:>28}│", "Conflicts", self.conflicts.len())?;
        writeln!(f, "│  {:<20} {:>28}│", "Errors", self.errors.len())?;
        writeln!(
            f,
            "│  {:<20} {:>26.3}s│",
            "Duration",
            self.duration.as_secs_f64()
        )?;
        writeln!(f, "└{sep}┘")?;

        if !self.conflicts.is_empty() {
            writeln!(f, "\nConflicts:")?;
            for c in &self.conflicts {
                writeln!(
                    f,
                    "  • {}/{} — local={} remote={}",
                    c.entity_type,
                    c.entity_id,
                    &c.local_hash[..8],
                    &c.remote_hash[..8]
                )?;
            }
        }

        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for e in &self.errors {
                writeln!(f, "  • {e}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn empty_report_is_clean() {
        let r = SyncReport::new();
        assert!(r.is_clean());
        assert_eq!(r.total_processed(), 0);
    }

    #[test]
    fn report_totals() {
        let mut r = SyncReport::new();
        r.created.push(("feature".into(), 1));
        r.updated.push(("wp".into(), 2));
        r.skipped.push(("feature".into(), 3));
        r.duration = Duration::from_millis(1234);
        assert_eq!(r.total_processed(), 3);
        assert!(r.is_clean());
    }

    #[test]
    fn report_display_contains_created() {
        let mut r = SyncReport::new();
        r.created.push(("feature".into(), 99));
        r.duration = Duration::from_secs(1);
        let s = r.to_string();
        assert!(s.contains("Created"));
        assert!(s.contains('1'));
    }

    #[test]
    fn is_clean_false_when_conflicts_present() {
        let mut r = SyncReport::new();
        let conflict = crate::conflict::SyncConflict::new(
            "feature", 1,
            serde_json::json!({}),
            serde_json::json!({}),
        );
        r.conflicts.push(conflict);
        assert!(!r.is_clean());
    }

    #[test]
    fn is_clean_false_when_errors_present() {
        let mut r = SyncReport::new();
        r.errors.push(crate::error::SyncError::EntityNotFound {
            entity_type: "test".into(),
            entity_id: 1,
        });
        assert!(!r.is_clean());
    }

    #[test]
    fn total_processed_counts_all_fields() {
        let mut r = SyncReport::new();
        r.created.push(("a".into(), 1));
        r.updated.push(("b".into(), 2));
        r.updated.push(("c".into(), 3));
        r.skipped.push(("d".into(), 4));
        let conflict = crate::conflict::SyncConflict::new("e", 5, serde_json::json!({}), serde_json::json!({}));
        r.conflicts.push(conflict);
        r.errors.push(crate::error::SyncError::EntityNotFound {
            entity_type: "f".into(),
            entity_id: 6,
        });
        assert_eq!(r.total_processed(), 6);
    }

    #[test]
    fn report_display_shows_conflicts() {
        let mut r = SyncReport::new();
        let conflict = crate::conflict::SyncConflict::new(
            "work_package", 42,
            serde_json::json!({"title": "A"}),
            serde_json::json!({"title": "B"}),
        );
        r.conflicts.push(conflict);
        r.duration = Duration::from_millis(500);
        let s = r.to_string();
        assert!(s.contains("Conflicts"));
        assert!(s.contains("work_package"));
        assert!(s.contains("42"));
    }

    #[test]
    fn report_display_shows_errors() {
        let mut r = SyncReport::new();
        r.errors.push(crate::error::SyncError::EntityNotFound {
            entity_type: "feature".into(),
            entity_id: 7,
        });
        r.duration = Duration::ZERO;
        let s = r.to_string();
        assert!(s.contains("Errors"));
    }

    #[test]
    fn report_display_shows_updated_count() {
        let mut r = SyncReport::new();
        r.updated.push(("wp".into(), 1));
        r.updated.push(("wp".into(), 2));
        r.duration = Duration::from_secs(2);
        let s = r.to_string();
        assert!(s.contains("Updated"));
    }

    #[test]
    fn report_display_shows_skipped_count() {
        let mut r = SyncReport::new();
        r.skipped.push(("epic".into(), 10));
        r.duration = Duration::from_secs(1);
        let s = r.to_string();
        assert!(s.contains("Skipped"));
    }

    #[test]
    fn report_display_shows_zero_duration() {
        let r = SyncReport::new();
        let s = r.to_string();
        assert!(s.contains("Duration"));
    }

    #[test]
    fn empty_report_display_renders() {
        let r = SyncReport::new();
        let s = r.to_string();
        assert!(s.contains("AgilePlus Sync Report"));
        assert!(s.contains("Created"));
        assert!(s.contains("0"));
    }
}
