// SPDX-License-Identifier: MIT OR Apache-2.0
//! Topology primitive tests: the exact layer decomposition, duplicate
//! dependency edges, and how the decomposition degrades on a cyclic graph.
//!
//! # Known limitation pinned here
//!
//! `WpGraph::parallel_layers` ranks a node while iterating the *topological*
//! order, which yields dependents before their prerequisites. Ranks are
//! therefore usually computed before the prerequisite has one, so the result
//! is a shallow two-level split (dependency-free nodes, then everything else)
//! rather than true longest-path layering. These tests pin that behaviour so a
//! future fix shows up as a deliberate change.

use crate::dto::PickedItem;
use crate::use_cases::triage::WpGraph;

fn make_item(id: &str, deps: Vec<&str>) -> PickedItem {
    PickedItem {
        wp_id: id.to_string(),
        title: format!("Title {id}"),
        state: "ready".to_string(),
        dependencies: deps.into_iter().map(String::from).collect(),
    }
}

/// Only dependency-free nodes are separated out; every dependent collapses
/// into the same layer, even across different depths of the diamond
/// (`A -> {B, C}`, `B -> D`, `C -> D`).
#[test]
fn parallel_layers_separates_only_dependency_free_nodes() {
    let graph = WpGraph::from_items(&[
        make_item("A", vec!["B", "C"]),
        make_item("B", vec!["D"]),
        make_item("C", vec!["D"]),
    ]);

    assert_eq!(
        graph.parallel_layers(),
        vec![
            vec!["D".to_string()],
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
        ]
    );
}

/// A single dependency-free node with a prerequisite-free child keeps the
/// same two-level shape.
#[test]
fn parallel_layers_keeps_dependents_together_for_a_chain() {
    let graph = WpGraph::from_items(&[make_item("B", vec!["C"]), make_item("A", vec!["B"])]);

    assert_eq!(
        graph.parallel_layers(),
        vec![
            vec!["C".to_string()],
            vec!["A".to_string(), "B".to_string()]
        ]
    );
}

/// A cyclic graph produces no linear order, so no node gets a rank and the
/// decomposition collapses to a single layer instead of dropping nodes.
#[test]
fn parallel_layers_collapses_cyclic_graph_into_one_layer() {
    let graph = WpGraph::from_items(&[make_item("A", vec!["B"]), make_item("B", vec!["A"])]);

    let topo = graph.topo_sort();
    assert!(
        topo.order.is_empty(),
        "no node is rankable without a cycle break"
    );
    assert_eq!(
        graph.parallel_layers(),
        vec![vec!["A".to_string(), "B".to_string()]]
    );
}

/// A repeated dependency edge inflates the in-degree count, but the matching
/// repeated decrement still reaches zero, so the node is emitted exactly once.
#[test]
fn topo_sort_tolerates_duplicate_dependency_edges() {
    let graph = WpGraph::from_items(&[make_item("A", vec!["B", "B"])]);

    let result = graph.topo_sort();
    assert_eq!(result.order, vec!["A", "B"]);
    assert!(result.cycle.is_none());
}
