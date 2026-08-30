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
        return decode_query_component(&v).ok_or_else(|| {
            ApiError::Unauthorized("Malformed percent encoding in API key".to_string())
        });
    }

    Err(ApiError::Unauthorized(
        "Missing API key (Authorization Bearer, X-API-Key header, or ?api_key= param required)"
            .to_string(),
    ))
}

fn decode_query_component(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' => {
                let high = *bytes.get(index + 1)?;
                let low = *bytes.get(index + 2)?;
                decoded.push((hex_value(high)? << 4) | hex_value(low)?);
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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
    fn query_api_key_is_percent_decoded() {
        let request = Request::builder()
            .uri("/protected?api_key=key%2Bwith+spaces")
            .body(axum::body::Body::empty())
            .unwrap();

        assert_eq!(
            extract_token(&HeaderMap::new(), &request).unwrap(),
            "key+with spaces"
        );
    }

    #[test]
    fn malformed_percent_encoded_query_api_key_is_rejected() {
        let request = Request::builder()
            .uri("/protected?api_key=broken%2")
            .body(axum::body::Body::empty())
            .unwrap();

        assert!(matches!(
            extract_token(&HeaderMap::new(), &request),
            Err(ApiError::Unauthorized(_))
        ));
    }

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
