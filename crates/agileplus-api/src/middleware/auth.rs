//! API key authentication middleware.
//!
//! Protected endpoints require the `X-API-Key` header, `Authorization: Bearer`,
//! or `?api_key=` query param.
//! Health, info, and webhook endpoints are always accessible without API key auth.
//!
//! Keys are validated via [`TokenVerifier`](super::token_verifier::TokenVerifier)
//! (Bearer / shared-secret path) or via the `CredentialStore` (legacy API-key path).
//! The raw key value is never logged.
//!
//! Traceability: FR-030 / WP11-T065 / FR-AGP-012

use axum::extract::Request;
use axum::http::HeaderMap;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use agileplus_domain::credentials::CredentialStore;

use crate::error::ApiError;
use crate::middleware::token_verifier::DynTokenVerifier;

/// Paths that do not require authentication.
const PUBLIC_PATHS: &[&str] = &["/health", "/info", "/webhooks"];

/// Extract a candidate token from `Authorization: Bearer`, `X-API-Key`, or `?api_key=`.
fn extract_token(headers: &HeaderMap, request: &Request) -> Result<String, ApiError> {
    if let Some(auth) = headers.get("Authorization").and_then(|v| v.to_str().ok())
        && let Some(token) = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "))
    {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    if let Some(header_val) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        return Ok(header_val.to_string());
    }

    if let Some(query) = request.uri().query()
        && let Some(v) = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == "api_key").then(|| v.to_string())
        })
    {
        return Ok(v);
    }

    Err(ApiError::Unauthorized(
        "Missing API key (Authorization Bearer, X-API-Key header, or ?api_key= param required)"
            .to_string(),
    ))
}

/// axum middleware that validates a bearer token / API key via [`DynTokenVerifier`].
///
/// Used by the composed router for protected `/api/*` routes.
pub async fn authorize(
    axum::extract::State(verifier): axum::extract::State<DynTokenVerifier>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let path = request.uri().path().to_string();

    if PUBLIC_PATHS.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    let token = extract_token(&headers, &request)?;
    let valid = verifier
        .verify(&token)
        .map_err(|e| ApiError::Internal(format!("token verifier error: {e}")))?;

    if !valid {
        let key_hint: String = token.chars().take(4).chain(['*'; 8]).collect();
        warn!(key_hint, "API authentication failed for key hint");
        return Err(ApiError::Unauthorized("Invalid API key".to_string()));
    }

    Ok(next.run(request).await)
}

/// axum middleware that validates the `X-API-Key` header (or `?api_key=` query
/// param) for all non-public endpoints via [`CredentialStore`].
///
/// Returns `401 Unauthorized` if the header/param is missing or the key is invalid.
pub async fn validate_api_key(
    axum::extract::State(creds): axum::extract::State<std::sync::Arc<dyn CredentialStore>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let path = request.uri().path().to_string();

    // Always allow public endpoints (health, info, webhooks).
    if PUBLIC_PATHS.iter().any(|p| path.starts_with(p)) {
        return Ok(next.run(request).await);
    }

    let api_key = extract_token(&headers, &request)?;

    let valid = validate_api_key_value(creds.as_ref(), &api_key)?;

    if !valid {
        // Log only a truncated hint for identification — never the raw key.
        let key_hint: String = api_key.chars().take(4).chain(['*'; 8]).collect();
        warn!(key_hint, "API authentication failed for key hint");
        return Err(ApiError::Unauthorized("Invalid API key".to_string()));
    }

    Ok(next.run(request).await)
}

fn validate_api_key_value(
    credentials: &dyn CredentialStore,
    api_key: &str,
) -> Result<bool, ApiError> {
    credentials
        .validate_api_key(api_key)
        .map_err(|error| ApiError::Internal(format!("credential store error: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::credentials::{InMemoryCredentialStore, format_api_key_hash, keys};
    use axum::{Router, middleware, routing::get};
    use axum_test::TestServer;

    #[test]
    fn protected_auth_reads_rotated_key_from_credential_store() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("old-key"))
            .unwrap();
        assert!(validate_api_key_value(&credentials, "old-key").unwrap());
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("new-key"))
            .unwrap();
        assert!(!validate_api_key_value(&credentials, "old-key").unwrap());
        assert!(validate_api_key_value(&credentials, "new-key").unwrap());
    }

    #[test]
    fn validate_api_key_empty_key_rejected() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("valid-key"))
            .unwrap();
        assert!(!validate_api_key_value(&credentials, "").unwrap());
    }

    #[test]
    fn validate_api_key_wrong_key_rejected() {
        let credentials = InMemoryCredentialStore::new();
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("correct-key"))
            .unwrap();
        assert!(!validate_api_key_value(&credentials, "wrong-key").unwrap());
    }

    #[test]
    fn validate_api_key_no_keys_stored() {
        let credentials = InMemoryCredentialStore::new();
        assert!(!validate_api_key_value(&credentials, "any-key").unwrap());
    }

    #[test]
    fn extract_token_from_bearer_header() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Bearer my-token"));
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "my-token");
    }

    #[test]
    fn extract_token_from_lowercase_bearer() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("bearer lowercase-token"));
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "lowercase-token");
    }

    #[test]
    fn extract_token_from_x_api_key_header() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_static("x-api-key-value"));
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "x-api-key-value");
    }

    #[test]
    fn extract_token_from_query_param() {
        use axum::http::HeaderMap;
        let headers = HeaderMap::new();
        let req = axum::http::Request::builder()
            .uri("/api/v1/features?api_key=query-key-123")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "query-key-123");
    }

    #[test]
    fn extract_token_missing_returns_unauthorized() {
        use axum::http::HeaderMap;
        let headers = HeaderMap::new();
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let err = extract_token(&headers, &req).unwrap_err();
        assert!(matches!(err, ApiError::Unauthorized(_)));
    }

    #[test]
    fn extract_token_bearer_takes_precedence_over_x_api_key() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Bearer bearer-val"));
        headers.insert("X-API-Key", HeaderValue::from_static("xkey-val"));
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "bearer-val");
    }

    #[test]
    fn extract_token_empty_bearer_falls_through_to_x_api_key() {
        use axum::http::{HeaderMap, HeaderValue};
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_static("Bearer "));
        headers.insert("X-API-Key", HeaderValue::from_static("fallback-key"));
        let req = axum::http::Request::builder()
            .uri("/api/v1/features")
            .body(axum::body::Body::empty())
            .unwrap();
        let token = extract_token(&headers, &req).unwrap();
        assert_eq!(token, "fallback-key");
    }

    #[tokio::test]
    async fn protected_axum_router_observes_in_place_rotation() {
        let credentials = std::sync::Arc::new(InMemoryCredentialStore::new());
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("old-key"))
            .unwrap();
        let shared: std::sync::Arc<dyn CredentialStore> = credentials.clone();
        let app = Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(middleware::from_fn_with_state(shared, validate_api_key));
        let server = TestServer::new(app);

        server
            .get("/protected")
            .add_header("X-API-Key", "old-key")
            .await
            .assert_status_ok();
        credentials
            .set("agileplus", keys::API_KEYS, &format_api_key_hash("new-key"))
            .unwrap();
        server
            .get("/protected")
            .add_header("X-API-Key", "old-key")
            .await
            .assert_status(axum::http::StatusCode::UNAUTHORIZED);
        server
            .get("/protected")
            .add_header("X-API-Key", "new-key")
            .await
            .assert_status_ok();
    }
}
