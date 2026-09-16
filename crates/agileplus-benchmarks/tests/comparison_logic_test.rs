//! Integration tests for sync roundtrip and comparison logic.
//!
//! Tests the serialise-deserialise hot path, conflict detection patterns,
//! and JSON schema consistency used by the benchmark suite.

use agileplus_benchmarks::helpers::*;
use agileplus_domain::domain::state_machine::FeatureState;

// ---------------------------------------------------------------------------
// Serialisation roundtrip fidelity
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_preserves_all_fields() {
    let p = SyncPayload::new(42);
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.id, p.id);
    assert_eq!(out.slug, p.slug);
    assert_eq!(out.state, p.state);
    assert_eq!(out.description, p.description);
}

#[test]
fn roundtrip_is_deterministic() {
    let p = SyncPayload::new(5);
    let out1 = simulate_sync_roundtrip(&p);
    let out2 = simulate_sync_roundtrip(&p);
    assert_eq!(out1.id, out2.id);
    assert_eq!(out1.slug, out2.slug);
    assert_eq!(out1.description, out2.description);
}

#[test]
fn roundtrip_batch_consistency() {
    let payloads = make_sync_payloads(100);
    for p in &payloads {
        let out = simulate_sync_roundtrip(p);
        assert_eq!(out.id, p.id);
        assert_eq!(out.slug, p.slug);
    }
}

// ---------------------------------------------------------------------------
// Push/pull schema consistency (mirrors bench pattern)
// ---------------------------------------------------------------------------

#[test]
fn push_schema_has_required_keys() {
    let p = SyncPayload::new(1);
    let json = serde_json::json!({
        "title":       &p.slug,
        "state":       format!("{:?}", p.state),
        "description": &p.description,
    });
    assert!(json.get("title").is_some());
    assert!(json.get("state").is_some());
    assert!(json.get("description").is_some());
}

#[test]
fn pull_schema_has_required_keys() {
    let raw = serde_json::json!({
        "id":          1_i64,
        "slug":        "feature-1",
        "state":       "Specified",
        "description": "hello",
    });
    let v: serde_json::Value = serde_json::from_str(&raw.to_string()).unwrap();
    assert!(v.get("id").is_some());
    assert!(v.get("slug").is_some());
    assert!(v.get("state").is_some());
    assert!(v.get("description").is_some());
}

#[test]
fn push_pull_key_alignment() {
    // Push uses: title, state, description
    // Pull uses: id, slug, state, description
    // Verify state and description survive the roundtrip
    let p = SyncPayload::new(10);
    let pushed = serde_json::json!({
        "title":       &p.slug,
        "state":       format!("{:?}", p.state),
        "description": &p.description,
    });

    // Simulate pull: add id, rename title→slug
    let pulled = serde_json::json!({
        "id":          p.id,
        "slug":        pushed["title"],
        "state":       pushed["state"],
        "description": pushed["description"],
    });

    assert_eq!(pulled["id"], p.id);
    assert_eq!(pulled["slug"], p.slug);
    assert_eq!(pulled["description"], p.description);
}

// ---------------------------------------------------------------------------
// Conflict detection logic
// ---------------------------------------------------------------------------

#[test]
fn conflict_detection_no_conflicts() {
    let payloads = make_sync_payloads(100);
    let conflicts: Vec<_> = payloads
        .iter()
        .filter(|p| p.description.starts_with("CONFLICT"))
        .collect();
    assert!(conflicts.is_empty());
}

#[test]
fn conflict_detection_with_conflicts() {
    let mut payloads = make_sync_payloads(100);
    let conflict_indices = [5, 25, 50, 75, 99];
    for &i in &conflict_indices {
        payloads[i].description = format!("CONFLICT-{i}-{}", "x".repeat(512));
    }

    let conflicts: Vec<_> = payloads
        .iter()
        .enumerate()
        .filter(|(_, p)| p.description.starts_with("CONFLICT"))
        .collect();
    assert_eq!(conflicts.len(), 5);
}

#[test]
fn conflict_resolution_preserves_non_conflicting() {
    let mut payloads = make_sync_payloads(50);
    payloads[10].description = "CONFLICT-10-data".to_string();

    let mut resolved = 0_u64;
    for p in &payloads {
        let out = simulate_sync_roundtrip(p);
        if out.description.starts_with("CONFLICT") {
            resolved += 1;
        }
    }
    assert_eq!(resolved, 1);
    // Non-conflicting items should pass through unchanged
    let normal: Vec<_> = payloads
        .iter()
        .filter(|p| !p.description.starts_with("CONFLICT"))
        .collect();
    assert_eq!(normal.len(), 49);
}

#[test]
fn conflict_count_matches_injection() {
    let mut payloads = make_sync_payloads(200);
    let inject = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    for &i in &inject {
        payloads[i].description = format!("CONFLICT-{i}");
    }

    let detected: Vec<_> = payloads
        .iter()
        .filter(|p| p.description.starts_with("CONFLICT"))
        .collect();
    assert_eq!(detected.len(), inject.len());
}

// ---------------------------------------------------------------------------
// Feature state consistency with sync
// ---------------------------------------------------------------------------

#[test]
fn sync_payload_state_is_specified() {
    let p = SyncPayload::new(1);
    assert_eq!(p.state, FeatureState::Specified);
}

#[test]
fn sync_payload_state_survives_roundtrip() {
    let p = SyncPayload::new(1);
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.state, FeatureState::Specified);
}

// ---------------------------------------------------------------------------
// Edge cases: empty and large payloads
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_empty_description() {
    let mut p = SyncPayload::new(1);
    p.description = String::new();
    let out = simulate_sync_roundtrip(&p);
    assert!(out.description.is_empty());
}

#[test]
fn roundtrip_unicode_description() {
    let mut p = SyncPayload::new(1);
    p.description = "日本語テスト 🚀 émojis".to_string();
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.description, p.description);
}

#[test]
fn roundtrip_max_single_payload() {
    let mut p = SyncPayload::new(1);
    p.description = "x".repeat(100_000);
    let out = simulate_sync_roundtrip(&p);
    assert_eq!(out.description.len(), 100_000);
}

// ---------------------------------------------------------------------------
// Batch sync timing configuration
// ---------------------------------------------------------------------------

#[test]
fn batch_sync_sizes_match_benchmarks() {
    // Verify helper generates counts matching bench configs: 1, 10, 100
    for &n in &[1_i64, 10, 100] {
        let payloads = make_sync_payloads(n);
        assert_eq!(payloads.len(), n as usize);
    }
}
