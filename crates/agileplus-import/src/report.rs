use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportReport {
    pub projects_created: usize,
    pub projects_updated: usize,
    pub modules_created: usize,
    pub modules_updated: usize,
    pub features_created: usize,
    pub features_updated: usize,
    pub cycles_created: usize,
    pub cycles_updated: usize,
    pub work_packages_created: usize,
    pub work_packages_updated: usize,
    pub module_links_created: usize,
    pub cycle_links_created: usize,
    pub artifacts_written: usize,
    pub audits_written: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_report_is_zero_and_equal_to_itself() {
        assert_eq!(ImportReport::default(), ImportReport::default());
    }

    #[test]
    fn partial_eq_detects_field_difference() {
        let base = ImportReport::default();
        let other = ImportReport {
            projects_created: 1,
            ..ImportReport::default()
        };

        assert_ne!(base, other);
    }

    #[test]
    fn clone_is_independent_and_equal() {
        let mut original = ImportReport {
            artifacts_written: 4,
            audits_written: 2,
            ..ImportReport::default()
        };

        let cloned = original.clone();
        original.artifacts_written = 99;

        assert_eq!(cloned.artifacts_written, 4);
        assert_eq!(cloned.audits_written, 2);
        assert_ne!(cloned, original);
    }

    #[test]
    fn debug_lists_every_counter_field() {
        let debug = format!("{:?}", ImportReport::default());
        for field in [
            "projects_created",
            "projects_updated",
            "modules_created",
            "modules_updated",
            "features_created",
            "features_updated",
            "cycles_created",
            "cycles_updated",
            "work_packages_created",
            "work_packages_updated",
            "module_links_created",
            "cycle_links_created",
            "artifacts_written",
            "audits_written",
        ] {
            assert!(debug.contains(field), "debug output missing {field}");
        }
    }

    #[test]
    fn deserialize_without_default_requires_all_fields() {
        let result: Result<ImportReport, _> = serde_json::from_str("{}");
        assert!(
            result.is_err(),
            "unspecified counters must not silently default"
        );
    }

    #[test]
    fn yaml_roundtrip_preserves_all_counters() {
        let report = ImportReport {
            projects_created: 1,
            projects_updated: 2,
            modules_created: 3,
            modules_updated: 4,
            features_created: 5,
            features_updated: 6,
            cycles_created: 7,
            cycles_updated: 8,
            work_packages_created: 9,
            work_packages_updated: 10,
            module_links_created: 11,
            cycle_links_created: 12,
            artifacts_written: 13,
            audits_written: 14,
        };

        let yaml = serde_yaml::to_string(&report).unwrap();
        let restored: ImportReport = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(restored, report);
    }

    #[test]
    fn extra_unknown_fields_are_ignored_on_deserialize() {
        let json = r#"{"projects_created":1,"projects_updated":0,"modules_created":0,
            "modules_updated":0,"features_created":0,"features_updated":0,"cycles_created":0,
            "cycles_updated":0,"work_packages_created":0,"work_packages_updated":0,
            "module_links_created":0,"cycle_links_created":0,"artifacts_written":0,
            "audits_written":0,"unknown_future_counter":99}"#;
        let report: ImportReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.projects_created, 1);
    }
}
