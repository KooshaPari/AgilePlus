//! Integration tests for agileplus-import ImportReport.

use agileplus_import::ImportReport;

#[test]
fn import_report_default_all_zero() {
    let report = ImportReport::default();
    assert_eq!(report.projects_created, 0);
    assert_eq!(report.projects_updated, 0);
    assert_eq!(report.modules_created, 0);
    assert_eq!(report.modules_updated, 0);
    assert_eq!(report.features_created, 0);
    assert_eq!(report.features_updated, 0);
    assert_eq!(report.cycles_created, 0);
    assert_eq!(report.cycles_updated, 0);
    assert_eq!(report.work_packages_created, 0);
    assert_eq!(report.work_packages_updated, 0);
    assert_eq!(report.module_links_created, 0);
    assert_eq!(report.cycle_links_created, 0);
    assert_eq!(report.artifacts_written, 0);
    assert_eq!(report.audits_written, 0);
}

#[test]
fn import_report_fields_are_mutable() {
    let report = ImportReport {
        projects_created: 3,
        projects_updated: 1,
        modules_created: 5,
        modules_updated: 2,
        features_created: 10,
        features_updated: 4,
        cycles_created: 2,
        cycles_updated: 1,
        work_packages_created: 20,
        work_packages_updated: 7,
        module_links_created: 8,
        cycle_links_created: 3,
        artifacts_written: 20,
        audits_written: 10,
    };

    assert_eq!(report.projects_created, 3);
    assert_eq!(report.projects_updated, 1);
    assert_eq!(report.modules_created, 5);
    assert_eq!(report.modules_updated, 2);
    assert_eq!(report.features_created, 10);
    assert_eq!(report.features_updated, 4);
    assert_eq!(report.cycles_created, 2);
    assert_eq!(report.cycles_updated, 1);
    assert_eq!(report.work_packages_created, 20);
    assert_eq!(report.work_packages_updated, 7);
    assert_eq!(report.module_links_created, 8);
    assert_eq!(report.cycle_links_created, 3);
    assert_eq!(report.artifacts_written, 20);
    assert_eq!(report.audits_written, 10);
}

#[test]
fn import_report_json_roundtrip() {
    let report = ImportReport {
        projects_created: 1,
        features_created: 5,
        artifacts_written: 10,
        ..Default::default()
    };

    let json = serde_json::to_string(&report).unwrap();
    let restored: ImportReport = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.projects_created, 1);
    assert_eq!(restored.features_created, 5);
    assert_eq!(restored.artifacts_written, 10);
    // Default fields should also roundtrip
    assert_eq!(restored.projects_updated, 0);
    assert_eq!(restored.audits_written, 0);
}

#[test]
fn import_report_clone() {
    let report = ImportReport {
        modules_created: 7,
        ..Default::default()
    };

    let cloned = report.clone();
    assert_eq!(cloned.modules_created, 7);
    assert_eq!(cloned.projects_created, 0);
}

#[test]
fn import_report_debug_format() {
    let report = ImportReport::default();
    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("ImportReport"));
    assert!(debug_str.contains("projects_created"));
}
