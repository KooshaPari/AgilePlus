//! tonic gRPC server implementing AgilePlusCoreService.
//!
//! Traceability: WP14-T079, T080

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(not(agileplus_proto_stubs))]
use tonic::transport::Server;
use tonic::{Request, Response, Status};
#[cfg(not(agileplus_proto_stubs))]
use tracing::info;

use agileplus_domain::domain::audit::AuditChain;
use agileplus_domain::domain::governance::{Evidence, EvidenceType, GovernanceRule};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{ContentStoragePort, StoragePort};
#[cfg(not(agileplus_proto_stubs))]
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreServiceServer;
use agileplus_proto::agileplus::v1::{
    CheckGovernanceGateRequest, CheckGovernanceGateResponse, CommandResponse,
    DispatchCommandRequest, DispatchCommandResponse, GateViolation as ProtoGateViolation,
    GetAuditTrailRequest, GetAuditTrailResponse, GetFeatureRequest, GetFeatureResponse,
    GetFeatureStateRequest, GetFeatureStateResponse, GetWorkPackageStatusRequest,
    GetWorkPackageStatusResponse, ListFeaturesRequest, ListFeaturesResponse,
    ListWorkPackagesRequest, ListWorkPackagesResponse, VerifyAuditChainRequest,
    VerifyAuditChainResponse, agile_plus_core_service_server::AgilePlusCoreService,
};

use crate::conversions::{audit_entry_to_proto, feature_to_proto, wp_to_proto};
use crate::event_bus::EventBus;
use crate::proxy::ProxyRouter;

pub mod integrations;

/// Map domain errors to gRPC Status codes consistently.
pub fn domain_error_to_status(e: agileplus_domain::error::DomainError) -> Status {
    use agileplus_domain::error::DomainError;
    match e {
        DomainError::NotFound(msg) => Status::not_found(msg),
        DomainError::InvalidTransition { from, to, reason } => {
            Status::failed_precondition(format!("invalid transition {from}->{to}: {reason}"))
        }
        DomainError::Conflict(msg) => Status::already_exists(msg),
        DomainError::NotImplemented => Status::unimplemented("not implemented"),
        other => Status::internal(other.to_string()),
    }
}

/// Parse the contract representation `FR-ID` or `FR-ID:evidence_type`.
fn parse_evidence_requirement(raw: &str) -> (&str, Option<EvidenceType>, bool) {
    let (fr_id, evidence_type) = raw
        .split_once(':')
        .map_or((raw, None), |(fr_id, kind)| (fr_id, Some(kind)));
    let had_type = evidence_type.is_some();
    let evidence_type = evidence_type.and_then(|kind| match kind {
        "test_result" => Some(EvidenceType::TestResult),
        "ci_output" => Some(EvidenceType::CiOutput),
        "review_approval" => Some(EvidenceType::ReviewApproval),
        "security_scan" => Some(EvidenceType::SecurityScan),
        "lint_result" => Some(EvidenceType::LintResult),
        "manual_attestation" => Some(EvidenceType::ManualAttestation),
        _ => None,
    });
    let recognized = !had_type || evidence_type.is_some();
    (fr_id, evidence_type, recognized)
}

fn evidence_satisfies_requirement(
    evidence: &[Evidence],
    feature_wp_ids: &HashSet<i64>,
    fr_id: &str,
    expected_type: Option<EvidenceType>,
) -> bool {
    evidence.iter().any(|candidate| {
        candidate.fr_id == fr_id
            && feature_wp_ids.contains(&candidate.wp_id)
            && expected_type.is_none_or(|kind| candidate.evidence_type == kind)
    })
}

fn missing_evidence_violation(
    rule: &GovernanceRule,
    fr_id: &str,
    requirement: &str,
) -> ProtoGateViolation {
    ProtoGateViolation {
        fr_id: fr_id.to_owned(),
        rule_id: rule.transition.clone(),
        message: format!("Missing required evidence for {requirement}"),
        remediation: format!("Provide evidence linked to {requirement}"),
    }
}

/// Compare transition identifiers while tolerating presentation-only spacing
/// around the planner's `->` separator. The WP prefix and state spelling stay
/// significant so a rule for one work package cannot match another.
fn governance_transition_matches(rule: &str, requested: &str) -> bool {
    fn normalized(value: &str) -> Option<(&str, &str)> {
        let (from, to) = value.split_once("->")?;
        let from = from.trim();
        let to = to.trim();
        (!from.is_empty() && !to.is_empty() && !to.contains("->")).then_some((from, to))
    }

    matches!(
        (normalized(rule), normalized(requested)),
        (Some(rule), Some(requested)) if rule == requested
    )
}

fn unsupported_core_command(command: &str) -> Status {
    Status::unimplemented(format!(
        "command '{command}' is not executable through the gRPC core"
    ))
}

/// gRPC server struct holding references to all port implementations.
pub struct AgilePlusCoreServer<S>
where
    S: StoragePort + 'static,
{
    storage: Arc<S>,
    #[allow(dead_code)] // reserved - injected for future event bus integration
    event_bus: Arc<EventBus>,
    proxy: Arc<ProxyRouter>,
}

impl<S> Clone for AgilePlusCoreServer<S>
where
    S: StoragePort + 'static,
{
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
            event_bus: Arc::clone(&self.event_bus),
            proxy: Arc::clone(&self.proxy),
        }
    }
}

impl<S> AgilePlusCoreServer<S>
where
    S: StoragePort + 'static,
{
    pub fn new(storage: Arc<S>, event_bus: Arc<EventBus>, proxy: Arc<ProxyRouter>) -> Self {
        Self {
            storage,
            event_bus,
            proxy,
        }
    }
}

#[tonic::async_trait]
impl<S> AgilePlusCoreService for AgilePlusCoreServer<S>
where
    S: StoragePort + 'static,
{
    // -------------------------------------------------------------------------
    // Feature RPCs
    // -------------------------------------------------------------------------

    async fn get_feature(
        &self,
        request: Request<GetFeatureRequest>,
    ) -> Result<Response<GetFeatureResponse>, Status> {
        let slug = request.into_inner().slug;
        match self.storage.get_feature_by_slug(&slug).await {
            Ok(Some(feature)) => Ok(Response::new(GetFeatureResponse {
                feature: Some(feature_to_proto(feature)),
            })),
            Ok(None) => Err(Status::not_found(format!("feature '{slug}' not found"))),
            Err(e) => Err(domain_error_to_status(e)),
        }
    }

    async fn list_features(
        &self,
        request: Request<ListFeaturesRequest>,
    ) -> Result<Response<ListFeaturesResponse>, Status> {
        let state_filter = request.into_inner().state_filter;
        let features = if state_filter.is_empty() {
            self.storage
                .list_all_features()
                .await
                .map_err(domain_error_to_status)?
        } else {
            let state: FeatureState = state_filter
                .parse()
                .map_err(|e: String| Status::invalid_argument(e))?;
            self.storage
                .list_features_by_state(state)
                .await
                .map_err(domain_error_to_status)?
        };
        let proto_features = features.into_iter().map(feature_to_proto).collect();
        Ok(Response::new(ListFeaturesResponse {
            features: proto_features,
        }))
    }

    async fn get_feature_state(
        &self,
        request: Request<GetFeatureStateRequest>,
    ) -> Result<Response<GetFeatureStateResponse>, Status> {
        let slug = request.into_inner().slug;
        let feature = self
            .storage
            .get_feature_by_slug(&slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("feature '{slug}' not found")))?;

        let state_str = feature.state.to_string();
        let next_cmd = match feature.state {
            FeatureState::Created => "specify",
            FeatureState::Specified => "research",
            FeatureState::Researched => "plan",
            FeatureState::Planned => "implement",
            FeatureState::Implementing => "validate",
            FeatureState::Validated => "ship",
            FeatureState::Shipped => "retrospective",
            FeatureState::Retrospected => "",
        };

        Ok(Response::new(GetFeatureStateResponse {
            feature_state: Some(agileplus_proto::agileplus::v1::FeatureState {
                state: state_str,
                next_command: next_cmd.to_string(),
                blockers: Vec::new(),
                governance: None,
            }),
        }))
    }

    // -------------------------------------------------------------------------
    // Work Package RPCs
    // -------------------------------------------------------------------------

    async fn list_work_packages(
        &self,
        request: Request<ListWorkPackagesRequest>,
    ) -> Result<Response<ListWorkPackagesResponse>, Status> {
        let req = request.into_inner();
        let feature = self
            .storage
            .get_feature_by_slug(&req.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", req.feature_slug))
            })?;

        let wps = self
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let filtered: Vec<_> = if req.state_filter.is_empty() {
            wps
        } else {
            wps.into_iter()
                .filter(|wp| {
                    format!("{:?}", wp.state).to_lowercase() == req.state_filter.to_lowercase()
                })
                .collect()
        };

        Ok(Response::new(ListWorkPackagesResponse {
            packages: filtered.into_iter().map(wp_to_proto).collect(),
        }))
    }

    async fn get_work_package_status(
        &self,
        request: Request<GetWorkPackageStatusRequest>,
    ) -> Result<Response<GetWorkPackageStatusResponse>, Status> {
        let req = request.into_inner();
        let feature = self
            .storage
            .get_feature_by_slug(&req.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", req.feature_slug))
            })?;

        let wps = self
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let wp = wps
            .into_iter()
            .find(|wp| wp.sequence == req.wp_sequence)
            .ok_or_else(|| {
                Status::not_found(format!("WP sequence {} not found", req.wp_sequence))
            })?;

        Ok(Response::new(GetWorkPackageStatusResponse {
            work_package_status: Some(wp_to_proto(wp)),
        }))
    }

    // -------------------------------------------------------------------------
    // Governance RPCs
    // -------------------------------------------------------------------------

    async fn check_governance_gate(
        &self,
        request: Request<CheckGovernanceGateRequest>,
    ) -> Result<Response<CheckGovernanceGateResponse>, Status> {
        let req = request.into_inner();
        let feature = self
            .storage
            .get_feature_by_slug(&req.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", req.feature_slug))
            })?;

        let contract = self
            .storage
            .get_latest_governance_contract(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        if contract.is_none() {
            return Ok(Response::new(CheckGovernanceGateResponse {
                passed: true,
                violations: Vec::new(),
            }));
        }

        let contract = contract.unwrap();
        let relevant_rules: Vec<_> = contract
            .rules
            .iter()
            .filter(|r| {
                r.transition.is_empty()
                    || governance_transition_matches(&r.transition, &req.transition)
            })
            .collect();

        let mut violations = Vec::new();
        let feature_wp_ids: HashSet<i64> = self
            .storage
            .list_wps_by_feature(feature.id)
            .await
            .map_err(domain_error_to_status)?
            .into_iter()
            .map(|wp| wp.id)
            .collect();

        for rule in &relevant_rules {
            for raw_requirement in &rule.required_evidence {
                let (fr_id, expected_type, recognized) =
                    parse_evidence_requirement(raw_requirement);
                let evidence = self
                    .storage
                    .get_evidence_by_fr(fr_id)
                    .await
                    .map_err(domain_error_to_status)?;
                if !recognized
                    || !evidence_satisfies_requirement(
                        &evidence,
                        &feature_wp_ids,
                        fr_id,
                        expected_type,
                    )
                {
                    violations.push(missing_evidence_violation(rule, fr_id, raw_requirement));
                }
            }
        }

        Ok(Response::new(CheckGovernanceGateResponse {
            passed: violations.is_empty(),
            violations,
        }))
    }

    // Server-streaming RPC for audit trail
    type GetAuditTrailStream =
        Pin<Box<dyn tokio_stream::Stream<Item = Result<GetAuditTrailResponse, Status>> + Send>>;

    #[allow(clippy::result_large_err)]
    async fn get_audit_trail(
        &self,
        request: Request<GetAuditTrailRequest>,
    ) -> Result<Response<Self::GetAuditTrailStream>, Status> {
        let req = request.into_inner();
        if req.limit < 0 {
            return Err(Status::invalid_argument("limit must be non-negative"));
        }
        let feature = self
            .storage
            .get_feature_by_slug(&req.feature_slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!("feature '{}' not found", req.feature_slug))
            })?;

        let entries = self
            .storage
            .get_audit_trail_page(
                feature.id,
                req.after_id,
                (req.limit > 0).then_some(req.limit as usize),
            )
            .await
            .map_err(domain_error_to_status)?;

        let slug = req.feature_slug.clone();

        let stream = tokio_stream::iter(entries.into_iter().map(move |entry| {
            let mut proto = audit_entry_to_proto(entry);
            proto.feature_slug = slug.clone();
            Ok(GetAuditTrailResponse {
                audit_entry: Some(proto),
            })
        }));

        Ok(Response::new(Box::pin(stream)))
    }

    async fn verify_audit_chain(
        &self,
        request: Request<VerifyAuditChainRequest>,
    ) -> Result<Response<VerifyAuditChainResponse>, Status> {
        let slug = request.into_inner().feature_slug;
        let feature = self
            .storage
            .get_feature_by_slug(&slug)
            .await
            .map_err(domain_error_to_status)?
            .ok_or_else(|| Status::not_found(format!("feature '{slug}' not found")))?;

        let entries = self
            .storage
            .get_audit_trail(feature.id)
            .await
            .map_err(domain_error_to_status)?;

        let count = entries.len() as i64;
        let chain = AuditChain { entries };

        match chain.verify_chain() {
            Ok(()) => Ok(Response::new(VerifyAuditChainResponse {
                valid: true,
                entries_verified: count,
                first_invalid_id: String::new(),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(VerifyAuditChainResponse {
                valid: false,
                entries_verified: 0,
                first_invalid_id: String::new(),
                error_message: e.to_string(),
            })),
        }
    }

    // -------------------------------------------------------------------------
    // Agent event streaming RPC
    // -------------------------------------------------------------------------

    type StreamAgentEventsStream = Pin<
        Box<
            dyn tokio_stream::Stream<
                    Item = Result<
                        agileplus_proto::agileplus::v1::StreamAgentEventsResponse,
                        Status,
                    >,
                > + Send,
        >,
    >;

    async fn stream_agent_events(
        &self,
        request: Request<agileplus_proto::agileplus::v1::StreamAgentEventsRequest>,
    ) -> Result<Response<Self::StreamAgentEventsStream>, Status> {
        let feature_slug = request.into_inner().feature_slug;
        let rx = self.event_bus.subscribe();
        let stream = crate::streaming::agent_event_stream(rx, feature_slug);
        Ok(Response::new(stream))
    }

    // -------------------------------------------------------------------------
    // Command dispatch RPC
    // -------------------------------------------------------------------------

    async fn dispatch_command(
        &self,
        request: Request<DispatchCommandRequest>,
    ) -> Result<Response<DispatchCommandResponse>, Status> {
        let req = request.into_inner();
        let cmd_req = req
            .command
            .ok_or_else(|| Status::invalid_argument("missing command field"))?;

        let command = cmd_req.command.as_str();
        let feature_slug = &cmd_req.feature_slug;
        let args = &cmd_req.args;

        // Agent commands forwarded through the proxy router
        let agent_commands = ["implement"];

        let (success, message, outputs) = if agent_commands.contains(&command) {
            let result = self
                .proxy
                .dispatch_agent_command(command, feature_slug, args)
                .await;
            (
                result.is_success(),
                result.message().to_string(),
                result.outputs(),
            )
        } else {
            match self
                .dispatch_core_command(command, feature_slug, args)
                .await
            {
                Ok((msg, outs)) => (true, msg, outs),
                Err(status) => return Err(status),
            }
        };

        Ok(Response::new(DispatchCommandResponse {
            result: Some(CommandResponse {
                success,
                message,
                outputs,
            }),
        }))
    }
}

impl<S> AgilePlusCoreServer<S>
where
    S: StoragePort + 'static,
{
    /// Dispatch core (non-agent) commands.
    async fn dispatch_core_command(
        &self,
        command: &str,
        _feature_slug: &str,
        _args: &HashMap<String, String>,
    ) -> Result<(String, HashMap<String, String>), Status> {
        match command {
            "specify" | "research" | "plan" | "validate" | "ship" | "retrospective" => {
                Err(unsupported_core_command(command))
            }
            other => Err(Status::unimplemented(format!("unknown command: '{other}'"))),
        }
    }
}

/// Start the gRPC server, binding to the given address.
#[cfg(not(agileplus_proto_stubs))]
pub async fn start_server<S>(
    addr: SocketAddr,
    storage: Arc<S>,
    event_bus: Arc<EventBus>,
    proxy: Arc<ProxyRouter>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: StoragePort + ContentStoragePort + 'static,
{
    let service = AgilePlusCoreServer::new(storage, event_bus, proxy);

    info!(%addr, "starting AgilePlus gRPC server");

    Server::builder()
        .add_service(AgilePlusCoreServiceServer::new(service.clone()))
        .add_service(
            agileplus_proto::agileplus::v1::integrations_service_server::IntegrationsServiceServer::new(
                service,
            ),
        )
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("gRPC server shut down gracefully");
    Ok(())
}

/// Report a clear runtime error when the crate was built without generated tonic services.
#[cfg(agileplus_proto_stubs)]
pub async fn start_server<S>(
    _addr: SocketAddr,
    _storage: Arc<S>,
    _event_bus: Arc<EventBus>,
    _proxy: Arc<ProxyRouter>,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: StoragePort + ContentStoragePort + 'static,
{
    Err(
        "AgilePlus gRPC network server is unavailable because protobuf generation was skipped"
            .into(),
    )
}

/// Listens for SIGTERM / SIGINT and resolves when either is received.
#[cfg(not(agileplus_proto_stubs))]
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::{
        audit::{AuditEntry, hash_entry},
        feature::Feature,
    };
    use agileplus_domain::error::DomainError;
    use agileplus_domain::ports::agent::{AgentConfig, AgentResult, AgentStatus, AgentTask};
    use agileplus_domain::ports::observability::{LogEntry, SpanContext};
    use agileplus_domain::ports::review::{CiStatus, PrInfo, ReviewComment, ReviewStatus};
    use agileplus_git::GitVcsAdapter;
    use agileplus_proto::agileplus::v1::CommandRequest;
    use agileplus_sqlite::SqliteStorageAdapter;
    use chrono::Utc;
    use tokio_stream::StreamExt;

    #[derive(Debug)]
    struct Unavailable;

    impl AgentPort for Unavailable {
        async fn dispatch(
            &self,
            _: AgentTask,
            _: &AgentConfig,
        ) -> Result<AgentResult, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn dispatch_async(
            &self,
            _: AgentTask,
            _: &AgentConfig,
        ) -> Result<String, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn query_status(&self, _: &str) -> Result<AgentStatus, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn cancel(&self, _: &str) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn send_instruction(&self, _: &str, _: &str) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }
    }

    impl ReviewPort for Unavailable {
        async fn get_review_status(&self, _: &str) -> Result<ReviewStatus, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn get_review_comments(&self, _: &str) -> Result<Vec<ReviewComment>, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn get_actionable_comments(
            &self,
            _: &str,
        ) -> Result<Vec<ReviewComment>, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn get_ci_status(&self, _: &str) -> Result<CiStatus, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn get_pr_info(&self, _: &str) -> Result<PrInfo, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn await_review(&self, _: &str, _: u64) -> Result<ReviewStatus, DomainError> {
            Err(DomainError::NotImplemented)
        }
        async fn await_ci(&self, _: &str, _: u64) -> Result<CiStatus, DomainError> {
            Err(DomainError::NotImplemented)
        }
    }

    impl ObservabilityPort for Unavailable {
        fn start_span(&self, _: &str, _: Option<&SpanContext>) -> SpanContext {
            SpanContext {
                trace_id: String::new(),
                span_id: String::new(),
                parent_span_id: None,
            }
        }
        fn end_span(&self, _: &SpanContext) {}
        fn add_span_event(&self, _: &SpanContext, _: &str, _: &[(&str, &str)]) {}
        fn set_span_error(&self, _: &SpanContext, _: &str) {}
        fn record_counter(&self, _: &str, _: u64, _: &[(&str, &str)]) {}
        fn record_histogram(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
        fn record_gauge(&self, _: &str, _: f64, _: &[(&str, &str)]) {}
        fn log(&self, _: &LogEntry) {}
        fn log_info(&self, _: &str) {}
        fn log_warn(&self, _: &str) {}
        fn log_error(&self, _: &str) {}
    }

    #[test]
    fn domain_error_mapping() {
        use agileplus_domain::error::DomainError;
        let s = domain_error_to_status(DomainError::NotFound("feat".into()));
        assert_eq!(s.code(), tonic::Code::NotFound);

        let s = domain_error_to_status(DomainError::InvalidTransition {
            from: "a".into(),
            to: "b".into(),
            reason: "test".into(),
        });
        assert_eq!(s.code(), tonic::Code::FailedPrecondition);

        let s = domain_error_to_status(DomainError::Conflict("x".into()));
        assert_eq!(s.code(), tonic::Code::AlreadyExists);
    }

    #[test]
    fn governance_requirement_uses_fr_id_and_optional_type() {
        assert_eq!(
            parse_evidence_requirement("FR-053:ci_output"),
            ("FR-053", Some(EvidenceType::CiOutput), true)
        );
        assert_eq!(parse_evidence_requirement("FR-054"), ("FR-054", None, true));
        assert_eq!(
            parse_evidence_requirement("FR-055:unknown"),
            ("FR-055", None, false)
        );
    }

    #[test]
    fn governance_transition_comparison_normalizes_arrow_spacing_only() {
        assert!(governance_transition_matches(
            "WP01: Doing -> Review",
            "WP01: Doing->Review"
        ));
        assert!(!governance_transition_matches(
            "WP01: Doing -> Review",
            "WP02: Doing -> Review"
        ));
        assert!(!governance_transition_matches(
            "WP01: Doing -> Review",
            "WP01: doing -> review"
        ));
        assert!(!governance_transition_matches(
            "malformed",
            "also malformed"
        ));
    }

    #[test]
    fn lifecycle_commands_are_explicitly_unsupported() {
        for command in [
            "specify",
            "research",
            "plan",
            "validate",
            "ship",
            "retrospective",
        ] {
            let status = unsupported_core_command(command);
            assert_eq!(status.code(), tonic::Code::Unimplemented);
            assert!(status.message().contains(command));
            assert!(!status.message().contains("queued"));
        }
    }

    #[tokio::test]
    async fn dispatch_command_rejects_unimplemented_lifecycle_command() {
        let storage =
            Arc::new(SqliteStorageAdapter::new(std::path::Path::new(":memory:")).unwrap());
        let service = AgilePlusCoreServer::new(
            storage,
            Arc::new(GitVcsAdapter::from_current_dir().unwrap()),
            Arc::new(Unavailable),
            Arc::new(Unavailable),
            Arc::new(Unavailable),
            Arc::new(EventBus::new(8)),
            Arc::new(ProxyRouter::new(None, None).await),
        );
        let error = service
            .dispatch_command(Request::new(DispatchCommandRequest {
                command: Some(CommandRequest {
                    command: "specify".to_owned(),
                    feature_slug: "feature-one".to_owned(),
                    args: HashMap::new(),
                }),
            }))
            .await
            .unwrap_err();

        assert_eq!(error.code(), tonic::Code::Unimplemented);
        assert!(!error.message().contains("queued"));
    }

    async fn test_service(
        storage: Arc<SqliteStorageAdapter>,
    ) -> AgilePlusCoreServer<
        SqliteStorageAdapter,
        GitVcsAdapter,
        Unavailable,
        Unavailable,
        Unavailable,
    > {
        AgilePlusCoreServer::new(
            storage,
            Arc::new(GitVcsAdapter::from_current_dir().unwrap()),
            Arc::new(Unavailable),
            Arc::new(Unavailable),
            Arc::new(Unavailable),
            Arc::new(EventBus::new(8)),
            Arc::new(ProxyRouter::new(None, None).await),
        )
    }

    #[tokio::test]
    async fn audit_trail_rejects_negative_limit() {
        let storage = Arc::new(SqliteStorageAdapter::in_memory().unwrap());
        let result = test_service(storage)
            .await
            .get_audit_trail(Request::new(GetAuditTrailRequest {
                feature_slug: "feature-one".into(),
                after_id: 0,
                limit: -1,
            }))
            .await;
        let error = match result {
            Ok(_) => panic!("negative audit limit must be rejected"),
            Err(error) => error,
        };
        assert_eq!(error.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn audit_trail_applies_after_id_and_latest_limit() {
        let storage = Arc::new(SqliteStorageAdapter::in_memory().unwrap());
        let feature_id = storage
            .create_feature(&Feature::new("audit-page", "Audit page", [0; 32], None))
            .await
            .unwrap();
        let mut previous = [0; 32];
        let mut ids = Vec::new();
        for _ in 0..3 {
            let mut entry = AuditEntry {
                id: 0,
                feature_id,
                wp_id: None,
                timestamp: Utc::now(),
                actor: "test".into(),
                transition: "test".into(),
                evidence_refs: Vec::new(),
                prev_hash: previous,
                hash: [0; 32],
                event_id: None,
                archived_to: None,
            };
            entry.hash = hash_entry(&entry);
            previous = entry.hash;
            ids.push(storage.append_audit_entry(&entry).await.unwrap());
        }
        let response = test_service(storage)
            .await
            .get_audit_trail(Request::new(GetAuditTrailRequest {
                feature_slug: "audit-page".into(),
                after_id: ids[0],
                limit: 1,
            }))
            .await
            .unwrap();
        let entries = response.into_inner().collect::<Vec<_>>().await;
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0]
                .as_ref()
                .unwrap()
                .audit_entry
                .as_ref()
                .unwrap()
                .id,
            ids[2]
        );
    }

    #[test]
    fn governance_evidence_must_belong_to_feature_and_match_type() {
        let evidence = vec![Evidence {
            id: 1,
            wp_id: 7,
            fr_id: "FR-053".to_owned(),
            evidence_type: EvidenceType::CiOutput,
            artifact_path: "ci://run/1".to_owned(),
            metadata: None,
            created_at: Utc::now(),
        }];

        assert!(evidence_satisfies_requirement(
            &evidence,
            &HashSet::from([7]),
            "FR-053",
            Some(EvidenceType::CiOutput),
        ));
        assert!(!evidence_satisfies_requirement(
            &evidence,
            &HashSet::from([8]),
            "FR-053",
            Some(EvidenceType::CiOutput),
        ));
        assert!(!evidence_satisfies_requirement(
            &evidence,
            &HashSet::from([7]),
            "FR-053",
            Some(EvidenceType::SecurityScan),
        ));
    }

    #[test]
    fn governance_violation_reports_requirement_not_feature_slug() {
        let rule = GovernanceRule {
            transition: "validate".to_owned(),
            required_evidence: vec!["FR-053:ci_output".to_owned()],
            policy_refs: Vec::new(),
        };
        let violation = missing_evidence_violation(&rule, "FR-053", "FR-053:ci_output");

        assert_eq!(violation.fr_id, "FR-053");
        assert_eq!(violation.rule_id, "validate");
        assert!(violation.message.contains("FR-053:ci_output"));
        assert!(!violation.message.contains("feature"));
    }
}
