// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data Transfer Objects for application-layer command/query boundaries.
//!
//! These are plain data structs with no domain logic. They cross the use-case
//! boundary and may be constructed by API handlers, CLI commands, or tests.

// ── Triage / CLI orchestration DTOs (FR-AGP-018, FR-AGP-019, FR-AGP-020) ──────
//
// These are the DTOs that flow through the use-case boundary for the
// CLI triage subcommands (pick, claim, dedup, scan, topology, etc.).
// They reference types from `agileplus-triage` (dedup candidates, claim
// records, repo info).
//
// We re-export the third-party types from this module so callers
// importing `crate::dto::*` get a closed surface.

pub use agileplus_triage::claim::{Claim, ClaimKind, ClaimReason, ClaimState, ClaimStore};
pub use agileplus_triage::dedup::DuplicateCandidate;
pub use agileplus_triage::repo_introspect::RepoInfo;

use serde::{Deserialize, Serialize};

// ── Pick / list ──────────────────────────────────────────────────────────────

/// Request: pick the next N work packages for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickRequest {
    pub agent_id: String,
    pub limit: usize,
    pub lane: Option<String>,
    pub category: Option<String>,
}

/// A picked work package summary returned by `AppState::pick`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickedItem {
    pub wp_id: String,
    pub title: String,
    pub state: String,
    pub dependencies: Vec<String>,
}

// ── Claim lifecycle ─────────────────────────────────────────────────────────

/// Request: claim a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub claim_id: String,
    pub resource: String,
    pub kind: ClaimKind,
    pub agent_id: String,
    pub ttl_seconds: i64,
    /// Structured reason for the claim. Optional on the wire; if
    /// absent, the use case layer falls back to `ClaimReason::default()`
    /// (`Manual("")`).
    #[serde(default)]
    pub reason: ClaimReason,
}

/// Request: refresh a claim's heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub claim_id: String,
}

/// Request: explicitly release a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseRequest {
    pub claim_id: String,
}

/// Request: mark a work package done and release its claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoneRequest {
    pub claim_id: String,
    pub wp_id: String,
    pub result: Option<String>,
}

// ── Dedup / scan / topology / export ────────────────────────────────────────

/// Request: find duplicate work-package candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupRequest {
    pub items: Vec<(String, String)>,
    pub threshold: f64,
}

/// Request: scan roots for repo metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRequest {
    pub roots: Vec<String>,
    pub max_depth: Option<usize>,
}

/// Request: produce a topology report from current work-package graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyRequest {
    pub root_wp: Option<String>,
}

/// Request: export current work-package set in the chosen format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub output_path: Option<String>,
    pub with_side: bool,
}

/// Output format selector for `AppState::export`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Markdown,
    Csv,
    Html,
    Mermaid,
    Dot,
}

// ── "Where am I?" snapshot ──────────────────────────────────────────────────

/// Request: snapshot the agent's current work context from a cwd.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereRequest {
    pub cwd: String,
}

/// Response: a snapshot of repo, claims, lane/category, and pickable items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereResponse {
    pub repo: Option<RepoInfo>,
    pub active_claims: Vec<Claim>,
    pub lane: Option<String>,
    pub category: Option<String>,
    pub next_pickable: Vec<PickedItem>,
}

// ── Domain DTOs (legacy per-use-case commands) ──────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── PickRequest ───────────────────────────────────────────────────────────

    #[test]
    fn pick_request_serializes_and_roundtrips() {
        let req = PickRequest {
            agent_id: "agent-1".into(),
            limit: 5,
            lane: Some("backend".into()),
            category: Some("bug".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PickRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.agent_id, "agent-1");
        assert_eq!(back.limit, 5);
        assert_eq!(back.lane.as_deref(), Some("backend"));
        assert_eq!(back.category.as_deref(), Some("bug"));
    }

    #[test]
    fn pick_request_optional_fields_none() {
        let req = PickRequest {
            agent_id: "a".into(),
            limit: 1,
            lane: None,
            category: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: PickRequest = serde_json::from_str(&json).unwrap();
        assert!(back.lane.is_none());
        assert!(back.category.is_none());
    }

    // ── PickedItem ────────────────────────────────────────────────────────────

    #[test]
    fn picked_item_serializes_and_roundtrips() {
        let item = PickedItem {
            wp_id: "wp-1".into(),
            title: "Do the thing".into(),
            state: "ready".into(),
            dependencies: vec!["wp-0".into()],
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: PickedItem = serde_json::from_str(&json).unwrap();
        assert_eq!(back.wp_id, "wp-1");
        assert_eq!(back.dependencies, vec!["wp-0"]);
    }

    #[test]
    fn picked_item_empty_dependencies() {
        let item = PickedItem {
            wp_id: "wp-99".into(),
            title: "No deps".into(),
            state: "done".into(),
            dependencies: vec![],
        };
        let json = serde_json::to_string(&item).unwrap();
        let back: PickedItem = serde_json::from_str(&json).unwrap();
        assert!(back.dependencies.is_empty());
    }

    // ── ClaimRequest ──────────────────────────────────────────────────────────

    #[test]
    fn claim_request_serializes_and_roundtrips() {
        let req = ClaimRequest {
            claim_id: "c-1".into(),
            resource: "worktree-foo".into(),
            kind: ClaimKind::Worktree,
            agent_id: "agent-1".into(),
            ttl_seconds: 300,
            reason: ClaimReason::TaskRef("wp-42".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ClaimRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.claim_id, "c-1");
        assert_eq!(back.kind, ClaimKind::Worktree);
        assert_eq!(back.ttl_seconds, 300);
        assert_eq!(back.reason, ClaimReason::TaskRef("wp-42".into()));
    }

    #[test]
    fn claim_request_default_reason() {
        // Deserialize with missing "reason" field — should use Default
        let json = r#"{"claim_id":"c-2","resource":"branch-x","kind":"branch","agent_id":"agent-2","ttl_seconds":60}"#;
        let back: ClaimRequest = serde_json::from_str(json).unwrap();
        assert_eq!(back.reason, ClaimReason::default());
    }

    // ── HeartbeatRequest / ReleaseRequest / DoneRequest ───────────────────────

    #[test]
    fn heartbeat_request_roundtrips() {
        let req = HeartbeatRequest {
            claim_id: "c-1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: HeartbeatRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.claim_id, "c-1");
    }

    #[test]
    fn release_request_roundtrips() {
        let req = ReleaseRequest {
            claim_id: "c-1".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ReleaseRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.claim_id, "c-1");
    }

    #[test]
    fn done_request_roundtrips_with_result() {
        let req = DoneRequest {
            claim_id: "c-3".into(),
            wp_id: "wp-7".into(),
            result: Some("all good".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DoneRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.result.as_deref(), Some("all good"));
    }

    #[test]
    fn done_request_roundtrips_without_result() {
        let req = DoneRequest {
            claim_id: "c-4".into(),
            wp_id: "wp-8".into(),
            result: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DoneRequest = serde_json::from_str(&json).unwrap();
        assert!(back.result.is_none());
    }

    // ── DedupRequest ──────────────────────────────────────────────────────────

    #[test]
    fn dedup_request_roundtrips() {
        let req = DedupRequest {
            items: vec![
                ("a".into(), "b".into()),
                ("c".into(), "d".into()),
            ],
            threshold: 0.75,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DedupRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.items.len(), 2);
        assert!((back.threshold - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn dedup_request_empty_items() {
        let req = DedupRequest {
            items: vec![],
            threshold: 0.0,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: DedupRequest = serde_json::from_str(&json).unwrap();
        assert!(back.items.is_empty());
    }

    // ── ScanRequest / TopologyRequest / ExportRequest ─────────────────────────

    #[test]
    fn scan_request_roundtrips() {
        let req = ScanRequest {
            roots: vec!["/a".into(), "/b".into()],
            max_depth: Some(3),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ScanRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.roots.len(), 2);
        assert_eq!(back.max_depth, Some(3));
    }

    #[test]
    fn scan_request_no_max_depth() {
        let req = ScanRequest {
            roots: vec![".".into()],
            max_depth: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ScanRequest = serde_json::from_str(&json).unwrap();
        assert!(back.max_depth.is_none());
    }

    #[test]
    fn topology_request_roundtrips() {
        let req = TopologyRequest {
            root_wp: Some("wp-1".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: TopologyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.root_wp.as_deref(), Some("wp-1"));
    }

    #[test]
    fn topology_request_no_root() {
        let req = TopologyRequest { root_wp: None };
        let json = serde_json::to_string(&req).unwrap();
        let back: TopologyRequest = serde_json::from_str(&json).unwrap();
        assert!(back.root_wp.is_none());
    }

    #[test]
    fn export_request_roundtrips() {
        let req = ExportRequest {
            format: ExportFormat::Markdown,
            output_path: Some("/tmp/out.md".into()),
            with_side: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ExportRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.format, ExportFormat::Markdown);
        assert!(back.with_side);
    }

    // ── ExportFormat enum ─────────────────────────────────────────────────────

    #[test]
    fn export_format_all_variants_roundtrip() {
        let formats = [
            ExportFormat::Markdown,
            ExportFormat::Csv,
            ExportFormat::Html,
            ExportFormat::Mermaid,
            ExportFormat::Dot,
        ];
        for fmt in &formats {
            let json = serde_json::to_string(fmt).unwrap();
            let back: ExportFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, fmt);
        }
    }

    #[test]
    fn export_format_snake_case_names() {
        assert_eq!(
            serde_json::to_string(&ExportFormat::Markdown).unwrap(),
            r#""markdown""#
        );
        assert_eq!(
            serde_json::to_string(&ExportFormat::Csv).unwrap(),
            r#""csv""#
        );
        assert_eq!(
            serde_json::to_string(&ExportFormat::Html).unwrap(),
            r#""html""#
        );
        assert_eq!(
            serde_json::to_string(&ExportFormat::Mermaid).unwrap(),
            r#""mermaid""#
        );
        assert_eq!(
            serde_json::to_string(&ExportFormat::Dot).unwrap(),
            r#""dot""#
        );
    }

    // ── WhereRequest / WhereResponse ──────────────────────────────────────────

    #[test]
    fn where_request_roundtrips() {
        let req = WhereRequest {
            cwd: "/home/user/project".into(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: WhereRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cwd, "/home/user/project");
    }

    #[test]
    fn where_response_roundtrips() {
        let resp = WhereResponse {
            repo: None,
            active_claims: vec![],
            lane: Some("backend".into()),
            category: Some("infra".into()),
            next_pickable: vec![PickedItem {
                wp_id: "wp-1".into(),
                title: "T".into(),
                state: "ready".into(),
                dependencies: vec![],
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: WhereResponse = serde_json::from_str(&json).unwrap();
        assert!(back.repo.is_none());
        assert_eq!(back.next_pickable.len(), 1);
        assert_eq!(back.lane.as_deref(), Some("backend"));
    }

    // ── Domain command DTOs (non-serde, Clone + Debug) ────────────────────────

    #[test]
    fn create_feature_cmd_clone_and_debug() {
        let cmd = CreateFeatureCmd {
            slug: "auth".into(),
            friendly_name: "Auth".into(),
            spec_hash: Some([42u8; 32]),
            target_branch: Some("feat/auth".into()),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.slug, "auth");
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("CreateFeatureCmd"));
        assert!(dbg.contains("auth"));
    }

    #[test]
    fn create_feature_cmd_defaults() {
        let cmd = CreateFeatureCmd {
            slug: "s".into(),
            friendly_name: "f".into(),
            spec_hash: None,
            target_branch: None,
        };
        let cloned = cmd.clone();
        assert!(cloned.spec_hash.is_none());
        assert!(cloned.target_branch.is_none());
    }

    #[test]
    fn feature_created_output_clone_and_debug() {
        let cmd = FeatureCreatedOutput {
            id: 1,
            feature: agileplus_domain::domain::feature::Feature::new(
                "slug",
                "Friendly",
                [0u8; 32],
                None,
            ),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.id, 1);
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("FeatureCreatedOutput"));
    }

    #[test]
    fn advance_feature_cmd_clone_and_debug() {
        let cmd = AdvanceFeatureCmd {
            feature_id: 42,
            target_state: "specified".into(),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.feature_id, 42);
        assert_eq!(cloned.target_state, "specified");
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("AdvanceFeatureCmd"));
    }

    #[test]
    fn create_story_cmd_clone_and_debug() {
        let cmd = CreateStoryCmd {
            epic_id: 1,
            project_id: 2,
            title: "Login".into(),
            points: Some(5),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.points, Some(5));
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("CreateStoryCmd"));
    }

    #[test]
    fn story_created_output_clone_and_debug() {
        let mut story = agileplus_domain::domain::story::Story::new(1, 2, "Login", Some(3)).unwrap();
        story.id = 10;
        let out = StoryCreatedOutput {
            id: 10,
            story,
        };
        let cloned = out.clone();
        assert_eq!(cloned.id, 10);
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("StoryCreatedOutput"));
    }

    #[test]
    fn transition_story_cmd_clone_and_debug() {
        let cmd = TransitionStoryCmd {
            story_id: 7,
            target_status: agileplus_domain::domain::story::StoryStatus::InProgress,
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.story_id, 7);
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("TransitionStoryCmd"));
    }

    #[test]
    fn create_epic_cmd_clone_and_debug() {
        let cmd = CreateEpicCmd {
            project_id: 3,
            title: "Auth Epic".into(),
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.project_id, 3);
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("CreateEpicCmd"));
    }

    #[test]
    fn epic_created_output_clone_and_debug() {
        let out = EpicCreatedOutput { id: 99 };
        let cloned = out.clone();
        assert_eq!(cloned.id, 99);
        let dbg = format!("{:?}", cloned);
        assert!(dbg.contains("EpicCreatedOutput"));
    }

    // ── DuplicateCandidate re-export ──────────────────────────────────────────

    #[test]
    fn duplicate_candidate_can_be_constructed() {
        // Verify the re-exported type from agileplus_triage is usable
        let candidate = DuplicateCandidate {
            a_id: "wp-1".into(),
            b_id: "wp-2".into(),
            hybrid_score: 0.85,
            token_jaccard: 0.9,
            fuzzy_ratio: 0.8,
            ngram_jaccard: 0.7,
            simhash_distance: 3,
        };
        assert!((candidate.hybrid_score - 0.85).abs() < f64::EPSILON);
        assert_eq!(candidate.a_id, "wp-1");
        assert_eq!(candidate.b_id, "wp-2");
    }
}
