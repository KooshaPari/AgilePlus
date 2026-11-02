//! Native HTTP witnesses for real-project provenance, snapshots and read-only GETs.
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Server {
    root: PathBuf,
    child: Child,
    origin: String,
}
impl Server {
    async fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "agileplus-source-regression-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join(".agileplus")).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q", "-b", "main"])
                .arg(&root)
                .status()
                .unwrap()
                .success()
        );
        let connection = rusqlite::Connection::open(root.join(".agileplus/agileplus.db")).unwrap();
        connection.execute_batch("CREATE TABLE features (id INTEGER PRIMARY KEY, slug TEXT, friendly_name TEXT,
            state TEXT, spec_hash BLOB, target_branch TEXT, created_at TEXT, updated_at TEXT, module_id INTEGER, labels TEXT);
            CREATE TABLE work_packages (id INTEGER PRIMARY KEY, feature_id INTEGER, title TEXT, state TEXT,
            sequence INTEGER, file_scope TEXT, acceptance_criteria TEXT, agent_id TEXT, pr_url TEXT,
            pr_state TEXT, worktree_path TEXT, created_at TEXT, updated_at TEXT);
            CREATE TABLE projects (id INTEGER PRIMARY KEY, slug TEXT, name TEXT, description TEXT, created_at TEXT, updated_at TEXT);
            CREATE TABLE epics (id INTEGER, title TEXT, status TEXT, requirement_id TEXT);
            CREATE TABLE stories (id INTEGER, epic_id INTEGER, title TEXT, status TEXT, requirement_id TEXT);
            INSERT INTO epics VALUES (9,'Selected project obligation','active','FR-REAL-900');
            INSERT INTO stories VALUES (10,9,'Selected project story','todo','FR-REAL-901');
            INSERT INTO projects VALUES (1,'selected','Selected Project','','2026-10-14T00:00:00Z','2026-10-14T00:00:00Z');
            INSERT INTO work_packages VALUES (1,1,'Actual WP','doing',1,'[]','FR-REAL-900 requires real proof',NULL,NULL,NULL,NULL,'2026-10-14T00:00:00Z','2026-10-14T00:00:00Z');").unwrap();
        connection.execute("INSERT INTO features VALUES (1,'actual-feature','Actual Feature','implementing',?1,'main','2026-10-14T00:00:00Z','2026-10-14T00:00:00Z',NULL,'[\"actual\"]')", [vec![7_u8;32]]).unwrap();
        drop(connection);
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let child = Command::new(env!("CARGO_BIN_EXE_agileplus-dashboard"))
            .current_dir(&root)
            .env("AGILEPLUS_PROJECT_ROOT", &root)
            .env("PORT", port.to_string())
            .env("DASHBOARD_HOST", "127.0.0.1")
            .env("DATABASE_PATH", root.join("unselected.db"))
            .env_remove("DATABASE_URL")
            .env_remove("AGILEPLUS_DASHBOARD_DEMO")
            .env_remove("PLANE_API_KEY")
            .env_remove("PLANE_WORKSPACE")
            .env_remove("PLANE_PROJECT")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut server = Self {
            root,
            child,
            origin: format!("http://127.0.0.1:{port}"),
        };
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            assert!(
                server.child.try_wait().unwrap().is_none(),
                "dashboard exited during startup"
            );
            if reqwest::get(format!("{}/api/dashboard/source.json", server.origin))
                .await
                .is_ok()
            {
                break;
            }
            assert!(Instant::now() < deadline, "dashboard did not start");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        server
    }
    async fn json(&self, path: &str) -> serde_json::Value {
        reqwest::get(format!("{}{path}", self.origin))
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[tokio::test]
async fn real_project_detail_does_not_invent_evidence_or_reports() {
    let server = Server::new("artifacts").await;
    let html = reqwest::get(format!("{}/features/1", server.origin))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(html.contains("Actual Feature"));
    for invented in [
        "feature_summary",
        "bundle-1-wp-1",
        "coverage-engine",
        "/artifacts/features/actual-feature.md",
    ] {
        assert!(
            !html.contains(invented),
            "real project rendered invented artifact: {invented}"
        );
    }
    assert!(!server.root.join(".agileplus/evidence").exists());
}

#[tokio::test]
async fn snapshot_does_not_expose_unbound_evidence_from_process_cwd() {
    let server = Server::new("unbound-evidence").await;
    fs::create_dir_all(server.root.join(".agileplus/evidence/1")).unwrap();
    fs::write(
        server.root.join(".agileplus/evidence/1/unbound.txt"),
        b"unverified artifact",
    )
    .unwrap();
    for endpoint in [
        "/api/features/1/evidence",
        "/api/dashboard/features/1/evidence.json",
        "/api/evidence/1/unbound.txt/content",
        "/api/evidence/1/unbound.txt/preview",
    ] {
        let response = reqwest::get(format!("{}{endpoint}", server.origin))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
            "snapshot must not pretend process-CWD artifacts are authoritative: {endpoint}"
        );
    }
}

#[tokio::test]
async fn get_reads_selected_project_and_never_creates_another_database() {
    let server = Server::new("selected-get").await;
    let response = server.json("/api/dashboard/epics-stories.json").await;
    assert_eq!(response["epics"][0]["title"], "Selected project obligation");
    assert_eq!(response["epics"][0]["requirement_id"], "FR-REAL-900");
    assert!(!server.root.join("unselected.db").exists());
    assert!(!server.root.join("agileplus.db").exists());
}

#[tokio::test]
async fn startup_snapshot_never_claims_an_ongoing_live_connection() {
    let server = Server::new("snapshot").await;
    let source_before = server.json("/api/dashboard/source.json").await;
    fs::rename(
        server.root.join(".agileplus/agileplus.db"),
        server.root.join("removed.db"),
    )
    .unwrap();
    let source_after = server.json("/api/dashboard/source.json").await;
    assert_eq!(source_after["mode"], "project_database_snapshot");
    assert_eq!(source_after["connected"], false);
    assert_eq!(source_after["loaded_at"], source_before["loaded_at"]);
    assert!(
        source_after["message"]
            .as_str()
            .unwrap()
            .contains("snapshot")
    );
    let html = reqwest::get(format!("{}/features/1", server.origin))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!html.contains("Live read-only"));
    assert!(html.contains("Snapshot loaded at"));
}
