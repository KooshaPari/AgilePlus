// SPDX-License-Identifier: MIT OR Apache-2.0
//! Server-Sent Events endpoint behaviour tests.
//!
//! `GET /api/v1/stream` subscribes the client to the shared broadcast channel
//! and forwards published domain events as SSE frames. The endpoint streams
//! forever, so the tests drive it two ways:
//!
//! - the handler is called directly and its body stream is polled frame by
//!   frame (deterministic framing assertions, including the lagged-receiver
//!   branch), and
//! - a real server started with `start_api` is driven over TCP to pin the
//!   end-to-end wire behaviour: auth boundary, `text/event-stream` content
//!   type, and forwarded frames.
//!
//! Traceability: WP11-T069

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use agileplus_api::routes::stream::stream_events;
use agileplus_api::start_api;
use axum::body::BodyDataStream;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_stream::StreamExt as _;

use crate::support::{
    MockStorage, TEST_API_KEY, setup_state_with_storage, setup_test_server,
};

const FRAME_TIMEOUT: Duration = Duration::from_secs(15);

/// Poll one SSE frame out of the response body.
async fn next_frame(body: &mut BodyDataStream) -> String {
    let chunk = tokio::time::timeout(FRAME_TIMEOUT, body.next())
        .await
        .expect("an SSE frame should arrive before the timeout")
        .expect("the SSE stream should yield a frame")
        .expect("the SSE frame should not be an error");
    String::from_utf8_lossy(&chunk).into_owned()
}

/// `GET /api/v1/stream` is registered on the protected router.
#[tokio::test]
async fn stream_endpoint_requires_api_key() {
    let server = setup_test_server().await;
    let resp = server.get("/api/v1/stream").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stream_is_served_as_event_stream() {
    let (state, _event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 16);
    let response = stream_events(State(state)).await.into_response();

    assert_eq!(response.status(), StatusCode::OK);
    let content_type = response
        .headers()
        .get("content-type")
        .expect("content-type header present")
        .to_str()
        .expect("utf-8 header")
        .to_string();
    assert!(
        content_type.starts_with("text/event-stream"),
        "SSE responses need the event-stream content type, got: {content_type}"
    );
}

#[tokio::test]
async fn stream_forwards_published_event_as_sse_frame() {
    let (state, event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 16);
    let mut body = stream_events(State(state)).await.into_response().into_body().into_data_stream();

    event_tx
        .send(serde_json::json!({
            "event_type": "feature.specified",
            "data": {"slug": "test-feature", "state": "specified"},
        }))
        .expect("the streaming endpoint holds a subscription");

    let frame = next_frame(&mut body).await;
    assert!(
        frame.contains("event: feature.specified"),
        "frame should carry the published event name, got: {frame}"
    );
    assert!(
        frame.contains("data: ") && frame.contains("test-feature"),
        "frame should carry the published payload, got: {frame}"
    );
}

#[tokio::test]
async fn stream_defaults_event_name_when_payload_has_none() {
    let (state, event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 16);
    let mut body = stream_events(State(state)).await.into_response().into_body().into_data_stream();

    event_tx
        .send(serde_json::json!({"data": {"n": 1}}))
        .expect("the streaming endpoint holds a subscription");

    let frame = next_frame(&mut body).await;
    assert!(
        frame.contains("event: event"),
        "frames without an event_type use the generic name, got: {frame}"
    );
    assert!(
        frame.contains("\"n\":1"),
        "frame should carry the published payload, got: {frame}"
    );
}

#[tokio::test]
async fn stream_renders_null_data_when_payload_has_none() {
    let (state, event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 16);
    let mut body = stream_events(State(state)).await.into_response().into_body().into_data_stream();

    event_tx
        .send(serde_json::json!({"event_type": "ping"}))
        .expect("the streaming endpoint holds a subscription");

    let frame = next_frame(&mut body).await;
    assert!(frame.contains("event: ping"), "got: {frame}");
    assert!(
        frame.contains("data: null"),
        "missing payloads render as JSON null, got: {frame}"
    );
}

#[tokio::test]
async fn stream_reports_lagging_subscriber_instead_of_dropping_connection() {
    // Capacity 2 with 5 unconsumed messages guarantees the receiver lags.
    let (state, event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 2);
    let mut body = stream_events(State(state)).await.into_response().into_body().into_data_stream();

    for n in 0..5 {
        let _ = event_tx.send(serde_json::json!({"event_type": "tick", "data": n}));
    }

    let frame = next_frame(&mut body).await;
    assert!(
        frame.contains("lagged"),
        "a lagged subscriber should get a keepalive comment, got: {frame}"
    );
}

// ── End-to-end over TCP ──────────────────────────────────────────────────────

async fn connect_with_retry(addr: SocketAddr) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => return stream,
            Err(error) => {
                assert!(
                    Instant::now() < deadline,
                    "server never accepted connections on {addr}: {error}"
                );
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        }
    }
}

/// Read from the socket until `needle` appears in the response text.
async fn read_until(stream: &mut TcpStream, needle: &str, what: &str) -> String {
    let mut buf = String::new();
    let mut chunk = [0u8; 4096];
    let read = tokio::time::timeout(FRAME_TIMEOUT, async {
        while !buf.contains(needle) {
            let n = stream.read(&mut chunk).await.expect("read from API server");
            if n == 0 {
                break;
            }
            buf.push_str(&String::from_utf8_lossy(&chunk[..n]));
        }
    })
    .await;
    assert!(read.is_ok(), "timed out waiting for {what}; saw: {buf:?}");
    assert!(buf.contains(needle), "never saw {what}; saw: {buf:?}");
    buf
}

/// Start the real API server and drive `GET /api/v1/stream` over TCP.
#[tokio::test]
async fn start_api_streams_events_and_enforces_auth_over_tcp() {
    let (state, event_tx) = setup_state_with_storage(MockStorage::with_test_data(), 16);

    // Reserve an ephemeral port, then let `start_api` bind it.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind an ephemeral port");
    let addr = listener.local_addr().expect("ephemeral address");
    drop(listener);

    let server = tokio::spawn(async move {
        let _ = start_api(addr, state).await;
    });

    // Unauthenticated requests are rejected before the stream opens.
    let mut unauthenticated = connect_with_retry(addr).await;
    unauthenticated
        .write_all(
            format!(
                "GET /api/v1/stream HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write unauthenticated request");
    let rejected = read_until(&mut unauthenticated, "Missing API key", "the 401 reply").await;
    assert!(
        rejected.starts_with("HTTP/1.1 401"),
        "missing credentials must fail closed, got: {rejected:?}"
    );
    drop(unauthenticated);

    // A present-but-wrong credential is rejected with a distinct message.
    let mut bad_key = connect_with_retry(addr).await;
    bad_key
        .write_all(
            format!(
                "GET /api/v1/stream HTTP/1.1\r\nHost: {addr}\r\nX-API-Key: not-a-real-key\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write invalid-key request");
    let rejected = read_until(&mut bad_key, "Invalid API key", "the 401 reply").await;
    assert!(
        rejected.starts_with("HTTP/1.1 401"),
        "a wrong API key must fail closed, got: {rejected:?}"
    );
    drop(bad_key);

    // Authenticated requests open the event stream and receive frames.
    let mut authenticated = connect_with_retry(addr).await;
    authenticated
        .write_all(
            format!(
                "GET /api/v1/stream HTTP/1.1\r\nHost: {addr}\r\nX-API-Key: {TEST_API_KEY}\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .expect("write authenticated request");

    let headers = read_until(&mut authenticated, "\r\n\r\n", "the SSE response head").await;
    assert!(
        headers.starts_with("HTTP/1.1 200"),
        "authenticated stream should open, got: {headers:?}"
    );
    assert!(
        headers.to_lowercase().contains("content-type: text/event-stream"),
        "stream should be served as SSE, got: {headers:?}"
    );

    event_tx
        .send(serde_json::json!({
            "event_type": "work_package.done",
            "data": {"wp_id": 1},
        }))
        .expect("the server holds a subscription");

    let frames = read_until(
        &mut authenticated,
        "event: work_package.done",
        "the forwarded SSE frame",
    )
    .await;
    assert!(
        frames.contains("\"wp_id\":1"),
        "forwarded frame should carry the published payload, got: {frames:?}"
    );

    server.abort();
}
