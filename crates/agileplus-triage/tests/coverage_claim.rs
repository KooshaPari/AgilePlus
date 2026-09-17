//! Deep coverage for agileplus-triage claim primitives:
//! claim kinds/states/reasons, store semantics, TTL expiry, handoff,
//! trait dispatch, and the claim event watcher.

use chrono::{Duration, Utc};

use agileplus_triage::claim::{
    Claim, ClaimError, ClaimKind, ClaimReason, ClaimState, ClaimStore, ClaimStoreTrait,
};
use agileplus_triage::claim_watcher::{ClaimEvent, ClaimWatcher};

fn reason() -> ClaimReason {
    ClaimReason::Manual("test".into())
}

fn make_claim(id: &str, resource: &str, kind: ClaimKind, ttl: i64) -> Claim {
    let now = Utc::now();
    Claim {
        id: id.into(),
        resource: resource.into(),
        kind,
        agent_id: "agent-a".into(),
        created_at: now,
        last_heartbeat: now,
        ttl_seconds: ttl,
        state: ClaimState::Active,
        reason: reason(),
    }
}

// ============================================================
// enums and serde
// ============================================================

#[test]
fn claim_kind_serde_snake_case() {
    assert_eq!(serde_json::to_string(&ClaimKind::Repo).unwrap(), "\"repo\"");
    assert_eq!(
        serde_json::to_string(&ClaimKind::Branch).unwrap(),
        "\"branch\""
    );
    assert_eq!(
        serde_json::to_string(&ClaimKind::Worktree).unwrap(),
        "\"worktree\""
    );
    assert_eq!(
        serde_json::to_string(&ClaimKind::Subproject).unwrap(),
        "\"subproject\""
    );
}

#[test]
fn claim_kind_roundtrip_all() {
    for kind in [
        ClaimKind::Repo,
        ClaimKind::Branch,
        ClaimKind::Worktree,
        ClaimKind::Subproject,
    ] {
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(serde_json::from_str::<ClaimKind>(&json).unwrap(), kind);
    }
}

#[test]
fn claim_kind_rejects_unknown() {
    assert!(serde_json::from_str::<ClaimKind>("\"galaxy\"").is_err());
}

#[test]
fn claim_kind_copy_and_eq() {
    let k = ClaimKind::Repo;
    let copy = k;
    assert_eq!(copy, k);
    assert_ne!(ClaimKind::Repo, ClaimKind::Branch);
}

#[test]
fn claim_kind_hash_distinguishes_variants() {
    use std::collections::HashSet;
    let set: HashSet<ClaimKind> = [
        ClaimKind::Repo,
        ClaimKind::Repo,
        ClaimKind::Branch,
        ClaimKind::Worktree,
    ]
    .into_iter()
    .collect();
    assert_eq!(set.len(), 3);
}

#[test]
fn claim_state_serde_snake_case() {
    assert_eq!(
        serde_json::to_string(&ClaimState::Active).unwrap(),
        "\"active\""
    );
    assert_eq!(
        serde_json::to_string(&ClaimState::Draining).unwrap(),
        "\"draining\""
    );
    assert_eq!(
        serde_json::to_string(&ClaimState::Expired).unwrap(),
        "\"expired\""
    );
}

#[test]
fn claim_state_roundtrip() {
    for state in [ClaimState::Active, ClaimState::Draining, ClaimState::Expired] {
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(serde_json::from_str::<ClaimState>(&json).unwrap(), state);
    }
}

#[test]
fn claim_reason_default_is_empty_manual() {
    assert_eq!(ClaimReason::default(), ClaimReason::Manual(String::new()));
    assert_eq!(ClaimReason::default().kind_str(), "manual");
    assert_eq!(ClaimReason::default().value(), "");
}

#[test]
fn claim_reason_kind_strs() {
    assert_eq!(ClaimReason::TaskRef("x".into()).kind_str(), "task_ref");
    assert_eq!(ClaimReason::Branch("x".into()).kind_str(), "branch");
    assert_eq!(ClaimReason::Subproject("x".into()).kind_str(), "subproject");
    assert_eq!(ClaimReason::WipRun("x".into()).kind_str(), "wip_run");
    assert_eq!(ClaimReason::Manual("x".into()).kind_str(), "manual");
}

#[test]
fn claim_reason_values() {
    assert_eq!(ClaimReason::TaskRef("wp-1".into()).value(), "wp-1");
    assert_eq!(ClaimReason::Branch("feat/x".into()).value(), "feat/x");
    assert_eq!(ClaimReason::Subproject("infra".into()).value(), "infra");
    assert_eq!(ClaimReason::WipRun("run".into()).value(), "run");
    assert_eq!(ClaimReason::Manual("note".into()).value(), "note");
}

#[test]
fn claim_reason_serde_wire_shape() {
    let r = ClaimReason::TaskRef("wp-1".into());
    assert_eq!(
        serde_json::to_string(&r).unwrap(),
        r#"{"kind":"task_ref","value":"wp-1"}"#
    );
}

#[test]
fn claim_reason_roundtrip_each_variant() {
    for r in [
        ClaimReason::TaskRef("wp-1".into()),
        ClaimReason::Branch("feat/login".into()),
        ClaimReason::Subproject("infra".into()),
        ClaimReason::WipRun("run-42".into()),
        ClaimReason::Manual("flaky".into()),
    ] {
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(serde_json::from_str::<ClaimReason>(&json).unwrap(), r);
    }
}

#[test]
fn claim_reason_rejects_unknown_kind() {
    assert!(serde_json::from_str::<ClaimReason>(r#"{"kind":"nope","value":"x"}"#).is_err());
}

#[test]
fn claim_reason_equality_and_clone() {
    let a = ClaimReason::Branch("b".into());
    assert_eq!(a.clone(), a);
    assert_ne!(a, ClaimReason::Branch("c".into()));
}

// ============================================================
// Claim::age_seconds / is_expired
// ============================================================

#[test]
fn age_seconds_positive_for_past_heartbeat() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 60);
    c.last_heartbeat = now - Duration::seconds(30);
    assert_eq!(c.age_seconds(now), 30);
}

#[test]
fn age_seconds_negative_for_future_heartbeat() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 60);
    c.last_heartbeat = now + Duration::seconds(5);
    assert_eq!(c.age_seconds(now), -5);
}

#[test]
fn is_expired_false_exactly_at_ttl_boundary() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 10);
    c.last_heartbeat = now - Duration::seconds(10);
    assert!(!c.is_expired(now));
}

#[test]
fn is_expired_true_just_past_ttl() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 10);
    c.last_heartbeat = now - Duration::seconds(10) - Duration::milliseconds(1);
    assert!(c.is_expired(now));
}

#[test]
fn is_expired_false_for_zero_ttl_at_exact_instant() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 0);
    c.last_heartbeat = now;
    assert!(!c.is_expired(now));
}

#[test]
fn is_expired_true_for_zero_ttl_after_a_millisecond() {
    let now = Utc::now();
    let mut c = make_claim("c", "r", ClaimKind::Repo, 0);
    c.last_heartbeat = now - Duration::milliseconds(2);
    assert!(c.is_expired(now));
}

#[test]
fn is_expired_true_for_negative_ttl_with_fresh_heartbeat() {
    let now = Utc::now();
    let c = make_claim("c", "r", ClaimKind::Repo, -5);
    assert!(c.is_expired(now));
}

#[test]
fn claim_serde_roundtrip() {
    let c = make_claim("c1", "repo:foo", ClaimKind::Repo, 120);
    let json = serde_json::to_string(&c).unwrap();
    let back: Claim = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "c1");
    assert_eq!(back.resource, "repo:foo");
    assert_eq!(back.kind, ClaimKind::Repo);
    assert_eq!(back.ttl_seconds, 120);
    assert_eq!(back.state, ClaimState::Active);
}

#[test]
fn claim_serde_defaults_reason_when_absent() {
    let c = make_claim("c1", "r", ClaimKind::Repo, 60);
    let mut json: serde_json::Value = serde_json::to_value(&c).unwrap();
    json.as_object_mut().unwrap().remove("reason");
    let back: Claim = serde_json::from_value(json).unwrap();
    assert_eq!(back.reason, ClaimReason::Manual(String::new()));
}

// ============================================================
// ClaimStore: claim()
// ============================================================

#[test]
fn claim_store_new_is_empty() {
    let s = ClaimStore::new();
    assert!(s.all().is_empty());
    assert!(s.active().is_empty());
}

#[test]
fn claim_store_default_is_empty() {
    let s = ClaimStore::default();
    assert!(s.all().is_empty());
}

#[test]
fn claim_store_claim_returns_record() {
    let mut s = ClaimStore::new();
    let c = s
        .claim("c1", "repo:foo", ClaimKind::Repo, "a", 60, reason())
        .unwrap();
    assert_eq!(c.id, "c1");
    assert_eq!(c.agent_id, "a");
    assert_eq!(c.state, ClaimState::Active);
    assert_eq!(c.ttl_seconds, 60);
}

#[test]
fn claim_store_claim_is_retrievable_by_resource() {
    let mut s = ClaimStore::new();
    s.claim("c1", "repo:foo", ClaimKind::Repo, "a", 60, reason());
    let found = s.lookup(ClaimKind::Repo, "repo:foo").unwrap();
    assert_eq!(found.id, "c1");
}

#[test]
fn claim_store_lookup_wrong_kind_misses() {
    let mut s = ClaimStore::new();
    s.claim("c1", "shared", ClaimKind::Repo, "a", 60, reason());
    assert!(s.lookup(ClaimKind::Branch, "shared").is_none());
    assert!(s.lookup(ClaimKind::Worktree, "shared").is_none());
}

#[test]
fn claim_store_second_claim_on_same_resource_is_rejected() {
    let mut s = ClaimStore::new();
    assert!(s
        .claim("c1", "repo:foo", ClaimKind::Repo, "a", 60, reason())
        .is_some());
    assert!(s
        .claim("c2", "repo:foo", ClaimKind::Repo, "b", 60, reason())
        .is_none());
    assert_eq!(s.all().len(), 1);
}

#[test]
fn claim_store_same_resource_different_kind_allowed() {
    let mut s = ClaimStore::new();
    assert!(s.claim("c1", "x", ClaimKind::Repo, "a", 60, reason()).is_some());
    assert!(s.claim("c2", "x", ClaimKind::Branch, "b", 60, reason()).is_some());
    assert_eq!(s.all().len(), 2);
}

#[test]
fn claim_store_different_resources_allowed() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r1", ClaimKind::Repo, "a", 60, reason());
    s.claim("c2", "r2", ClaimKind::Repo, "a", 60, reason());
    assert_eq!(s.active().len(), 2);
}

#[test]
fn claim_store_reclaim_same_id_updates_record() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r1", ClaimKind::Repo, "a", 60, reason());
    let c = s
        .claim("c1", "r1", ClaimKind::Repo, "a", 120, ClaimReason::TaskRef("wp".into()))
        .unwrap();
    assert_eq!(c.ttl_seconds, 120);
    assert_eq!(c.reason, ClaimReason::TaskRef("wp".into()));
    assert_eq!(s.all().len(), 1);
}

#[test]
fn claim_store_claim_after_release_succeeds() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.release("c1"));
    assert!(s.claim("c2", "r", ClaimKind::Repo, "b", 60, reason()).is_some());
}

#[test]
fn claim_store_claim_after_expiry_reap_succeeds() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 0, reason());
    std::thread::sleep(std::time::Duration::from_millis(3));
    assert_eq!(s.reap_expired(Utc::now()), 1);
    assert!(s.claim("c2", "r", ClaimKind::Repo, "b", 60, reason()).is_some());
}

// ============================================================
// ClaimStore: heartbeat / release
// ============================================================

#[test]
fn heartbeat_existing_returns_true() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.heartbeat("c1"));
}

#[test]
fn heartbeat_missing_returns_false() {
    let mut s = ClaimStore::new();
    assert!(!s.heartbeat("nope"));
}

#[test]
fn heartbeat_refreshes_last_heartbeat() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 3600, reason());
    let before = s.lookup(ClaimKind::Repo, "r").unwrap().last_heartbeat;
    std::thread::sleep(std::time::Duration::from_millis(5));
    s.heartbeat("c1");
    let after = s.lookup(ClaimKind::Repo, "r").unwrap().last_heartbeat;
    assert!(after >= before);
}

#[test]
fn release_existing_returns_true() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.release("c1"));
}

#[test]
fn release_missing_returns_false() {
    let mut s = ClaimStore::new();
    assert!(!s.release("nope"));
}

#[test]
fn release_removes_claim_and_frees_resource() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    s.release("c1");
    assert!(s.all().is_empty());
    assert!(s.lookup(ClaimKind::Repo, "r").is_none());
}

#[test]
fn release_twice_second_fails() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.release("c1"));
    assert!(!s.release("c1"));
}

// ============================================================
// ClaimStore: reap_expired
// ============================================================

#[test]
fn reap_expired_returns_zero_when_nothing_expired() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 3600, reason());
    assert_eq!(s.reap_expired(Utc::now()), 0);
}

#[test]
fn reap_expired_removes_only_expired() {
    let mut s = ClaimStore::new();
    s.claim("short", "r1", ClaimKind::Repo, "a", 0, reason());
    s.claim("long", "r2", ClaimKind::Repo, "b", 3600, reason());
    std::thread::sleep(std::time::Duration::from_millis(3));
    assert_eq!(s.reap_expired(Utc::now()), 1);
    assert!(s.lookup(ClaimKind::Repo, "r1").is_none());
    assert!(s.lookup(ClaimKind::Repo, "r2").is_some());
}

#[test]
fn reap_expired_with_future_now_reaps_all() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r1", ClaimKind::Repo, "a", 60, reason());
    s.claim("c2", "r2", ClaimKind::Repo, "b", 60, reason());
    let far_future = Utc::now() + Duration::days(1);
    assert_eq!(s.reap_expired(far_future), 2);
    assert!(s.all().is_empty());
}

#[test]
fn reap_is_idempotent() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 0, reason());
    std::thread::sleep(std::time::Duration::from_millis(3));
    assert_eq!(s.reap_expired(Utc::now()), 1);
    assert_eq!(s.reap_expired(Utc::now()), 0);
}

// ============================================================
// ClaimStore: active / all
// ============================================================

#[test]
fn active_excludes_draining() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r1", ClaimKind::Repo, "a", 60, reason());
    s.claim("c2", "r2", ClaimKind::Repo, "b", 60, reason());
    s.claim_transfer("c1", "c3", "c").unwrap();
    let ids: Vec<String> = s.active().into_iter().map(|c| c.id).collect();
    assert!(!ids.contains(&"c1".to_string()));
    assert!(ids.contains(&"c2".to_string()));
    assert!(ids.contains(&"c3".to_string()));
}

#[test]
fn all_includes_draining() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r1", ClaimKind::Repo, "a", 60, reason());
    s.claim_transfer("c1", "c2", "b").unwrap();
    assert_eq!(s.all().len(), 2);
}

// ============================================================
// ClaimStore: claim_transfer
// ============================================================

#[test]
fn transfer_inherits_resource_kind_ttl_reason() {
    let mut s = ClaimStore::new();
    s.claim(
        "c1",
        "branch:feat",
        ClaimKind::Branch,
        "a",
        3600,
        ClaimReason::Branch("feat/login".into()),
    );
    let new = s.claim_transfer("c1", "c2", "agent-b").unwrap();
    assert_eq!(new.id, "c2");
    assert_eq!(new.agent_id, "agent-b");
    assert_eq!(new.resource, "branch:feat");
    assert_eq!(new.kind, ClaimKind::Branch);
    assert_eq!(new.ttl_seconds, 3600);
    assert_eq!(new.reason, ClaimReason::Branch("feat/login".into()));
    assert_eq!(new.state, ClaimState::Active);
}

#[test]
fn transfer_marks_source_draining() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    s.claim_transfer("c1", "c2", "b").unwrap();
    let old = s.all().into_iter().find(|c| c.id == "c1").unwrap();
    assert_eq!(old.state, ClaimState::Draining);
}

#[test]
fn transfer_repoints_resource_lookup_to_new_claim() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    s.claim_transfer("c1", "c2", "b").unwrap();
    assert_eq!(s.lookup(ClaimKind::Repo, "r").unwrap().id, "c2");
}

#[test]
fn transfer_missing_source_is_not_found() {
    let mut s = ClaimStore::new();
    match s.claim_transfer("ghost", "c2", "b") {
        Err(ClaimError::NotFound(id)) => assert_eq!(id, "ghost"),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn transfer_draining_source_is_wrong_state() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    s.claim_transfer("c1", "c2", "b").unwrap();
    assert!(matches!(
        s.claim_transfer("c1", "c3", "c"),
        Err(ClaimError::WrongState)
    ));
}

#[test]
fn transfer_chain_from_new_claim_is_allowed() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    s.claim_transfer("c1", "c2", "b").unwrap();
    // c2 is Active, so it can be transferred onward.
    let c3 = s.claim_transfer("c2", "c3", "c").unwrap();
    assert_eq!(c3.agent_id, "c");
    assert_eq!(c3.resource, "r");
}

#[test]
fn transfer_to_same_id_overwrites() {
    let mut s = ClaimStore::new();
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    // from_id == to_id: old marked Draining then new inserted under same id.
    let new = s.claim_transfer("c1", "c1", "b").unwrap();
    assert_eq!(new.id, "c1");
    assert_eq!(new.agent_id, "b");
    assert_eq!(new.state, ClaimState::Active);
}

// ============================================================
// ClaimError display
// ============================================================

#[test]
fn claim_error_not_found_display() {
    assert!(ClaimError::NotFound("abc".into()).to_string().contains("abc"));
}

#[test]
fn claim_error_wrong_owner_display() {
    let msg = ClaimError::WrongOwner {
        expected: "alice".into(),
        actual: "bob".into(),
    }
    .to_string();
    assert!(msg.contains("alice") && msg.contains("bob"));
}

#[test]
fn claim_error_wrong_state_display() {
    assert_eq!(
        ClaimError::WrongState.to_string(),
        "wrong state for operation"
    );
}

// ============================================================
// ClaimStoreTrait dispatch
// ============================================================

#[test]
fn trait_claim_and_lookup() {
    let mut s: Box<dyn ClaimStoreTrait> = Box::new(ClaimStore::new());
    assert!(s
        .claim("c1", "r", ClaimKind::Repo, "a", 60, reason())
        .is_some());
    assert_eq!(s.all().len(), 1);
    assert_eq!(s.active().len(), 1);
    assert!(s.lookup(ClaimKind::Repo, "r").is_some());
}

#[test]
fn trait_conflict_returns_none() {
    let mut s: Box<dyn ClaimStoreTrait> = Box::new(ClaimStore::new());
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.claim("c2", "r", ClaimKind::Repo, "b", 60, reason()).is_none());
}

#[test]
fn trait_heartbeat_and_release() {
    let mut s: Box<dyn ClaimStoreTrait> = Box::new(ClaimStore::new());
    s.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert!(s.heartbeat("c1"));
    assert!(!s.heartbeat("nope"));
    assert!(s.release("c1"));
    assert!(!s.release("c1"));
}

#[test]
fn trait_reap_and_transfer() {
    let mut s: Box<dyn ClaimStoreTrait> = Box::new(ClaimStore::new());
    s.claim("c1", "r1", ClaimKind::Repo, "a", 0, reason());
    s.claim("c2", "r2", ClaimKind::Repo, "b", 3600, reason());
    std::thread::sleep(std::time::Duration::from_millis(3));
    assert_eq!(s.reap_expired(Utc::now()), 1);
    let new = s.claim_transfer("c2", "c3", "c").unwrap();
    assert_eq!(new.id, "c3");
}

#[test]
fn trait_object_is_send_usable() {
    // The trait is object-safe; store it behind a boxed trait object.
    let s: Box<dyn ClaimStoreTrait> = Box::new(ClaimStore::new());
    assert!(s.all().is_empty());
}

// ============================================================
// ClaimWatcher
// ============================================================

#[test]
fn watcher_new_is_empty() {
    let w = ClaimWatcher::new();
    assert!(w.all().is_empty());
    assert!(w.active().is_empty());
}

#[test]
fn watcher_default_is_empty() {
    let w: ClaimWatcher = Default::default();
    assert!(w.all().is_empty());
}

#[test]
fn watcher_with_store_preserves_existing_claims() {
    let mut store = ClaimStore::new();
    store.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    let w = ClaimWatcher::with_store(store);
    assert_eq!(w.all().len(), 1);
}

#[test]
fn watcher_emits_claimed_to_listener() {
    let mut w = ClaimWatcher::new();
    let mut rx = w.watch("c1");
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    assert_eq!(rx.try_recv().unwrap(), ClaimEvent::Claimed);
}

#[test]
fn watcher_emits_heartbeat_to_listener() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    let mut rx = w.watch("c1");
    w.heartbeat("c1");
    assert_eq!(rx.try_recv().unwrap(), ClaimEvent::Heartbeat);
}

#[test]
fn watcher_emits_released_to_listener() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    let mut rx = w.watch("c1");
    w.release("c1");
    assert_eq!(rx.try_recv().unwrap(), ClaimEvent::Released);
}

#[test]
fn watcher_emits_expired_for_reaped_claims() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 0, reason());
    let mut rx = w.watch("c1");
    std::thread::sleep(std::time::Duration::from_millis(3));
    assert_eq!(w.reap_expired(Utc::now()), 1);
    assert_eq!(rx.try_recv().unwrap(), ClaimEvent::Expired);
}

#[test]
fn watcher_emits_transferred_and_claimed_on_transfer() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    let mut rx_from = w.watch("c1");
    let mut rx_to = w.watch("c2");
    w.claim_transfer("c1", "c2", "b").unwrap();
    assert_eq!(
        rx_from.try_recv().unwrap(),
        ClaimEvent::Transferred {
            from: "c1".into(),
            to: "c2".into()
        }
    );
    assert_eq!(rx_to.try_recv().unwrap(), ClaimEvent::Claimed);
}

#[test]
fn watcher_failed_claim_emits_nothing() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    let mut rx = w.watch("c2");
    assert!(w.claim("c2", "r", ClaimKind::Repo, "b", 60, reason()).is_none());
    assert!(rx.try_recv().is_err());
}

#[test]
fn watcher_failed_heartbeat_emits_nothing() {
    let mut w = ClaimWatcher::new();
    let mut rx = w.watch("ghost");
    assert!(!w.heartbeat("ghost"));
    assert!(rx.try_recv().is_err());
}

#[test]
fn watcher_failed_release_emits_nothing() {
    let mut w = ClaimWatcher::new();
    let mut rx = w.watch("ghost");
    assert!(!w.release("ghost"));
    assert!(rx.try_recv().is_err());
}

#[test]
fn watcher_failed_transfer_emits_nothing() {
    let mut w = ClaimWatcher::new();
    let mut rx = w.watch("ghost");
    assert!(w.claim_transfer("ghost", "c2", "b").is_err());
    assert!(rx.try_recv().is_err());
}

#[test]
fn watcher_release_clears_store_after_event() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Repo, "a", 60, reason());
    w.heartbeat("c1");
    w.release("c1");
    // After release the underlying store no longer holds the claim.
    assert!(w.store().all().is_empty());
    assert!(w.lookup(ClaimKind::Repo, "r").is_none());
}

#[test]
fn watcher_stores_and_lookup_delegate() {
    let mut w = ClaimWatcher::new();
    w.claim("c1", "r", ClaimKind::Worktree, "a", 60, reason());
    assert!(w.lookup(ClaimKind::Worktree, "r").is_some());
    assert_eq!(w.active().len(), 1);
}

#[test]
fn watcher_store_mut_allows_direct_mutation() {
    let mut w = ClaimWatcher::new();
    w.store_mut()
        .claim("direct", "r", ClaimKind::Repo, "a", 60, reason());
    assert_eq!(w.all().len(), 1);
}

#[test]
fn claim_event_variants_distinct() {
    assert_ne!(ClaimEvent::Claimed, ClaimEvent::Heartbeat);
    assert_ne!(ClaimEvent::Released, ClaimEvent::Expired);
    assert_ne!(
        ClaimEvent::Transferred {
            from: "a".into(),
            to: "b".into()
        },
        ClaimEvent::Transferred {
            from: "a".into(),
            to: "c".into()
        }
    );
}

#[test]
fn claim_event_debug_and_clone() {
    let e = ClaimEvent::Transferred {
        from: "a".into(),
        to: "b".into(),
    };
    assert!(format!("{e:?}").contains("Transferred"));
    assert_eq!(e.clone(), e);
}
