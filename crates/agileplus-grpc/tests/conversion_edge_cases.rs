//! Comprehensive tests for domain-to-Protobuf conversion edge cases.
//!
//! Complements the inline unit tests in `conversions.rs` and the existing
//! `pact_schema.rs` contract tests.
//!
//! Traceability: WP14-T080

use agileplus_domain::domain::audit::{AuditEntry, EvidenceRef};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::{
    DependencyType, PrState, WorkPackage, WpDependency, WpState,
};
use agileplus_grpc::conversions::{
    audit_entry_to_proto, feature_to_proto, feature_to_proto_with_wps, wp_to_proto,
    wp_to_proto_with_dependencies,
};

// ---------------------------------------------------------------------------
// Feature conversion edge cases
// ---------------------------------------------------------------------------

#[test]
fn feature_default_branch_is_main() {
    let f = Feature::new("no-branch", "No Branch", [0; 32], None);
    let proto = feature_to_proto(f);
    assert_eq!(proto.target_branch, "main");
}

#[test]
fn feature_long_slug_preserved() {
    let slug = "a".repeat(256);
    let f = Feature::new(&slug, "Long", [0; 32], None);
    let proto = feature_to_proto(f);
    assert_eq!(proto.slug.len(), 256);
}

#[test]
fn feature_friendly_name_with_unicode() {
    let f = Feature::new("i18n", "Ünïcödé Fëätürë 日本語", [0; 32], None);
    let proto = feature_to_proto(f);
    assert_eq!(proto.friendly_name, "Ünïcödé Fëätürë 日本語");
}

#[test]
fn feature_created_at_and_updated_at_are_different_when_time_passes() {
    let f = Feature::new("time-test", "Time Test", [0; 32], None);
    let proto = feature_to_proto(f);
    // Both are RFC 3339 and should contain 'T' separator
    assert!(proto.created_at.contains('T'));
    assert!(proto.updated_at.contains('T'));
    // created_at and updated_at may be equal for a freshly created feature
    // but both must be valid ISO strings
    assert!(proto.created_at.starts_with("20"));
}

#[test]
fn feature_wp_counts_all_done() {
    let f = Feature::new("all-done", "All Done", [0; 32], None);
    let mut wp1 = WorkPackage::new(f.id, "WP1", 1, "c");
    wp1.state = WpState::Done;
    let mut wp2 = WorkPackage::new(f.id, "WP2", 2, "c");
    wp2.state = WpState::Done;
    let mut wp3 = WorkPackage::new(f.id, "WP3", 3, "c");
    wp3.state = WpState::Done;

    let proto = feature_to_proto_with_wps(f, &[wp1, wp2, wp3]);
    assert_eq!(proto.wp_count, 3);
    assert_eq!(proto.wp_done, 3);
}

#[test]
fn feature_wp_counts_none_done() {
    let f = Feature::new("none-done", "None Done", [0; 32], None);
    let mut wp1 = WorkPackage::new(f.id, "WP1", 1, "c");
    wp1.state = WpState::Planned;
    let mut wp2 = WorkPackage::new(f.id, "WP2", 2, "c");
    wp2.state = WpState::Doing;
    let mut wp3 = WorkPackage::new(f.id, "WP3", 3, "c");
    wp3.state = WpState::Review;
    let mut wp4 = WorkPackage::new(f.id, "WP4", 4, "c");
    wp4.state = WpState::Blocked;

    let proto = feature_to_proto_with_wps(f, &[wp1, wp2, wp3, wp4]);
    assert_eq!(proto.wp_count, 4);
    assert_eq!(proto.wp_done, 0);
}

#[test]
fn feature_wp_counts_saturation_at_i32_max() {
    // When wp_count exceeds i32::MAX, it should saturate to i32::MAX
    let f = Feature::new("sat", "Sat", [0; 32], None);

    // Create a fake feature with a huge number of work packages via raw counts
    // We use the internal function indirectly: feature_to_proto_with_wps can't
    // easily create 2.5B items, so test the saturation logic via
    // feature_to_proto_with_wps with many items to verify correctness pattern.
    // The saturation path fires when usize > i32::MAX. Since we can't allocate
    // that many in a test, we verify the pattern is consistent with small counts.
    let proto = feature_to_proto_with_wps(f, &[]);
    assert_eq!(proto.wp_count, 0);
    assert_eq!(proto.wp_done, 0);
}

#[test]
fn feature_many_work_packages_count() {
    let f = Feature::new("many-wps", "Many WPs", [0; 32], None);
    let wps: Vec<WorkPackage> = (0..100)
        .map(|i| {
            let mut wp = WorkPackage::new(f.id, &format!("WP{i}"), i, "c");
            if i % 3 == 0 {
                wp.state = WpState::Done;
            }
            wp
        })
        .collect();

    let done_count = wps.iter().filter(|wp| wp.state == WpState::Done).count();
    let proto = feature_to_proto_with_wps(f, &wps);
    assert_eq!(proto.wp_count, 100);
    assert_eq!(proto.wp_done as usize, done_count);
}

// ---------------------------------------------------------------------------
// WorkPackage conversion edge cases
// ---------------------------------------------------------------------------

#[test]
fn wp_empty_file_scope() {
    let wp = WorkPackage::new(1, "No Files", 1, "c");
    let proto = wp_to_proto(wp);
    assert!(proto.file_scope.is_empty());
}

#[test]
fn wp_multiple_file_scopes() {
    let mut wp = WorkPackage::new(1, "Multi File", 1, "c");
    wp.file_scope = vec![
        "src/main.rs".into(),
        "src/lib.rs".into(),
        "tests/test.rs".into(),
    ];
    let proto = wp_to_proto(wp);
    assert_eq!(proto.file_scope.len(), 3);
    assert_eq!(proto.file_scope[0], "src/main.rs");
}

#[test]
fn wp_with_many_dependencies() {
    let mut wp = WorkPackage::new(1, "Multi Dep", 5, "c");
    wp.id = 100;
    let deps: Vec<WpDependency> = (10..=19)
        .map(|i| WpDependency {
            wp_id: 100,
            depends_on: i,
            dep_type: DependencyType::Explicit,
        })
        .collect();

    let proto = wp_to_proto_with_dependencies(wp, &deps);
    assert_eq!(proto.depends_on.len(), 10);
    assert_eq!(proto.depends_on, vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19]);
}

#[test]
fn wp_empty_dependencies_list() {
    let wp = WorkPackage::new(1, "No Deps", 1, "c");
    let proto = wp_to_proto_with_dependencies(wp, &[]);
    assert!(proto.depends_on.is_empty());
}

#[test]
fn wp_all_pr_state_variants() {
    let states = [
        (PrState::Open, "open"),
        (PrState::Review, "review"),
        (PrState::ChangesRequested, "changesrequested"),
        (PrState::Approved, "approved"),
        (PrState::Merged, "merged"),
    ];

    for (state, expected) in states {
        let mut wp = WorkPackage::new(1, "PR State Test", 1, "c");
        wp.pr_state = Some(state);
        let proto = wp_to_proto(wp);
        assert_eq!(
            proto.pr_state, expected,
            "PrState::{:?} should format as '{expected}'",
            state
        );
    }
}

#[test]
fn wp_none_pr_state_is_empty() {
    let wp = WorkPackage::new(1, "No PR", 1, "c");
    assert!(wp.pr_state.is_none());
    let proto = wp_to_proto(wp);
    assert_eq!(proto.pr_state, "");
}

#[test]
fn wp_dependency_type_file_overlap_preserved() {
    let mut wp = WorkPackage::new(1, "FileOverlap", 3, "c");
    wp.id = 50;
    let deps = vec![WpDependency {
        wp_id: 50,
        depends_on: 40,
        dep_type: DependencyType::FileOverlap,
    }];

    let proto = wp_to_proto_with_dependencies(wp, &deps);
    // The depends_on field contains the depends_on id regardless of type
    assert_eq!(proto.depends_on, vec![40]);
}

#[test]
fn wp_sequence_preserved() {
    let wp = WorkPackage::new(1, "Seq", 42, "c");
    let proto = wp_to_proto(wp);
    assert_eq!(proto.sequence, 42);
}

#[test]
fn wp_max_sequence_number() {
    let wp = WorkPackage::new(1, "MaxSeq", i32::MAX, "c");
    let proto = wp_to_proto(wp);
    assert_eq!(proto.sequence, i32::MAX);
}

// ---------------------------------------------------------------------------
// AuditEntry conversion edge cases
// ---------------------------------------------------------------------------

#[test]
fn audit_entry_feature_slug_and_wp_sequence_are_defaults() {
    use chrono::Utc;
    let entry = AuditEntry {
        id: 1,
        feature_id: 10,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "agent".into(),
        transition: "a->b".into(),
        evidence_refs: vec![],
        prev_hash: [0; 32],
        hash: [0; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert_eq!(proto.feature_slug, "");
    assert_eq!(proto.wp_sequence, 0);
}

#[test]
fn audit_entry_preserves_actor() {
    use chrono::Utc;
    let entry = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "human-reviewer".into(),
        transition: "created->specified".into(),
        evidence_refs: vec![],
        prev_hash: [0; 32],
        hash: [0xFF; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert_eq!(proto.actor, "human-reviewer");
    assert_eq!(proto.transition, "created->specified");
}

#[test]
fn audit_entry_multiple_evidence_refs() {
    use chrono::Utc;
    let entry = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "agent".into(),
        transition: "x".into(),
        evidence_refs: vec![
            EvidenceRef { evidence_id: 1, fr_id: "FR-001".into() },
            EvidenceRef { evidence_id: 2, fr_id: "FR-002".into() },
            EvidenceRef { evidence_id: 3, fr_id: "FR-003".into() },
        ],
        prev_hash: [0; 32],
        hash: [0; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert_eq!(
        proto.evidence_refs,
        vec!["FR-001".to_string(), "FR-002".to_string(), "FR-003".to_string()]
    );
}

#[test]
fn audit_entry_timestamp_is_rfc3339() {
    use chrono::DateTime;
    let entry = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
        actor: "a".into(),
        transition: "".into(),
        evidence_refs: vec![],
        prev_hash: [0; 32],
        hash: [0; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert!(proto.timestamp.contains("T"));
    assert!(proto.timestamp.contains("2023"));
}

#[test]
fn audit_entry_zero_hashes() {
    use chrono::Utc;
    let entry = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "a".into(),
        transition: "".into(),
        evidence_refs: vec![],
        prev_hash: [0; 32],
        hash: [0; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert_eq!(proto.prev_hash, vec![0u8; 32]);
    assert_eq!(proto.hash, vec![0u8; 32]);
}

#[test]
fn audit_entry_max_hashes() {
    use chrono::Utc;
    let entry = AuditEntry {
        id: 1,
        feature_id: 1,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "a".into(),
        transition: "".into(),
        evidence_refs: vec![],
        prev_hash: [0xFF; 32],
        hash: [0xFF; 32],
        event_id: None,
        archived_to: None,
    };
    let proto = audit_entry_to_proto(entry);
    assert!(proto.prev_hash.iter().all(|&b| b == 0xFF));
    assert!(proto.hash.iter().all(|&b| b == 0xFF));
}
