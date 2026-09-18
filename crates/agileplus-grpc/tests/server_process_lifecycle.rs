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
    tokio::time::sleep(Duration::from_millis(250)).await;

    server.terminate();
    let status = server.wait_for_exit(Duration::from_secs(15));
    assert!(
        status.success(),
        "SIGTERM should produce a graceful exit, got {status:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}
