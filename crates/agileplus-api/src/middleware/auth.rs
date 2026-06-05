//! Authentication middleware for protected API routes.
//!
//! The default verifier is a shared-secret/API-key verifier that uses
//! constant-time comparison. JWT and Authvault integration remain follow-up
//! work once the token-verifier port is exercised in production.

use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use tracing::warn;

use crate::error::ApiError;
use crate::middleware::token_verifier::DynTokenVerifier;

/// axum middleware that authorizes protected routes.
///
/// Accepts either `Authorization: Bearer <token>` or `X-API-Key: <token>`.
/// Returns `401 Unauthorized` when the credential is missing or invalid.
pub async fn authorize(
    State(verifier): State<DynTokenVerifier>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_token(request.headers())
        .ok_or_else(|| ApiError::Unauthorized("Missing bearer token or API key".to_string()))?;

    match verifier.verify(&token) {
        Ok(true) => Ok(next.run(request).await),
        Ok(false) => {
            warn!(token_hint = %token_hint(&token), "API authorization failed");
            Err(ApiError::Unauthorized(
                "Invalid bearer token or API key".to_string(),
            ))
        }
        Err(err) => Err(ApiError::Internal(format!(
            "token verification failed: {err}"
        ))),
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
    format!("{prefix}...")
}

#[cfg(test)]
mod tests {
    use crate::middleware::token_verifier::SharedSecretVerifier;
    use crate::middleware::token_verifier::TokenVerifier;
    use std::sync::Arc;

    #[test]
    fn shared_secret_verifier_accepts_matching_token() {
        let verifier = Arc::new(SharedSecretVerifier::new(vec!["alpha".to_string()]));
        assert!(verifier.verify("alpha").unwrap());
    }

    #[test]
    fn shared_secret_verifier_rejects_non_matching_token() {
        let verifier = Arc::new(SharedSecretVerifier::new(vec!["alpha".to_string()]));
        assert!(!verifier.verify("omega").unwrap());
    }

    #[test]
    fn shared_secret_verifier_rejects_length_mismatch() {
        let verifier = Arc::new(SharedSecretVerifier::new(vec!["alpha".to_string()]));
        assert!(!verifier.verify("alph").unwrap());
    }

    #[test]
    fn shared_secret_verifier_parses_csv_tokens() {
        let verifier = Arc::new(SharedSecretVerifier::new(vec![
            "alpha".to_string(),
            "beta".to_string(),
        ]));
        assert!(verifier.verify("beta").unwrap());
    }
}
