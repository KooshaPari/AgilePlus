use crate::loaders::{TraceDocument, TraceDocumentKind};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceGraph {
    pub nodes: BTreeMap<String, TraceNode>,
    pub duplicate_ids: BTreeMap<String, Vec<PathBuf>>,
}

impl TraceGraph {
    #[must_use]
    pub fn from_documents(documents: &[TraceDocument]) -> Self {
        let mut nodes = BTreeMap::new();
        let mut duplicate_ids: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

        for document in documents {
            let node = TraceNode {
                id: document.id.clone(),
                kind: TraceNodeKind::from(document.kind),
                refs: document.refs.iter().cloned().collect(),
                path: document.path.clone(),
            };

            if let Some(existing) = nodes.insert(document.id.clone(), node) {
                duplicate_ids
                    .entry(document.id.clone())
                    .or_default()
                    .push(existing.path);
                duplicate_ids
                    .entry(document.id.clone())
                    .or_default()
                    .push(document.path.clone());
            }
        }

        for paths in duplicate_ids.values_mut() {
            paths.sort();
            paths.dedup();
        }

        Self {
            nodes,
            duplicate_ids,
        }
    }

    #[must_use]
    pub fn incoming_refs(&self, id: &str) -> Vec<&TraceNode> {
        self.nodes
            .values()
            .filter(|node| node.refs.contains(id))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceNode {
    pub id: String,
    pub kind: TraceNodeKind,
    pub refs: BTreeSet<String>,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceNodeKind {
    FunctionalRequirement,
    NonFunctionalRequirement,
    Test,
    Code,
}

impl From<TraceDocumentKind> for TraceNodeKind {
    fn from(kind: TraceDocumentKind) -> Self {
        match kind {
            TraceDocumentKind::FunctionalRequirement => Self::FunctionalRequirement,
            TraceDocumentKind::NonFunctionalRequirement => Self::NonFunctionalRequirement,
            TraceDocumentKind::Test => Self::Test,
            TraceDocumentKind::Code => Self::Code,
        }
    }
}
