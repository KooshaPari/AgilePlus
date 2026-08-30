use std::sync::Arc;

use agileplus_grpc::{event_bus::EventBus, proxy::ProxyRouter, server::AgilePlusCoreServer};
use agileplus_proto::agileplus::v1::{
    ClassifyInputRequest, CreateBacklogItemRequest, DetectGitHubConflictsRequest,
    DetectPlaneConflictsRequest, GenerateRouterRequest, ListBacklogRequest,
    PromoteBacklogItemRequest, SyncBugToGitHubRequest, SyncFeatureToPlaneRequest,
    SyncIssueStatusRequest, SyncWpToPlaneRequest,
    integrations_service_client::IntegrationsServiceClient,
    integrations_service_server::IntegrationsServiceServer,
};
use agileplus_sqlite::SqliteStorageAdapter;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

fn assert_unimplemented<T>(result: Result<tonic::Response<T>, tonic::Status>) {
    let status = match result {
        Err(status) => status,
        Ok(_) => panic!("unsupported RPC must not silently succeed"),
    };
    assert_eq!(status.code(), tonic::Code::Unimplemented);
}

#[tokio::test]
async fn canonical_queue_round_trip() {
    let storage = Arc::new(SqliteStorageAdapter::in_memory().expect("in-memory SQLite storage"));
    let service = AgilePlusCoreServer::new(
        storage,
        Arc::new(EventBus::default()),
        Arc::new(ProxyRouter::new(None, None).await),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback listener");
    let address = listener.local_addr().expect("loopback address");
    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(IntegrationsServiceServer::new(service))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("serve integrations service");
    });

    let mut client = IntegrationsServiceClient::connect(format!("http://{address}"))
        .await
        .expect("connect real tonic client");
    let created = client
        .create_backlog_item(CreateBacklogItemRequest {
            r#type: "task".to_string(),
            title: "canonical queue".to_string(),
            body: "persist this".to_string(),
            priority: "high".to_string(),
            feature_id: "ignored-feature".to_string(),
            wp_id: "ignored-wp".to_string(),
            triaged_by: "grpc-contract".to_string(),
        })
        .await
        .expect("create queue item")
        .into_inner()
        .item
        .expect("created item");

    let listed = client
        .list_backlog(ListBacklogRequest {
            type_filter: "task".to_string(),
            state_filter: String::new(),
            feature_slug: String::new(),
        })
        .await
        .expect("list queue items")
        .into_inner()
        .items;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);
    assert_eq!(listed[0].body, "persist this");

    let promotion = client
        .promote_backlog_item(PromoteBacklogItemRequest {
            backlog_item_id: created.id,
            target_type: "story".to_string(),
        })
        .await
        .expect_err("promotion must not report success before it mutates storage");
    assert_eq!(promotion.code(), tonic::Code::Unimplemented);
    assert!(promotion.message().contains("atomic"));

    let invalid_target = client
        .promote_backlog_item(PromoteBacklogItemRequest {
            backlog_item_id: created.id,
            target_type: "   ".to_string(),
        })
        .await
        .expect_err("blank promotion target must be rejected");
    assert_eq!(invalid_target.code(), tonic::Code::InvalidArgument);

    assert_unimplemented(
        client
            .sync_feature_to_plane(SyncFeatureToPlaneRequest {
                feature_slug: "feature".to_string(),
                state: "planned".to_string(),
            })
            .await,
    );
    assert_unimplemented(
        client
            .sync_wp_to_plane(SyncWpToPlaneRequest {
                feature_slug: "feature".to_string(),
                wp_sequence: 1,
                state: "planned".to_string(),
                pr_url: String::new(),
            })
            .await,
    );
    assert_unimplemented(
        client
            .detect_plane_conflicts(DetectPlaneConflictsRequest {
                mirror: "plane".to_string(),
            })
            .await,
    );
    assert_unimplemented(
        client
            .sync_bug_to_git_hub(SyncBugToGitHubRequest {
                title: "bug".to_string(),
                body: String::new(),
                labels: Vec::new(),
                feature_slug: String::new(),
                wp_sequence: 0,
            })
            .await,
    );
    assert_unimplemented(
        client
            .sync_issue_status(SyncIssueStatusRequest {
                mirror: "github".to_string(),
                mirror_id: "1".to_string(),
            })
            .await,
    );
    assert_unimplemented(
        client
            .detect_git_hub_conflicts(DetectGitHubConflictsRequest {
                mirror: "github".to_string(),
            })
            .await,
    );
    assert_unimplemented(
        client
            .classify_input(ClassifyInputRequest {
                input: "triage this".to_string(),
                feature_slug: String::new(),
                wp_sequence: 0,
            })
            .await,
    );
    assert_unimplemented(
        client
            .generate_router(GenerateRouterRequest {
                project_path: String::new(),
                sub_commands: Vec::new(),
            })
            .await,
    );

    server.abort();
}
