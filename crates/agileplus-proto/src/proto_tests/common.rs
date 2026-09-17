// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::*;
use prost::Message;

// ── common.proto ─────────────────────────────────────────────────────────────

roundtrip!(
    rt_project_scope,
    ProjectScope,
    ProjectScope {
        canonical_repo_root: "/Users/dev/repo".to_string(),
    }
);

roundtrip!(
    rt_feature,
    Feature,
    Feature {
        id: 42,
        slug: "test-feature".to_string(),
        friendly_name: "Test Feature".to_string(),
        state: "Implementing".to_string(),
        target_branch: "feat/test".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
        wp_count: 5,
        wp_done: 2,
    }
);

roundtrip!(
    rt_feature_state,
    FeatureState,
    FeatureState {
        state: "Planned".to_string(),
        next_command: "implement".to_string(),
        blockers: vec!["missing spec".to_string(), "no owner".to_string()],
        governance: Some(GovernanceStatus {
            all_gates_passed: false,
            total_rules: 10,
            passed_rules: 8,
            outstanding: vec![GateViolation {
                fr_id: "FR-AGP-011".to_string(),
                rule_id: "gate-1".to_string(),
                message: "coverage below floor".to_string(),
                remediation: "add tests".to_string(),
            }],
        }),
    }
);

roundtrip!(
    rt_work_package_status,
    WorkPackageStatus,
    WorkPackageStatus {
        id: 7,
        title: "WP-7".to_string(),
        state: "Done".to_string(),
        sequence: 7,
        agent_id: "agent-7".to_string(),
        pr_url: "https://github.com/o/r/pull/7".to_string(),
        pr_state: "merged".to_string(),
        depends_on: vec![5, 6],
        file_scope: vec!["src/lib.rs".to_string(), "tests/it.rs".to_string()],
    }
);

roundtrip!(
    rt_gate_violation,
    GateViolation,
    GateViolation {
        fr_id: "FR-1".to_string(),
        rule_id: "R-1".to_string(),
        message: "boom".to_string(),
        remediation: "fix it".to_string(),
    }
);

roundtrip!(
    rt_governance_status,
    GovernanceStatus,
    GovernanceStatus {
        all_gates_passed: true,
        total_rules: 3,
        passed_rules: 3,
        outstanding: Vec::new(),
    }
);

roundtrip!(
    rt_audit_entry,
    AuditEntry,
    AuditEntry {
        id: 99,
        feature_slug: "audit-feature".to_string(),
        wp_sequence: 3,
        timestamp: "2024-05-05T12:00:00Z".to_string(),
        actor: "jcode".to_string(),
        transition: "validated->shipped".to_string(),
        evidence_refs: vec!["pr#1".to_string(), "ci-run#2".to_string()],
        prev_hash: vec![0xab; 32],
        hash: vec![0xcd; 32],
    }
);

roundtrip!(
    rt_command_request,
    CommandRequest,
    CommandRequest {
        command: "specify".to_string(),
        feature_slug: "cmd-feature".to_string(),
        args: [("force".to_string(), "true".to_string())]
            .into_iter()
            .collect(),
    }
);

roundtrip!(
    rt_command_response,
    CommandResponse,
    CommandResponse {
        success: true,
        message: "ok".to_string(),
        outputs: [("path".to_string(), "kitty-specs/x".to_string())]
            .into_iter()
            .collect(),
    }
);

roundtrip!(
    rt_agent_event,
    AgentEvent,
    AgentEvent {
        event_type: "wp.started".to_string(),
        feature_slug: "evt-feature".to_string(),
        wp_sequence: 2,
        agent_id: "agent-2".to_string(),
        payload: "{\"k\":1}".to_string(),
        timestamp: "2024-06-01T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_agent_event_ack,
    AgentEventAck,
    AgentEventAck {
        event_id: "evt-1".to_string(),
        acknowledged: true,
        instruction: "continue".to_string(),
    }
);

