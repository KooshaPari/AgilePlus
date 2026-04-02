//! API key management route handlers.
//!
//! - POST /api/v1/keys — generate a new API key
//!
//! Traceability: WP11-T064

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::credentials::keys;

use crate::api_key::{generate_plaintext_key, hash_key};
use crate::error::ApiError;
use crate::state::AppState;

pub fn routes<S, V, O>() -> Router<AppState<S, V, O>>
where
    S: agileplus_domain::ports::StoragePort + Send + Sync + 'static,
    V: agileplus_domain::ports::VcsPort + Send + Sync + 'static,
    O: agileplus_domain::ports::ObservabilityPort + Send + Sync + 'static,
{
    Router::new().route("/", post(create_api_key::<S, V, O>))
}

#[derive(Debug, Serialize)]
pub struct CreateKeyResponse {
    pub key: String,
    pub key_hint: String,
    pub hashed: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: Option<String>,
}

/// `POST /api/v1/keys`
pub async fn create_api_key<S, V, O>(
    State(app): State<AppState<S, V, O>>,
    Json(body): Json<CreateKeyRequest>,
) -> Result<(StatusCode, Json<CreateKeyResponse>), ApiError>
where
    S: agileplus_domain::ports::StoragePort + Send + Sync + 'static,
    V: agileplus_domain::ports::VcsPort + Send + Sync + 'static,
    O: agileplus_domain::ports::ObservabilityPort + Send + Sync + 'static,
{
    let _name = body.name.unwrap_or_else(|| "default".to_string());

    let plaintext = generate_plaintext_key();
    let hashed = hash_key(&plaintext);
    let hashed_hex: String = hashed.iter().map(|b| format!("{b:02x}")).collect();

    app.credentials
        .set("agileplus", keys::API_KEYS, &plaintext)
        .map_err(|e| ApiError::Internal(format!("failed to store key: {e}")))?;

    let key_hint = if plaintext.len() > 8 {
        format!("{}...", &plaintext[..8])
    } else {
        "[key too short]".to_string()
    };

    Ok((
        StatusCode::CREATED,
        Json(CreateKeyResponse {
            key: plaintext,
            key_hint,
            hashed: hashed_hex,
        }),
    ))
}
