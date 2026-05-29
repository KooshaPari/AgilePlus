//! Authentication middleware for protected API routes.
//!
//! The default verifier is a shared-secret/API-key verifier that uses
//! constant-time comparison. JWT and Authvault integration remain follow-up
//! work once the token-verifier port is exercised in production.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use subtle::ConstantTimeEq;
use tracing::warn;

use crate::error::ApiError;

/// Port for validating bearer tokens or API keys on protected routes.
pub trait TokenVerifier: Send + Sync {
    fn verify(&self, token: &str) -> bool;
}

/// Default token verifier backed by one or more shared secrets.
#[derive(Clone, Debug)]
pub struct SharedSecretTokenVerifier {
    allowed_tokens: Arc<[String]>,
}

impl SharedSecretTokenVerifier {
    pub fn new(tokens: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            allowed_tokens: tokens.into_iter().map(Into::into).collect(),
        }
    }

    pub fn from_csv(raw: Option<String>) -> Self {
        let raw_str = raw.unwrap_or_default();
        let tokens: Vec<&str> = raw_str
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .collect();
        Self::new(tokens)
    }
}

impl TokenVerifier for SharedSecretTokenVerifier {
    fn verify(&self, token: &str) -> bool {
        let candidate = token.as_bytes();
        self.allowed_tokens
            .iter()
            .any(|expected| expected.as_bytes().ct_eq(candidate).into())
    }
}

/// axum middleware that authorizes protected routes.
///
/// Accepts either `Authorization: Bearer <token>` or `X-API-Key: <token>`.
/// Returns `401 Unauthorized` when the credential is missing or invalid.
pub async fn authorize(
    State(verifier): State<Arc<dyn TokenVerifier>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_token(request.headers()).ok_or_else(|| {
        ApiError::Unauthorized("Missing bearer token or API key".to_string())
    })?;

    if verifier.verify(&token) {
        Ok(next.run(request).await)
    } else {
        warn!(token_hint = %token_hint(&token), "API authorization failed");
        Err(ApiError::Unauthorized("Invalid bearer token or API key".to_string()))
    }
}

fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    {
        return Some(value.trim().to_string());
    }

    headers
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
}

fn token_hint(token: &str) -> String {
    let prefix = token.chars().take(4).collect::<String>();
    format!("{prefix}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_secret_verifier_accepts_matching_token() {
        let verifier = SharedSecretTokenVerifier::new(["alpha"]);
        assert!(verifier.verify("alpha"));
    }

    #[test]
    fn shared_secret_verifier_rejects_non_matching_token() {
        let verifier = SharedSecretTokenVerifier::new(["alpha"]);
        assert!(!verifier.verify("omega"));
    }

    #[test]
    fn shared_secret_verifier_rejects_length_mismatch() {
        let verifier = SharedSecretTokenVerifier::new(["alpha"]);
        assert!(!verifier.verify("alph"));
    }

    #[test]
    fn shared_secret_verifier_parses_csv_tokens() {
        let verifier = SharedSecretTokenVerifier::from_csv(Some("alpha, beta".to_string()));
        assert!(verifier.verify("beta"));
    }
}
