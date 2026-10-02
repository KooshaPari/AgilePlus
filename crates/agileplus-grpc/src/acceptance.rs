//! Authenticated gRPC acceptance decorator over the existing core service.
//! Ordinary RPCs retain their implementation; validate uses the same atomic port
//! as CLI/HTTP. Shipping remains fail-closed until promotion has its own service.

use std::{collections::HashMap, sync::Arc};
use agileplus_domain::{
    credentials::CredentialStore,
    domain::{acceptance::AcceptFeatureCommand, governance_evaluator::evaluate_governance},
    error::DomainError,
    ports::{StoragePort, execution::AtomicAcceptancePort},
};
use agileplus_proto::agileplus::v1::{
    agile_plus_core_service_server::AgilePlusCoreService,
    CheckGovernanceGateRequest, CheckGovernanceGateResponse, CommandResponse, GateViolation,
    DispatchCommandRequest, DispatchCommandResponse, GetAuditTrailRequest,
    GetFeatureRequest, GetFeatureResponse, GetFeatureStateRequest, GetFeatureStateResponse,
    GetWorkPackageStatusRequest, GetWorkPackageStatusResponse, ListFeaturesRequest, ListFeaturesResponse,
    ListWorkPackagesRequest, ListWorkPackagesResponse, StreamAgentEventsRequest,
    VerifyAuditChainRequest, VerifyAuditChainResponse,
};
use tonic::{Request, Response, Status};

pub struct AcceptanceCore<T, S> {
    inner: T,
    storage: Arc<S>,
    credentials: Arc<dyn CredentialStore>,
    canonical_repo_root: String,
}

impl<T, S> AcceptanceCore<T, S> {
    pub fn new(inner: T, storage: Arc<S>, credentials: Arc<dyn CredentialStore>, canonical_repo_root: String) -> Self {
        Self { inner, storage, credentials, canonical_repo_root }
    }
}

fn domain_status(error: DomainError) -> Status {
    match error {
        DomainError::Validation(message) => Status::failed_precondition(message),
        DomainError::Conflict(message) => Status::aborted(message),
        DomainError::NotFound(_) => Status::not_found("feature or acceptance record not found"),
        DomainError::NotImplemented => Status::unimplemented("atomic acceptance is not configured"),
        other => {
            tracing::error!(%other, "gRPC atomic acceptance failed");
            Status::internal("acceptance persistence failed")
        }
    }
}

#[tonic::async_trait]
impl<T, S> AgilePlusCoreService for AcceptanceCore<T, S>
where T: AgilePlusCoreService, S: StoragePort + AtomicAcceptancePort + 'static,
{
    async fn list_features(&self, r: Request<ListFeaturesRequest>) -> Result<Response<ListFeaturesResponse>, Status> {
        self.inner.list_features(r).await
    }
    async fn get_feature(&self, r: Request<GetFeatureRequest>) -> Result<Response<GetFeatureResponse>, Status> {
        self.inner.get_feature(r).await
    }
    async fn get_feature_state(&self, r: Request<GetFeatureStateRequest>) -> Result<Response<GetFeatureStateResponse>, Status> {
        self.inner.get_feature_state(r).await
    }
    async fn list_work_packages(&self, r: Request<ListWorkPackagesRequest>) -> Result<Response<ListWorkPackagesResponse>, Status> {
        self.inner.list_work_packages(r).await
    }
    async fn get_work_package_status(&self, r: Request<GetWorkPackageStatusRequest>) -> Result<Response<GetWorkPackageStatusResponse>, Status> {
        self.inner.get_work_package_status(r).await
    }
    async fn check_governance_gate(&self, r: Request<CheckGovernanceGateRequest>) -> Result<Response<CheckGovernanceGateResponse>, Status> {
        let r = r.into_inner();
        if r.project_scope.as_ref().map(|s| s.canonical_repo_root.as_str()) != Some(self.canonical_repo_root.as_str()) {
            return Err(Status::permission_denied("project scope does not match governance service"));
        }
        let feature = self.storage.get_feature_by_slug(&r.feature_slug).await.map_err(domain_status)?
            .ok_or_else(|| Status::not_found("feature not found"))?;
        let Some(mut contract) = self.storage.get_latest_governance_contract(feature.id).await.map_err(domain_status)? else {
            return Ok(Response::new(CheckGovernanceGateResponse { passed:false, violations:vec![GateViolation {
                fr_id:String::new(), rule_id:r.transition, message:"governance NotConfigured".into(),
                remediation:"accept an applicable governance contract".into(),
            }] }));
        };
        contract.rules.retain(|rule| rule.transition.is_empty() || rule.transition == r.transition);
        let evaluation = evaluate_governance(self.storage.as_ref(), &contract, feature.id).await.map_err(domain_status)?;
        let passed = evaluation.passed(&contract);
        let mut violations: Vec<GateViolation> = evaluation.missing_evidence.iter().map(|(fr, kind)| GateViolation {
            fr_id:fr.clone(), rule_id:r.transition.clone(), message:format!("missing {kind} evidence"),
            remediation:"provide current feature-scoped evidence".into(),
        }).collect();
        violations.extend(evaluation.policy_results.iter().filter(|p| !p.passed).map(|p| GateViolation {
            fr_id:String::new(), rule_id:p.policy_id.to_string(), message:p.message.clone(),
            remediation:"satisfy the governing policy".into(),
        }));
        if !passed && violations.is_empty() {
            violations.push(GateViolation { fr_id:String::new(), rule_id:r.transition,
                message:"governance NotConfigured".into(), remediation:"configure applicable nonempty rules".into() });
        }
        Ok(Response::new(CheckGovernanceGateResponse { passed, violations }))
    }
    type GetAuditTrailStream = T::GetAuditTrailStream;
    async fn get_audit_trail(&self, r: Request<GetAuditTrailRequest>) -> Result<Response<Self::GetAuditTrailStream>, Status> {
        self.inner.get_audit_trail(r).await
    }
    async fn verify_audit_chain(&self, r: Request<VerifyAuditChainRequest>) -> Result<Response<VerifyAuditChainResponse>, Status> {
        self.inner.verify_audit_chain(r).await
    }
    type StreamAgentEventsStream = T::StreamAgentEventsStream;
    async fn stream_agent_events(&self, r: Request<StreamAgentEventsRequest>) -> Result<Response<Self::StreamAgentEventsStream>, Status> {
        self.inner.stream_agent_events(r).await
    }
    async fn dispatch_command(&self, request: Request<DispatchCommandRequest>) -> Result<Response<DispatchCommandResponse>, Status> {
        if request.get_ref().command.as_ref().is_none_or(|c| c.command != "validate") {
            return self.inner.dispatch_command(request).await;
        }
        let token = request.metadata().get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|value| !value.is_empty())
            .ok_or_else(|| Status::unauthenticated("bearer credential required for acceptance"))?;
        if !self.credentials.validate_api_key(token).map_err(|error| {
            tracing::error!(%error, "acceptance credential verification failed");
            Status::internal("credential verification failed")
        })? {
            return Err(Status::unauthenticated("invalid acceptance credential"));
        }
        let request = request.into_inner();
        let scope = request.project_scope.as_ref()
            .ok_or_else(|| Status::invalid_argument("project scope required"))?;
        if scope.canonical_repo_root != self.canonical_repo_root {
            return Err(Status::permission_denied("project scope does not match acceptance service"));
        }
        let command = request.command.ok_or_else(|| Status::invalid_argument("command required"))?;
        if command.args.keys().any(|key| key != "request_id" && key != "expected_governance_version") {
            return Err(Status::invalid_argument("acceptance arguments may not override authority or grading"));
        }
        let request_id = command.args.get("request_id")
            .cloned().ok_or_else(|| Status::invalid_argument("acceptance request_id required"))?;
        let expected_governance_version = command.args.get("expected_governance_version")
            .map(|v| v.parse::<i32>()).transpose()
            .map_err(|_| Status::invalid_argument("invalid governance version"))?;
        let feature = self.storage.get_feature_by_slug(&command.feature_slug).await.map_err(domain_status)?
            .ok_or_else(|| Status::not_found("feature not found"))?;
        let accept = AcceptFeatureCommand {
            request_id, feature_id: feature.id, actor: "grpc:api-key".into(), expected_governance_version,
        };
        accept.validate().map_err(|_| Status::invalid_argument("invalid acceptance command identity"))?;
        let outcome = self.storage.accept_feature_atomic(&accept).await.map_err(domain_status)?;
        let mut outputs = HashMap::new();
        outputs.insert("acceptance_receipt".into(), serde_json::to_string(&outcome.receipt)
            .map_err(|_| Status::internal("cannot serialize acceptance receipt"))?);
        outputs.insert("replayed".into(), outcome.replayed.to_string());
        outputs.insert("accepted_state".into(), "validated".into());
        Ok(Response::new(DispatchCommandResponse {
            result: Some(CommandResponse { success: true,
                message: if outcome.replayed { "historical acceptance receipt replayed" }
                    else { "acceptance committed atomically" }.into(), outputs }),
        }))
    }
}
