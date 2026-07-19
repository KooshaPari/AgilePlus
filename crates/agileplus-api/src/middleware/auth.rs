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
    if let Some(auth) = headers.get("Authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth
            .strip_prefix("Bearer ")
            .or_else(|| auth.strip_prefix("bearer "))
        {
            let token = token.trim();
            if !token.is_empty() {
                return Ok(token.to_string());
            }
        }
    }

    if let Some(header_val) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        return Ok(header_val.to_string());
    }

    if let Some(query) = request.uri().query() {
        if let Some(v) = query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == "api_key").then(|| v.to_string())
        }) {
            return Ok(v);
        }
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

    let valid = creds
        .validate_api_key(&api_key)
        .map_err(|e| ApiError::Internal(format!("credential store error: {e}")))?;

    if !valid {
        // Log only a truncated hint for identification — never the raw key.
        let key_hint: String = api_key.chars().take(4).chain(['*'; 8]).collect();
        warn!(key_hint, "API authentication failed for key hint");
        return Err(ApiError::Unauthorized("Invalid API key".to_string()));
    }

    Ok(next.run(request).await)
}
