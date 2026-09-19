// SPDX-License-Identifier: MIT OR Apache-2.0
//! Authentication middleware behaviour tests.
//!
//! Two middlewares guard the API:
//!
//! - `middleware::auth::validate_api_key` — the `CredentialStore`-backed key
//!   check that the production router installs on every `/api/*` route.
//! - `middleware::auth::authorize` — the hexagonal `TokenVerifier` path used
//!   when an auth backend (shared secret, later JWT) is wired into the router.
//!
//! Both are exercised here over real HTTP requests: public-path bypass, token
//! extraction precedence, the 401 contract (status, JSON shape, message), and
//! the 500 contract when the verifier backend itself fails.
//!
//! Traceability: FR-030 / FR-AGP-012

use std::sync::Arc;

use agileplus_api::middleware::auth::{authorize, validate_api_key};
use agileplus_api::middleware::token_verifier::{
    DynTokenVerifier, SharedSecretVerifier, TokenVerifier,
};
use agileplus_domain::credentials::{
    CredentialStore, InMemoryCredentialStore, format_api_key_hash, keys as cred_keys,
};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router, middleware};
use axum_test::TestServer;

const API_KEY_HEADER: &str = "X-API-Key";
const VALID_KEY: &str = "auth-middleware-test-key";

/// Verifier that fails, standing in for an unreachable keystore.
///
/// `TokenVerifier::verify` is documented to treat *invalid* tokens as
/// `Ok(false)`; only infrastructure failures return `Err`.
struct FailingVerifier;

impl TokenVerifier for FailingVerifier {
    fn verify(&self, _token: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Err("keystore unreachable".into())
    }
}

fn shared_secret_verifier(keys: &[&str]) -> DynTokenVerifier {
    Arc::new(SharedSecretVerifier::new(
        keys.iter().map(|k| (*k).to_owned()).collect(),
    ))
}

/// Router guarded by the `TokenVerifier`-based `authorize` middleware.
fn authorize_app(verifier: DynTokenVerifier) -> TestServer {
    let app = Router::new()
        .route(
            "/protected",
            get(|| async { Json(serde_json::json!({"ok": true})) }),
        )
        .route("/health", get(|| async { "healthy" }))
        .route("/info", get(|| async { "info" }))
        .route("/webhooks/github", get(|| async { "hook" }))
        .layer(middleware::from_fn_with_state(verifier, authorize));
    TestServer::new(app)
}

/// Router guarded by the `CredentialStore`-based `validate_api_key` middleware.
fn credential_app() -> TestServer {
    let creds: Arc<dyn CredentialStore> = {
        let store = InMemoryCredentialStore::new();
        store
            .set("agileplus", cred_keys::API_KEYS, &format_api_key_hash(VALID_KEY))
            .expect("seeding the test API key should succeed");
        Arc::new(store)
    };

    let app = Router::new()
        .route(
            "/protected",
            get(|| async { Json(serde_json::json!({"ok": true})) }),
        )
        .route("/health", get(|| async { "healthy" }))
        .route("/info", get(|| async { "info" }))
        .route("/webhooks/github", get(|| async { "hook" }))
        .layer(middleware::from_fn_with_state(creds, validate_api_key));
    TestServer::new(app)
}

// ── authorize (TokenVerifier path) ───────────────────────────────────────────

#[tokio::test]
async fn authorize_accepts_bearer_token() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    let resp = server
        .get("/protected")
        .add_header("Authorization", format!("Bearer {VALID_KEY}"))
        .await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<serde_json::Value>()["ok"], true);
}

#[tokio::test]
async fn authorize_accepts_x_api_key_header() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, VALID_KEY)
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn authorize_rejects_invalid_token() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, "not-the-key")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.json::<serde_json::Value>()["error"],
        "Invalid API key",
        "401 body must describe the failure without echoing the rejected token"
    );
}

#[tokio::test]
async fn authorize_rejects_request_without_credentials() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    let resp = server.get("/protected").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    let message = resp.json::<serde_json::Value>()["error"]
        .as_str()
        .expect("401 body carries an error message")
        .to_string();
    for expected in ["Authorization Bearer", "X-API-Key", "api_key"] {
        assert!(
            message.contains(expected),
            "401 message should name the accepted credential sources, got: {message}"
        );
    }
}

#[tokio::test]
async fn authorize_passes_public_paths_without_credentials() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    for path in ["/health", "/info", "/webhooks/github"] {
        let resp = server.get(path).await;
        resp.assert_status_ok();
    }
}

#[tokio::test]
async fn authorize_returns_generic_500_when_verifier_fails() {
    let server = authorize_app(Arc::new(FailingVerifier));
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, VALID_KEY)
        .await;
    resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    let body = resp.text();
    assert!(
        !body.contains("keystore unreachable"),
        "verifier errors must not leak to clients, got: {body}"
    );
    assert!(
        body.contains("internal server error"),
        "500 body should be the generic envelope, got: {body}"
    );
}

// ── validate_api_key (CredentialStore path) ──────────────────────────────────

#[tokio::test]
async fn validate_api_key_passes_public_paths_without_credentials() {
    let server = credential_app();
    for path in ["/health", "/info", "/webhooks/github"] {
        let resp = server.get(path).await;
        resp.assert_status_ok();
    }
}

#[tokio::test]
async fn validate_api_key_accepts_bearer_token() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header("Authorization", format!("Bearer {VALID_KEY}"))
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn validate_api_key_accepts_query_param_alongside_other_params() {
    let server = credential_app();
    let resp = server
        .get(&format!("/protected?limit=5&api_key={VALID_KEY}"))
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn validate_api_key_falls_back_when_authorization_scheme_is_not_bearer() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header("Authorization", "Basic dXNlcjpwYXNzd29yZA==")
        .add_header(API_KEY_HEADER, VALID_KEY)
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn validate_api_key_rejects_empty_header_value() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, "")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.json::<serde_json::Value>()["error"],
        "Invalid API key",
        "an empty key is present-but-invalid, not missing"
    );
}

#[tokio::test]
async fn validate_api_key_rejects_empty_query_param_value() {
    let server = credential_app();
    let resp = server.get("/protected?api_key=").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    assert_eq!(
        resp.json::<serde_json::Value>()["error"],
        "Invalid API key"
    );
}

/// A credential header that is not valid UTF-8 cannot be read as a token, so
/// the middleware must treat the request as *missing* credentials ("Missing API
/// key") rather than as a present-but-wrong key ("Invalid API key"). The
/// `to_str().ok()` guard is what decides this.
#[tokio::test]
async fn validate_api_key_treats_a_non_utf8_header_as_missing() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, &b"\xff\xfe\xfd"[..])
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    let error = resp.json::<serde_json::Value>()["error"]
        .as_str()
        .expect("the 401 body carries an error string")
        .to_string();
    assert!(
        error.starts_with("Missing API key"),
        "unreadable bytes are no credential at all, got: {error}"
    );
}

/// An unreadable `Authorization` header must not abort extraction: the
/// `X-API-Key` fallback still authenticates the request.
#[tokio::test]
async fn validate_api_key_skips_a_non_utf8_authorization_header() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header("Authorization", &b"Bearer \xff\xfe"[..])
        .add_header(API_KEY_HEADER, VALID_KEY)
        .await;
    resp.assert_status_ok();
}

/// Same guard on the `bearer ` prefix path: unreadable bytes plus a valid query
/// param must still succeed.
#[tokio::test]
async fn authorize_skips_a_non_utf8_authorization_header() {
    let server = authorize_app(shared_secret_verifier(&[VALID_KEY]));
    let resp = server
        .get("/protected")
        .add_header("Authorization", &b"\xff\xfe\xfd"[..])
        .add_header(API_KEY_HEADER, VALID_KEY)
        .await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn validate_api_key_401_is_json() {
    let server = credential_app();
    let resp = server
        .get("/protected")
        .add_header(API_KEY_HEADER, "wrong")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
    let content_type = resp
        .headers()
        .get("content-type")
        .expect("content-type header")
        .to_str()
        .expect("utf-8 header")
        .to_string();
    assert!(
        content_type.contains("application/json"),
        "401 responses should be JSON, got: {content_type}"
    );
}
