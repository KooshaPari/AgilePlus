use std::{path::PathBuf, sync::Arc};

use agileplus_domain::{
    credentials::CredentialStore,
    domain::backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus, Intent},
    ports::ContentStoragePort,
};
use agileplus_proto::agileplus::v1::{
    self as proto, integrations_service_server::IntegrationsService,
};
use tonic::{Request, Response, Status};

use super::domain_error_to_status;

pub struct QueueService<S> {
    root: PathBuf,
    storage: Arc<S>,
    credentials: Arc<dyn CredentialStore>,
}

impl<S> QueueService<S> {
    pub fn new(root: PathBuf, storage: Arc<S>, credentials: Arc<dyn CredentialStore>) -> Self {
        Self {
            root,
            storage,
            credentials,
        }
    }

    fn authorize<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .filter(|v| !v.is_empty())
            .ok_or_else(|| Status::unauthenticated("bearer credential required for queue"))?;
        if !self
            .credentials
            .validate_api_key(token)
            .map_err(|_| Status::internal("credential verification failed"))?
        {
            return Err(Status::unauthenticated("invalid queue credential"));
        }
        Ok(())
    }

    fn validate_scope<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let root = self
            .root
            .to_str()
            .ok_or_else(|| Status::internal("core repository root is not valid UTF-8"))?;
        let bytes = request
            .metadata()
            .get_bin("agileplus-project-root-bin")
            .map(|value| value.to_bytes())
            .transpose()
            .map_err(|_| Status::invalid_argument("invalid project scope metadata"))?;
        let scope = bytes
            .as_ref()
            .map(|value| std::str::from_utf8(value))
            .transpose()
            .map_err(|_| Status::invalid_argument("project scope is not UTF-8"))?;
        match scope {
            None | Some("") => Err(Status::invalid_argument("project scope is required")),
            Some(scope) if scope == root => Ok(()),
            Some(_) => Err(Status::permission_denied(
                "project scope does not match this core server repository",
            )),
        }
    }
}

fn to_proto(item: BacklogItem) -> proto::BacklogItem {
    proto::BacklogItem {
        id: item.id.unwrap_or_default(),
        r#type: item.intent.to_string(),
        title: item.title,
        body: item.description,
        priority: item.priority.to_string(),
        state: item.status.to_string(),
        external_ref: String::new(),
        created_at: item.created_at.to_rfc3339(),
    }
}

#[tonic::async_trait]
impl<S: ContentStoragePort + 'static> IntegrationsService for QueueService<S> {
    async fn sync_feature_to_plane(
        &self,
        _: Request<proto::SyncFeatureToPlaneRequest>,
    ) -> Result<Response<proto::SyncFeatureToPlaneResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn sync_wp_to_plane(
        &self,
        _: Request<proto::SyncWpToPlaneRequest>,
    ) -> Result<Response<proto::SyncWpToPlaneResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn detect_plane_conflicts(
        &self,
        _: Request<proto::DetectPlaneConflictsRequest>,
    ) -> Result<Response<proto::DetectPlaneConflictsResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn sync_bug_to_git_hub(
        &self,
        _: Request<proto::SyncBugToGitHubRequest>,
    ) -> Result<Response<proto::SyncBugToGitHubResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn sync_issue_status(
        &self,
        _: Request<proto::SyncIssueStatusRequest>,
    ) -> Result<Response<proto::SyncIssueStatusResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn detect_git_hub_conflicts(
        &self,
        _: Request<proto::DetectGitHubConflictsRequest>,
    ) -> Result<Response<proto::DetectGitHubConflictsResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn classify_input(
        &self,
        _: Request<proto::ClassifyInputRequest>,
    ) -> Result<Response<proto::ClassifyInputResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn promote_backlog_item(
        &self,
        _: Request<proto::PromoteBacklogItemRequest>,
    ) -> Result<Response<proto::PromoteBacklogItemResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }
    async fn generate_router(
        &self,
        _: Request<proto::GenerateRouterRequest>,
    ) -> Result<Response<proto::GenerateRouterResponse>, Status> {
        Err(Status::unimplemented("integration is not configured"))
    }

    async fn create_backlog_item(
        &self,
        request: Request<proto::CreateBacklogItemRequest>,
    ) -> Result<Response<proto::CreateBacklogItemResponse>, Status> {
        self.authorize(&request)?;
        self.validate_scope(&request)?;
        let request = request.into_inner();
        if !request.wp_id.is_empty() {
            return Err(Status::invalid_argument(
                "wp_id is not supported by backlog storage",
            ));
        }
        if request.title.trim().is_empty() {
            return Err(Status::invalid_argument("title must not be empty"));
        }
        let intent = if request.r#type.is_empty() {
            Intent::Task
        } else {
            request
                .r#type
                .parse::<Intent>()
                .map_err(Status::invalid_argument)?
        };
        let mut item =
            BacklogItem::from_triage(request.title, request.body, intent, request.triaged_by)
                .with_feature_slug((!request.feature_id.is_empty()).then_some(request.feature_id));
        if !request.priority.is_empty() {
            item.priority = request
                .priority
                .parse::<BacklogPriority>()
                .map_err(Status::invalid_argument)?;
        }
        item.id = Some(
            self.storage
                .create_backlog_item(&item)
                .await
                .map_err(domain_error_to_status)?,
        );
        Ok(Response::new(proto::CreateBacklogItemResponse {
            item: Some(to_proto(item)),
        }))
    }

    async fn list_backlog(
        &self,
        request: Request<proto::ListBacklogRequest>,
    ) -> Result<Response<proto::ListBacklogResponse>, Status> {
        self.authorize(&request)?;
        self.validate_scope(&request)?;
        let request = request.into_inner();
        let filters = BacklogFilters {
            intent: if request.type_filter.is_empty() {
                None
            } else {
                Some(
                    request
                        .type_filter
                        .parse::<Intent>()
                        .map_err(Status::invalid_argument)?,
                )
            },
            status: if request.state_filter.is_empty() {
                None
            } else {
                Some(
                    request
                        .state_filter
                        .parse::<BacklogStatus>()
                        .map_err(Status::invalid_argument)?,
                )
            },
            feature_slug: (!request.feature_slug.is_empty()).then_some(request.feature_slug),
            ..Default::default()
        };
        let items = self
            .storage
            .list_backlog_items(&filters)
            .await
            .map_err(domain_error_to_status)?;
        Ok(Response::new(proto::ListBacklogResponse {
            items: items.into_iter().map(to_proto).collect(),
        }))
    }
}
