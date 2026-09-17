#[derive(Debug, Default, Clone)]
pub struct ExportStats {
    pub events_exported: usize,
    pub snapshots_exported: usize,
    pub sync_mappings_exported: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct EntityRef {
    pub entity_type: String,
    pub entity_id: i64,
}

#[cfg(test)]
mod deep_tests {
    use super::*;

    #[test]
    fn export_stats_default_zeroed() {
        let s = ExportStats::default();
        assert_eq!(s.events_exported, 0);
        assert_eq!(s.snapshots_exported, 0);
        assert_eq!(s.sync_mappings_exported, 0);
        assert_eq!(s.duration_ms, 0);
    }

    #[test]
    fn export_stats_clone_and_debug() {
        let s = ExportStats {
            events_exported: 3,
            snapshots_exported: 2,
            sync_mappings_exported: 1,
            duration_ms: 9,
        };
        let c = s.clone();
        assert_eq!(c.events_exported, 3);
        assert!(format!("{s:?}").contains("events_exported"));
    }

    #[test]
    fn entity_ref_fields() {
        let e = EntityRef {
            entity_type: "Feature".into(),
            entity_id: 42,
        };
        assert_eq!(e.entity_type, "Feature");
        assert_eq!(e.entity_id, 42);
    }

    #[test]
    fn entity_ref_clone_and_debug() {
        let e = EntityRef {
            entity_type: "Epic".into(),
            entity_id: 1,
        };
        let c = e.clone();
        assert_eq!(c.entity_type, "Epic");
        assert!(format!("{e:?}").contains("Epic"));
    }
}
