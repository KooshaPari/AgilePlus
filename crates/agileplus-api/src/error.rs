//! API error type mapped to HTTP status codes.
//!
//! Traceability: WP15-T086

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// Errors returned by API handlers.
///
/// Each variant maps to a specific HTTP status code and produces a JSON body:
/// `{"error": "<message>"}`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Template(String),
    #[error("internal server error")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            ApiError::Template(m) => {
                tracing::error!(error.message = %m, kind = "template_render", "template render failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "template render error".to_string(),
                )
            }
            ApiError::Internal(m) => {
                tracing::error!(error.message = %m, kind = "internal", "internal api error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<agileplus_domain::error::DomainError> for ApiError {
    fn from(e: agileplus_domain::error::DomainError) -> Self {
        match e {
            agileplus_domain::error::DomainError::NotFound(m) => ApiError::NotFound(m),
            agileplus_domain::error::DomainError::Conflict(m) => ApiError::Conflict(m),
            agileplus_domain::error::DomainError::InvalidTransition { from, to, reason } => {
                ApiError::Conflict(format!("invalid transition {from} -> {to}: {reason}"))
            }
            other => ApiError::Internal(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    #[test]
    fn not_found_returns_404() {
        let err = ApiError::NotFound("missing".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn bad_request_returns_400() {
        let err = ApiError::BadRequest("invalid".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn unauthorized_returns_401() {
        let err = ApiError::Unauthorized("no key".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn conflict_returns_409() {
        let err = ApiError::Conflict("dup".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn template_returns_500() {
        let err = ApiError::Template("bad tpl".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn internal_returns_500() {
        let err = ApiError::Internal("something broke".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(ApiError::NotFound("x".into()).to_string(), "x");
        assert_eq!(ApiError::BadRequest("y".into()).to_string(), "y");
        assert_eq!(ApiError::Unauthorized("z".into()).to_string(), "z");
        assert_eq!(ApiError::Conflict("c".into()).to_string(), "c");
    }

    #[test]
    fn internal_error_display_is_generic() {
        let err = ApiError::Internal("db down".into());
        assert_eq!(err.to_string(), "internal server error");
    }

    #[test]
    fn domain_not_found_maps_to_api_not_found() {
        let domain_err = agileplus_domain::error::DomainError::NotFound("feat".into());
        let api_err: ApiError = domain_err.into();
        match api_err {
            ApiError::NotFound(msg) => assert_eq!(msg, "feat"),
            _ => panic!("expected NotFound"),
        }
    }

    #[test]
    fn domain_conflict_maps_to_api_conflict() {
        let domain_err = agileplus_domain::error::DomainError::Conflict("dup".into());
        let api_err: ApiError = domain_err.into();
        match api_err {
            ApiError::Conflict(msg) => assert_eq!(msg, "dup"),
            _ => panic!("expected Conflict"),
        }
    }

    #[test]
    fn domain_invalid_transition_maps_to_api_conflict() {
        let domain_err = agileplus_domain::error::DomainError::InvalidTransition {
            from: "a".into(),
            to: "b".into(),
            reason: "no".into(),
        };
        let api_err: ApiError = domain_err.into();
        match api_err {
            ApiError::Conflict(msg) => {
                assert!(msg.contains("a"));
                assert!(msg.contains("b"));
                assert!(msg.contains("no"));
            }
            _ => panic!("expected Conflict for InvalidTransition"),
        }
    }

    #[test]
    fn domain_not_implemented_maps_to_api_internal() {
        let domain_err = agileplus_domain::error::DomainError::NotImplemented;
        let api_err: ApiError = domain_err.into();
        match api_err {
            ApiError::Internal(_) => {}
            _ => panic!("expected Internal for NotImplemented"),
        }
    }

    // ── Response bodies ──────────────────────────────────────────────────────
    //
    // 500-class variants deliberately replace the internal message with a fixed
    // string so a template or storage error can never reach a client. The status
    // codes are asserted above; these pin the bodies.

    async fn body_json(resp: Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .expect("error responses carry a body");
        serde_json::from_slice(&bytes).expect("error bodies are JSON")
    }

    #[tokio::test]
    async fn template_error_body_is_generic_and_hides_details() {
        let resp =
            ApiError::Template("cycle_detail.html: field `nope` missing".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["error"], "template render error");
        assert!(
            !json.to_string().contains("cycle_detail.html"),
            "the template name and field must not leak, got: {json}"
        );
    }

    #[tokio::test]
    async fn internal_error_body_is_generic_and_hides_details() {
        let resp =
            ApiError::Internal("sqlite: disk I/O error at /var/db/x.sqlite".into()).into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let json = body_json(resp).await;
        assert_eq!(json["error"], "internal server error");
        assert!(
            !json.to_string().contains("sqlite"),
            "storage details must not leak, got: {json}"
        );
    }

    #[tokio::test]
    async fn client_error_body_uses_the_error_envelope() {
        let resp = ApiError::Unauthorized("Invalid API key".into()).into_response();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        let json = body_json(resp).await;
        assert_eq!(json["error"], "Invalid API key");
        assert_eq!(
            json.as_object().expect("object").len(),
            1,
            "the envelope is exactly {{\"error\": ...}}, got: {json}"
        );
    }
}
