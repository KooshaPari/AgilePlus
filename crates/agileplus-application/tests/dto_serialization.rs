// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for DTO serialization, construction, and traits.

use agileplus_application::dto::*;
use agileplus_domain::domain::story::StoryStatus;
use agileplus_triage::claim::ClaimState;
use serde_json;

// ── PickRequest ─────────────────────────────────────────────────────────────

#[test]
fn pick_request_roundtrip_with_all_fields() {
    let req = PickRequest {
        agent_id: "agent-42".into(),
        limit: 10,
        lane: Some("backend".into()),
        category: Some("bug".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: PickRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.agent_id, "agent-42");
    assert_eq!(back.limit, 10);
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

#[test]
fn pick_request_clone_and_debug() {
    let req = PickRequest {
        agent_id: "x".into(),
        limit: 5,
        lane: None,
        category: None,
    };
    let cloned = req.clone();
    assert_eq!(cloned.agent_id, "x");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("PickRequest"));
}

// ── PickedItem ──────────────────────────────────────────────────────────────

#[test]
fn picked_item_roundtrip_with_deps() {
    let item = PickedItem {
        wp_id: "wp-10".into(),
        title: "Implement auth".into(),
        state: "ready".into(),
        dependencies: vec!["wp-0".into(), "wp-5".into()],
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: PickedItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.wp_id, "wp-10");
    assert_eq!(back.dependencies, vec!["wp-0", "wp-5"]);
    assert_eq!(back.title, "Implement auth");
}

#[test]
fn picked_item_empty_deps_roundtrip() {
    let item = PickedItem {
        wp_id: "wp-99".into(),
        title: "Standalone".into(),
        state: "done".into(),
        dependencies: vec![],
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: PickedItem = serde_json::from_str(&json).unwrap();
    assert!(back.dependencies.is_empty());
}

#[test]
fn picked_item_clone_and_debug() {
    let item = PickedItem {
        wp_id: "wp-1".into(),
        title: "T".into(),
        state: "s".into(),
        dependencies: vec![],
    };
    let cloned = item.clone();
    assert_eq!(cloned.wp_id, "wp-1");
    let dbg = format!("{:?}", item);
    assert!(dbg.contains("PickedItem"));
}

// ── ClaimRequest ────────────────────────────────────────────────────────────

#[test]
fn claim_request_roundtrip_with_reason() {
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
fn claim_request_default_reason_on_missing_field() {
    let json = r#"{"claim_id":"c-2","resource":"branch-x","kind":"branch","agent_id":"a","ttl_seconds":60}"#;
    let req: ClaimRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.reason, ClaimReason::default());
}

#[test]
fn claim_request_various_kinds() {
    for kind in [ClaimKind::Worktree, ClaimKind::Branch, ClaimKind::Repo, ClaimKind::Subproject] {
        let req = ClaimRequest {
            claim_id: "c".into(),
            resource: "r".into(),
            kind,
            agent_id: "a".into(),
            ttl_seconds: 10,
            reason: ClaimReason::default(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ClaimRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.kind, kind);
    }
}

#[test]
fn claim_request_clone_and_debug() {
    let req = ClaimRequest {
        claim_id: "c".into(),
        resource: "r".into(),
        kind: ClaimKind::Worktree,
        agent_id: "a".into(),
        ttl_seconds: 10,
        reason: ClaimReason::default(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.claim_id, "c");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ClaimRequest"));
}

// ── HeartbeatRequest / ReleaseRequest ───────────────────────────────────────

#[test]
fn heartbeat_request_roundtrip() {
    let req = HeartbeatRequest {
        claim_id: "c-99".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: HeartbeatRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.claim_id, "c-99");
}

#[test]
fn heartbeat_request_clone_and_debug() {
    let req = HeartbeatRequest {
        claim_id: "c".into(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.claim_id, "c");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("HeartbeatRequest"));
}

#[test]
fn release_request_roundtrip() {
    let req = ReleaseRequest {
        claim_id: "c-1".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ReleaseRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.claim_id, "c-1");
}

#[test]
fn release_request_clone_and_debug() {
    let req = ReleaseRequest {
        claim_id: "c".into(),
    };
    let cloned = req.clone();
    assert_eq!(cloned.claim_id, "c");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ReleaseRequest"));
}

// ── DoneRequest ─────────────────────────────────────────────────────────────

#[test]
fn done_request_roundtrip_with_result() {
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
fn done_request_roundtrip_without_result() {
    let req = DoneRequest {
        claim_id: "c-4".into(),
        wp_id: "wp-8".into(),
        result: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: DoneRequest = serde_json::from_str(&json).unwrap();
    assert!(back.result.is_none());
}

#[test]
fn done_request_clone_and_debug() {
    let req = DoneRequest {
        claim_id: "c".into(),
        wp_id: "w".into(),
        result: None,
    };
    let cloned = req.clone();
    assert_eq!(cloned.claim_id, "c");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("DoneRequest"));
}

// ── DedupRequest ────────────────────────────────────────────────────────────

#[test]
fn dedup_request_roundtrip() {
    let req = DedupRequest {
        items: vec![
            ("a".into(), "b".into()),
            ("c".into(), "d".into()),
            ("e".into(), "f".into()),
        ],
        threshold: 0.75,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: DedupRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.items.len(), 3);
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
    assert!((back.threshold - 0.0).abs() < f64::EPSILON);
}

#[test]
fn dedup_request_clone_and_debug() {
    let req = DedupRequest {
        items: vec![("a".into(), "b".into())],
        threshold: 0.5,
    };
    let cloned = req.clone();
    assert_eq!(cloned.items.len(), 1);
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("DedupRequest"));
}

// ── ScanRequest ─────────────────────────────────────────────────────────────

#[test]
fn scan_request_roundtrip_with_depth() {
    let req = ScanRequest {
        roots: vec!["/a".into(), "/b".into(), "/c".into()],
        max_depth: Some(5),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ScanRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.roots.len(), 3);
    assert_eq!(back.max_depth, Some(5));
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
    assert_eq!(back.roots.len(), 1);
}

#[test]
fn scan_request_clone_and_debug() {
    let req = ScanRequest {
        roots: vec![],
        max_depth: None,
    };
    let cloned = req.clone();
    assert!(cloned.roots.is_empty());
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ScanRequest"));
}

// ── TopologyRequest ─────────────────────────────────────────────────────────

#[test]
fn topology_request_roundtrip_with_root() {
    let req = TopologyRequest {
        root_wp: Some("wp-1".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: TopologyRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.root_wp.as_deref(), Some("wp-1"));
}

#[test]
fn topology_request_roundtrip_no_root() {
    let req = TopologyRequest { root_wp: None };
    let json = serde_json::to_string(&req).unwrap();
    let back: TopologyRequest = serde_json::from_str(&json).unwrap();
    assert!(back.root_wp.is_none());
}

#[test]
fn topology_request_clone_and_debug() {
    let req = TopologyRequest { root_wp: None };
    let cloned = req.clone();
    assert!(cloned.root_wp.is_none());
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("TopologyRequest"));
}

// ── ExportRequest / ExportFormat ─────────────────────────────────────────────

#[test]
fn export_request_roundtrip() {
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

#[test]
fn export_request_no_output_path() {
    let req = ExportRequest {
        format: ExportFormat::Csv,
        output_path: None,
        with_side: false,
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: ExportRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.format, ExportFormat::Csv);
    assert!(!back.with_side);
    assert!(back.output_path.is_none());
}

#[test]
fn export_request_clone_and_debug() {
    let req = ExportRequest {
        format: ExportFormat::Html,
        output_path: None,
        with_side: false,
    };
    let cloned = req.clone();
    assert_eq!(cloned.format, ExportFormat::Html);
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("ExportRequest"));
}

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
fn export_format_snake_case_serialization() {
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

#[test]
fn export_format_equality_and_copy() {
    let a = ExportFormat::Markdown;
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn export_format_debug() {
    let dbg = format!("{:?}", ExportFormat::Mermaid);
    assert_eq!(dbg, "Mermaid");
}

// ── WhereRequest / WhereResponse ────────────────────────────────────────────

#[test]
fn where_request_roundtrip() {
    let req = WhereRequest {
        cwd: "/home/user/project".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: WhereRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cwd, "/home/user/project");
}

#[test]
fn where_request_clone_and_debug() {
    let req = WhereRequest { cwd: "/x".into() };
    let cloned = req.clone();
    assert_eq!(cloned.cwd, "/x");
    let dbg = format!("{:?}", req);
    assert!(dbg.contains("WhereRequest"));
}

#[test]
fn where_response_roundtrip_with_data() {
    let resp = WhereResponse {
        repo: None,
        active_claims: vec![],
        lane: Some("backend".into()),
        category: Some("infra".into()),
        next_pickable: vec![
            PickedItem {
                wp_id: "wp-1".into(),
                title: "T1".into(),
                state: "ready".into(),
                dependencies: vec![],
            },
            PickedItem {
                wp_id: "wp-2".into(),
                title: "T2".into(),
                state: "blocked".into(),
                dependencies: vec!["wp-1".into()],
            },
        ],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: WhereResponse = serde_json::from_str(&json).unwrap();
    assert!(back.repo.is_none());
    assert_eq!(back.next_pickable.len(), 2);
    assert_eq!(back.lane.as_deref(), Some("backend"));
    assert_eq!(back.category.as_deref(), Some("infra"));
    assert_eq!(back.next_pickable[1].dependencies, vec!["wp-1"]);
}

#[test]
fn where_response_empty_lists() {
    let resp = WhereResponse {
        repo: None,
        active_claims: vec![],
        lane: None,
        category: None,
        next_pickable: vec![],
    };
    let json = serde_json::to_string(&resp).unwrap();
    let back: WhereResponse = serde_json::from_str(&json).unwrap();
    assert!(back.active_claims.is_empty());
    assert!(back.next_pickable.is_empty());
    assert!(back.lane.is_none());
    assert!(back.category.is_none());
}

#[test]
fn where_response_clone_and_debug() {
    let resp = WhereResponse {
        repo: None,
        active_claims: vec![],
        lane: None,
        category: None,
        next_pickable: vec![],
    };
    let cloned = resp.clone();
    assert!(cloned.next_pickable.is_empty());
    let dbg = format!("{:?}", resp);
    assert!(dbg.contains("WhereResponse"));
}

// ── Domain command DTOs (Clone + Debug, no Serialize) ──────────────────────

#[test]
fn create_feature_cmd_clone_debug() {
    let cmd = CreateFeatureCmd {
        slug: "auth".into(),
        friendly_name: "Auth".into(),
        spec_hash: Some([42u8; 32]),
        target_branch: Some("feat/auth".into()),
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.slug, "auth");
    assert_eq!(cloned.spec_hash, Some([42u8; 32]));
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("CreateFeatureCmd"));
    assert!(dbg.contains("auth"));
}

#[test]
fn create_feature_cmd_none_defaults() {
    let cmd = CreateFeatureCmd {
        slug: "s".into(),
        friendly_name: "f".into(),
        spec_hash: None,
        target_branch: None,
    };
    assert!(cmd.spec_hash.is_none());
    assert!(cmd.target_branch.is_none());
}

#[test]
fn feature_created_output_clone_debug() {
    let feature = agileplus_domain::domain::feature::Feature::new(
        "test-slug",
        "Test Feature",
        [0u8; 32],
        None,
    );
    let out = FeatureCreatedOutput {
        id: 42,
        feature,
    };
    let cloned = out.clone();
    assert_eq!(cloned.id, 42);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("FeatureCreatedOutput"));
}

#[test]
fn advance_feature_cmd_clone_debug() {
    let cmd = AdvanceFeatureCmd {
        feature_id: 99,
        target_state: "specified".into(),
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.feature_id, 99);
    assert_eq!(cloned.target_state, "specified");
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("AdvanceFeatureCmd"));
}

#[test]
fn create_story_cmd_clone_debug() {
    let cmd = CreateStoryCmd {
        epic_id: 1,
        project_id: 2,
        title: "Login".into(),
        points: Some(5),
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.points, Some(5));
    assert_eq!(cloned.epic_id, 1);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("CreateStoryCmd"));
}

#[test]
fn create_story_cmd_no_points() {
    let cmd = CreateStoryCmd {
        epic_id: 1,
        project_id: 2,
        title: "Task".into(),
        points: None,
    };
    let cloned = cmd.clone();
    assert!(cloned.points.is_none());
}

#[test]
fn story_created_output_clone_debug() {
    let mut story =
        agileplus_domain::domain::story::Story::new(1, 2, "Login", Some(3)).unwrap();
    story.id = 10;
    let out = StoryCreatedOutput { id: 10, story };
    let cloned = out.clone();
    assert_eq!(cloned.id, 10);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("StoryCreatedOutput"));
}

#[test]
fn transition_story_cmd_clone_debug() {
    let cmd = TransitionStoryCmd {
        story_id: 7,
        target_status: StoryStatus::InProgress,
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.story_id, 7);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("TransitionStoryCmd"));
}

#[test]
fn transition_story_cmd_all_status_variants() {
    let statuses = [
        StoryStatus::Todo,
        StoryStatus::InProgress,
        StoryStatus::Done,
    ];
    for status in statuses {
        let cmd = TransitionStoryCmd {
            story_id: 1,
            target_status: status,
        };
        assert_eq!(cmd.target_status, status);
    }
}

#[test]
fn create_epic_cmd_clone_debug() {
    let cmd = CreateEpicCmd {
        project_id: 3,
        title: "Auth Epic".into(),
    };
    let cloned = cmd.clone();
    assert_eq!(cloned.project_id, 3);
    assert_eq!(cloned.title, "Auth Epic");
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("CreateEpicCmd"));
}

#[test]
fn epic_created_output_clone_debug() {
    let out = EpicCreatedOutput { id: 99 };
    let cloned = out.clone();
    assert_eq!(cloned.id, 99);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("EpicCreatedOutput"));
}

// ── DuplicateCandidate re-export ────────────────────────────────────────────

#[test]
fn duplicate_candidate_construction_and_fields() {
    let c = DuplicateCandidate {
        a_id: "wp-1".into(),
        b_id: "wp-2".into(),
        hybrid_score: 0.85,
        token_jaccard: 0.9,
        fuzzy_ratio: 0.8,
        ngram_jaccard: 0.7,
        simhash_distance: 3,
    };
    assert_eq!(c.a_id, "wp-1");
    assert_eq!(c.b_id, "wp-2");
    assert!((c.hybrid_score - 0.85).abs() < f64::EPSILON);
    assert!((c.token_jaccard - 0.9).abs() < f64::EPSILON);
    assert_eq!(c.simhash_distance, 3);
}

#[test]
fn duplicate_candidate_clone_debug() {
    let c = DuplicateCandidate {
        a_id: "a".into(),
        b_id: "b".into(),
        hybrid_score: 0.5,
        token_jaccard: 0.5,
        fuzzy_ratio: 0.5,
        ngram_jaccard: 0.5,
        simhash_distance: 0,
    };
    let cloned = c.clone();
    assert_eq!(cloned.a_id, "a");
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("DuplicateCandidate"));
}

// ── ClaimKind / ClaimReason / ClaimState re-exports ────────────────────────

#[test]
fn claim_kind_variants_debug_and_clone() {
    let kinds = [
        ClaimKind::Worktree,
        ClaimKind::Branch,
        ClaimKind::Repo,
        ClaimKind::Subproject,
    ];
    for kind in kinds {
        let dbg = format!("{:?}", kind);
        assert!(!dbg.is_empty());
        // ClaimKind should be Copy or Clone
        let _cloned = kind;
    }
}

#[test]
fn claim_reason_debug() {
    let reason = ClaimReason::TaskRef("wp-1".into());
    let dbg = format!("{:?}", reason);
    assert!(dbg.contains("TaskRef"));
}

#[test]
fn claim_state_default() {
    let state = ClaimState::Active;
    let dbg = format!("{:?}", state);
    assert!(!dbg.is_empty());
}
