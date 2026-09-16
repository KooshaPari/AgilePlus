//! Integration tests for inbound sync module.
//!
//! Covers: InboundSync processing, LocalEntityStore trait, event routing.

use std::collections::HashMap;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_plane::inbound::{InboundOutcome, InboundSync, LocalEntityStore};
use agileplus_plane::state_mapper::PlaneStateMapper;
use agileplus_plane::webhook::{
    PlaneInboundEvent, PlaneWebhookCycle, PlaneWebhookIssue, PlaneWebhookModule,
};

// ── Mock Store ──────────────────────────────────────────────

struct MockStore {
    hashes: HashMap<String, String>,
    archived: Vec<String>,
    imported: Vec<PlaneWebhookIssue>,
}

impl MockStore {
    fn new() -> Self {
        Self {
            hashes: HashMap::new(),
            archived: Vec::new(),
            imported: Vec::new(),
        }
    }

    fn with_hash(mut self, id: &str, hash: &str) -> Self {
        self.hashes.insert(id.into(), hash.into());
        self
    }
}

impl LocalEntityStore for MockStore {
    fn get_content_hash(&self, id: &str) -> Option<String> {
        self.hashes.get(id).cloned()
    }

    fn apply_update(
        &mut self,
        id: &str,
        _state: FeatureState,
        hash: String,
    ) -> anyhow::Result<()> {
        self.hashes.insert(id.into(), hash);
        Ok(())
    }

    fn mark_archived(&mut self, id: &str) -> anyhow::Result<()> {
        self.archived.push(id.into());
        Ok(())
    }

    fn auto_import(
        &mut self,
        webhook_issue: &PlaneWebhookIssue,
        _state: FeatureState,
    ) -> anyhow::Result<()> {
        self.imported.push(webhook_issue.clone());
        Ok(())
    }
}

fn webhook_issue(id: &str, name: &str, state: Option<&str>) -> PlaneWebhookIssue {
    PlaneWebhookIssue {
        id: id.into(),
        name: name.into(),
        description_html: None,
        state: state.map(|s| s.into()),
        labels: vec![],
        project: None,
    }
}

// ── IssueCreated ────────────────────────────────────────────

#[test]
fn issue_created_auto_imports_when_enabled() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::IssueCreated(webhook_issue("i1", "New Issue", Some("backlog")));
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::AutoImported { .. }));
    if let InboundOutcome::AutoImported { issue_id, .. } = outcome {
        assert_eq!(issue_id, "i1");
    }
    assert_eq!(store.imported.len(), 1);
    assert_eq!(store.imported[0].id, "i1");
}

#[test]
fn issue_created_not_tracked_when_disabled() {
    let sync = InboundSync::new(PlaneStateMapper::new(), false);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::IssueCreated(webhook_issue("i2", "Issue", Some("started")));
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::NotTracked { .. }));
    if let InboundOutcome::NotTracked { issue_id } = outcome {
        assert_eq!(issue_id, "i2");
    }
    assert!(store.imported.is_empty());
}

#[test]
fn issue_created_already_tracked_treats_as_update() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new().with_hash("i3", "old_hash");

    let event = PlaneInboundEvent::IssueCreated(webhook_issue("i3", "Updated Issue", Some("started")));
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::Updated { .. }));
}

// ── IssueUpdated ────────────────────────────────────────────

#[test]
fn issue_updated_tracked_entity_updates_hash() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new().with_hash("i4", "old_hash");

    let event = PlaneInboundEvent::IssueUpdated(webhook_issue("i4", "Updated", Some("started")));
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::Updated { .. }));
    // The store should have a new hash
    assert!(store.hashes.contains_key("i4"));
}

#[test]
fn issue_updated_untracked_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();

    let event = PlaneInboundEvent::IssueUpdated(webhook_issue("i5", "Untracked", Some("backlog")));
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::NotTracked { .. }));
}

// ── IssueDeleted ────────────────────────────────────────────

#[test]
fn issue_deleted_tracked_archives() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new().with_hash("i6", "hash");

    let event = PlaneInboundEvent::IssueDeleted { issue_id: "i6".into() };
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::Archived { .. }));
    assert!(store.archived.contains(&"i6".to_string()));
}

#[test]
fn issue_deleted_untracked_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();

    let event = PlaneInboundEvent::IssueDeleted { issue_id: "i7".into() };
    let outcome = sync.process(event, &mut store).unwrap();

    assert!(matches!(outcome, InboundOutcome::NotTracked { .. }));
    assert!(store.archived.is_empty());
}

// ── Module/Cycle events return NotTracked ───────────────────

#[test]
fn module_updated_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::ModuleUpdated(PlaneWebhookModule {
        id: "mod-1".into(),
        name: "Auth".into(),
        description: None,
    });
    let outcome = sync.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "mod-1"),
        other => panic!("expected NotTracked, got {:?}", other),
    }
}

#[test]
fn module_deleted_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::ModuleDeleted { module_id: "mod-2".into() };
    let outcome = sync.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "mod-2"),
        other => panic!("expected NotTracked, got {:?}", other),
    }
}

#[test]
fn cycle_updated_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::CycleUpdated(PlaneWebhookCycle {
        id: "cyc-1".into(),
        name: "Sprint 1".into(),
        start_date: None,
        end_date: None,
    });
    let outcome = sync.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "cyc-1"),
        other => panic!("expected NotTracked, got {:?}", other),
    }
}

#[test]
fn cycle_deleted_returns_not_tracked() {
    let sync = InboundSync::new(PlaneStateMapper::new(), true);
    let mut store = MockStore::new();
    let event = PlaneInboundEvent::CycleDeleted { cycle_id: "cyc-2".into() };
    let outcome = sync.process(event, &mut store).unwrap();

    match outcome {
        InboundOutcome::NotTracked { issue_id } => assert_eq!(issue_id, "cyc-2"),
        other => panic!("expected NotTracked, got {:?}", other),
    }
}

// ── InboundOutcome variants ─────────────────────────────────

#[test]
fn inbound_outcome_auto_imported_debug() {
    let outcome = InboundOutcome::AutoImported {
        issue_id: "1".into(),
        title: "Test".into(),
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("AutoImported"));
}

#[test]
fn inbound_outcome_updated_debug() {
    let outcome = InboundOutcome::Updated {
        issue_id: "1".into(),
        new_hash: "abc".into(),
        new_state: FeatureState::Implementing,
    };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Updated"));
}

#[test]
fn inbound_outcome_unchanged_debug() {
    let outcome = InboundOutcome::Unchanged { issue_id: "1".into() };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Unchanged"));
}

#[test]
fn inbound_outcome_archived_debug() {
    let outcome = InboundOutcome::Archived { issue_id: "1".into() };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Archived"));
}

#[test]
fn inbound_outcome_not_tracked_debug() {
    let outcome = InboundOutcome::NotTracked { issue_id: "1".into() };
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("NotTracked"));
}

// ── InboundOutcome equality ─────────────────────────────────

#[test]
fn inbound_outcome_partial_eq() {
    assert_eq!(
        InboundOutcome::Archived { issue_id: "1".into() },
        InboundOutcome::Archived { issue_id: "1".into() }
    );
    assert_ne!(
        InboundOutcome::Archived { issue_id: "1".into() },
        InboundOutcome::Archived { issue_id: "2".into() }
    );
}

#[test]
fn inbound_outcome_serialization_roundtrip() {
    let outcome = InboundOutcome::Updated {
        issue_id: "i1".into(),
        new_hash: "abc".into(),
        new_state: FeatureState::Implementing,
    };
    let json = serde_json::to_string(&outcome).unwrap();
    let restored: InboundOutcome = serde_json::from_str(&json).unwrap();
    assert_eq!(
        outcome,
        restored
    );
}
