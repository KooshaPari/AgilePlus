use std::collections::HashSet;

use super::{DependencyGraph, DependencyType, WorkPackage, WpDependency, WpState};

#[test]
fn wp_planned_to_doing() {
    let mut wp = WorkPackage::new(1, "test", 1, "criteria");
    wp.transition(WpState::Doing).unwrap();
    assert_eq!(wp.state, WpState::Doing);
}

#[test]
fn wp_doing_to_review() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    assert_eq!(wp.state, WpState::Review);
}

#[test]
fn wp_review_to_done() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    wp.transition(WpState::Done).unwrap();
    assert_eq!(wp.state, WpState::Done);
}

#[test]
fn wp_invalid_planned_to_done() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    assert!(wp.transition(WpState::Done).is_err());
}

#[test]
fn wp_blocked_and_back() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Blocked).unwrap();
    wp.transition(WpState::Planned).unwrap();
    assert_eq!(wp.state, WpState::Planned);
}

#[test]
fn wp_file_overlap() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.file_scope = vec!["src/main.rs".into(), "src/lib.rs".into()];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.file_scope = vec!["src/lib.rs".into(), "src/other.rs".into()];
    assert_eq!(a.has_file_overlap(&b), vec!["src/lib.rs".to_string()]);
}

#[test]
fn graph_empty() {
    let g = DependencyGraph::new();
    assert!(!g.has_cycle());
}

#[test]
fn graph_linear_order() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 2,
        dep_type: DependencyType::Explicit,
    });
    let order = g.execution_order().unwrap();
    assert_eq!(order, vec![vec![1], vec![2], vec![3]]);
}

#[test]
fn graph_parallel() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    let order = g.execution_order().unwrap();
    assert_eq!(order.len(), 2);
    assert_eq!(order[0], vec![1]);
    assert!(order[1].contains(&2) && order[1].contains(&3));
}

#[test]
fn graph_cycle_detected() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 1,
        depends_on: 2,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    assert!(g.has_cycle());
}

#[test]
fn graph_ready_wps() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    g.add_edge(WpDependency {
        wp_id: 3,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    let done = HashSet::new();
    let mut ready = g.ready_wps(&done);
    ready.sort();
    assert_eq!(ready, vec![1]);
    let done: HashSet<i64> = [1].into();
    let mut ready = g.ready_wps(&done);
    ready.sort();
    assert_eq!(ready, vec![2, 3]);
}

#[test]
fn graph_file_overlap_edges() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.id = 1;
    a.file_scope = vec!["f.rs".into()];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.id = 2;
    b.file_scope = vec!["f.rs".into()];
    let mut g = DependencyGraph::new();
    g.add_file_overlap_edges(&[a, b]);
    assert_eq!(g.execution_order().unwrap(), vec![vec![1], vec![2]]);
}

// --- Additional tests for uncovered paths ---

#[test]
fn wp_new_defaults() {
    let wp = WorkPackage::new(42, "My WP", 7, "criteria text");
    assert_eq!(wp.id, 0);
    assert_eq!(wp.feature_id, 42);
    assert_eq!(wp.title, "My WP");
    assert_eq!(wp.state, WpState::Planned);
    assert_eq!(wp.sequence, 7);
    assert_eq!(wp.acceptance_criteria, "criteria text");
    assert!(wp.file_scope.is_empty());
    assert!(wp.agent_id.is_none());
    assert!(wp.pr_url.is_none());
    assert!(wp.pr_state.is_none());
    assert!(wp.worktree_path.is_none());
    assert!(wp.plane_sub_issue_id.is_none());
    assert!(wp.base_commit.is_none());
    assert!(wp.head_commit.is_none());
}

#[test]
fn wp_blocked_to_doing() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Blocked).unwrap();
    wp.transition(WpState::Doing).unwrap();
    assert_eq!(wp.state, WpState::Doing);
}

#[test]
fn wp_review_back_to_doing() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    wp.transition(WpState::Doing).unwrap();
    assert_eq!(wp.state, WpState::Doing);
}

#[test]
fn wp_no_file_overlap() {
    let a = WorkPackage::new(1, "a", 1, "c");
    let b = WorkPackage::new(1, "b", 2, "c");
    assert!(a.has_file_overlap(&b).is_empty());
}

#[test]
fn wp_file_overlap_empty_scopes() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.file_scope = vec![];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.file_scope = vec!["a.rs".into()];
    assert!(a.has_file_overlap(&b).is_empty());
}

#[test]
fn wp_file_overlap_symmetric() {
    let mut a = WorkPackage::new(1, "a", 1, "c");
    a.file_scope = vec!["shared.rs".into()];
    let mut b = WorkPackage::new(1, "b", 2, "c");
    b.file_scope = vec!["shared.rs".into()];
    assert_eq!(a.has_file_overlap(&b), vec!["shared.rs".to_string()]);
    assert_eq!(b.has_file_overlap(&a), vec!["shared.rs".to_string()]);
}

#[test]
fn wp_transition_error_contains_states() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    let err = wp.transition(WpState::Done).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Planned"));
    assert!(msg.contains("Done"));
}

#[test]
fn wp_invalid_blocked_to_review() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Blocked).unwrap();
    assert!(wp.transition(WpState::Review).is_err());
}

#[test]
fn wp_invalid_done_to_anything() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    wp.transition(WpState::Doing).unwrap();
    wp.transition(WpState::Review).unwrap();
    wp.transition(WpState::Done).unwrap();
    assert!(wp.transition(WpState::Planned).is_err());
    assert!(wp.transition(WpState::Doing).is_err());
    assert!(wp.transition(WpState::Review).is_err());
    assert!(wp.transition(WpState::Blocked).is_err());
}

#[test]
fn can_transition_to_valid_transitions() {
    assert!(WpState::Planned.can_transition_to(WpState::Doing));
    assert!(WpState::Planned.can_transition_to(WpState::Blocked));
    assert!(WpState::Doing.can_transition_to(WpState::Review));
    assert!(WpState::Doing.can_transition_to(WpState::Blocked));
    assert!(WpState::Review.can_transition_to(WpState::Done));
    assert!(WpState::Review.can_transition_to(WpState::Doing));
    assert!(WpState::Blocked.can_transition_to(WpState::Planned));
    assert!(WpState::Blocked.can_transition_to(WpState::Doing));
}

#[test]
fn can_transition_to_invalid_transitions() {
    // Planned
    assert!(!WpState::Planned.can_transition_to(WpState::Planned));
    assert!(!WpState::Planned.can_transition_to(WpState::Review));
    assert!(!WpState::Planned.can_transition_to(WpState::Done));
    // Doing
    assert!(!WpState::Doing.can_transition_to(WpState::Doing));
    assert!(!WpState::Doing.can_transition_to(WpState::Planned));
    assert!(!WpState::Doing.can_transition_to(WpState::Done));
    // Review
    assert!(!WpState::Review.can_transition_to(WpState::Review));
    assert!(!WpState::Review.can_transition_to(WpState::Planned));
    assert!(!WpState::Review.can_transition_to(WpState::Blocked));
    // Done
    assert!(!WpState::Done.can_transition_to(WpState::Planned));
    assert!(!WpState::Done.can_transition_to(WpState::Doing));
    assert!(!WpState::Done.can_transition_to(WpState::Review));
    assert!(!WpState::Done.can_transition_to(WpState::Done));
    assert!(!WpState::Done.can_transition_to(WpState::Blocked));
    // Blocked
    assert!(!WpState::Blocked.can_transition_to(WpState::Blocked));
    assert!(!WpState::Blocked.can_transition_to(WpState::Review));
    assert!(!WpState::Blocked.can_transition_to(WpState::Done));
}

#[test]
fn wp_transition_updates_timestamp() {
    let mut wp = WorkPackage::new(1, "t", 1, "c");
    let before = wp.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(10));
    wp.transition(WpState::Doing).unwrap();
    assert!(wp.updated_at >= before);
}

#[test]
fn wp_serde_roundtrip() {
    let wp = WorkPackage::new(1, "Serializable", 3, "must work");
    let json = serde_json::to_string(&wp).unwrap();
    let back: WorkPackage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, wp.title);
    assert_eq!(back.state, wp.state);
    assert_eq!(back.sequence, wp.sequence);
    assert_eq!(back.acceptance_criteria, wp.acceptance_criteria);
}

#[test]
fn graph_ready_wps_all_done() {
    let mut g = DependencyGraph::new();
    g.add_edge(WpDependency {
        wp_id: 2,
        depends_on: 1,
        dep_type: DependencyType::Explicit,
    });
    let done: HashSet<i64> = [1, 2].into();
    let ready = g.ready_wps(&done);
    assert!(ready.is_empty());
}

#[test]
fn graph_execution_order_single_node() {
    let g = DependencyGraph::new();
    // No edges => empty execution order.
    assert!(g.execution_order().unwrap().is_empty());
}
