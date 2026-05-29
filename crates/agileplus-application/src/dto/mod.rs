//! Data Transfer Objects for application-layer command/query boundaries.
//!
//! These are plain data structs with no domain logic. They cross the use-case
//! boundary and may be constructed by API handlers, CLI commands, or tests.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::story::{Story, StoryStatus};

// ── Feature ──────────────────────────────────────────────────────────────────

/// Command: create a new Feature.
#[derive(Debug, Clone)]
pub struct CreateFeatureCmd {
    pub slug: String,
    pub friendly_name: String,
    /// Optional SHA-256 spec hash; defaults to all-zeros if absent.
    pub spec_hash: Option<[u8; 32]>,
    pub target_branch: Option<String>,
}

/// Output of `CreateFeature::execute`.
#[derive(Debug, Clone)]
pub struct FeatureCreatedOutput {
    pub id: i64,
    pub feature: Feature,
}

/// Command: advance a Feature through its lifecycle.
#[derive(Debug, Clone)]
pub struct AdvanceFeatureCmd {
    pub feature_id: i64,
    /// Target state, expressed as the lowercase string (e.g. "specified").
    pub target_state: String,
}

// ── Story ─────────────────────────────────────────────────────────────────────

/// Command: create a new Story under an Epic.
#[derive(Debug, Clone)]
pub struct CreateStoryCmd {
    pub epic_id: i64,
    pub project_id: i64,
    pub title: String,
    pub points: Option<u32>,
}

/// Output of `CreateStory::execute`.
#[derive(Debug, Clone)]
pub struct StoryCreatedOutput {
    pub id: i64,
    pub story: Story,
}

/// Command: transition a Story's status.
#[derive(Debug, Clone)]
pub struct TransitionStoryCmd {
    pub story_id: i64,
    pub target_status: StoryStatus,
}

// ── Epic ──────────────────────────────────────────────────────────────────────

/// Command: create a new Epic.
#[derive(Debug, Clone)]
pub struct CreateEpicCmd {
    pub project_id: i64,
    pub title: String,
}

/// Output of `CreateEpic::execute`.
#[derive(Debug, Clone)]
pub struct EpicCreatedOutput {
    pub id: i64,
}
