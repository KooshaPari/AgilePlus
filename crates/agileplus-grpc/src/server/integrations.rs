use tonic::{Request, Response, Status};

use agileplus_domain::{
    domain::backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogStatus, Intent},
    ports::{ContentStoragePort, StoragePort},
};
use agileplus_proto::agileplus::v1::{
    ClassifyInputRequest, ClassifyInputResponse, CreateBacklogItemRequest,
    CreateBacklogItemResponse, DetectGitHubConflictsRequest, DetectGitHubConflictsResponse,
    DetectPlaneConflictsRequest, DetectPlaneConflictsResponse, GenerateRouterRequest,
    GenerateRouterResponse, ListBacklogRequest, ListBacklogResponse, PromoteBacklogItemRequest,
    PromoteBacklogItemResponse, SyncBugToGitHubRequest, SyncBugToGitHubResponse,
    SyncFeatureToPlaneRequest, SyncFeatureToPlaneResponse, SyncIssueStatusRequest,
    SyncIssueStatusResponse, SyncWpToPlaneRequest, SyncWpToPlaneResponse,
    integrations_service_server::IntegrationsService,
};

use super::{AgilePlusCoreServer, domain_error_to_status};
use crate::conversions::backlog_item_to_proto;

#[tonic::async_trait]
impl<S> IntegrationsService for AgilePlusCoreServer<S>
where
    S: StoragePort + ContentStoragePort + 'static,
{
    async fn sync_feature_to_plane(
        &self,
        _request: Request<SyncFeatureToPlaneRequest>,
    ) -> Result<Response<SyncFeatureToPlaneResponse>, Status> {
        Err(Status::unimplemented(
            "Plane synchronization is not implemented",
        ))
    }

    async fn sync_wp_to_plane(
        &self,
        _request: Request<SyncWpToPlaneRequest>,
    ) -> Result<Response<SyncWpToPlaneResponse>, Status> {
        Err(Status::unimplemented(
            "Plane synchronization is not implemented",
        ))
    }

    async fn detect_plane_conflicts(
        &self,
        _request: Request<DetectPlaneConflictsRequest>,
    ) -> Result<Response<DetectPlaneConflictsResponse>, Status> {
        Err(Status::unimplemented(
            "Plane conflict detection is not implemented",
        ))
    }

    async fn sync_bug_to_git_hub(
        &self,
        _request: Request<SyncBugToGitHubRequest>,
    ) -> Result<Response<SyncBugToGitHubResponse>, Status> {
        Err(Status::unimplemented(
            "GitHub synchronization is not implemented",
        ))
    }

    async fn sync_issue_status(
        &self,
        _request: Request<SyncIssueStatusRequest>,
    ) -> Result<Response<SyncIssueStatusResponse>, Status> {
        Err(Status::unimplemented(
            "GitHub synchronization is not implemented",
        ))
    }

    async fn detect_git_hub_conflicts(
        &self,
        _request: Request<DetectGitHubConflictsRequest>,
    ) -> Result<Response<DetectGitHubConflictsResponse>, Status> {
        Err(Status::unimplemented(
            "GitHub conflict detection is not implemented",
        ))
    }

    async fn classify_input(
        &self,
        _request: Request<ClassifyInputRequest>,
    ) -> Result<Response<ClassifyInputResponse>, Status> {
        Err(Status::unimplemented(
            "gRPC input classification is not implemented",
        ))
    }

    async fn create_backlog_item(
        &self,
        request: Request<CreateBacklogItemRequest>,
    ) -> Result<Response<CreateBacklogItemResponse>, Status> {
        let request = request.into_inner();
        let mut item = BacklogItem::from_triage(
            request.title,
            request.body,
            parse_intent(&request.r#type)?,
            if request.triaged_by.is_empty() {
                "grpc".to_string()
            } else {
                request.triaged_by
            },
        );
        if !request.priority.is_empty() {
            item.priority = parse_priority(&request.priority)?;
        }
        if !request.wp_id.trim().is_empty() {
            return Err(Status::invalid_argument(
                "wp_id backlog association is not supported by the canonical queue model",
            ));
        }
        if !request.feature_id.trim().is_empty() {
            item.feature_slug = Some(request.feature_id);
        }

        let id = self
            .storage
            .create_backlog_item(&item)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(CreateBacklogItemResponse {
            item: Some(backlog_item_to_proto(BacklogItem {
                id: Some(id),
                ..item
            })),
        }))
    }

    async fn list_backlog(
        &self,
        request: Request<ListBacklogRequest>,
    ) -> Result<Response<ListBacklogResponse>, Status> {
        let request = request.into_inner();
        let filters = BacklogFilters {
            intent: parse_intent_opt(&request.type_filter)?,
            status: parse_status_opt(&request.state_filter)?,
            feature_slug: (!request.feature_slug.is_empty()).then_some(request.feature_slug),
            ..Default::default()
        };
        let items = self
            .storage
            .list_backlog_items(&filters)
            .await
            .map_err(domain_error_to_status)?;

        Ok(Response::new(ListBacklogResponse {
            items: items.into_iter().map(backlog_item_to_proto).collect(),
        }))
    }

    async fn promote_backlog_item(
        &self,
        request: Request<PromoteBacklogItemRequest>,
    ) -> Result<Response<PromoteBacklogItemResponse>, Status> {
        let request = request.into_inner();
        let target_type = request.target_type.trim();
        if target_type.is_empty() {
            return Err(Status::invalid_argument("target_type must not be empty"));
        }

        Err(Status::unimplemented(
            "backlog promotion requires atomic target creation and backlog transition",
        ))
    }

    async fn generate_router(
        &self,
        _request: Request<GenerateRouterRequest>,
    ) -> Result<Response<GenerateRouterResponse>, Status> {
        Err(Status::unimplemented(
            "gRPC router generation is not implemented",
        ))
    }
}

fn parse_intent(value: &str) -> Result<Intent, Status> {
    value.parse::<Intent>().map_err(Status::invalid_argument)
}

fn parse_intent_opt(value: &str) -> Result<Option<Intent>, Status> {
    (!value.is_empty()).then(|| parse_intent(value)).transpose()
}

fn parse_priority(value: &str) -> Result<BacklogPriority, Status> {
    value
        .parse::<BacklogPriority>()
        .map_err(Status::invalid_argument)
}

fn parse_status_opt(value: &str) -> Result<Option<BacklogStatus>, Status> {
    (!value.is_empty())
        .then(|| {
            value
                .parse::<BacklogStatus>()
                .map_err(Status::invalid_argument)
        })
        .transpose()
}
