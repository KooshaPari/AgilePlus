//! Shared in-process HTTP mock for `agileplus-github` integration tests.
//!
//! Every test that exercises `GitHubClient` or `GitHubSyncAdapter` speaks HTTP
//! to a socket owned by the test process. No test in this crate reaches the
//! real GitHub API, and no test depends on wall-clock time.

#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use agileplus_domain::domain::backlog::{BacklogItem, BacklogPriority, BacklogStatus};
use agileplus_triage::Intent;
use chrono::{DateTime, Utc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A request the mock server received, captured verbatim so tests can assert
/// on method, path, headers and body independently of the client code.
#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl RecordedRequest {
    /// Case-insensitive header lookup.
    pub fn header(&self, name: &str) -> Option<&str> {
        let needle = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(recorded, _)| *recorded == needle)
            .map(|(_, value)| value.as_str())
    }

    /// The body parsed as JSON (panics if the captured body is not JSON).
    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body).expect("captured request body must be valid JSON")
    }
}

/// A canned HTTP response.
#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: &'static str,
    pub content_type: &'static str,
    pub body: String,
}

impl MockResponse {
    /// A `200 OK` JSON response.
    pub fn json(body: impl Into<String>) -> Self {
        Self {
            status: "200 OK",
            content_type: "application/json",
            body: body.into(),
        }
    }

    /// An arbitrary status line with the supplied body.
    pub fn error(status: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.into(),
        }
    }

    /// A `200 OK` response carrying a GitHub-issue-shaped payload.
    pub fn issue(number: i64, title: &str, body: Option<&str>, state: &str) -> Self {
        Self::json(issue_json(number, title, body, state))
    }
}

/// Render a GitHub issue payload; `body` serialises as `null` when absent.
pub fn issue_json(number: i64, title: &str, body: Option<&str>, state: &str) -> String {
    serde_json::json!({
        "number": number,
        "title": title,
        "body": body,
        "state": state,
        "labels": [{"name": "bug"}],
        "updated_at": "2025-01-15T10:30:00Z",
    })
    .to_string()
}

type Handler = Arc<dyn Fn(&RecordedRequest) -> MockResponse + Send + Sync>;

/// A single-route-per-request HTTP/1.1 server bound to an ephemeral loopback
/// port. Connections are answered one request at a time and closed, so the
/// client cannot reuse a socket between assertions.
pub struct MockGitHub {
    base_url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    handle: tokio::task::JoinHandle<()>,
}

impl MockGitHub {
    /// Start a mock server that answers every request through `handler`.
    pub async fn start<F>(handler: F) -> Self
    where
        F: Fn(&RecordedRequest) -> MockResponse + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral loopback port");
        let addr = listener.local_addr().expect("local_addr");
        let handler: Handler = Arc::new(handler);
        let requests: Arc<Mutex<Vec<RecordedRequest>>> = Arc::new(Mutex::new(Vec::new()));

        let recorded = Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let handler = Arc::clone(&handler);
                let recorded = Arc::clone(&recorded);
                tokio::spawn(async move {
                    let Ok(request) = read_request(&mut socket).await else {
                        return;
                    };
                    let response = handler(&request);
                    recorded
                        .lock()
                        .expect("mock request log must not be poisoned")
                        .push(request);
                    let _ = write_response(&mut socket, &response).await;
                });
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            requests,
            handle,
        }
    }

    /// Base URL to hand to `GitHubClient::new` / `LiveGhDataSource::new`.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// A snapshot of every request received so far, in arrival order.
    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("mock request log must not be poisoned")
            .clone()
    }

    /// How many requests have arrived so far.
    pub fn request_count(&self) -> usize {
        self.requests
            .lock()
            .expect("mock request log must not be poisoned")
            .len()
    }
}

impl Drop for MockGitHub {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// A loopback URL that nothing is listening on, for transport-failure paths.
///
/// The port is obtained by binding and immediately dropping a listener, which
/// yields a port the OS will not hand out again during the test run.
pub async fn closed_loopback_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral loopback port");
    let addr = listener.local_addr().expect("local_addr");
    drop(listener);
    format!("http://{addr}")
}

/// A backlog bug with fixed timestamps, so any body derived from it is stable.
pub fn sample_bug(id: i64) -> BacklogItem {
    let created: DateTime<Utc> = DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
        .expect("fixed RFC 3339 timestamp")
        .with_timezone(&Utc);
    BacklogItem {
        id: Some(id),
        title: "Login crash".to_string(),
        description: "App crashes when clicking login".to_string(),
        intent: Intent::Bug,
        priority: BacklogPriority::High,
        status: BacklogStatus::New,
        source: "user-report".to_string(),
        feature_slug: Some("auth".to_string()),
        tags: Vec::new(),
        created_at: created,
        updated_at: created,
    }
}

/// SHA-256 of `content` as lowercase hex, computed independently of the crate.
pub fn sha256_hex(content: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(content.as_bytes()))
}

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|window| window == b"\r\n\r\n")
}

async fn read_request(socket: &mut TcpStream) -> std::io::Result<RecordedRequest> {
    let mut buf: Vec<u8> = Vec::new();
    let mut chunk = [0_u8; 4096];

    let head_end = loop {
        if let Some(position) = find_double_crlf(&buf) {
            break position + 4;
        }
        let read = socket.read(&mut chunk).await?;
        if read == 0 {
            break buf.len();
        }
        buf.extend_from_slice(&chunk[..read]);
    };

    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split(' ');
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut headers: Vec<(String, String)> = Vec::new();
    let mut content_length = 0_usize;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if name == "content-length" {
            content_length = value.parse().unwrap_or(0);
        }
        headers.push((name, value));
    }

    let mut body = buf.get(head_end..).unwrap_or_default().to_vec();
    while body.len() < content_length {
        let read = socket.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }

    Ok(RecordedRequest {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

async fn write_response(socket: &mut TcpStream, response: &MockResponse) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {}\r\ncontent-type: {}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        response.status,
        response.content_type,
        response.body.len()
    );
    socket.write_all(head.as_bytes()).await?;
    socket.write_all(response.body.as_bytes()).await?;
    socket.flush().await?;
    socket.shutdown().await
}
