use tonic::{Response, Status};

use agileplus_domain::domain::backlog::{BacklogFilters, BacklogItem};
use agileplus_domain::ports::{AgentPort, ContentStoragePort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    integrations_service_server::IntegrationsService, ClassifyInputRequest, ClassifyInputResponse,
    CreateBacklogItemRequest, CreateBacklogItemResponse, DetectGitHubConflictsRequest,
    DetectGitHubConflictsResponse, DetectPlaneConflictsRequest, DetectPlaneConflictsResponse,
    GenerateRouterRequest, GenerateRouterResponse, ListBacklogRequest, ListBacklogResponse,
    PromoteBacklogItemRequest, PromoteBacklogItemResponse, SyncBugToGitHubRequest,
    SyncBugToGitHubResponse, SyncFeatureToPlaneRequest, SyncFeatureToPlaneResponse,
    SyncIssueStatusRequest, SyncIssueStatusResponse, SyncWpToPlaneRequest, SyncWpToPlaneResponse,
};

use super::AgilePlusCoreServer;
use crate::conversions::backlog_item_to_proto;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + ContentStoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    async fn create_backlog_item_impl(
        &self,
        request: CreateBacklogItemRequest,
    ) -> Result<Response<CreateBacklogItemResponse>, Status> {
        if !request.wp_id.is_empty() {
            return Err(Status::invalid_argument(
                "wp_id associations are unsupported by the core backlog contract",
            ));
        }
        let intent = request.r#type.parse().map_err(Status::invalid_argument)?;
        let mut item = BacklogItem::from_triage(
            request.title,
            request.body,
            intent,
            if request.triaged_by.is_empty() { "grpc".into() } else { request.triaged_by },
        )
        .with_feature_slug((!request.feature_id.is_empty()).then_some(request.feature_id));
        if !request.priority.is_empty() {
            item.priority = request.priority.parse().map_err(Status::invalid_argument)?;
        }
        let id = self.storage.create_backlog_item(&item).await.map_err(super::domain_error_to_status)?;
        item.id = Some(id);
        Ok(Response::new(CreateBacklogItemResponse { item: Some(backlog_item_to_proto(item)) }))
    }

    async fn list_backlog_items_impl(
        &self,
        request: ListBacklogRequest,
    ) -> Result<Response<ListBacklogResponse>, Status> {
        let filters = BacklogFilters {
            intent: (!request.type_filter.is_empty()).then(|| request.type_filter.parse()).transpose().map_err(Status::invalid_argument)?,
            status: (!request.state_filter.is_empty()).then(|| request.state_filter.parse()).transpose().map_err(Status::invalid_argument)?,
            feature_slug: (!request.feature_slug.is_empty()).then_some(request.feature_slug),
            ..Default::default()
        };
        let items = self.storage.list_backlog_items(&filters).await.map_err(super::domain_error_to_status)?;
        Ok(Response::new(ListBacklogResponse { items: items.into_iter().map(backlog_item_to_proto).collect() }))
    }
}

#[tonic::async_trait]
impl<S, V, A, R, O> IntegrationsService for AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + ContentStoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    async fn sync_feature_to_plane(&self, _: tonic::Request<SyncFeatureToPlaneRequest>) -> Result<Response<SyncFeatureToPlaneResponse>, Status> { Err(Status::unimplemented("Plane sync is not implemented by the core")) }
    async fn sync_wp_to_plane(&self, _: tonic::Request<SyncWpToPlaneRequest>) -> Result<Response<SyncWpToPlaneResponse>, Status> { Err(Status::unimplemented("Plane sync is not implemented by the core")) }
    async fn detect_plane_conflicts(&self, _: tonic::Request<DetectPlaneConflictsRequest>) -> Result<Response<DetectPlaneConflictsResponse>, Status> { Err(Status::unimplemented("Plane sync is not implemented by the core")) }
    async fn sync_bug_to_git_hub(&self, _: tonic::Request<SyncBugToGitHubRequest>) -> Result<Response<SyncBugToGitHubResponse>, Status> { Err(Status::unimplemented("GitHub sync is not implemented by the core")) }
    async fn sync_issue_status(&self, _: tonic::Request<SyncIssueStatusRequest>) -> Result<Response<SyncIssueStatusResponse>, Status> { Err(Status::unimplemented("GitHub sync is not implemented by the core")) }
    async fn detect_git_hub_conflicts(&self, _: tonic::Request<DetectGitHubConflictsRequest>) -> Result<Response<DetectGitHubConflictsResponse>, Status> { Err(Status::unimplemented("GitHub sync is not implemented by the core")) }
    async fn classify_input(&self, _: tonic::Request<ClassifyInputRequest>) -> Result<Response<ClassifyInputResponse>, Status> { Err(Status::unimplemented("input classification is not implemented by the core")) }
    async fn create_backlog_item(&self, request: tonic::Request<CreateBacklogItemRequest>) -> Result<Response<CreateBacklogItemResponse>, Status> { self.create_backlog_item_impl(request.into_inner()).await }
    async fn list_backlog(&self, request: tonic::Request<ListBacklogRequest>) -> Result<Response<ListBacklogResponse>, Status> { self.list_backlog_items_impl(request.into_inner()).await }
    async fn promote_backlog_item(&self, _: tonic::Request<PromoteBacklogItemRequest>) -> Result<Response<PromoteBacklogItemResponse>, Status> { Err(Status::unimplemented("backlog promotion is not implemented by the core")) }
    async fn generate_router(&self, _: tonic::Request<GenerateRouterRequest>) -> Result<Response<GenerateRouterResponse>, Status> { Err(Status::unimplemented("router generation is not implemented by the core")) }
}
