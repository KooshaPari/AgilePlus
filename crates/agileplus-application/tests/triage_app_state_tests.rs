//! Public-boundary tests for `use_cases::triage::AppState`.
//!
//! These drive the crate the way a CLI subcommand does: through `AppState` and
//! the public `dto` request types. The in-crate unit tests use
//! `crate::test_mocks::InMemoryWpRepo`, which is `#[cfg(test)]` and therefore
//! invisible from an integration test — so this file supplies its own
//! `WpRepository` double and observes real outcomes: which query was forwarded
//! to the port, whether a claim was actually released, and what the graph
//! queries returned.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use agileplus_application::dto::{
    ClaimReason, ClaimRequest, DedupRequest, DoneRequest, HeartbeatRequest, PickRequest,
    PickedItem, ReleaseRequest, ScanRequest, TopologyRequest, WhereRequest,
};
use agileplus_application::use_cases::triage::{AppState, WpRepository};
use agileplus_triage::claim::{ClaimKind, ClaimState};

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn item(id: &str, deps: &[&str]) -> PickedItem {
    PickedItem {
        wp_id: id.to_string(),
        title: format!("Title {id}"),
        state: "ready".to_string(),
        dependencies: deps.iter().map(|d| d.to_string()).collect(),
    }
}

fn claim_req(id: &str, resource: &str) -> ClaimRequest {
    ClaimRequest {
        claim_id: id.to_string(),
        resource: resource.to_string(),
        kind: ClaimKind::Worktree,
        agent_id: "agent-1".to_string(),
        ttl_seconds: 300,
        reason: ClaimReason::default(),
    }
}

// ── WpRepository double ──────────────────────────────────────────────────────

/// `(agent, lane, category, limit)` as observed by the port.
type QueryLog = Vec<(String, Option<String>, Option<String>, usize)>;

#[derive(Default)]
struct RecordingWpRepo {
    items: Vec<PickedItem>,
    queries: Mutex<QueryLog>,
    done: Mutex<Vec<String>>,
    fail_pick: AtomicBool,
    fail_export: AtomicBool,
    fail_mark_done: AtomicBool,
}

impl RecordingWpRepo {
    fn with_items(items: Vec<PickedItem>) -> Self {
        Self {
            items,
            ..Self::default()
        }
    }

    fn failing_pick(self) -> Self {
        self.fail_pick.store(true, Ordering::SeqCst);
        self
    }

    fn failing_export(self) -> Self {
        self.fail_export.store(true, Ordering::SeqCst);
        self
    }

    fn failing_mark_done(self) -> Self {
        self.fail_mark_done.store(true, Ordering::SeqCst);
        self
    }

    fn queries(&self) -> QueryLog {
        self.queries.lock().unwrap().clone()
    }

    fn done_ids(&self) -> Vec<String> {
        self.done.lock().unwrap().clone()
    }
}

impl WpRepository for RecordingWpRepo {
    fn list_pickable(
        &self,
        agent: &str,
        lane: Option<&str>,
        category: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<PickedItem>> {
        if self.fail_pick.load(Ordering::SeqCst) {
            anyhow::bail!("wp store offline");
        }
        self.queries.lock().unwrap().push((
            agent.to_string(),
            lane.map(str::to_string),
            category.map(str::to_string),
            limit,
        ));
        Ok(self.items.iter().take(limit).cloned().collect())
    }

    fn all_for_export(&self, _with_side: bool) -> anyhow::Result<Vec<PickedItem>> {
        if self.fail_export.load(Ordering::SeqCst) {
            anyhow::bail!("wp export offline");
        }
        Ok(self.items.clone())
    }

    fn add_dependency(&mut self, _from: &str, _to: &str) -> anyhow::Result<()> {
        Ok(())
    }

    fn mark_done(&mut self, wp_id: &str) -> anyhow::Result<()> {
        if self.fail_mark_done.load(Ordering::SeqCst) {
            anyhow::bail!("wp store offline");
        }
        self.done.lock().unwrap().push(wp_id.to_string());
        Ok(())
    }

    fn claim_count(&self) -> usize {
        self.done.lock().unwrap().len()
    }

    fn wp_count(&self) -> usize {
        self.items.len()
    }

    fn stage_count(&self) -> usize {
        3
    }
}

// ── pick ─────────────────────────────────────────────────────────────────────

#[test]
fn pick_forwards_agent_lane_category_and_limit_to_the_port() {
    let repo = RecordingWpRepo::with_items(vec![item("WP01", &[]), item("WP02", &[])]);
    let state = AppState::new(repo);

    let picked = state
        .pick(&PickRequest {
            agent_id: "agent-7".into(),
            limit: 1,
            lane: Some("backend".into()),
            category: Some("bug".into()),
        })
        .unwrap();

    assert_eq!(picked.len(), 1, "limit is honoured by the port call");
    assert_eq!(
        state.wp_repo.queries(),
        vec![(
            "agent-7".to_string(),
            Some("backend".to_string()),
            Some("bug".to_string()),
            1
        )]
    );
}

#[test]
fn pick_surfaces_repository_failure() {
    let state = AppState::new(RecordingWpRepo::default().failing_pick());
    let err = state
        .pick(&PickRequest {
            agent_id: "a".into(),
            limit: 5,
            lane: None,
            category: None,
        })
        .unwrap_err();
    assert!(err.to_string().contains("wp store offline"));
}

// ── claim lifecycle ──────────────────────────────────────────────────────────

#[test]
fn claim_then_heartbeat_then_release_round_trips() {
    let state = AppState::new(RecordingWpRepo::default());

    let claim = state.claim(&claim_req("c1", "worktree-auth")).unwrap();
    assert_eq!(claim.id, "c1");
    assert_eq!(claim.state, ClaimState::Active);

    assert!(
        state
            .heartbeat(&HeartbeatRequest {
                claim_id: "c1".into()
            })
            .unwrap()
    );

    assert!(
        state
            .release(&ReleaseRequest {
                claim_id: "c1".into()
            })
            .unwrap()
    );
    assert!(
        !state
            .heartbeat(&HeartbeatRequest {
                claim_id: "c1".into()
            })
            .unwrap(),
        "a released claim can no longer be heartbeated"
    );
    assert!(state.claim_store.lock().unwrap().active().is_empty());
}

#[test]
fn claim_is_rejected_when_the_resource_is_held_by_another_claim() {
    let state = AppState::new(RecordingWpRepo::default());
    state.claim(&claim_req("c1", "worktree-auth")).unwrap();

    let err = state.claim(&claim_req("c2", "worktree-auth")).unwrap_err();
    assert!(err.to_string().contains("already claimed"), "got: {err}");
    assert_eq!(state.claim_store.lock().unwrap().active().len(), 1);
}

// ── done ─────────────────────────────────────────────────────────────────────

#[test]
fn done_releases_the_claim_and_marks_the_wp_done() {
    let repo = RecordingWpRepo::with_items(vec![item("WP01", &[])]);
    let mut state = AppState::new(repo);
    state.claim(&claim_req("c1", "WP01")).unwrap();

    let ok = state
        .done(&DoneRequest {
            claim_id: "c1".into(),
            wp_id: "WP01".into(),
            result: Some("shipped".into()),
        })
        .unwrap();

    assert!(ok);
    assert_eq!(state.wp_repo.done_ids(), vec!["WP01".to_string()]);
    assert!(
        state.claim_store.lock().unwrap().active().is_empty(),
        "done must release the claim it was handed"
    );
}

#[test]
fn done_surfaces_a_mark_done_failure_but_still_releases_the_claim() {
    let repo = RecordingWpRepo::with_items(vec![item("WP01", &[])]).failing_mark_done();
    let mut state = AppState::new(repo);
    state.claim(&claim_req("c1", "WP01")).unwrap();

    let err = state
        .done(&DoneRequest {
            claim_id: "c1".into(),
            wp_id: "WP01".into(),
            result: None,
        })
        .unwrap_err();

    assert!(err.to_string().contains("wp store offline"));
    assert!(
        state.claim_store.lock().unwrap().active().is_empty(),
        "the release precedes the failing write"
    );
}

// ── dedup / scan / topology / where ──────────────────────────────────────────

#[test]
fn dedup_reports_near_duplicate_work_descriptions() {
    let state = AppState::new(RecordingWpRepo::default());
    let candidates = state
        .dedup(&DedupRequest {
            items: vec![
                ("wp-1".into(), "add user login endpoint".into()),
                ("wp-2".into(), "add user login endpoints".into()),
                ("wp-3".into(), "unrelated billing work".into()),
            ],
            threshold: 0.6,
        })
        .unwrap();

    assert_eq!(
        candidates.len(),
        1,
        "only the near-identical pair qualifies"
    );
    assert_eq!(candidates[0].a_id, "wp-1");
    assert_eq!(candidates[0].b_id, "wp-2");
    assert!(candidates[0].hybrid_score >= 0.6);
}

#[test]
fn scan_reports_directories_and_skips_missing_paths() {
    let state = AppState::new(RecordingWpRepo::default());
    let tmp = tempfile::tempdir().unwrap();

    let infos = state
        .scan(&ScanRequest {
            roots: vec![
                tmp.path().to_string_lossy().to_string(),
                "/definitely/not/here/12345".to_string(),
            ],
            max_depth: None,
        })
        .unwrap();

    assert_eq!(infos.len(), 1, "only the real directory is inspected");
}

#[test]
fn topology_orders_dependencies_and_groups_parallel_layers() {
    let repo = RecordingWpRepo::with_items(vec![
        item("WP01", &[]),
        item("WP02", &["WP01"]),
        item("WP03", &["WP01"]),
    ]);
    let state = AppState::new(repo);

    let report = state.topology(&TopologyRequest { root_wp: None }).unwrap();

    assert_eq!(report.topo.order.len(), 3);
    assert!(report.topo.cycle.is_none());
    let pos = |n: &str| report.topo.order.iter().position(|x| x == n).unwrap();
    // Dependents are emitted before their prerequisites: WP02/WP03 need WP01,
    // so WP01 must come last.
    assert!(pos("WP02") < pos("WP01"));
    assert!(pos("WP03") < pos("WP01"));

    let flattened: Vec<&String> = report.layers.iter().flatten().collect();
    assert_eq!(flattened.len(), 3);
    assert!(
        report
            .layers
            .iter()
            .any(|l| l.iter().any(|n| n == "WP02") && l.iter().any(|n| n == "WP03")),
        "WP02 and WP03 share a dependency depth and must be in one layer"
    );
}

#[test]
fn topology_reports_a_cycle_instead_of_silently_truncating() {
    let repo = RecordingWpRepo::with_items(vec![
        item("A", &["B"]),
        item("B", &["C"]),
        item("C", &["A"]),
    ]);
    let state = AppState::new(repo);

    let report = state.topology(&TopologyRequest { root_wp: None }).unwrap();

    let cycle = report
        .topo
        .cycle
        .expect("a cyclic graph must report a cycle");
    assert_eq!(cycle.len(), 3);
}

#[test]
fn topology_surfaces_an_export_failure() {
    let state = AppState::new(RecordingWpRepo::default().failing_export());
    let err = state
        .topology(&TopologyRequest { root_wp: None })
        .unwrap_err();
    assert!(err.to_string().contains("wp export offline"));
}

#[test]
fn where_am_i_snapshots_the_repo_and_pickable_items() {
    let repo = RecordingWpRepo::with_items(vec![item("WP01", &[])]);
    let state = AppState::new(repo);
    let tmp = tempfile::tempdir().unwrap();

    let resp = state
        .where_am_i(&WhereRequest {
            cwd: tmp.path().to_string_lossy().to_string(),
        })
        .unwrap();

    assert!(resp.repo.is_some(), "an existing directory is inspected");
    assert_eq!(resp.next_pickable.len(), 1);
    assert_eq!(
        state.wp_repo.queries(),
        vec![("anonymous".to_string(), None, None, 5)],
        "where_am_i queries anonymously with lane/category undiscovered"
    );
}

#[test]
fn where_am_i_with_a_missing_cwd_reports_no_repo() {
    let state = AppState::new(RecordingWpRepo::default());
    let resp = state
        .where_am_i(&WhereRequest {
            cwd: "/definitely/not/here/12345".into(),
        })
        .unwrap();
    assert!(resp.repo.is_none());
}
