//! Process-level tests for the gRPC core server: startup, the real wire
//! protocol, and graceful shutdown.
//!
//! The in-process tests drive the handler bodies directly. This file covers
//! what only shows up in a real process: `main`, `CoreConfig::from_env` and
//! `ProjectContext` discovery, `start_server` binding a socket, tonic client /
//! server serialization, and the SIGTERM shutdown path.
//!
//! The child process receives its configuration through its own environment, so
//! nothing here mutates the test runner's environment.
//!
//! Traceability: WP14-T079, T080

#![cfg(unix)]

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use agileplus_proto::agileplus::v1::agile_plus_core_service_client::AgilePlusCoreServiceClient;
use agileplus_proto::agileplus::v1::{
    GetFeatureRequest, ListFeaturesRequest, ProjectScope, VerifyAuditChainRequest,
};
use tonic::Code;

/// Kills the child process if the test panics before the explicit shutdown.
struct ServerProcess {
    child: Child,
}

impl ServerProcess {
    fn spawn(root: &Path, port: u16) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_agileplus-grpc"))
            .env("AGILEPLUS_GRPC_BIND", format!("127.0.0.1:{port}"))
            .env("AGILEPLUS_PROJECT_ROOT", root)
            .env("AGILEPLUS_API_KEY", "fixture-queue-key")
            .env_remove("AGILEPLUS_CORE_DATABASE_PATH")
            .env_remove("AGILEPLUS_DB_PATH")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("the agileplus-grpc binary should spawn");
        Self { child }
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Request a graceful shutdown through SIGTERM.
    fn terminate(&self) {
        let ok = Command::new("kill")
            .args(["-TERM", &self.pid().to_string()])
            .status()
            .expect("kill should be available")
            .success();
        assert!(ok, "SIGTERM should be delivered to pid {}", self.pid());
    }

    /// Wait for exit, returning the exit code.
    fn wait_for_exit(&mut self, within: Duration) -> std::process::ExitStatus {
        let deadline = Instant::now() + within;
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("child status should be readable")
            {
                return status;
            }
            if Instant::now() >= deadline {
                let _ = self.child.kill();
                panic!("the server did not exit within {within:?}");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Reserve a port by binding and immediately releasing it.
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("a loopback port should be bindable")
        .local_addr()
        .expect("the bound address should be readable")
        .port()
}

/// Create a throwaway git worktree for the server to own.
fn temporary_project_root() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("the clock should be after the epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "agileplus-grpc-server-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temporary project root should be creatable");
    let init = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .output()
        .expect("git init should run");
    assert!(init.status.success(), "git init failed: {init:?}");
    std::fs::canonicalize(&dir).expect("the project root should canonicalize")
}

/// Poll the socket until the server accepts connections.
async fn await_listening(child: &mut ServerProcess, port: u16) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.child.try_wait().expect("child status") {
            panic!("the server exited before it started listening: {status}");
        }
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the server never accepted a connection on 127.0.0.1:{port}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn server_binary_serves_requests_and_shuts_down_on_sigterm() {
    let root = temporary_project_root();
    let port = free_port();
    let mut server = ServerProcess::spawn(&root, port);

    await_listening(&mut server, port).await;

    let mut queue = agileplus_proto::agileplus::v1::integrations_service_client::IntegrationsServiceClient::connect(format!("http://127.0.0.1:{port}"))
        .await.expect("queue client connects");
    use agileplus_proto::agileplus::v1::{
        CreateBacklogItemRequest, ListBacklogRequest, PromoteBacklogItemRequest,
    };
    let scope = ProjectScope {
        canonical_repo_root: root.to_string_lossy().to_string(),
    };
    let request = CreateBacklogItemRequest {
        r#type: "task".into(),
        title: "isolated queue item".into(),
        body: "test body".into(),
        feature_id: "feature-a".into(),
        triaged_by: "test".into(),
        project_scope: Some(scope.clone()),
        ..Default::default()
    };
    assert_eq!(
        queue
            .create_backlog_item(request.clone())
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let mut wrong = tonic::Request::new(request.clone());
    wrong
        .metadata_mut()
        .insert("authorization", "Bearer invalid-key".parse().unwrap());
    assert_eq!(
        queue.create_backlog_item(wrong).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    assert_eq!(
        queue
            .list_backlog(ListBacklogRequest {
                project_scope: Some(scope.clone()),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let mut wrong_list = tonic::Request::new(ListBacklogRequest {
        project_scope: Some(scope.clone()),
        ..Default::default()
    });
    wrong_list
        .metadata_mut()
        .insert("authorization", "Bearer invalid-key".parse().unwrap());
    assert_eq!(
        queue.list_backlog(wrong_list).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    drop(queue);
    let channel = tonic::transport::Channel::from_shared(format!("http://127.0.0.1:{port}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut queue = agileplus_proto::agileplus::v1::integrations_service_client::IntegrationsServiceClient::with_interceptor(channel, |mut r: tonic::Request<()>| {
        r.metadata_mut().insert("authorization", "Bearer fixture-queue-key".parse().unwrap());
        Ok(r)
    });
    let mut unscoped = request.clone();
    unscoped.project_scope = None;
    assert_eq!(
        queue
            .create_backlog_item(unscoped)
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut foreign = request.clone();
    foreign.project_scope = Some(ProjectScope {
        canonical_repo_root: "/foreign".into(),
    });
    assert_eq!(
        queue.create_backlog_item(foreign).await.unwrap_err().code(),
        Code::PermissionDenied
    );
    let mut unsupported_wp = request.clone();
    unsupported_wp.wp_id = "WP01".into();
    assert_eq!(
        queue
            .create_backlog_item(unsupported_wp)
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let unscoped_list = ListBacklogRequest::default();
    assert_eq!(
        queue.list_backlog(unscoped_list).await.unwrap_err().code(),
        Code::InvalidArgument
    );
    let item = queue
        .create_backlog_item(request)
        .await
        .expect("scoped create")
        .into_inner()
        .item
        .expect("item");
    assert_eq!(item.title, "isolated queue item");
    let list = queue
        .list_backlog(ListBacklogRequest {
            feature_slug: "feature-a".into(),
            project_scope: Some(scope.clone()),
            ..Default::default()
        })
        .await
        .expect("scoped list")
        .into_inner();
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].id, item.id);
    assert_eq!(
        queue
            .promote_backlog_item(PromoteBacklogItemRequest {
                backlog_item_id: item.id,
                target_type: "feature".into()
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unimplemented
    );

    // The server's canonical repository root is the canonicalized project
    // root, so scope-bound requests must name exactly that path.
    let canonical_root = root.to_string_lossy().to_string();
    let scope = Some(ProjectScope {
        canonical_repo_root: canonical_root.clone(),
    });

    let mut client = AgilePlusCoreServiceClient::connect(format!("http://127.0.0.1:{port}"))
        .await
        .expect("a client should connect to the running server");

    let listed = client
        .list_features(ListFeaturesRequest {
            state_filter: String::new(),
            project_scope: scope.clone(),
        })
        .await
        .expect("list_features should succeed over the wire")
        .into_inner();
    assert!(
        listed.features.is_empty(),
        "a fresh core starts with no features"
    );

    // A real repository directory owns real state, so a malformed scope must
    // still be refused at the transport boundary.
    let denied = client
        .list_features(ListFeaturesRequest {
            state_filter: String::new(),
            project_scope: Some(ProjectScope {
                canonical_repo_root: "/repo/not-the-project".into(),
            }),
        })
        .await
        .expect_err("a foreign scope must be rejected");
    assert_eq!(denied.code(), Code::PermissionDenied);

    // Domain errors survive the wire as the mapped gRPC status.
    let missing = client
        .get_feature(GetFeatureRequest {
            slug: "does-not-exist".into(),
            project_scope: scope.clone(),
        })
        .await
        .expect_err("an unknown feature must fail");
    assert_eq!(missing.code(), Code::NotFound);
    assert!(missing.message().contains("does-not-exist"));

    // An empty audit chain is reported as an error payload, not a status.
    let chain = client
        .verify_audit_chain(VerifyAuditChainRequest {
            feature_slug: "does-not-exist".into(),
            project_scope: scope,
        })
        .await
        .expect_err("verifying an unknown feature must fail");
    assert_eq!(chain.code(), Code::NotFound);

    // The database lives inside the project's `.agileplus` state directory.
    assert!(
        root.join(".agileplus").join("agileplus.db").exists(),
        "the server should create its SQLite store under the project root"
    );

    // Observed behaviour: `serve_with_shutdown` waits for every open connection,
    // and tonic counts an accepted socket as in-flight until the client closes
    // it, so a held-open channel stalls shutdown indefinitely. Release the
    // client before signalling, the way a draining deploy would.
    drop(client);
    drop(queue);
    tokio::time::sleep(Duration::from_millis(250)).await;

    server.terminate();
    let status = server.wait_for_exit(Duration::from_secs(15));
    assert!(
        status.success(),
        "SIGTERM should produce a graceful exit, got {status:?}"
    );

    let mut restarted = ServerProcess::spawn(&root, port);
    await_listening(&mut restarted, port).await;
    let channel = tonic::transport::Channel::from_shared(format!("http://127.0.0.1:{port}"))
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut queue = agileplus_proto::agileplus::v1::integrations_service_client::IntegrationsServiceClient::with_interceptor(channel, |mut r: tonic::Request<()>| {
        r.metadata_mut().insert("authorization", "Bearer fixture-queue-key".parse().unwrap());
        Ok(r)
    });
    let items = queue
        .list_backlog(ListBacklogRequest {
            feature_slug: "feature-a".into(),
            project_scope: Some(ProjectScope {
                canonical_repo_root: root.to_string_lossy().to_string(),
            }),
            ..Default::default()
        })
        .await
        .expect("list after restart")
        .into_inner()
        .items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item.id);
    drop(queue);
    tokio::time::sleep(Duration::from_millis(250)).await;
    restarted.terminate();
    assert!(restarted.wait_for_exit(Duration::from_secs(15)).success());

    let _ = std::fs::remove_dir_all(&root);
}
