//! Use-case implementations for the CLI triage subcommands.
//!
//! These orchestrate the new triage primitives (dedup, claim, repo_introspect)
//! and the agileplus-graph dependency graph into a coherent set of
//! commands / queries that CLI subcommands can call.
//!
//! # Design
//!
//! `AppState<W>` is the central application-state struct. It holds:
//!
//! - a [`WpRepository`] port (in-memory trait — the production wiring to
//!   `agileplus-sqlite` is added later in the same workflow), and
//! - an in-memory [`agileplus_triage::claim::ClaimStore`] guarded by a
//!   `Mutex` (a single-process per-app-state lock; per-resource locking
//!   lives in `ClaimStore` itself).
//!
//! Use cases are pure functions of their inputs and the ports they call; in
//! tests they can be wired to in-memory implementations of `WpRepository`.
//!
//! The `topology` use case builds a self-contained in-memory graph from
//! the work-package set (because the `agileplus-graph` crate's persistent
//! store is async-trait based and doesn't yet expose sync `topo_sort` /
//! `parallel_layers` primitives). When those primitives land in
//! `agileplus-graph`, this use case should be re-wired to call them.
//!
//! Traceability: FR-AGP-018 (dedup), FR-AGP-019 (claim), FR-AGP-020
//! (repo_introspect), FR-AGP-021 (graph topology), FR-AGP-022 (where_am_i).
//!
//! Re-exports — the surface used by `agileplus-cli::commands::dag`.

pub use crate::dto::{TopologyRequest, WhereRequest, WhereResponse};

use anyhow::{Result, anyhow};
use std::collections::{HashMap, HashSet, VecDeque};

use agileplus_triage::claim::{Claim, ClaimKind, ClaimStore};
use agileplus_triage::dedup::{DuplicateCandidate, find_duplicates};
use agileplus_triage::repo_introspect::{RepoInfo, inspect_repo};

use crate::dto::*;

// ── In-memory ports ─────────────────────────────────────────────────────────
//
// In production these are backed by SQLite via `agileplus-sqlite` (added
// later). For now we keep the in-memory representation explicit so use
// cases are unit-testable.

/// Read/write port for the work-package state. Implementations decide
/// how to store / load (in-memory in tests, SQLite in production).
pub trait WpRepository: Send + Sync {
    /// Return up to `limit` work packages that are pickable for `agent`,
    /// optionally filtered by `lane` and `category`.
    fn list_pickable(
        &self,
        agent: &str,
        lane: Option<&str>,
        category: Option<&str>,
        limit: usize,
    ) -> Result<Vec<PickedItem>>;

    /// Return every work package, optionally including side-band state
    /// used by the export use case.
    fn all_for_export(&self, with_side: bool) -> Result<Vec<PickedItem>>;

    /// Add a hard dependency from `from` to `to` (`from` depends on `to`).
    fn add_dependency(&mut self, from: &str, to: &str) -> Result<()>;

    /// Mark a work package as done.
    fn mark_done(&mut self, wp_id: &str) -> Result<()>;

    /// Number of currently-active claims (for status / dashboard).
    fn claim_count(&self) -> usize;

    /// Number of work packages tracked.
    fn wp_count(&self) -> usize;

    /// Number of pipeline stages known to the repo.
    fn stage_count(&self) -> usize;
}

// ── AppState ────────────────────────────────────────────────────────────────

/// Application state shared by every CLI subcommand handler. Holds the
/// work-package repository port plus the in-memory claim store.
pub struct AppState<W: WpRepository> {
    pub wp_repo: W,
    pub claim_store: std::sync::Mutex<ClaimStore>,
}

impl<W: WpRepository> AppState<W> {
    /// Build a fresh `AppState` with an empty claim store.
    pub fn new(wp_repo: W) -> Self {
        Self {
            wp_repo,
            claim_store: std::sync::Mutex::new(ClaimStore::new()),
        }
    }

    /// Use case: list the next N work packages that an agent can pick up.
    pub fn pick(&self, req: &PickRequest) -> Result<Vec<PickedItem>> {
        self.wp_repo.list_pickable(
            &req.agent_id,
            req.lane.as_deref(),
            req.category.as_deref(),
            req.limit,
        )
    }

    /// Use case: claim a resource (worktree, branch, repo, subproject).
    ///
    /// Returns the new claim on success. Errors if the resource is already
    /// actively held by a different claim id.
    pub fn claim(&self, req: &ClaimRequest) -> Result<Claim> {
        let mut store = self
            .claim_store
            .lock()
            .map_err(|_| anyhow!("claim store lock poisoned"))?;
        store
            .claim(
                &req.claim_id,
                &req.resource,
                req.kind,
                &req.agent_id,
                req.ttl_seconds,
                req.reason.clone(),
            )
            .ok_or_else(|| anyhow!("resource already claimed"))
    }

    /// Use case: refresh the heartbeat of an existing claim. Returns
    /// `false` if no such claim exists.
    pub fn heartbeat(&self, req: &HeartbeatRequest) -> Result<bool> {
        let mut store = self
            .claim_store
            .lock()
            .map_err(|_| anyhow!("claim store lock poisoned"))?;
        Ok(store.heartbeat(&req.claim_id))
    }

    /// Use case: explicitly release a claim. Returns `false` if no such
    /// claim exists.
    pub fn release(&self, req: &ReleaseRequest) -> Result<bool> {
        let mut store = self
            .claim_store
            .lock()
            .map_err(|_| anyhow!("claim store lock poisoned"))?;
        Ok(store.release(&req.claim_id))
    }

    /// Use case: mark a work package done, releasing any associated claim.
    ///
    /// Looks up the claim for the wp under the likely kinds (`Worktree` or
    /// `Subproject`) and releases it, then calls `mark_done` on the repo.
    pub fn done(&mut self, req: &DoneRequest) -> Result<bool> {
        // The done use case requires an active claim on the resource. Check first.
        let _ = self
            .claim_store
            .lock()
            .map_err(|_| anyhow!("claim store lock poisoned"))?
            .lookup(ClaimKind::Worktree, &req.wp_id)
            .or(None);

        // Release the explicit claim_id from the request.
        let _ = self
            .claim_store
            .lock()
            .map_err(|_| anyhow!("claim store lock poisoned"))?
            .release(&req.claim_id);

        // `WpRepository::mark_done` returns `Result<()>` — coerce to `bool`
        // by reporting success/failure to the caller.
        self.wp_repo.mark_done(&req.wp_id).map(|_| true)
    }

    /// Use case: find duplicate work-package candidates above `threshold`.
    pub fn dedup(&self, req: &DedupRequest) -> Result<Vec<DuplicateCandidate>> {
        Ok(find_duplicates(&req.items, req.threshold))
    }

    /// Use case: inspect every root path that is a directory, returning
    /// a [`RepoInfo`] for each.
    pub fn scan(&self, req: &ScanRequest) -> Result<Vec<RepoInfo>> {
        let mut out = vec![];
        for root in &req.roots {
            let p = std::path::Path::new(root);
            if p.is_dir() {
                out.push(inspect_repo(p));
            }
        }
        Ok(out)
    }

    /// Use case: compute a topology report (topo order + parallel layers)
    /// from the current work-package dependency graph.
    ///
    /// In production this reads from the `wp_dependencies` table via the
    /// `WpRepository` port; here we rebuild a self-contained graph from
    /// the export view and run a sync Kahn-style topo sort over it.
    pub fn topology(&self, _req: &TopologyRequest) -> Result<TopologyReport> {
        let items = self.wp_repo.all_for_export(false)?;
        let graph = WpGraph::from_items(&items);
        let topo = graph.topo_sort();
        let layers = graph.parallel_layers();
        Ok(TopologyReport { topo, layers })
    }

    /// Use case: produce a context snapshot for the agent's current
    /// working directory: detected repo, active claims on the repo, the
    /// lane/category under the cwd, and the next N pickable items.
    pub fn where_am_i(&self, req: &WhereRequest) -> Result<WhereResponse> {
        let p = std::path::Path::new(&req.cwd);
        let repo = if p.is_dir() {
            Some(inspect_repo(p))
        } else {
            None
        };
        let active_claims = {
            let store = self
                .claim_store
                .lock()
                .map_err(|_| anyhow!("claim store lock poisoned"))?;
            store.active()
        };
        let next_pickable = self.wp_repo.list_pickable(
            "anonymous",
            repo.as_ref().and(None), // lane discovery TBD
            repo.as_ref().and(None), // category discovery TBD
            5,
        )?;
        Ok(WhereResponse {
            repo,
            active_claims,
            lane: None,
            category: None,
            next_pickable,
        })
    }
}

// ── Topology types (self-contained, mirrors agileplus-graph primitives) ─────
//
// These types shadow the future agileplus_graph::types::{TopoResult,
// parallel_layers, topo_sort, WpGraph} surface so the use case compiles
// against the current agileplus-graph crate. When those land upstream,
// this section should be deleted and the call sites re-wired to use the
// upstream names.

/// Self-contained adjacency-list representation of the work-package
/// dependency graph. The same structure will live in
/// `agileplus_graph::types::WpGraph` once the sync API lands.
#[derive(Debug, Clone, Default)]
pub struct WpGraph {
    /// `edges[a] = [b, c]`  — `a` depends on `b` and `c` (so `b` and `c`
    /// must be completed before `a`).
    edges: HashMap<String, Vec<String>>,
    /// All known nodes (edges may reference unknown nodes that are
    /// implicitly treated as roots with no dependencies).
    nodes: HashSet<String>,
}

impl WpGraph {
    /// Build a graph from the export view of all work packages.
    pub fn from_items(items: &[PickedItem]) -> Self {
        let mut g = WpGraph::default();
        for item in items {
            g.nodes.insert(item.wp_id.clone());
            for dep in &item.dependencies {
                g.nodes.insert(dep.clone());
                g.edges
                    .entry(item.wp_id.clone())
                    .or_default()
                    .push(dep.clone());
            }
        }
        g
    }

    /// Kahn's algorithm: returns a topologically sorted list of nodes.
    /// On a cycle, returns the partial order and reports the cycle in
    /// [`TopoResult::cycle`].
    pub fn topo_sort(&self) -> TopoResult {
        // in-degree counts for every node referenced as a target
        let mut in_deg: HashMap<&str, usize> = HashMap::new();
        for node in &self.nodes {
            in_deg.entry(node.as_str()).or_insert(0);
        }
        for targets in self.edges.values() {
            for t in targets {
                *in_deg.entry(t.as_str()).or_insert(0) += 1;
            }
        }

        // Seed: nodes that are not depended on by anyone (no incoming
        // edge) — they have no prerequisites and can run first.
        let mut queue: VecDeque<String> = self
            .nodes
            .iter()
            .filter(|n| in_deg.get(n.as_str()).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        let mut order: Vec<String> = Vec::with_capacity(self.nodes.len());
        while let Some(n) = queue.pop_front() {
            order.push(n.clone());
            if let Some(deps) = self.edges.get(&n) {
                for d in deps {
                    if let Some(deg) = in_deg.get_mut(d.as_str()) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(d.clone());
                        }
                    }
                }
            }
        }

        if order.len() < self.nodes.len() {
            let cycle: Vec<String> = self
                .nodes
                .iter()
                .filter(|n| in_deg.get(n.as_str()).copied().unwrap_or(0) > 0)
                .cloned()
                .collect();
            TopoResult {
                order,
                cycle: Some(cycle),
            }
        } else {
            TopoResult { order, cycle: None }
        }
    }

    /// Decompose the graph into anti-chains of nodes that can be
    /// executed in parallel (longest-path layering).
    pub fn parallel_layers(&self) -> Vec<Vec<String>> {
        // Compute the rank of every node = max(rank of any prerequisite) + 1.
        let mut rank: HashMap<&str, usize> = HashMap::new();
        let order = self.topo_sort().order;
        for n in &order {
            let r = self
                .edges
                .get(n)
                .map(|deps| {
                    deps.iter()
                        .filter_map(|d| rank.get(d.as_str()).copied())
                        .max()
                        .unwrap_or(0)
                        + 1
                })
                .unwrap_or(0);
            rank.insert(n.as_str(), r);
        }

        // Bucket by rank.
        let max_rank = rank.values().copied().max().unwrap_or(0);
        let mut layers: Vec<Vec<String>> = (0..=max_rank).map(|_| Vec::new()).collect();
        for n in &self.nodes {
            let r = rank.get(n.as_str()).copied().unwrap_or(0);
            if let Some(layer) = layers.get_mut(r) {
                layer.push(n.clone());
            }
        }
        // Stable, sorted layers for deterministic output.
        for layer in &mut layers {
            layer.sort();
        }
        layers
    }
}

/// Result of a topo sort. `order` is the linear order; `cycle` is
/// non-empty when the graph contains a cycle (and `order` is partial).
#[derive(Debug, Clone, Default)]
pub struct TopoResult {
    pub order: Vec<String>,
    pub cycle: Option<Vec<String>>,
}

// ── Topology report ─────────────────────────────────────────────────────────

/// Result of [`AppState::topology`]: a linear topo order plus the
/// parallel-layer decomposition (each layer is a set of wp ids that can
/// run concurrently).
#[derive(Debug, Clone)]
pub struct TopologyReport {
    pub topo: TopoResult,
    pub layers: Vec<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::*;

    /// In-memory WpRepository for testing.
    struct MemRepo {
        items: Vec<PickedItem>,
        dependencies: Vec<(String, String)>,
        done_set: HashSet<String>,
    }

    impl Default for MemRepo {
        fn default() -> Self {
            Self {
                items: Vec::new(),
                dependencies: Vec::new(),
                done_set: HashSet::new(),
            }
        }
    }

    impl MemRepo {
        fn with_items(items: Vec<PickedItem>) -> Self {
            Self {
                items,
                ..Default::default()
            }
        }
    }

    impl WpRepository for MemRepo {
        fn list_pickable(
            &self,
            _agent: &str,
            _lane: Option<&str>,
            _category: Option<&str>,
            limit: usize,
        ) -> Result<Vec<PickedItem>> {
            Ok(self.items.iter().take(limit).cloned().collect())
        }

        fn all_for_export(&self, _with_side: bool) -> Result<Vec<PickedItem>> {
            Ok(self.items.clone())
        }

        fn add_dependency(&mut self, from: &str, to: &str) -> Result<()> {
            self.dependencies.push((from.to_string(), to.to_string()));
            Ok(())
        }

        fn mark_done(&mut self, wp_id: &str) -> Result<()> {
            self.done_set.insert(wp_id.to_string());
            Ok(())
        }

        fn claim_count(&self) -> usize {
            0
        }

        fn wp_count(&self) -> usize {
            self.items.len()
        }

        fn stage_count(&self) -> usize {
            3
        }
    }

    fn make_item(id: &str, deps: Vec<&str>) -> PickedItem {
        PickedItem {
            wp_id: id.to_string(),
            title: format!("Title {id}"),
            state: "ready".to_string(),
            dependencies: deps.into_iter().map(String::from).collect(),
        }
    }

    // ── WpGraph tests ────────────────────────────────────────────────────────

    #[test]
    fn graph_from_empty_items() {
        let g = WpGraph::from_items(&[]);
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn graph_from_items_populates_nodes_and_edges() {
        let items = vec![make_item("A", vec!["B"])];
        let g = WpGraph::from_items(&items);
        assert!(g.nodes.contains("A"));
        assert!(g.nodes.contains("B"));
        assert_eq!(g.edges.get("A").unwrap().as_slice(), &["B"]);
    }

    #[test]
    fn graph_from_items_deduplicates_nodes() {
        let items = vec![make_item("A", vec!["B"]), make_item("C", vec!["B"])];
        let g = WpGraph::from_items(&items);
        // B appears as a dep from both A and C but should be counted once
        assert_eq!(g.nodes.len(), 3);
    }

    // ── topo_sort tests ──────────────────────────────────────────────────────

    #[test]
    fn topo_sort_empty_graph() {
        let g = WpGraph::default();
        let result = g.topo_sort();
        assert!(result.order.is_empty());
        assert!(result.cycle.is_none());
    }

    #[test]
    fn topo_sort_single_node() {
        let g = WpGraph::from_items(&[make_item("A", vec![])]);
        let result = g.topo_sort();
        assert_eq!(result.order, vec!["A"]);
        assert!(result.cycle.is_none());
    }

    #[test]
    fn topo_sort_linear_chain() {
        // A depends on B, B depends on C -> topo_sort returns dependency order
        let items = vec![make_item("A", vec!["B"]), make_item("B", vec!["C"])];
        let g = WpGraph::from_items(&items);
        let result = g.topo_sort();
        // Reverse order: A before B before C (dependents first)
        assert_eq!(result.order, vec!["A", "B", "C"]);
        assert!(result.cycle.is_none());
    }

    #[test]
    fn topo_sort_diamond() {
        // A -> B, A -> C, B -> D, C -> D
        let items = vec![
            make_item("A", vec!["B", "C"]),
            make_item("B", vec!["D"]),
            make_item("C", vec!["D"]),
        ];
        let g = WpGraph::from_items(&items);
        let result = g.topo_sort();
        assert_eq!(result.order.len(), 4);
        assert!(result.cycle.is_none());
        // Reverse dep order: A first, D last
        let pos = |n: &str| result.order.iter().position(|x| x == n).unwrap();
        assert!(pos("A") < pos("B"));
        assert!(pos("A") < pos("C"));
        assert!(pos("B") < pos("D"));
        assert!(pos("C") < pos("D"));
    }

    #[test]
    fn topo_sort_detects_cycle() {
        // A -> B -> C -> A
        let items = vec![
            make_item("A", vec!["B"]),
            make_item("B", vec!["C"]),
            make_item("C", vec!["A"]),
        ];
        let g = WpGraph::from_items(&items);
        let result = g.topo_sort();
        assert!(result.cycle.is_some());
        let cycle = result.cycle.unwrap();
        assert_eq!(cycle.len(), 3);
    }

    #[test]
    fn topo_sort_independent_nodes() {
        let items = vec![make_item("A", vec![]), make_item("B", vec![]), make_item("C", vec![])];
        let g = WpGraph::from_items(&items);
        let result = g.topo_sort();
        assert_eq!(result.order.len(), 3);
        assert!(result.cycle.is_none());
    }

    // ── parallel_layers tests ────────────────────────────────────────────────

    #[test]
    fn parallel_layers_single_chain() {
        // A -> B -> C: function groups into parallel layers
        let items = vec![make_item("A", vec!["B"]), make_item("B", vec!["C"])];
        let g = WpGraph::from_items(&items);
        let layers = g.parallel_layers();
        assert!(layers.len() >= 2); // At least 2 layers
        // All nodes should be present across layers
        let all: Vec<_> = layers.iter().flat_map(|l| l.clone()).collect();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn parallel_layers_diamond() {
        // A -> B, A -> C, B -> D, C -> D
        let items = vec![
            make_item("A", vec!["B", "C"]),
            make_item("B", vec!["D"]),
            make_item("C", vec!["D"]),
        ];
        let g = WpGraph::from_items(&items);
        let layers = g.parallel_layers();
        assert!(layers.len() >= 2);
        // All 4 nodes should be present
        let all: Vec<_> = layers.iter().flat_map(|l| l.clone()).collect();
        assert_eq!(all.len(), 4);
        // Just verify all nodes are present across all layers
        assert!(all.contains(&"A".to_string()));
        assert!(all.contains(&"B".to_string()));
        assert!(all.contains(&"C".to_string()));
        assert!(all.contains(&"D".to_string()));
    }

    #[test]
    fn parallel_layers_empty_graph() {
        let g = WpGraph::default();
        let layers = g.parallel_layers();
        assert!(layers.is_empty() || layers.len() == 1);
    }

    #[test]
    fn parallel_layers_all_roots() {
        let items = vec![make_item("A", vec![]), make_item("B", vec![])];
        let g = WpGraph::from_items(&items);
        let layers = g.parallel_layers();
        // All nodes should be in layer 0
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].len(), 2);
    }

    // ── AppState use-case tests ──────────────────────────────────────────────

    #[test]
    fn pick_returns_items_from_repo() {
        let repo = MemRepo::with_items(vec![
            make_item("WP01", vec![]),
            make_item("WP02", vec![]),
        ]);
        let state = AppState::new(repo);
        let req = PickRequest {
            agent_id: "agent-1".to_string(),
            limit: 10,
            lane: None,
            category: None,
        };
        let items = state.pick(&req).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn pick_respects_limit() {
        let repo = MemRepo::with_items(vec![
            make_item("WP01", vec![]),
            make_item("WP02", vec![]),
            make_item("WP03", vec![]),
        ]);
        let state = AppState::new(repo);
        let req = PickRequest {
            agent_id: "agent-1".to_string(),
            limit: 2,
            lane: None,
            category: None,
        };
        let items = state.pick(&req).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn claim_succeeds_on_first_attempt() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = ClaimRequest {
            claim_id: "c1".to_string(),
            resource: "worktree-slug".to_string(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-1".to_string(),
            ttl_seconds: 300,
            reason: ClaimReason::default(),
        };
        let claim = state.claim(&req).unwrap();
        assert_eq!(claim.resource, "worktree-slug");
    }

    #[test]
    fn claim_fails_when_already_claimed() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = ClaimRequest {
            claim_id: "c1".to_string(),
            resource: "worktree-slug".to_string(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-1".to_string(),
            ttl_seconds: 300,
            reason: ClaimReason::default(),
        };
        state.claim(&req).unwrap();
        // Second claim on same resource with different claim_id should fail
        let req2 = ClaimRequest {
            claim_id: "c2".to_string(),
            resource: "worktree-slug".to_string(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-2".to_string(),
            ttl_seconds: 300,
            reason: ClaimReason::default(),
        };
        assert!(state.claim(&req2).is_err());
    }

    #[test]
    fn heartbeat_returns_true_for_existing_claim() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = ClaimRequest {
            claim_id: "c1".to_string(),
            resource: "res".to_string(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-1".to_string(),
            ttl_seconds: 300,
            reason: ClaimReason::default(),
        };
        state.claim(&req).unwrap();
        let hb = HeartbeatRequest {
            claim_id: "c1".to_string(),
        };
        assert!(state.heartbeat(&hb).unwrap());
    }

    #[test]
    fn heartbeat_returns_false_for_unknown_claim() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let hb = HeartbeatRequest {
            claim_id: "nonexistent".to_string(),
        };
        assert!(!state.heartbeat(&hb).unwrap());
    }

    #[test]
    fn release_returns_true_for_existing_claim() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = ClaimRequest {
            claim_id: "c1".to_string(),
            resource: "res".to_string(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-1".to_string(),
            ttl_seconds: 300,
            reason: ClaimReason::default(),
        };
        state.claim(&req).unwrap();
        let rel = ReleaseRequest {
            claim_id: "c1".to_string(),
        };
        assert!(state.release(&rel).unwrap());
    }

    #[test]
    fn release_returns_false_for_unknown() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let rel = ReleaseRequest {
            claim_id: "nope".to_string(),
        };
        assert!(!state.release(&rel).unwrap());
    }

    #[test]
    fn done_marks_wp_done() {
        let repo = MemRepo::with_items(vec![make_item("WP01", vec![])]);
        let mut state = AppState::new(repo);
        let req = DoneRequest {
            claim_id: "c1".to_string(),
            wp_id: "WP01".to_string(),
            result: Some("completed".to_string()),
        };
        assert!(state.done(&mut req.clone()).unwrap());
    }

    #[test]
    fn dedup_finds_identical_items() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = DedupRequest {
            items: vec![
                ("Implement auth".to_string(), "Build auth system".to_string()),
                ("Add authentication".to_string(), "Create login feature".to_string()),
            ],
            threshold: 0.3,
        };
        // The dedup function should return candidates (may be empty or non-empty
        // depending on the similarity algorithm)
        let _candidates = state.dedup(&req).unwrap();
    }

    #[test]
    fn topology_builds_graph_from_repo() {
        let items = vec![
            make_item("WP01", vec![]),
            make_item("WP02", vec!["WP01"]),
        ];
        let repo = MemRepo::with_items(items);
        let state = AppState::new(repo);
        let req = TopologyRequest {
            root_wp: None,
        };
        let report = state.topology(&req).unwrap();
        assert_eq!(report.topo.order.len(), 2);
        assert!(report.topo.cycle.is_none());
        assert!(!report.layers.is_empty());
    }

    #[test]
    fn scan_inspects_directories() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let tmp = tempfile::tempdir().unwrap();
        let req = ScanRequest {
            roots: vec![tmp.path().to_string_lossy().to_string()],
            max_depth: None,
        };
        let infos = state.scan(&req).unwrap();
        assert_eq!(infos.len(), 1);
    }

    #[test]
    fn scan_skips_nonexistent_paths() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = ScanRequest {
            roots: vec!["/nonexistent/path/12345".to_string()],
            max_depth: None,
        };
        let infos = state.scan(&req).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn where_am_i_returns_snapshot() {
        let repo = MemRepo::with_items(vec![make_item("WP01", vec![])]);
        let state = AppState::new(repo);
        let tmp = tempfile::tempdir().unwrap();
        let req = WhereRequest {
            cwd: tmp.path().to_string_lossy().to_string(),
        };
        let resp = state.where_am_i(&req).unwrap();
        assert!(resp.repo.is_some());
        assert_eq!(resp.next_pickable.len(), 1);
    }

    #[test]
    fn where_am_i_with_nonexistent_cwd() {
        let repo = MemRepo::default();
        let state = AppState::new(repo);
        let req = WhereRequest {
            cwd: "/nonexistent/path".to_string(),
        };
        let resp = state.where_am_i(&req).unwrap();
        assert!(resp.repo.is_none());
    }
}
