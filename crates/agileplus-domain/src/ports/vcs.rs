//! VCS port value objects.

use serde::{Deserialize, Serialize};

/// Information about a git branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
    pub commit: String,
    pub is_remote: bool,
}

/// Information about a git worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub commit: String,
}

/// Result of a merge operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub success: bool,
    pub commit: Option<String>,
    pub message: Option<String>,
}

/// A file-level conflict detected during merge analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub file_path: String,
    pub conflict_type: String,
}

/// Collection of artifacts discovered for a feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureArtifacts {
    pub spec: Option<String>,
    pub research: Option<String>,
    pub plan: Option<String>,
    pub other: Vec<String>,
}
