//! RPC handler tests for `DispatchCommand`.
//!
//! Exercises the agent-proxy branch, every core command's state transition,
//! the governance gate applied to `validate`, and the rejection paths
//! (unknown command, unknown feature, invalid transition, missing scope).
//!
//! Traceability: WP14-T079, T080b

mod support;

use agileplus_domain::domain::governance::EvidenceType;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::WpState;
use agileplus_domain::ports::StoragePort;
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreService;
use agileplus_proto::agileplus::v1::{CommandResponse, DispatchCommandRequest};
use support::{Harness, command_request, rule, scope};
use tonic::{Code, Request, Status};

/// Dispatch `command` against `slug` and return the response payload.
async fn dispatch(
    harness: &Harness,
    command: &str,
    slug: &str,
    args: &[(&str, &str)],
) -> Result<CommandResponse, Status> {
    harness
        .server
        .dispatch_command(Request::new(DispatchCommandRequest {
            command: Some(command_request(command, slug, args)),
            project_scope: scope(),
        }))
        .await
        .map(|response| {
            response
                .into_inner()
                .result
                .expect("command response payload should be present")
        })
}

// ---------------------------------------------------------------------------
// Request validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_command_without_a_command_field_is_invalid_argument() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .dispatch_command(Request::new(DispatchCommandRequest {
            command: None,
            project_scope: scope(),
        }))
        .await
        .expect_err("a command-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("command"));
}

#[tokio::test]
async fn dispatch_command_requires_project_scope() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let status = harness
        .server
        .dispatch_command(Request::new(DispatchCommandRequest {
            command: Some(command_request("specify", "alpha", &[])),
            project_scope: None,
        }))
        .await
        .expect_err("a scope-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn dispatch_command_unknown_command_is_unimplemented() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let status = dispatch(&harness, "deploy", "alpha", &[])
        .await
        .expect_err("an unknown command must be rejected");

    assert_eq!(status.code(), Code::Unimplemented);
    assert!(status.message().contains("deploy"));
}

#[tokio::test]
async fn dispatch_command_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = dispatch(&harness, "specify", "ghost", &[])
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

// ---------------------------------------------------------------------------
// Agent-proxied commands
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_command_implement_is_never_acknowledged_without_the_agents_service() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Planned).await;

    let result = dispatch(&harness, "implement", "alpha", &[("wp", "1")])
        .await
        .expect("the agent proxy path returns a payload, not a status");

    assert!(
        !result.success,
        "an unavailable agents service must not report success"
    );
    assert!(result.message.contains("stub"));
    assert!(result.message.contains("implement"));
    assert!(result.message.contains("alpha"));
    assert!(result.outputs.is_empty());

    // The agent path must not mutate the feature itself.
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Planned
    );
}

// ---------------------------------------------------------------------------
// Core commands: happy paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_command_specify_persists_the_new_state_and_echoes_args() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let result = dispatch(&harness, "specify", "alpha", &[("author", "koosha")])
        .await
        .expect("specify should succeed");

    assert!(result.success);
    assert!(result.message.contains("specify"));
    assert!(result.message.contains("alpha"));
    assert!(result.message.contains("specified"));
    assert_eq!(
        result.outputs.get("state").map(String::as_str),
        Some("specified")
    );
    assert_eq!(
        result.outputs.get("author").map(String::as_str),
        Some("koosha")
    );
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Specified
    );
}

#[tokio::test]
async fn dispatch_command_drives_the_feature_through_the_whole_lifecycle() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Created).await;

    let steps = [
        ("specify", "specified"),
        ("research", "researched"),
        ("plan", "planned"),
    ];
    for (command, expected_state) in steps {
        let result = dispatch(&harness, command, "alpha", &[])
            .await
            .unwrap_or_else(|status| panic!("{command} failed: {status:?}"));
        assert!(result.success, "{command} should report success");
        assert!(result.message.contains(expected_state));
        assert_eq!(
            harness.persisted_feature("alpha").await.state.to_string(),
            expected_state
        );
    }

    // `implement` is an agent command, so the lifecycle jumps straight from
    // planned to implementing once the agent completes.
    harness
        .set_feature_state(feature.id, FeatureState::Implementing)
        .await;

    for (command, expected_state) in [
        ("validate", "validated"),
        ("ship", "shipped"),
        ("retrospective", "retrospected"),
    ] {
        let result = dispatch(&harness, command, "alpha", &[])
            .await
            .unwrap_or_else(|status| panic!("{command} failed: {status:?}"));
        assert!(result.success, "{command} should report success");
        assert!(result.message.contains(expected_state));
        assert_eq!(
            harness.persisted_feature("alpha").await.state.to_string(),
            expected_state
        );
    }
}

#[tokio::test]
async fn dispatch_command_plan_binds_a_default_governance_contract() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Researched)
        .await;

    let result = dispatch(&harness, "plan", "alpha", &[])
        .await
        .expect("plan should succeed");
    assert!(result.success);

    let contract = harness
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .expect("contract lookup should succeed")
        .expect("planning should bind a contract");

    assert_eq!(contract.feature_id, feature.id);
    assert_eq!(contract.version, 1);
    assert_eq!(contract.rules.len(), 1);
    assert!(contract.rules[0].transition.is_empty());
    assert!(contract.rules[0].required_evidence.is_empty());
}

#[tokio::test]
async fn dispatch_command_validate_skips_the_gate_when_no_contract_exists() {
    let harness = Harness::new().await;
    harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;

    let result = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect("validate should succeed without a contract");

    assert!(result.success);
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Validated
    );
}

// ---------------------------------------------------------------------------
// Core commands: rejection paths
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_command_rejects_an_invalid_transition() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let status = dispatch(&harness, "ship", "alpha", &[])
        .await
        .expect_err("shipping a created feature must fail");

    assert_eq!(status.code(), Code::FailedPrecondition);
    assert!(status.message().contains("invalid transition"));
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Created,
        "a rejected transition must not be persisted"
    );
}

#[tokio::test]
async fn dispatch_command_rejects_a_repeated_transition() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Specified).await;

    let status = dispatch(&harness, "specify", "alpha", &[])
        .await
        .expect_err("re-applying specify must fail");

    assert_eq!(status.code(), Code::FailedPrecondition);
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Specified
    );
}

#[tokio::test]
async fn dispatch_command_validate_blocks_when_required_evidence_is_missing() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-7:test_result"])])
        .await;

    let status = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect_err("a governance violation must block validate");

    assert_eq!(status.code(), Code::FailedPrecondition);
    assert!(status.message().contains("governance violation"));
    assert!(status.message().contains("FR-7:test_result"));
    // Observed ordering: `dispatch_core_command` persists the new state before it
    // evaluates the governance gate, so a gate failure still leaves the feature
    // in `validated`. Pinned here so the behaviour change is noticed when the
    // ordering is fixed (defect report: WP14-T079 follow-up).
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Validated
    );
}

#[tokio::test]
async fn dispatch_command_validate_blocks_on_an_unrecognized_evidence_type() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_evidence(wp.id, "FR-7", EvidenceType::TestResult)
        .await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-7:scan"])])
        .await;

    let status = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect_err("an undefined evidence type cannot be satisfied");

    assert_eq!(status.code(), Code::FailedPrecondition);
    assert!(status.message().contains("FR-7:scan"));
}

#[tokio::test]
async fn dispatch_command_validate_blocks_on_evidence_from_another_feature() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let other = harness
        .seed_feature("beta", FeatureState::Implementing)
        .await;
    let other_wp = harness.seed_wp(other.id, 1, WpState::Done).await;
    harness
        .seed_evidence(other_wp.id, "FR-7", EvidenceType::TestResult)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-7:test_result"])])
        .await;

    let status = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect_err("evidence from another feature must not satisfy the contract");

    assert_eq!(status.code(), Code::FailedPrecondition);
    assert!(status.message().contains("governance violation"));
}

#[tokio::test]
async fn dispatch_command_validate_ignores_contract_rules_for_other_transitions() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_contract(feature.id, 1, vec![rule("ship", &["FR-7:test_result"])])
        .await;

    let result = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect("a ship-scoped rule must not block validate");

    assert!(result.success);
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Validated
    );
}

#[tokio::test]
async fn dispatch_command_validate_accepts_evidence_from_a_feature_work_package() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Implementing)
        .await;
    let wp = harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness
        .seed_evidence(wp.id, "FR-7", EvidenceType::TestResult)
        .await;
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-7:test_result"])])
        .await;

    let result = dispatch(&harness, "validate", "alpha", &[])
        .await
        .expect("matching evidence should satisfy the gate");

    assert!(result.success);
    assert_eq!(
        result.outputs.get("state").map(String::as_str),
        Some("validated")
    );
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Validated
    );
}

#[tokio::test]
async fn dispatch_command_plan_keeps_an_existing_contract_when_binding_fails() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("alpha", FeatureState::Researched)
        .await;
    // Version 1 already exists, so binding the default contract hits the
    // unique (feature_id, version) constraint. The command must degrade
    // gracefully instead of failing the transition.
    harness
        .seed_contract(feature.id, 1, vec![rule("validate", &["FR-1:test_result"])])
        .await;

    let result = dispatch(&harness, "plan", "alpha", &[])
        .await
        .expect("plan should still succeed");

    assert!(result.success);
    assert_eq!(
        harness.persisted_feature("alpha").await.state,
        FeatureState::Planned
    );

    let contract = harness
        .storage
        .get_latest_governance_contract(feature.id)
        .await
        .expect("contract lookup should succeed")
        .expect("the pre-existing contract should remain");
    assert_eq!(contract.version, 1);
    assert_eq!(contract.rules.len(), 1);
    assert_eq!(contract.rules[0].transition, "validate");
    assert_eq!(
        contract.rules[0].required_evidence,
        vec!["FR-1:test_result"]
    );
}

#[tokio::test]
async fn dispatch_command_implement_is_forwarded_when_the_agents_service_is_reachable() {
    // A bound listener is enough: the router only probes TCP reachability.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let port = listener
        .local_addr()
        .expect("listener address should be readable")
        .port();
    let harness = Harness::with_downstream(Some(format!("127.0.0.1:{port}")), None).await;
    harness.seed_feature("alpha", FeatureState::Planned).await;

    let result = dispatch(&harness, "implement", "alpha", &[("wp", "3")])
        .await
        .expect("a reachable agents service should produce a payload");

    assert!(result.success, "a forwarded command reports success");
    assert!(result.message.contains("forwarded"));
    assert!(result.message.contains("implement"));
    assert!(result.message.contains("alpha"));
    // Forwarded dispatches echo the caller's arguments and add no state key.
    assert_eq!(result.outputs.get("wp").map(String::as_str), Some("3"));
    assert!(!result.outputs.contains_key("state"));
    assert_eq!(result.outputs.len(), 1);
    drop(listener);
}
