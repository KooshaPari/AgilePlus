//! Integration tests for sync module.
//!
//! Covers: SyncState, SyncOutcome, and sync adapter construction.

use agileplus_plane::sync::{PlaneSyncAdapter, SyncOutcome, SyncState};
use agileplus_plane::PlaneClient;

// ── SyncState ───────────────────────────────────────────────

#[test]
fn sync_state_new_defaults() {
    let state = SyncState::new("my-feature".into());
    assert_eq!(state.feature_slug, "my-feature");
    assert!(state.plane_issue_id.is_none());
    assert!(state.last_synced_at.is_none());
    assert!(state.content_hash.is_none());
    assert!(state.wp_mappings.is_empty());
}

#[test]
fn sync_state_set_fields() {
    let mut state = SyncState::new("feat".into());
    state.plane_issue_id = Some("issue-123".into());
    state.content_hash = Some("abc123".into());
    state.last_synced_at = Some(chrono::Utc::now());
    state
        .wp_mappings
        .insert("WP01".into(), "sub-456".into());
    state
        .wp_mappings
        .insert("WP02".into(), "sub-789".into());

    assert_eq!(state.plane_issue_id, Some("issue-123".into()));
    assert_eq!(state.wp_mappings.len(), 2);
    assert_eq!(state.wp_mappings["WP01"], "sub-456");
}

#[test]
fn sync_state_serialization_roundtrip() {
    let mut state = SyncState::new("feat".into());
    state.plane_issue_id = Some("issue-123".into());
    state.content_hash = Some("hash-abc".into());
    state
        .wp_mappings
        .insert("WP01".into(), "sub-456".into());

    let json = serde_json::to_string(&state).unwrap();
    let restored: SyncState = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.feature_slug, "feat");
    assert_eq!(restored.plane_issue_id, Some("issue-123".into()));
    assert_eq!(restored.content_hash, Some("hash-abc".into()));
    assert_eq!(restored.wp_mappings["WP01"], "sub-456");
}

#[test]
fn sync_state_empty_wp_mappings() {
    let state = SyncState::new("feat".into());
    let json = serde_json::to_string(&state).unwrap();
    let restored: SyncState = serde_json::from_str(&json).unwrap();
    assert!(restored.wp_mappings.is_empty());
}

#[test]
fn sync_state_debug_format() {
    let state = SyncState::new("feat".into());
    let debug = format!("{:?}", state);
    assert!(debug.contains("SyncState"));
    assert!(debug.contains("feat"));
}

#[test]
fn sync_state_clone() {
    let mut state = SyncState::new("feat".into());
    state.plane_issue_id = Some("i1".into());
    state
        .wp_mappings
        .insert("WP1".into(), "sub1".into());
    let cloned = state.clone();
    assert_eq!(cloned.feature_slug, "feat");
    assert_eq!(cloned.plane_issue_id, Some("i1".into()));
    assert_eq!(cloned.wp_mappings["WP1"], "sub1");
}

// ── SyncOutcome ─────────────────────────────────────────────

#[test]
fn sync_outcome_created() {
    let outcome = SyncOutcome::Created("id-1".into());
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Created"));
    assert!(debug.contains("id-1"));
}

#[test]
fn sync_outcome_updated() {
    let outcome = SyncOutcome::Updated("id-2".into());
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Updated"));
    assert!(debug.contains("id-2"));
}

#[test]
fn sync_outcome_skipped() {
    let outcome = SyncOutcome::Skipped;
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Skipped"));
}

#[test]
fn sync_outcome_conflict() {
    let outcome = SyncOutcome::Conflict("id-3".into());
    let debug = format!("{:?}", outcome);
    assert!(debug.contains("Conflict"));
    assert!(debug.contains("id-3"));
}

#[test]
fn sync_outcome_variants_are_distinct() {
    let created = SyncOutcome::Created("a".into());
    let updated = SyncOutcome::Updated("b".into());
    let skipped = SyncOutcome::Skipped;
    let conflict = SyncOutcome::Conflict("c".into());

    assert_ne!(format!("{:?}", created), format!("{:?}", updated));
    assert_ne!(format!("{:?}", created), format!("{:?}", skipped));
    assert_ne!(format!("{:?}", created), format!("{:?}", conflict));
    assert_ne!(format!("{:?}", updated), format!("{:?}", skipped));
    assert_ne!(format!("{:?}", updated), format!("{:?}", conflict));
    assert_ne!(format!("{:?}", skipped), format!("{:?}", conflict));
}

#[test]
fn sync_outcome_partial_eq() {
    assert_eq!(
        SyncOutcome::Created("a".into()),
        SyncOutcome::Created("a".into())
    );
    assert_ne!(
        SyncOutcome::Created("a".into()),
        SyncOutcome::Created("b".into())
    );
    assert_ne!(
        SyncOutcome::Created("a".into()),
        SyncOutcome::Updated("a".into())
    );
    assert_eq!(SyncOutcome::Skipped, SyncOutcome::Skipped);
}

// ── PlaneSyncAdapter construction ───────────────────────────

#[test]
fn sync_adapter_construction() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "key".into(),
        "ws".into(),
        "proj".into(),
    );
    let adapter = PlaneSyncAdapter::new(client);
    let debug = format!("{:?}", adapter);
    assert!(debug.contains("PlaneSyncAdapter"));
}

#[test]
fn sync_adapter_debug_format() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "key".into(),
        "ws".into(),
        "proj".into(),
    );
    let adapter = PlaneSyncAdapter::new(client);
    let debug = format!("{:?}", adapter);
    assert!(debug.contains("PlaneSyncAdapter"));
}
