// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::*;
use prost::Message;

// ── integrations.proto ───────────────────────────────────────────────────────

roundtrip!(
    rt_sync_feature_to_plane_request,
    SyncFeatureToPlaneRequest,
    SyncFeatureToPlaneRequest {
        feature_slug: "f".to_string(),
        state: "Shipped".to_string(),
    }
);

roundtrip!(
    rt_sync_feature_to_plane_response,
    SyncFeatureToPlaneResponse,
    SyncFeatureToPlaneResponse {
        success: true,
        mirror_id: "m-1".to_string(),
        mirror_url: "https://plane.so/m-1".to_string(),
        message: "synced".to_string(),
    }
);

roundtrip!(
    rt_sync_wp_to_plane_request,
    SyncWpToPlaneRequest,
    SyncWpToPlaneRequest {
        feature_slug: "f".to_string(),
        wp_sequence: 2,
        state: "Done".to_string(),
        pr_url: "https://github.com/o/r/pull/2".to_string(),
    }
);

roundtrip!(
    rt_sync_wp_to_plane_response,
    SyncWpToPlaneResponse,
    SyncWpToPlaneResponse {
        success: false,
        mirror_id: String::new(),
        mirror_url: String::new(),
        message: "conflict".to_string(),
    }
);

roundtrip!(
    rt_sync_bug_to_github_request,
    SyncBugToGitHubRequest,
    SyncBugToGitHubRequest {
        title: "bug".to_string(),
        body: "steps to repro".to_string(),
        labels: vec!["bug".to_string(), "p1".to_string()],
        feature_slug: "f".to_string(),
        wp_sequence: 1,
    }
);

roundtrip!(
    rt_sync_bug_to_github_response,
    SyncBugToGitHubResponse,
    SyncBugToGitHubResponse {
        success: true,
        mirror_id: "GH-1".to_string(),
        mirror_url: "https://github.com/o/r/issues/1".to_string(),
        message: "created".to_string(),
    }
);

roundtrip!(
    rt_sync_issue_status_request,
    SyncIssueStatusRequest,
    SyncIssueStatusRequest {
        mirror: "github".to_string(),
        mirror_id: "GH-1".to_string(),
    }
);

roundtrip!(
    rt_sync_issue_status_response,
    SyncIssueStatusResponse,
    SyncIssueStatusResponse {
        success: true,
        mirror_id: "GH-1".to_string(),
        mirror_url: "https://github.com/o/r/issues/1".to_string(),
        message: "closed".to_string(),
    }
);

roundtrip!(
    rt_detect_plane_conflicts_request,
    DetectPlaneConflictsRequest,
    DetectPlaneConflictsRequest {
        mirror: "plane".to_string(),
    }
);

roundtrip!(
    rt_detect_plane_conflicts_response,
    DetectPlaneConflictsResponse,
    DetectPlaneConflictsResponse {
        conflicts: vec![Conflict {
            entity_type: "feature".to_string(),
            entity_id: 1,
            mirror_id: "m-1".to_string(),
            description: "diverged".to_string(),
            local_hash: "aaa".to_string(),
            remote_hash: "bbb".to_string(),
        }],
    }
);

roundtrip!(
    rt_detect_github_conflicts_request,
    DetectGitHubConflictsRequest,
    DetectGitHubConflictsRequest {
        mirror: "github".to_string(),
    }
);

roundtrip!(
    rt_detect_github_conflicts_response,
    DetectGitHubConflictsResponse,
    DetectGitHubConflictsResponse {
        conflicts: Vec::new(),
    }
);

roundtrip!(
    rt_conflict,
    Conflict,
    Conflict {
        entity_type: "wp".to_string(),
        entity_id: 9,
        mirror_id: "m-9".to_string(),
        description: "hash mismatch".to_string(),
        local_hash: "111".to_string(),
        remote_hash: "222".to_string(),
    }
);

roundtrip!(
    rt_classify_input_request,
    ClassifyInputRequest,
    ClassifyInputRequest {
        input: "the build is broken".to_string(),
        feature_slug: "f".to_string(),
        wp_sequence: 1,
    }
);

roundtrip!(
    rt_classify_input_response,
    ClassifyInputResponse,
    ClassifyInputResponse {
        r#type: "bug".to_string(),
        confidence: "0.91".to_string(),
        suggested_title: "Fix build".to_string(),
        suggested_priority: "high".to_string(),
    }
);

roundtrip!(
    rt_create_backlog_item_request,
    CreateBacklogItemRequest,
    CreateBacklogItemRequest {
        r#type: "story".to_string(),
        title: "Add caching".to_string(),
        body: "details".to_string(),
        priority: "medium".to_string(),
        feature_id: "f".to_string(),
        wp_id: "1".to_string(),
        triaged_by: "jcode".to_string(),
    }
);

roundtrip!(
    rt_create_backlog_item_response,
    CreateBacklogItemResponse,
    CreateBacklogItemResponse {
        item: Some(BacklogItem {
            id: 1,
            r#type: "story".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_backlog_item,
    BacklogItem,
    BacklogItem {
        id: 77,
        r#type: "bug".to_string(),
        title: "Crash on start".to_string(),
        body: "stack trace".to_string(),
        priority: "high".to_string(),
        state: "open".to_string(),
        external_ref: "github:77".to_string(),
        created_at: "2024-08-01T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_list_backlog_request,
    ListBacklogRequest,
    ListBacklogRequest {
        type_filter: "bug".to_string(),
        state_filter: "open".to_string(),
        feature_slug: "f".to_string(),
    }
);

roundtrip!(
    rt_list_backlog_response,
    ListBacklogResponse,
    ListBacklogResponse {
        items: vec![BacklogItem {
            id: 1,
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_promote_backlog_item_request,
    PromoteBacklogItemRequest,
    PromoteBacklogItemRequest {
        backlog_item_id: 5,
        target_type: "feature".to_string(),
    }
);

roundtrip!(
    rt_promote_backlog_item_response,
    PromoteBacklogItemResponse,
    PromoteBacklogItemResponse {
        success: true,
        created_entity_id: "42".to_string(),
        message: "promoted".to_string(),
    }
);

roundtrip!(
    rt_generate_router_request,
    GenerateRouterRequest,
    GenerateRouterRequest {
        project_path: "/repo".to_string(),
        sub_commands: vec!["specify".to_string(), "plan".to_string()],
    }
);

roundtrip!(
    rt_generate_router_response,
    GenerateRouterResponse,
    GenerateRouterResponse {
        success: true,
        claude_md_path: "CLAUDE.md".to_string(),
        agents_md_path: "AGENTS.md".to_string(),
        message: "generated".to_string(),
    }
);

