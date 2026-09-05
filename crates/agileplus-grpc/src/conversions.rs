//! Domain ↔ Protobuf conversion functions.
//!
//! Rust's orphan rule prevents implementing foreign traits on foreign types.
//! We use free conversion functions instead of `From`/`Into` impls.
//!
//! Traceability: WP14-T080

use agileplus_domain::domain::audit::AuditEntry as DomainAuditEntry;
use agileplus_domain::domain::backlog::BacklogItem as DomainBacklogItem;
use agileplus_domain::domain::feature::Feature as DomainFeature;
use agileplus_domain::domain::work_package::WorkPackage as DomainWorkPackage;
use agileplus_proto::agileplus::v1::{
    AuditEntry as ProtoAuditEntry, BacklogItem as ProtoBacklogItem, Feature as ProtoFeature,
    WorkPackageStatus as ProtoWpStatus,
};

/// Convert a domain Feature to its Protobuf representation.
pub fn feature_to_proto(f: DomainFeature) -> ProtoFeature {
    ProtoFeature {
        id: f.id,
        slug: f.slug,
        friendly_name: f.friendly_name,
        state: f.state.to_string(),
        target_branch: f.target_branch,
        created_at: f.created_at.to_rfc3339(),
        updated_at: f.updated_at.to_rfc3339(),
        wp_count: 0, // Populated separately when needed
        wp_done: 0,
    }
}

/// Convert a domain WorkPackage to its Protobuf representation.
pub fn wp_to_proto(wp: DomainWorkPackage) -> ProtoWpStatus {
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
        depends_on: Vec::new(), // Populated separately when needed
        file_scope: wp.file_scope,
    }
}

/// Convert a domain backlog item to the current integrations protobuf shape.
pub fn backlog_item_to_proto(item: DomainBacklogItem) -> ProtoBacklogItem {
    ProtoBacklogItem {
        id: item.id.unwrap_or_default(),
        r#type: item.intent.to_string(),
        title: item.title,
        body: item.description,
        priority: item.priority.to_string(),
        state: item.status.to_string(),
        external_ref: item.source,
        created_at: item.created_at.to_rfc3339(),
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
    use agileplus_domain::domain::work_package::WorkPackage;

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
    fn wp_conversion() {
        let wp = WorkPackage::new(1, "Test WP", 1, "criteria");
        let proto = wp_to_proto(wp);
        assert_eq!(proto.title, "Test WP");
        assert_eq!(proto.state, "planned");
        assert_eq!(proto.sequence, 1);
    }

    #[test]
    fn backlog_conversion_uses_current_proto_fields() {
        use agileplus_domain::domain::backlog::{BacklogItem, Intent};

        let item = BacklogItem::from_triage(
            "Repair backlog bridge".to_string(),
            "Persist through SQLite".to_string(),
            Intent::Task,
            "qe".to_string(),
        );
        let proto = backlog_item_to_proto(item);

        assert_eq!(proto.r#type, "task");
        assert_eq!(proto.title, "Repair backlog bridge");
        assert_eq!(proto.body, "Persist through SQLite");
        assert_eq!(proto.external_ref, "qe");
        assert!(!proto.created_at.is_empty());
    }
}
