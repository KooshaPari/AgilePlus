// SPDX-License-Identifier: MIT OR Apache-2.0
//! `AppState` use-case tests: request forwarding, claim lifecycle, dedup and
//! scan results, and the error paths that reach the caller.

use agileplus_triage::claim::{ClaimKind, ClaimReason, ClaimState};
use agileplus_triage::repo_introspect::RepoState;

use crate::dto::*;
use crate::test_mocks::InMemoryWpRepo;
use crate::use_cases::triage::AppState;

// ── helpers ─────────────────────────────────────────────────────────────────

fn make_item(id: &str, deps: Vec<&str>) -> PickedItem {
    PickedItem {
        wp_id: id.to_string(),
        title: format!("Title {id}"),
        state: "ready".to_string(),
        dependencies: deps.into_iter().map(String::from).collect(),
    }
}

fn claim_req(claim_id: &str, resource: &str, kind: ClaimKind) -> ClaimRequest {
    ClaimRequest {
        claim_id: claim_id.to_string(),
        resource: resource.to_string(),
        kind,
        agent_id: "agent-1".to_string(),
        ttl_seconds: 300,
        reason: ClaimReason::TaskRef(resource.to_string()),
    }
}

// ── pick ────────────────────────────────────────────────────────────────────

/// `pick` forwards every request field to the port unchanged (the use case
/// must not silently drop lane/category filtering).
#[test]
fn pick_forwards_agent_lane_category_and_limit_to_the_port() {
    let repo = InMemoryWpRepo::with_items(vec![
        make_item("WP01", vec![]),
        make_item("WP02", vec![]),
        make_item("WP03", vec![]),
    ]);
    let state = AppState::new(repo);

    let items = state
        .pick(&PickRequest {
            agent_id: "agent-7".to_string(),
            limit: 2,
            lane: Some("backend".to_string()),
            category: Some("bug".to_string()),
        })
        .unwrap();

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].wp_id, "WP01");
    let queries = state.wp_repo.recorded_queries();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].agent, "agent-7");
    assert_eq!(queries[0].lane.as_deref(), Some("backend"));
    assert_eq!(queries[0].category.as_deref(), Some("bug"));
    assert_eq!(queries[0].limit, 2);
}

/// A port failure is surfaced, not swallowed.
#[test]
fn pick_surfaces_port_failure() {
    let state = AppState::new(InMemoryWpRepo::default().failing_pick());
    let err = state
        .pick(&PickRequest {
            agent_id: "agent-1".to_string(),
            limit: 5,
            lane: None,
            category: None,
        })
        .unwrap_err();
    assert_eq!(err.to_string(), "wp store offline");
}

// ── claim lifecycle ─────────────────────────────────────────────────────────

/// The returned claim carries the request's agent, kind, ttl and reason.
#[test]
fn claim_returns_request_metadata() {
    let state = AppState::new(InMemoryWpRepo::default());

    let claim = state
        .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
        .unwrap();

    assert_eq!(claim.id, "c1");
    assert_eq!(claim.resource, "WP01");
    assert_eq!(claim.agent_id, "agent-1");
    assert_eq!(claim.kind, ClaimKind::Worktree);
    assert_eq!(claim.ttl_seconds, 300);
    assert_eq!(claim.state, ClaimState::Active);
    assert_eq!(claim.reason, ClaimReason::TaskRef("WP01".to_string()));
    assert!(claim.last_heartbeat >= claim.created_at);
}

/// Resources are keyed by `(kind, resource)`: the same name under a different
/// kind is a different resource and can be held concurrently.
#[test]
fn claim_same_resource_with_different_kind_is_allowed() {
    let state = AppState::new(InMemoryWpRepo::default());

    state
        .claim(&claim_req("c1", "shared-name", ClaimKind::Repo))
        .unwrap();
    let second = state.claim(&claim_req("c2", "shared-name", ClaimKind::Worktree));
    assert!(second.is_ok(), "different kind must not collide");

    // …while the same kind under a new claim id is rejected.
    let conflict = state.claim(&claim_req("c3", "shared-name", ClaimKind::Worktree));
    assert_eq!(
        conflict.unwrap_err().to_string(),
        "resource already claimed"
    );
}

/// A zero TTL claim is still `Active` until someone reaps it: the use case
/// does not silently drop expired claims.
#[test]
fn claim_with_zero_ttl_is_active_until_reaped() {
    let state = AppState::new(InMemoryWpRepo::default());
    let mut req = claim_req("c1", "WP01", ClaimKind::Worktree);
    req.ttl_seconds = 0;

    let claim = state.claim(&req).unwrap();
    assert_eq!(claim.ttl_seconds, 0);
    assert_eq!(claim.state, ClaimState::Active);
    assert_eq!(state.claim_store.lock().unwrap().active().len(), 1);
}

/// `done` releases the claim and marks the work package done.
#[test]
fn done_releases_claim_and_marks_wp_done() {
    let repo = InMemoryWpRepo::with_items(vec![make_item("WP01", vec![])]);
    let mut state = AppState::new(repo);
    state
        .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
        .unwrap();

    let ok = state
        .done(&DoneRequest {
            claim_id: "c1".to_string(),
            wp_id: "WP01".to_string(),
            result: Some("completed".to_string()),
        })
        .unwrap();

    assert!(ok);
    assert_eq!(state.wp_repo.done_ids(), vec!["WP01"]);
    assert!(
        state.claim_store.lock().unwrap().active().is_empty(),
        "the claim must be released"
    );
}

/// When the repo rejects `mark_done`, the error reaches the caller — but the
/// claim has already been released.
#[test]
fn done_surfaces_repo_failure_after_releasing_claim() {
    let repo = InMemoryWpRepo::with_items(vec![make_item("WP01", vec![])]).failing_mark_done();
    let mut state = AppState::new(repo);
    state
        .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
        .unwrap();

    let err = state
        .done(&DoneRequest {
            claim_id: "c1".to_string(),
            wp_id: "WP01".to_string(),
            result: None,
        })
        .unwrap_err();

    assert_eq!(err.to_string(), "wp store offline");
    assert!(state.wp_repo.done_ids().is_empty());
    assert!(
        state.claim_store.lock().unwrap().active().is_empty(),
        "release happens before mark_done"
    );
}

/// `done` releases exactly the claim named in the request. Other active
/// claims survive, so only the finished resource becomes claimable again.
#[test]
fn done_releases_only_the_requested_claim() {
    let repo =
        InMemoryWpRepo::with_items(vec![make_item("WP01", vec![]), make_item("WP02", vec![])]);
    let mut state = AppState::new(repo);
    state
        .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
        .unwrap();
    state
        .claim(&claim_req("c2", "WP02", ClaimKind::Branch))
        .unwrap();

    let ok = state
        .done(&DoneRequest {
            claim_id: "c1".to_string(),
            wp_id: "WP01".to_string(),
            result: None,
        })
        .unwrap();

    assert!(ok);
    assert_eq!(state.wp_repo.done_ids(), vec!["WP01"]);

    let active = state.claim_store.lock().unwrap().active();
    assert_eq!(active.len(), 1, "the untouched claim stays held");
    assert_eq!(active[0].id, "c2");
    assert_eq!(active[0].resource, "WP02");

    // WP01 is free again; WP02 is still taken.
    state
        .claim(&claim_req("c3", "WP01", ClaimKind::Worktree))
        .expect("the finished resource is claimable again");
    assert_eq!(
        state
            .claim(&claim_req("c4", "WP02", ClaimKind::Branch))
            .unwrap_err()
            .to_string(),
        "resource already claimed"
    );
}

/// Lock poisoning is reported as a clear error from every entry point rather
/// than panicking the caller.
#[test]
fn poisoned_claim_store_is_reported_by_every_entry_point() {
    let mut state = AppState::new(InMemoryWpRepo::with_items(vec![make_item("WP01", vec![])]));

    // Poison the mutex on purpose: hold the guard, then unwind.
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = state.claim_store.lock().unwrap();
        panic!("poison the claim store on purpose");
    }));
    assert!(poisoned.is_err(), "the helper must have unwound");

    let expected = "claim store lock poisoned";
    assert_eq!(
        state
            .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
            .unwrap_err()
            .to_string(),
        expected
    );
    assert_eq!(
        state
            .heartbeat(&HeartbeatRequest {
                claim_id: "c1".to_string(),
            })
            .unwrap_err()
            .to_string(),
        expected
    );
    assert_eq!(
        state
            .release(&ReleaseRequest {
                claim_id: "c1".to_string(),
            })
            .unwrap_err()
            .to_string(),
        expected
    );
    assert_eq!(
        state
            .done(&DoneRequest {
                claim_id: "c1".to_string(),
                wp_id: "WP01".to_string(),
                result: None,
            })
            .unwrap_err()
            .to_string(),
        expected
    );
    assert_eq!(
        state
            .where_am_i(&WhereRequest {
                cwd: "/tmp".to_string(),
            })
            .unwrap_err()
            .to_string(),
        expected
    );
}

// ── dedup ───────────────────────────────────────────────────────────────────

/// Near-identical descriptions are reported as one candidate pair carrying the
/// input ids and a perfect similarity breakdown.
#[test]
fn dedup_reports_near_identical_descriptions() {
    let state = AppState::new(InMemoryWpRepo::default());

    let candidates = state
        .dedup(&DedupRequest {
            items: vec![
                ("WP01".to_string(), "Add login page".to_string()),
                ("WP02".to_string(), "Add login page".to_string()),
                ("WP03".to_string(), "Rewrite the billing engine".to_string()),
            ],
            threshold: 0.5,
        })
        .unwrap();

    assert_eq!(
        candidates.len(),
        1,
        "only the identical pair is a candidate"
    );
    let c = &candidates[0];
    assert_eq!((c.a_id.as_str(), c.b_id.as_str()), ("WP01", "WP02"));
    assert!(
        (c.hybrid_score - 1.0).abs() < 1e-9,
        "got {}",
        c.hybrid_score
    );
    assert_eq!(c.token_jaccard, 1.0);
    assert_eq!(c.fuzzy_ratio, 1.0);
    assert_eq!(c.simhash_distance, 0);
}

/// Candidates come back ranked by descending hybrid score.
#[test]
fn dedup_ranks_candidates_by_descending_score() {
    let state = AppState::new(InMemoryWpRepo::default());

    let candidates = state
        .dedup(&DedupRequest {
            items: vec![
                ("WP01".to_string(), "Add login page".to_string()),
                ("WP02".to_string(), "Add login page".to_string()),
                ("WP03".to_string(), "Add login screen".to_string()),
            ],
            threshold: 0.3,
        })
        .unwrap();

    assert_eq!(candidates.len(), 3, "every pair clears the low threshold");
    assert_eq!(
        (candidates[0].a_id.as_str(), candidates[0].b_id.as_str()),
        ("WP01", "WP02"),
        "the identical pair ranks first"
    );
    assert!(
        candidates
            .windows(2)
            .all(|w| w[0].hybrid_score >= w[1].hybrid_score),
        "scores must be non-increasing: {:?}",
        candidates
            .iter()
            .map(|c| c.hybrid_score)
            .collect::<Vec<_>>()
    );
}

/// A threshold above the maximum attainable score yields no candidates.
#[test]
fn dedup_threshold_above_max_similarity_returns_nothing() {
    let state = AppState::new(InMemoryWpRepo::default());

    let candidates = state
        .dedup(&DedupRequest {
            items: vec![
                ("WP01".to_string(), "Add login page".to_string()),
                ("WP02".to_string(), "Add login page".to_string()),
            ],
            threshold: 1.01,
        })
        .unwrap();

    assert!(candidates.is_empty());
}

// ── scan ────────────────────────────────────────────────────────────────────

/// Only directories are inspected; plain files in the roots list are skipped.
#[test]
fn scan_reports_only_directories() {
    let state = AppState::new(InMemoryWpRepo::default());
    let tmp = tempfile::tempdir().unwrap();
    let file = tmp.path().join("not-a-repo.txt");
    std::fs::write(&file, "hi").unwrap();

    let infos = state
        .scan(&ScanRequest {
            roots: vec![
                tmp.path().to_string_lossy().to_string(),
                file.to_string_lossy().to_string(),
            ],
            max_depth: None,
        })
        .unwrap();

    assert_eq!(infos.len(), 1, "the file is skipped");
    assert_eq!(infos[0].path, tmp.path().to_string_lossy());
}

/// A plain directory without `.git` and without source files is rated as
/// `NoGit`; this is the same classification used by the CLI triage report.
#[test]
fn scan_marks_empty_directory_as_no_git() {
    let state = AppState::new(InMemoryWpRepo::default());
    let tmp = tempfile::tempdir().unwrap();

    let infos = state
        .scan(&ScanRequest {
            roots: vec![tmp.path().to_string_lossy().to_string()],
            max_depth: None,
        })
        .unwrap();

    assert_eq!(infos.len(), 1);
    assert_eq!(infos[0].state, RepoState::NoGit);
    assert_eq!(infos[0].hygiene_score, 30);
    assert!(infos[0].current_branch.is_none());
}

// ── topology ────────────────────────────────────────────────────────────────

/// A cycle is reported through the use case: the caller gets the partial
/// order plus the members of the cycle, not a silent truncation.
#[test]
fn topology_reports_cycle_with_partial_order() {
    let repo = InMemoryWpRepo::with_items(vec![
        make_item("WP01", vec!["WP02"]),
        make_item("WP02", vec!["WP01"]),
    ]);
    let state = AppState::new(repo);

    let report = state.topology(&TopologyRequest { root_wp: None }).unwrap();

    let cycle = report.topo.cycle.expect("cycle must be reported");
    assert_eq!(cycle.len(), 2);
    assert!(report.topo.order.len() < 2, "order is partial");
    let all: Vec<&String> = report.layers.iter().flatten().collect();
    assert_eq!(all.len(), 2, "layers still describe both nodes");
}

/// A dependency-only node (referenced but never listed) still appears in the
/// topology as a prerequisite.
#[test]
fn topology_includes_dependency_only_nodes() {
    let repo = InMemoryWpRepo::with_items(vec![make_item("WP01", vec!["WP00"])]);
    let state = AppState::new(repo);

    let report = state.topology(&TopologyRequest { root_wp: None }).unwrap();

    assert_eq!(report.topo.order, vec!["WP01", "WP00"]);
    assert_eq!(report.layers.len(), 2);
    assert_eq!(report.layers[0], vec!["WP00"]);
    assert_eq!(report.layers[1], vec!["WP01"]);
}

/// An export failure is surfaced with its own message.
#[test]
fn topology_surfaces_export_failure() {
    let state = AppState::new(InMemoryWpRepo::default().failing_export());
    let err = state
        .topology(&TopologyRequest { root_wp: None })
        .unwrap_err();
    assert_eq!(err.to_string(), "wp store offline");
}

// ── where_am_i ──────────────────────────────────────────────────────────────

/// `where_am_i` queries the port for the anonymous agent, asking for the next
/// five pickable items, and reports the active claims.
#[test]
fn where_am_i_reports_claims_and_forwards_the_anonymous_query() {
    let repo = InMemoryWpRepo::with_items(vec![make_item("WP01", vec![])]);
    let state = AppState::new(repo);
    let tmp = tempfile::tempdir().unwrap();
    state
        .claim(&claim_req("c1", "WP01", ClaimKind::Worktree))
        .unwrap();

    let resp = state
        .where_am_i(&WhereRequest {
            cwd: tmp.path().to_string_lossy().to_string(),
        })
        .unwrap();

    assert_eq!(resp.active_claims.len(), 1);
    assert_eq!(resp.active_claims[0].resource, "WP01");
    assert!(resp.lane.is_none());
    assert!(resp.category.is_none());
    assert_eq!(resp.next_pickable.len(), 1);

    let queries = state.wp_repo.recorded_queries();
    assert_eq!(queries.len(), 1);
    assert_eq!(queries[0].agent, "anonymous");
    assert!(queries[0].lane.is_none());
    assert!(queries[0].category.is_none());
    assert_eq!(queries[0].limit, 5);
}

/// A pick failure is surfaced even though the repo snapshot itself succeeded.
#[test]
fn where_am_i_surfaces_pick_failure() {
    let state = AppState::new(InMemoryWpRepo::default().failing_pick());
    let tmp = tempfile::tempdir().unwrap();

    let err = state
        .where_am_i(&WhereRequest {
            cwd: tmp.path().to_string_lossy().to_string(),
        })
        .unwrap_err();

    assert_eq!(err.to_string(), "wp store offline");
}
