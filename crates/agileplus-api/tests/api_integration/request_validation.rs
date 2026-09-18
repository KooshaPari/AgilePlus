// SPDX-License-Identifier: MIT OR Apache-2.0
//! HTTP-contract behaviour that sits *outside* the handlers.
//!
//! Covers the extractor rejection shapes axum produces for malformed requests
//! (unsupported media type, unparseable path/query parameters, malformed JSON
//! bodies), the permissive CORS layer, and the statically-served asset mount.
//! None of these were exercised before: the existing suites only ever sent
//! well-formed `application/json` requests to valid paths.
//!
//! Traceability: WP11-T065, WP15-T086

use axum::body::Bytes;
use axum::http::{Method, StatusCode};

use crate::support::{TEST_API_KEY, setup_test_server};

const KEY: &str = "X-API-Key";

// ── Request validation: extractor rejections ─────────────────────────────────

#[tokio::test]
async fn non_json_content_type_is_415() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .text("title=not-json")
        .await;
    resp.assert_status(StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn malformed_json_body_is_400() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features")
        .add_header(KEY, TEST_API_KEY)
        .content_type("application/json")
        .bytes(Bytes::from_static(br#"{"title": "unterminated"#))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn non_numeric_path_segment_is_400() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/not-a-number")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn non_numeric_query_filter_is_400() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/events?entity_id=not-a-number")
        .add_header(KEY, TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

/// Auth is enforced before the handler's extractors, so a malformed request
/// without credentials must still fail closed with 401 rather than 400/415.
#[tokio::test]
async fn malformed_request_without_credentials_is_401() {
    let server = setup_test_server().await;
    let resp = server.post("/api/v1/features").text("title=not-json").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

// ── CORS ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn cors_allow_origin_is_sent_for_simple_requests() {
    let server = setup_test_server().await;
    let resp = server
        .get("/health")
        .add_header("origin", "https://example.com")
        .await;
    resp.assert_status_ok();
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .expect("CORS layer should answer a cross-origin request")
            .to_str()
            .expect("header is ascii"),
        "*",
        "the API is configured permissive"
    );
}

/// Preflight requests carry no credentials; the CORS layer must answer them
/// itself rather than letting the auth middleware reject the browser.
#[tokio::test]
async fn cors_preflight_is_answered_without_credentials() {
    let server = setup_test_server().await;
    let resp = server
        .method(Method::OPTIONS, "/api/v1/features")
        .add_header("origin", "https://example.com")
        .add_header("access-control-request-method", "POST")
        .add_header("access-control-request-headers", "x-api-key")
        .await;

    assert!(
        resp.status_code().is_success(),
        "preflight must not be rejected by auth, got: {}",
        resp.status_code()
    );
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .expect("preflight response advertises the allowed origin")
            .to_str()
            .expect("header is ascii"),
        "*"
    );
    assert!(
        resp.headers().get("access-control-allow-methods").is_some(),
        "preflight response should advertise the allowed methods"
    );
}

// ── Static asset mount ───────────────────────────────────────────────────────

/// `/static` is mounted on the public part of the router: a missing asset is a
/// plain 404, not a 401.
#[tokio::test]
async fn static_asset_missing_is_404_without_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/static/definitely-absent.css").await;
    resp.assert_status(StatusCode::NOT_FOUND);
}
