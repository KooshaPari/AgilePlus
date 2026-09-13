//! Domain ↔ Protobuf conversion functions.
//!
//! Rust's orphan rule prevents implementing foreign traits on foreign types.
//! We use free conversion functions instead of `From`/`Into` impls.
//!
//! Traceability: WP14-T080

use agileplus_domain::domain::audit::AuditEntry as DomainAuditEntry;
use agileplus_domain::domain::feature::Feature as DomainFeature;
use agileplus_domain::domain::work_package::{
    WorkPackage as DomainWorkPackage, WpDependency, WpState,
};
use agileplus_proto::agileplus::v1::{
    AuditEntry as ProtoAuditEntry, Feature as ProtoFeature, WorkPackageStatus as ProtoWpStatus,
};

/// Convert a domain Feature to its Protobuf representation.
///
/// This conversion has no storage context, so it is retained for callers that
/// only need the feature's intrinsic fields. RPC handlers should use
/// [`feature_to_proto_with_wps`] so the count fields reflect persisted state.
pub fn feature_to_proto(f: DomainFeature) -> ProtoFeature {
    feature_to_proto_with_wps(f, &[])
}

/// Convert a domain Feature and its work packages to Protobuf.
pub fn feature_to_proto_with_wps(
    f: DomainFeature,
    work_packages: &[DomainWorkPackage],
) -> ProtoFeature {
    let wp_count = work_packages.len();
    let wp_done = work_packages
        .iter()
        .filter(|wp| wp.state == WpState::Done)
        .count();
    feature_to_proto_with_counts(f, wp_count, wp_done)
}

fn feature_to_proto_with_counts(f: DomainFeature, wp_count: usize, wp_done: usize) -> ProtoFeature {
    ProtoFeature {
        id: f.id,
        slug: f.slug,
        friendly_name: f.friendly_name,
        state: f.state.to_string(),
        target_branch: f.target_branch,
        created_at: f.created_at.to_rfc3339(),
        updated_at: f.updated_at.to_rfc3339(),
        // Protobuf uses int32 for these fields. Saturating conversion keeps a
        // malformed or extremely large store from wrapping into a negative
        // count on the wire.
        wp_count: i32::try_from(wp_count).unwrap_or(i32::MAX),
        wp_done: i32::try_from(wp_done).unwrap_or(i32::MAX),
    }
}

/// Convert a domain WorkPackage to its Protobuf representation.
pub fn wp_to_proto(wp: DomainWorkPackage) -> ProtoWpStatus {
    wp_to_proto_with_dependencies(wp, &[])
}

/// Convert a domain WorkPackage and its persisted dependency edges to Protobuf.
pub fn wp_to_proto_with_dependencies(
    wp: DomainWorkPackage,
    dependencies: &[WpDependency],
) -> ProtoWpStatus {
    ProtoWpStatus {
        id: wp.id,
        title: wp.title,
        state: format!("{:?}", wp.state).to_lowercase(),
        sequence: wp.sequence,
        agent_id: wp.agent_id.unwrap_or_default(),
        pr_url: wp.pr_url.unwrap_or_default(),
        pr_state: wp
            .pr_state
            .map(|ps| format!("{:?}", ps).to_lowercase())
            .unwrap_or_default(),
        depends_on: dependencies
            .iter()
            .map(|dependency| dependency.depends_on)
            .collect(),
        file_scope: wp.file_scope,
    }
}

/// Convert a domain AuditEntry to its Protobuf representation.
///
/// The caller must set `feature_slug` and `wp_sequence` from context.
pub fn audit_entry_to_proto(e: DomainAuditEntry) -> ProtoAuditEntry {
    ProtoAuditEntry {
        id: e.id,
        feature_slug: String::new(), // Caller fills from context
        wp_sequence: 0,              // Caller fills when needed
        timestamp: e.timestamp.to_rfc3339(),
        actor: e.actor,
        transition: e.transition,
        evidence_refs: e.evidence_refs.iter().map(|r| r.fr_id.clone()).collect(),
        prev_hash: e.prev_hash.to_vec(),
        hash: e.hash.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::feature::Feature;
    use agileplus_domain::domain::work_package::{DependencyType, WorkPackage, WpState};

    #[test]
    fn feature_conversion() {
        let f = Feature::new("my-feature", "My Feature", [0u8; 32], Some("main"));
        let proto = feature_to_proto(f);
        assert_eq!(proto.slug, "my-feature");
        assert_eq!(proto.friendly_name, "My Feature");
        assert_eq!(proto.state, "created");
        assert_eq!(proto.target_branch, "main");
    }

    #[test]
    fn feature_conversion_reports_work_package_counts() {
        let feature = Feature::new("counted-feature", "Counted Feature", [0u8; 32], None);
        let mut done = WorkPackage::new(feature.id, "Done WP", 1, "done");
        done.state = WpState::Done;
        let planned = WorkPackage::new(feature.id, "Planned WP", 2, "planned");

        let proto = feature_to_proto_with_wps(feature, &[done, planned]);

        assert_eq!(proto.wp_count, 2);
        assert_eq!(proto.wp_done, 1);
    }

    #[test]
    fn wp_conversion() {
        let wp = WorkPackage::new(1, "Test WP", 1, "criteria");
        let proto = wp_to_proto(wp);
        assert_eq!(proto.title, "Test WP");
        assert_eq!(proto.state, "planned");
        assert_eq!(proto.sequence, 1);
    }

    #[test]
    fn wp_conversion_reports_dependency_ids() {
        let mut wp = WorkPackage::new(1, "Dependent WP", 2, "done");
        wp.id = 20;
        let dependency = WpDependency {
            wp_id: 20,
            depends_on: 10,
            dep_type: DependencyType::Explicit,
        };

        let proto = wp_to_proto_with_dependencies(wp, &[dependency]);

        assert_eq!(proto.depends_on, vec![10]);
    }

    #[test]
    fn wp_conversion_preserves_large_dependency_ids() {
        let mut wp = WorkPackage::new(1, "Dependent WP", 2, "done");
        wp.id = 20;
        let large_id = i64::from(i32::MAX) + 1;
        let dependency = WpDependency {
            wp_id: 20,
            depends_on: large_id,
            dep_type: DependencyType::Explicit,
        };

        let proto = wp_to_proto_with_dependencies(wp, &[dependency]);

        assert_eq!(proto.depends_on, vec![large_id]);
    }

    // --- Additional tests for uncovered paths ---

    #[test]
    fn feature_conversion_preserves_id() {
        let mut f = Feature::new("id-test", "ID Test", [0; 32], None);
        f.id = 99;
        let proto = feature_to_proto(f);
        assert_eq!(proto.id, 99);
    }

    #[test]
    fn feature_conversion_preserves_spec_hash_as_state() {
        let f = Feature::new("s", "S", [0xFF; 32], None);
        let proto = feature_to_proto(f);
        assert_eq!(proto.state, "created");
    }

    #[test]
    fn feature_with_no_work_packages_zeroes_counts() {
        let f = Feature::new("empty", "Empty", [0; 32], None);
        let proto = feature_to_proto_with_wps(f, &[]);
        assert_eq!(proto.wp_count, 0);
        assert_eq!(proto.wp_done, 0);
    }

    #[test]
    fn feature_wp_done_counts_only_done_state() {
        let f = Feature::new("mixed", "Mixed", [0; 32], None);
        let mut wp1 = WorkPackage::new(f.id, "WP1", 1, "c");
        wp1.state = WpState::Done;
        let mut wp2 = WorkPackage::new(f.id, "WP2", 2, "c");
        wp2.state = WpState::Doing;
        let mut wp3 = WorkPackage::new(f.id, "WP3", 3, "c");
        wp3.state = WpState::Done;

        let proto = feature_to_proto_with_wps(f, &[wp1, wp2, wp3]);
        assert_eq!(proto.wp_count, 3);
        assert_eq!(proto.wp_done, 2);
    }

    #[test]
    fn wp_conversion_with_optional_fields() {
        let mut wp = WorkPackage::new(1, "Optional", 5, "criteria");
        wp.agent_id = Some("agent-42".to_string());
        wp.pr_url = Some("https://github.com/pr/123".to_string());
        wp.pr_state = Some(agileplus_domain::domain::work_package::PrState::Approved);
        wp.file_scope = vec!["src/main.rs".into()];

        let proto = wp_to_proto(wp);
        assert_eq!(proto.agent_id, "agent-42");
        assert_eq!(proto.pr_url, "https://github.com/pr/123");
        assert_eq!(proto.pr_state, "approved");
        assert_eq!(proto.file_scope, vec!["src/main.rs"]);
    }

    #[test]
    fn wp_conversion_default_optional_fields() {
        let wp = WorkPackage::new(1, "Defaults", 1, "c");
        let proto = wp_to_proto(wp);
        assert_eq!(proto.agent_id, "");
        assert_eq!(proto.pr_url, "");
        assert_eq!(proto.pr_state, "");
        assert!(proto.depends_on.is_empty());
        assert!(proto.file_scope.is_empty());
    }

    #[test]
    fn wp_state_formatted_lowercase() {
        let states = [
            (WpState::Planned, "planned"),
            (WpState::Doing, "doing"),
            (WpState::Review, "review"),
            (WpState::Done, "done"),
            (WpState::Blocked, "blocked"),
        ];
        for (state, expected) in states {
            let mut wp = WorkPackage::new(1, "t", 1, "c");
            wp.state = state;
            let proto = wp_to_proto(wp);
            assert_eq!(proto.state, expected, "state {state:?} should format as {expected}");
        }
    }

    #[test]
    fn audit_entry_to_proto_conversion() {
        use chrono::DateTime;

        let entry = DomainAuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: Some(5),
            timestamp: DateTime::from_timestamp(1_000_000, 0).unwrap(),
            actor: "test-user".to_string(),
            transition: "Created->Specified".to_string(),
            evidence_refs: vec![
                agileplus_domain::domain::audit::EvidenceRef {
                    evidence_id: 1,
                    fr_id: "FR-001".to_string(),
                },
                agileplus_domain::domain::audit::EvidenceRef {
                    evidence_id: 2,
                    fr_id: "FR-002".to_string(),
                },
            ],
            prev_hash: [0xAB; 32],
            hash: [0xCD; 32],
            event_id: None,
            archived_to: None,
        };

        let proto = audit_entry_to_proto(entry);
        assert_eq!(proto.id, 1);
        assert_eq!(proto.actor, "test-user");
        assert_eq!(proto.transition, "Created->Specified");
        assert_eq!(proto.evidence_refs, vec!["FR-001", "FR-002"]);
        assert_eq!(proto.prev_hash, vec![0xAB; 32]);
        assert_eq!(proto.hash, vec![0xCD; 32]);
        assert_eq!(proto.feature_slug, ""); // caller fills
        assert_eq!(proto.wp_sequence, 0); // caller fills
    }

    #[test]
    fn audit_entry_to_proto_empty_evidence() {
        use chrono::DateTime;

        let entry = DomainAuditEntry {
            id: 2,
            feature_id: 1,
            wp_id: None,
            timestamp: DateTime::from_timestamp(2_000_000, 0).unwrap(),
            actor: "actor".to_string(),
            transition: "A->B".to_string(),
            evidence_refs: vec![],
            prev_hash: [0; 32],
            hash: [0; 32],
            event_id: None,
            archived_to: None,
        };

        let proto = audit_entry_to_proto(entry);
        assert!(proto.evidence_refs.is_empty());
        assert!(proto.prev_hash.is_empty() || proto.prev_hash == vec![0; 32]);
    }

    #[test]
    fn feature_conversion_timestamps_are_rfc3339() {
        let f = Feature::new("ts", "Timestamps", [0; 32], None);
        let proto = feature_to_proto(f);
        assert!(proto.created_at.contains("T"));
        assert!(proto.updated_at.contains("T"));
    }
}
