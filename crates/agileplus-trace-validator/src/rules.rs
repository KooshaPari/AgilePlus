use crate::graph::{TraceGraph, TraceNode, TraceNodeKind};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub kind: ValidationIssueKind,
    pub node_id: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationIssueKind {
    BrokenReference,
    DuplicateId,
    MissingTestCoverage,
    OrphanFunctionalRequirement,
}

#[must_use]
pub fn validate_graph(graph: &TraceGraph) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    validate_duplicate_ids(graph, &mut issues);
    validate_broken_refs(graph, &mut issues);
    validate_functional_requirements(graph, &mut issues);
    issues.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then(format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    issues
}

fn validate_duplicate_ids(graph: &TraceGraph, issues: &mut Vec<ValidationIssue>) {
    for (id, paths) in &graph.duplicate_ids {
        issues.push(ValidationIssue {
            kind: ValidationIssueKind::DuplicateId,
            node_id: id.clone(),
            message: format!("duplicate trace id {id} appears in {} files", paths.len()),
            path: paths.first().cloned(),
        });
    }
}

fn validate_broken_refs(graph: &TraceGraph, issues: &mut Vec<ValidationIssue>) {
    for node in graph.nodes.values() {
        for reference in &node.refs {
            if !graph.nodes.contains_key(reference) {
                issues.push(ValidationIssue {
                    kind: ValidationIssueKind::BrokenReference,
                    node_id: node.id.clone(),
                    message: format!("{} references missing trace id {reference}", node.id),
                    path: Some(node.path.clone()),
                });
            }
        }
    }
}

fn validate_functional_requirements(graph: &TraceGraph, issues: &mut Vec<ValidationIssue>) {
    for node in graph
        .nodes
        .values()
        .filter(|node| node.kind == TraceNodeKind::FunctionalRequirement)
    {
        let has_refs = !node.refs.is_empty();
        let has_incoming = !graph.incoming_refs(&node.id).is_empty();
        if !has_refs && !has_incoming {
            issues.push(ValidationIssue {
                kind: ValidationIssueKind::OrphanFunctionalRequirement,
                node_id: node.id.clone(),
                message: format!("functional requirement {} has no trace links", node.id),
                path: Some(node.path.clone()),
            });
        }

        if !has_test_coverage(graph, node) {
            issues.push(ValidationIssue {
                kind: ValidationIssueKind::MissingTestCoverage,
                node_id: node.id.clone(),
                message: format!("functional requirement {} is not linked to a test", node.id),
                path: Some(node.path.clone()),
            });
        }
    }
}

fn has_test_coverage(graph: &TraceGraph, requirement: &TraceNode) -> bool {
    requirement.refs.iter().any(|reference| {
        graph
            .nodes
            .get(reference)
            .is_some_and(|node| node.kind == TraceNodeKind::Test)
    }) || graph
        .incoming_refs(&requirement.id)
        .iter()
        .any(|node| node.kind == TraceNodeKind::Test)
}
