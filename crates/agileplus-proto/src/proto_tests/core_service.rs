// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::*;
use prost::Message;

// ── core.proto ───────────────────────────────────────────────────────────────

roundtrip!(
    rt_get_feature_request,
    GetFeatureRequest,
    GetFeatureRequest {
        slug: "f".to_string(),
        project_scope: Some(ProjectScope {
            canonical_repo_root: "/r".to_string(),
        }),
    }
);

roundtrip!(
    rt_get_feature_response,
    GetFeatureResponse,
    GetFeatureResponse {
        feature: Some(Feature {
            id: 1,
            slug: "f".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_list_features_request,
    ListFeaturesRequest,
    ListFeaturesRequest {
        state_filter: "Shipped".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_list_features_response,
    ListFeaturesResponse,
    ListFeaturesResponse {
        features: vec![Feature {
            id: 1,
            slug: "a".to_string(),
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_get_feature_state_request,
    GetFeatureStateRequest,
    GetFeatureStateRequest {
        slug: "s".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_get_feature_state_response,
    GetFeatureStateResponse,
    GetFeatureStateResponse {
        feature_state: Some(FeatureState {
            state: "Created".to_string(),
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_list_work_packages_request,
    ListWorkPackagesRequest,
    ListWorkPackagesRequest {
        feature_slug: "f".to_string(),
        state_filter: "Doing".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_list_work_packages_response,
    ListWorkPackagesResponse,
    ListWorkPackagesResponse {
        packages: vec![WorkPackageStatus {
            id: 1,
            sequence: 1,
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_get_work_package_status_request,
    GetWorkPackageStatusRequest,
    GetWorkPackageStatusRequest {
        feature_slug: "f".to_string(),
        wp_sequence: 4,
        project_scope: None,
    }
);

roundtrip!(
    rt_get_work_package_status_response,
    GetWorkPackageStatusResponse,
    GetWorkPackageStatusResponse {
        work_package_status: Some(WorkPackageStatus {
            id: 4,
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_check_governance_gate_request,
    CheckGovernanceGateRequest,
    CheckGovernanceGateRequest {
        feature_slug: "f".to_string(),
        transition: "planned->implementing".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_check_governance_gate_response,
    CheckGovernanceGateResponse,
    CheckGovernanceGateResponse {
        passed: false,
        violations: vec![GateViolation {
            fr_id: "FR-2".to_string(),
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_get_audit_trail_request,
    GetAuditTrailRequest,
    GetAuditTrailRequest {
        feature_slug: "f".to_string(),
        after_id: 1234,
        project_scope: None,
    }
);

roundtrip!(
    rt_get_audit_trail_response,
    GetAuditTrailResponse,
    GetAuditTrailResponse {
        audit_entry: Some(AuditEntry {
            id: 9,
            hash: vec![1, 2, 3],
            ..Default::default()
        }),
    }
);

roundtrip!(
    rt_verify_audit_chain_request,
    VerifyAuditChainRequest,
    VerifyAuditChainRequest {
        feature_slug: "f".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_verify_audit_chain_response,
    VerifyAuditChainResponse,
    VerifyAuditChainResponse {
        valid: false,
        entries_verified: 10,
        first_invalid_id: "7".to_string(),
        error_message: "hash mismatch".to_string(),
    }
);

roundtrip!(
    rt_dispatch_command_request,
    DispatchCommandRequest,
    DispatchCommandRequest {
        command: Some(CommandRequest {
            command: "validate".to_string(),
            feature_slug: "f".to_string(),
            args: Default::default(),
        }),
        project_scope: Some(ProjectScope {
            canonical_repo_root: "/r".to_string(),
        }),
    }
);

roundtrip!(
    rt_dispatch_command_response,
    DispatchCommandResponse,
    DispatchCommandResponse {
        result: Some(CommandResponse {
            success: true,
            message: "done".to_string(),
            outputs: Default::default(),
        }),
    }
);

roundtrip!(
    rt_stream_agent_events_request,
    StreamAgentEventsRequest,
    StreamAgentEventsRequest {
        feature_slug: "f".to_string(),
        project_scope: None,
    }
);

roundtrip!(
    rt_stream_agent_events_response,
    StreamAgentEventsResponse,
    StreamAgentEventsResponse {
        event: Some(AgentEvent {
            event_type: "tick".to_string(),
            ..Default::default()
        }),
    }
);

