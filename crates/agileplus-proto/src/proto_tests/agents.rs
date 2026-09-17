// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::*;
use prost::Message;

// ── agents.proto ─────────────────────────────────────────────────────────────

roundtrip!(
    rt_spawn_agent_request,
    SpawnAgentRequest,
    SpawnAgentRequest {
        feature_slug: "f".to_string(),
        wp_sequence: 1,
        harness: "jcode".to_string(),
        prompt_path: "prompts/wp1.md".to_string(),
        context_paths: vec!["AGENTS.md".to_string()],
        worktree_path: "AgilePlus-wtrees/wp1".to_string(),
        max_agents: 3,
    }
);

roundtrip!(
    rt_spawn_agent_response,
    SpawnAgentResponse,
    SpawnAgentResponse {
        success: true,
        agent_id: "agent-1".to_string(),
        message: "spawned".to_string(),
    }
);

roundtrip!(
    rt_get_agent_status_request,
    GetAgentStatusRequest,
    GetAgentStatusRequest {
        agent_id: "agent-1".to_string(),
    }
);

roundtrip!(
    rt_get_agent_status_response,
    GetAgentStatusResponse,
    GetAgentStatusResponse {
        status: Some(AgentStatus {
            agent_id: "agent-1".to_string(),
            state: "running".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_agent_status,
    AgentStatus,
    AgentStatus {
        agent_id: "agent-1".to_string(),
        state: "reviewing".to_string(),
        feature_slug: "f".to_string(),
        wp_sequence: 2,
        pr_url: "https://github.com/o/r/pull/2".to_string(),
        review_cycles: 3,
        last_activity: "2024-07-01T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_cancel_agent_request,
    CancelAgentRequest,
    CancelAgentRequest {
        agent_id: "agent-1".to_string(),
        reason: "superseded".to_string(),
    }
);

roundtrip!(
    rt_cancel_agent_response,
    CancelAgentResponse,
    CancelAgentResponse {
        success: true,
        message: "cancelled".to_string(),
    }
);

roundtrip!(
    rt_agent_events_request,
    AgentEventsRequest,
    AgentEventsRequest {
        event: Some(AgentEvent {
            event_type: "log".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_agent_events_response,
    AgentEventsResponse,
    AgentEventsResponse {
        ack: Some(AgentEventAck {
            event_id: "e".to_string(),
            acknowledged: true,
            instruction: "wait".to_string(),
        }),
    }
);

roundtrip!(
    rt_start_review_loop_request,
    StartReviewLoopRequest,
    StartReviewLoopRequest {
        feature_slug: "f".to_string(),
        wp_sequence: 1,
        pr_url: "https://github.com/o/r/pull/1".to_string(),
        agent_id: "agent-1".to_string(),
        max_cycles: 5,
    }
);

roundtrip!(
    rt_start_review_loop_response,
    StartReviewLoopResponse,
    StartReviewLoopResponse {
        success: true,
        review_loop_id: "loop-1".to_string(),
        message: "started".to_string(),
    }
);

roundtrip!(
    rt_get_review_status_request,
    GetReviewStatusRequest,
    GetReviewStatusRequest {
        review_loop_id: "loop-1".to_string(),
    }
);

roundtrip!(
    rt_get_review_status_response,
    GetReviewStatusResponse,
    GetReviewStatusResponse {
        status: Some(ReviewStatus {
            review_loop_id: "loop-1".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_review_status,
    ReviewStatus,
    ReviewStatus {
        review_loop_id: "loop-1".to_string(),
        state: "awaiting_ci".to_string(),
        cycle: 2,
        max_cycles: 5,
        pending_comments: vec!["fix lint".to_string()],
        ci_passing: false,
    }
);

