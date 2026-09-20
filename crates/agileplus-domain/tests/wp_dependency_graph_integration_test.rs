//! Integration contract tests for the work-package dependency graph in
//! `agileplus_domain::domain::work_package::dependency`.
//!
//! `DependencyGraph` is what turns a flat WP list into an executable schedule:
//! it derives file-overlap edges, answers "what can start now", and refuses to
//! produce an execution order for a cyclic plan. Getting the edge direction or
//! the layering wrong reorders work behind every WP in the project, so these
//! tests pin direction, readiness, layering, and cycle detection.
//!
//! Traceability: FR-014 (work-package scheduling), WP08

use agileplus_domain::domain::work_package::{
    DependencyGraph, DependencyType, WorkPackage, WpDependency,
};
use agileplus_domain::error::DomainError;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn wp(id: i64, sequence: i32, file_scope: &[&str]) -> WorkPackage {
    let mut package = WorkPackage::new(
        1,
        &format!("WP{id:02}"),
        sequence,
        &format!("criteria-{id}"),
    );
    package.id = id;
    package.file_scope = file_scope.iter().map(|s| (*s).to_string()).collect();
    package
}

fn dep(wp_id: i64, depends_on: i64) -> WpDependency {
    WpDependency {
        wp_id,
        depends_on,
        dep_type: DependencyType::Explicit,
    }
}

fn done(ids: &[i64]) -> HashSet<i64> {
    ids.iter().copied().collect()
}

// ---------------------------------------------------------------------------
// Empty graph
// ---------------------------------------------------------------------------

#[test]
fn an_empty_graph_has_no_nodes_no_layers_and_no_cycle() {
    let graph = DependencyGraph::new();

    assert!(graph.ready_wps(&done(&[])).is_empty());
    assert_eq!(graph.execution_order().unwrap(), Vec::<Vec<i64>>::new());
    assert!(!graph.has_cycle());
}

// ---------------------------------------------------------------------------
// Edge direction and readiness
// ---------------------------------------------------------------------------

#[test]
fn a_dependency_edge_blocks_the_dependent_until_the_target_is_done() {
    let mut graph = DependencyGraph::new();
    // WP2 depends on WP1.
    graph.add_edge(dep(2, 1));

    // Only the dependency-free node can start.
    assert_eq!(graph.ready_wps(&done(&[])), vec![1]);
    assert_eq!(graph.ready_wps(&done(&[1])), vec![2]);
}

#[test]
fn ready_wps_excludes_dependents_of_unfinished_work() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(2, 1));

    // Nothing done: only the dependency-free node is ready.
    assert_eq!(graph.ready_wps(&done(&[])), vec![1]);
    // Dependency done: the dependent becomes ready and the finished node drops.
    assert_eq!(graph.ready_wps(&done(&[1])), vec![2]);
    // Everything done: nothing left.
    assert!(graph.ready_wps(&done(&[1, 2])).is_empty());
}

#[test]
fn ready_wps_handles_a_node_with_multiple_unfinished_dependencies() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(3, 1));
    graph.add_edge(dep(3, 2));

    let mut ready = graph.ready_wps(&done(&[]));
    ready.sort();
    assert_eq!(ready, vec![1, 2]);

    assert_eq!(graph.ready_wps(&done(&[1])), vec![2]);
    assert_eq!(graph.ready_wps(&done(&[1, 2])), vec![3]);
}

#[test]
fn an_isolated_dependency_target_is_still_a_node_in_the_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(5, 4));

    let mut ready = graph.ready_wps(&done(&[]));
    ready.sort();
    // `all_node_ids` includes both endpoints, but only 4 has no deps.
    assert_eq!(ready, vec![4]);
}

// ---------------------------------------------------------------------------
// Execution order (layering)
// ---------------------------------------------------------------------------

#[test]
fn execution_order_groups_independent_work_into_sorted_layers() {
    let mut graph = DependencyGraph::new();
    // Chain: 3 -> 2 -> 1 (2 depends on 1, 3 depends on 2).
    graph.add_edge(dep(2, 1));
    graph.add_edge(dep(3, 2));

    let layers = graph.execution_order().unwrap();
    assert_eq!(layers, vec![vec![1], vec![2], vec![3]]);
    assert!(!graph.has_cycle());
}

#[test]
fn execution_order_places_siblings_in_the_same_sorted_layer() {
    let mut graph = DependencyGraph::new();
    // 5 and 3 both depend on 1; 9 depends on 5.
    graph.add_edge(dep(5, 1));
    graph.add_edge(dep(3, 1));
    graph.add_edge(dep(9, 5));

    let layers = graph.execution_order().unwrap();
    assert_eq!(layers, vec![vec![1], vec![3, 5], vec![9]]);
}

#[test]
fn a_diamond_lays_out_three_layers() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(2, 1));
    graph.add_edge(dep(3, 1));
    graph.add_edge(dep(4, 2));
    graph.add_edge(dep(4, 3));

    let layers = graph.execution_order().unwrap();
    assert_eq!(layers, vec![vec![1], vec![2, 3], vec![4]]);
}

// ---------------------------------------------------------------------------
// Cycle detection
// ---------------------------------------------------------------------------

#[test]
fn a_two_node_cycle_is_reported_as_a_domain_error_and_has_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(1, 2));
    graph.add_edge(dep(2, 1));

    assert!(graph.has_cycle());
    match graph.execution_order().unwrap_err() {
        DomainError::InvalidTransition { from, to, reason } => {
            assert_eq!(from, "graph");
            assert_eq!(to, "execution_order");
            assert_eq!(reason, "cycle detected in dependency graph");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn a_self_dependency_is_a_cycle() {
    let mut graph = DependencyGraph::new();
    graph.add_edge(dep(1, 1));

    assert!(graph.has_cycle());
    assert!(graph.execution_order().is_err());
}

#[test]
fn a_longer_cycle_that_contains_acyclic_nodes_fails_the_whole_order() {
    let mut graph = DependencyGraph::new();
    // Acyclic leader plus a 3-cycle: 10 -> 11 -> 12 -> 10.
    graph.add_edge(dep(10, 11));
    graph.add_edge(dep(11, 12));
    graph.add_edge(dep(12, 10));

    assert!(graph.has_cycle());
    assert!(graph.execution_order().is_err());
}

// ---------------------------------------------------------------------------
// File-overlap edge derivation
// ---------------------------------------------------------------------------

#[test]
fn file_overlap_creates_a_dependency_from_the_later_wp_to_the_earlier_one() {
    let mut graph = DependencyGraph::new();
    graph.add_file_overlap_edges(&[
        wp(1, 1, &["src/auth.rs", "src/lib.rs"]),
        wp(2, 2, &["src/lib.rs"]),
    ]);

    // Layer 1 is the earlier WP; the later WP depends on it.
    assert_eq!(graph.execution_order().unwrap(), vec![vec![1], vec![2]]);
    assert_eq!(graph.ready_wps(&done(&[1])), vec![2]);
}

#[test]
fn file_overlap_direction_follows_sequence_not_input_order() {
    let mut graph = DependencyGraph::new();
    // The later-sequence WP is passed first; the edge must still point from the
    // earlier sequence to the later one.
    graph.add_file_overlap_edges(&[wp(20, 9, &["shared.rs"]), wp(10, 1, &["shared.rs"])]);

    assert_eq!(graph.execution_order().unwrap(), vec![vec![10], vec![20]]);
}

#[test]
fn equal_sequences_fall_back_to_input_order_as_the_tie_break() {
    let mut graph = DependencyGraph::new();
    graph.add_file_overlap_edges(&[wp(1, 5, &["shared.rs"]), wp(2, 5, &["shared.rs"])]);

    assert_eq!(graph.execution_order().unwrap(), vec![vec![1], vec![2]]);
}

#[test]
fn non_overlapping_file_scopes_produce_no_edges_and_therefore_no_nodes() {
    let mut graph = DependencyGraph::new();
    graph.add_file_overlap_edges(&[
        wp(1, 1, &["src/a.rs"]),
        wp(2, 2, &["src/b.rs"]),
        wp(3, 3, &["docs/readme.md"]),
    ]);

    // The graph is edge-defined: with no edges there are no nodes, so there is
    // nothing to schedule and nothing is "ready".
    assert_eq!(graph.execution_order().unwrap(), Vec::<Vec<i64>>::new());
    assert!(graph.ready_wps(&done(&[])).is_empty());
    assert!(!graph.has_cycle());
}

#[test]
fn partial_overlap_still_creates_exactly_one_edge() {
    let mut graph = DependencyGraph::new();
    graph.add_file_overlap_edges(&[
        wp(1, 1, &["a.rs", "b.rs", "c.rs"]),
        wp(2, 2, &["b.rs", "c.rs"]),
    ]);

    // Duplicate overlap entries do not duplicate the dependency edge.
    assert_eq!(graph.execution_order().unwrap(), vec![vec![1], vec![2]]);
}

#[test]
fn an_empty_wp_list_produces_an_empty_graph() {
    let mut graph = DependencyGraph::new();
    graph.add_file_overlap_edges(&[]);

    assert!(graph.ready_wps(&done(&[])).is_empty());
    assert!(!graph.has_cycle());
}

#[test]
fn explicit_and_derived_edges_combine_into_the_same_schedule() {
    let mut graph = DependencyGraph::new();
    // Derived: WP2 overlaps WP1's file scope.
    graph.add_file_overlap_edges(&[wp(1, 1, &["shared.rs"]), wp(2, 2, &["shared.rs"])]);
    // Explicit: WP3 depends on WP2 even though their scopes differ.
    graph.add_edge(dep(3, 2));

    assert_eq!(
        graph.execution_order().unwrap(),
        vec![vec![1], vec![2], vec![3]]
    );
    assert!(!graph.has_cycle());
}

#[test]
fn dependency_types_are_distinct_wire_values() {
    // The three origins must stay distinguishable in serialized plans.
    assert_ne!(DependencyType::Explicit, DependencyType::FileOverlap);
    assert_ne!(DependencyType::FileOverlap, DependencyType::Data);
    assert_ne!(DependencyType::Explicit, DependencyType::Data);

    let mut graph = DependencyGraph::new();
    graph.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Data,
    });
    assert_eq!(graph.execution_order().unwrap(), vec![vec![1], vec![2]]);
}
