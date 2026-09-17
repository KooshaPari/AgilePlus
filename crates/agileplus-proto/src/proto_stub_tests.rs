// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for the hand-written stubs compiled when `protoc` is absent.
//!
//! The stubs are plain Rust structs (no `prost::Message`), so these tests cover
//! construction, defaults, equality/clone semantics, `Debug` formatting and the
//! `NamedService` wiring that the rest of the workspace depends on.
//!
//! Traceability: FR-AGP-011

use super::agileplus::v1::*;

#[test]
fn stub_feature_state_default_and_populated() {
    let default = FeatureState::default();
    assert_eq!(default.state, "");
    assert_eq!(default.next_command, "");
    assert!(default.blockers.is_empty());
    assert!(default.governance.is_none());

    let state = FeatureState {
        state: "Implementing".to_string(),
        next_command: "validate".to_string(),
        blockers: vec!["b1".to_string()],
        governance: Some(GovernanceStatus::default()),
    };
    assert_eq!(state, state.clone());
    assert_eq!(state.blockers.len(), 1);
}

#[test]
fn stub_governance_status_default_and_violations() {
    let status = GovernanceStatus::default();
    assert!(!status.all_gates_passed);
    assert_eq!(status.total_rules, 0);
    assert!(status.outstanding.is_empty());

    let status = GovernanceStatus {
        all_gates_passed: false,
        total_rules: 2,
        passed_rules: 1,
        outstanding: vec![GateViolation {
            fr_id: "FR-1".to_string(),
            rule_id: "R-1".to_string(),
            message: "m".to_string(),
            remediation: "fix".to_string(),
        }],
    };
    assert_eq!(status.outstanding[0].rule_id, "R-1");
}

#[test]
fn stub_gate_violation_fields() {
    let v = GateViolation {
        fr_id: "FR-AGP-011".to_string(),
        rule_id: "gate".to_string(),
        message: "msg".to_string(),
        remediation: "rem".to_string(),
    };
    assert_eq!(v.clone(), v);
    assert_eq!(GateViolation::default().fr_id, "");
}

#[test]
fn stub_command_response_hashmap() {
    use std::collections::HashMap;
    let mut outputs = HashMap::new();
    outputs.insert("k".to_string(), "v".to_string());
    let resp = CommandResponse {
        success: true,
        message: "ok".to_string(),
        outputs,
    };
    assert_eq!(resp.outputs.get("k"), Some(&"v".to_string()));
    assert!(CommandResponse::default().outputs.is_empty());
}

#[test]
fn stub_feature_default_and_debug() {
    let feature = Feature::default();
    assert_eq!(feature.id, 0);
    assert_eq!(feature.wp_count, 0);

    let feature = Feature {
        id: 7,
        slug: "slug".to_string(),
        friendly_name: "Name".to_string(),
        state: "Shipped".to_string(),
        target_branch: "main".to_string(),
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
        wp_count: 3,
        wp_done: 1,
    };
    let rendered = format!("{feature:?}");
    assert!(rendered.contains("slug"));
    assert_eq!(feature.clone(), feature);
}

#[test]
fn stub_core_request_response_wrappers() {
    let req = GetFeatureRequest {
        slug: "s".to_string(),
    };
    assert_eq!(req.slug, "s");

    let resp = GetFeatureResponse {
        feature: Some(Feature::default()),
    };
    assert!(resp.feature.is_some());

    let list_req = ListFeaturesRequest {
        state_filter: "Shipped".to_string(),
    };
    assert_eq!(list_req.state_filter, "Shipped");
    assert!(ListFeaturesResponse::default().features.is_empty());

    let state_req = GetFeatureStateRequest {
        slug: "s".to_string(),
    };
    assert_eq!(state_req.slug, "s");
    assert!(GetFeatureStateResponse::default().feature_state.is_none());
}

#[test]
fn stub_work_package_status_and_wrappers() {
    let wp = WorkPackageStatus {
        id: 1,
        title: "WP".to_string(),
        state: "Done".to_string(),
        sequence: 1,
        agent_id: "a".to_string(),
        pr_url: "u".to_string(),
        pr_state: "merged".to_string(),
        depends_on: vec![0],
        file_scope: vec!["src/lib.rs".to_string()],
    };
    assert_eq!(wp.clone(), wp);
    assert_eq!(wp.depends_on, vec![0]);

    let list_req = ListWorkPackagesRequest {
        feature_slug: "f".to_string(),
        state_filter: String::new(),
    };
    assert_eq!(list_req.feature_slug, "f");
    assert_eq!(
        ListWorkPackagesResponse {
            packages: vec![wp.clone()]
        }
        .packages
        .len(),
        1
    );

    let get_req = GetWorkPackageStatusRequest {
        feature_slug: "f".to_string(),
        wp_sequence: 2,
    };
    assert_eq!(get_req.wp_sequence, 2);
    assert!(GetWorkPackageStatusResponse::default()
        .work_package_status
        .is_none());
}

#[test]
fn stub_governance_gate_and_audit_types() {
    let req = CheckGovernanceGateRequest {
        feature_slug: "f".to_string(),
        transition: "t".to_string(),
    };
    assert_eq!(req.transition, "t");

    let resp = CheckGovernanceGateResponse {
        passed: true,
        violations: Vec::new(),
    };
    assert!(resp.passed);

    let audit_req = GetAuditTrailRequest {
        feature_slug: "f".to_string(),
        after_id: 10,
    };
    assert_eq!(audit_req.after_id, 10);

    let entry = AuditEntry {
        id: 1,
        feature_slug: "f".to_string(),
        wp_sequence: 1,
        timestamp: "t".to_string(),
        actor: "jcode".to_string(),
        transition: "a->b".to_string(),
        evidence_refs: vec!["e".to_string()],
        prev_hash: vec![0u8; 32],
        hash: vec![1u8; 32],
    };
    assert_eq!(entry.hash.len(), 32);
    assert_eq!(
        GetAuditTrailResponse {
            audit_entry: Some(entry.clone())
        }
        .audit_entry
        .unwrap(),
        entry
    );

    let verify_req = VerifyAuditChainRequest {
        feature_slug: "f".to_string(),
    };
    assert_eq!(verify_req.feature_slug, "f");
    let verify = VerifyAuditChainResponse {
        valid: false,
        entries_verified: 5,
        first_invalid_id: "2".to_string(),
        error_message: "bad".to_string(),
    };
    assert_eq!(verify.entries_verified, 5);
}

#[test]
fn stub_agent_event_and_stream_wrappers() {
    let event = AgentEvent {
        event_type: "tick".to_string(),
        feature_slug: "f".to_string(),
        wp_sequence: 1,
        agent_id: "a".to_string(),
        payload: "{}".to_string(),
        timestamp: "t".to_string(),
    };
    assert_eq!(event.clone(), event);

    let req = StreamAgentEventsRequest {
        feature_slug: "f".to_string(),
    };
    assert_eq!(req.feature_slug, "f");
    assert!(StreamAgentEventsResponse::default().event.is_none());
}

#[test]
fn stub_dispatch_command_types() {
    use std::collections::HashMap;
    let mut args = HashMap::new();
    args.insert("force".to_string(), "true".to_string());
    let cmd = DispatchCommand {
        command: "specify".to_string(),
        feature_slug: "f".to_string(),
        args,
    };
    assert_eq!(cmd.args.len(), 1);

    let req = DispatchCommandRequest {
        command: Some(cmd),
    };
    assert_eq!(req.command.as_ref().unwrap().command, "specify");

    let resp = DispatchCommandResponse {
        result: Some(CommandResponse::default()),
    };
    assert!(resp.result.is_some());
}

#[test]
fn stub_backlog_types() {
    let item = BacklogItem {
        id: 1,
        title: "T".to_string(),
        description: "D".to_string(),
        r#type: "bug".to_string(),
        priority: "high".to_string(),
        status: "open".to_string(),
        source: "github".to_string(),
        feature_slug: "f".to_string(),
        tags: vec!["bug".to_string()],
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };
    assert_eq!(item.r#type, "bug");
    assert_eq!(item.clone(), item);

    let create = CreateBacklogItemRequest {
        title: "T".to_string(),
        description: "D".to_string(),
        r#type: "bug".to_string(),
        priority: "high".to_string(),
        source: "github".to_string(),
        feature_slug: "f".to_string(),
        tags: vec![],
    };
    assert_eq!(create.priority, "high");

    assert!(CreateBacklogItemResponse::default().item.is_none());
    assert_eq!(
        ImportBacklogRequest {
            items: vec![create]
        }
        .items
        .len(),
        1
    );
    assert!(ImportBacklogResponse::default().items.is_empty());

    let get_req = GetBacklogItemRequest {
        backlog_item_id: 5,
    };
    assert_eq!(get_req.backlog_item_id, 5);
    assert!(GetBacklogItemResponse::default().item.is_none());

    let list = ListBacklogRequest {
        type_filter: "bug".to_string(),
        state_filter: "open".to_string(),
        priority_filter: "high".to_string(),
        feature_slug: "f".to_string(),
        source_filter: "github".to_string(),
        limit: 10,
        sort: "created".to_string(),
    };
    assert_eq!(list.limit, 10);
    assert!(ListBacklogResponse::default().items.is_empty());

    let update = UpdateBacklogStatusRequest {
        backlog_item_id: 1,
        target_status: "closed".to_string(),
    };
    assert_eq!(update.target_status, "closed");
    let updated = UpdateBacklogStatusResponse {
        backlog_item_id: 1,
        from_status: "open".to_string(),
        to_status: "closed".to_string(),
    };
    assert_eq!(updated.from_status, "open");

    let pop = PopBacklogRequest { count: 3 };
    assert_eq!(pop.count, 3);
    assert!(PopBacklogResponse::default().items.is_empty());
}

#[test]
fn stub_work_item_types() {
    assert_eq!(ListProjectsRequest::default(), ListProjectsRequest {});

    let project = ProjectProto {
        id: 1,
        slug: "s".to_string(),
        name: "n".to_string(),
        description: "d".to_string(),
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };
    assert_eq!(project.clone(), project);
    assert_eq!(
        ListProjectsResponse {
            projects: vec![project.clone()]
        }
        .projects
        .len(),
        1
    );

    assert_eq!(ListEpicsRequest { project_id: 1 }.project_id, 1);
    let epic = EpicProto {
        id: 1,
        project_id: 1,
        title: "t".to_string(),
        description: "d".to_string(),
        status: "Open".to_string(),
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };
    assert_eq!(epic.status, "Open");
    assert_eq!(ListEpicsResponse { epics: vec![epic] }.epics.len(), 1);

    assert_eq!(ListStoriesRequest { epic_id: 2 }.epic_id, 2);
    let story = StoryProto {
        id: 3,
        epic_id: 2,
        project_id: 1,
        title: "t".to_string(),
        description: "d".to_string(),
        status: "InProgress".to_string(),
        points: 5,
        requirement_id: "FR-1".to_string(),
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };
    assert_eq!(story.points, 5);
    assert_eq!(ListStoriesResponse { stories: vec![story] }.stories.len(), 1);

    let sync = SyncRepositoryRequest {
        repo: "acme/backend".to_string(),
        project_id: 1,
        epic_id: 2,
    };
    assert_eq!(sync.repo, "acme/backend");
    let sync_resp = SyncRepositoryResponse {
        stories_synced: 3,
        stories_skipped: 1,
        errors: vec!["e".to_string()],
    };
    assert_eq!(sync_resp.stories_synced, 3);
}

#[test]
fn stub_default_equality_is_reflexive() {
    assert_eq!(Feature::default(), Feature::default());
    assert_eq!(AgentEvent::default(), AgentEvent::default());
    assert_eq!(BacklogItem::default(), BacklogItem::default());
    assert_eq!(StoryProto::default(), StoryProto::default());
}

#[test]
fn stub_named_service_constants_match_proto_packages() {
    use tonic::server::NamedService;
    assert_eq!(
        IntegrationsServiceServer::<()>::NAME,
        "agileplus.v1.IntegrationsService"
    );
    let _ = IntegrationsServiceServer::new(());
}
