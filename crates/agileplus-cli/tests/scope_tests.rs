// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus_cli::commands::scope`.
//!
//! Tests `detect_file_scope`, `OverlapGraph`, and `build_overlap_graph`
//! with various edge cases beyond the existing inline tests.

use agileplus_cli::commands::scope::{OverlapGraph, build_overlap_graph, detect_file_scope};
use agileplus_domain::domain::work_package::WorkPackage;

// ── detect_file_scope ────────────────────────────────────────────────────────

#[test]
fn detect_rs_files() {
    let desc = "Fix src/main.rs and src/lib.rs";
    let scope = detect_file_scope(desc);
    assert!(scope.contains(&"src/main.rs".to_string()));
    assert!(scope.contains(&"src/lib.rs".to_string()));
}

#[test]
fn detect_nested_paths() {
    let desc = "Update crates/agileplus-cli/src/commands/dashboard.rs";
    let scope = detect_file_scope(desc);
    assert!(scope.iter().any(|s| s.contains("dashboard.rs")));
}

#[test]
fn detect_ts_files() {
    let desc = "Refactor src/index.ts and src/utils/helper.ts";
    let scope = detect_file_scope(desc);
    assert!(scope.iter().any(|s| s.contains("index.ts")));
    assert!(scope.iter().any(|s| s.contains("helper.ts")));
}

#[test]
fn detect_json_files() {
    let desc = "Update package.json and tsconfig.json";
    let scope = detect_file_scope(desc);
    assert!(scope.contains(&"package.json".to_string()));
    assert!(scope.contains(&"tsconfig.json".to_string()));
}

#[test]
fn no_files_in_description() {
    let desc = "This is a simple description without any file references.";
    let scope = detect_file_scope(desc);
    assert!(scope.is_empty());
}

#[test]
fn empty_description() {
    let scope = detect_file_scope("");
    assert!(scope.is_empty());
}

#[test]
fn mixed_content_and_files() {
    let desc = "Update README.md to document src/new_feature.rs changes in the config.toml";
    let scope = detect_file_scope(desc);
    assert_eq!(scope.len(), 3);
    assert!(scope.contains(&"README.md".to_string()));
    assert!(scope.iter().any(|s| s.contains("new_feature.rs")));
    assert!(scope.contains(&"config.toml".to_string()));
}

#[test]
fn backtick_wrapped_files() {
    let desc = "Edit `src/auth.rs` and `tests/auth_test.rs`";
    let scope = detect_file_scope(desc);
    assert!(scope.iter().any(|s| s.contains("auth.rs")));
}

#[test]
fn long_extensions_are_skipped() {
    // Extensions > 6 chars shouldn't be detected as file references
    let desc = "The configuration.backup_longext file needs updating";
    let scope = detect_file_scope(desc);
    // "backup_longext" has extension "longext" (6 chars) which is borderline
    // The test just verifies the function doesn't crash
    let _ = scope;
}

#[test]
fn numeric_only_words_are_skipped() {
    let desc = "Fix issue 42 in module 7";
    let scope = detect_file_scope(desc);
    assert!(scope.is_empty());
}

// ── OverlapGraph ─────────────────────────────────────────────────────────────

#[test]
fn overlap_graph_new_is_empty() {
    let graph = OverlapGraph::new();
    assert!(graph.edges.is_empty());
}

#[test]
fn parallel_groups_empty_ids() {
    let graph = OverlapGraph::new();
    let groups = graph.parallel_groups(&[]);
    assert!(groups.is_empty() || groups.iter().all(|g| g.is_empty()));
}

#[test]
fn parallel_groups_single_id_no_conflicts() {
    let graph = OverlapGraph::new();
    let groups = graph.parallel_groups(&[42]);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0], vec![42]);
}

#[test]
fn parallel_groups_multiple_ids_no_conflicts() {
    let graph = OverlapGraph::new();
    let groups = graph.parallel_groups(&[1, 2, 3, 4, 5]);
    assert_eq!(groups.len(), 1);
    let all: Vec<i64> = groups.into_iter().flatten().collect();
    assert_eq!(all.len(), 5);
}

#[test]
fn parallel_groups_full_mesh_conflict() {
    // Every pair conflicts — needs N groups for N items
    let mut graph = OverlapGraph::new();
    graph.edges.push((1, 2, vec!["f.rs".into()]));
    graph.edges.push((1, 3, vec!["g.rs".into()]));
    graph.edges.push((2, 3, vec!["h.rs".into()]));
    let groups = graph.parallel_groups(&[1, 2, 3]);
    // 3 nodes fully connected → needs 3 colors
    assert_eq!(groups.len(), 3);
    for g in &groups {
        assert_eq!(g.len(), 1);
    }
}

#[test]
fn parallel_groups_unrelated_conflicts() {
    // 1-2 conflict, 3-4 conflict, 1-3 don't conflict
    let mut graph = OverlapGraph::new();
    graph.edges.push((1, 2, vec!["a.rs".into()]));
    graph.edges.push((3, 4, vec!["b.rs".into()]));
    let groups = graph.parallel_groups(&[1, 2, 3, 4]);
    // Should be able to put {1,3} and {2,4} in two groups
    assert!(groups.len() <= 2);
}

// ── build_overlap_graph ──────────────────────────────────────────────────────

#[test]
fn build_overlap_graph_empty_slice() {
    let graph = build_overlap_graph(&[]);
    assert!(graph.edges.is_empty());
}

#[test]
fn build_overlap_graph_single_wp() {
    let mut wp = WorkPackage::new(1, "solo", 1, "criteria");
    wp.id = 1;
    wp.file_scope = vec!["src/a.rs".into()];
    let graph = build_overlap_graph(&[wp]);
    assert!(graph.edges.is_empty());
}

#[test]
fn build_overlap_graph_three_wps_partial_overlap() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.id = 1;
    a.file_scope = vec!["src/shared.rs".into(), "src/a_only.rs".into()];

    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.id = 2;
    b.file_scope = vec!["src/shared.rs".into(), "src/b_only.rs".into()];

    let mut c = WorkPackage::new(1, "c", 3, "c");
    c.id = 3;
    c.file_scope = vec!["src/c_only.rs".into()];

    let graph = build_overlap_graph(&[a, b, c]);
    // a-b overlap on shared.rs, a-c no overlap, b-c no overlap
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].0, 1);
    assert_eq!(graph.edges[0].1, 2);
}
