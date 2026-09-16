//! Integration tests for error types, conversions, and display formatting.
//! Complements the inline unit tests in src/error.rs.

use agileplus_governance::*;

#[test]
fn error_display_all_variants() {
    let cases: Vec<(GovernanceError, &str)> = vec![
        (GovernanceError::Config("url required".into()), "Configuration error: url required"),
        (GovernanceError::Database("table missing".into()), "Database error: table missing"),
        (GovernanceError::Network("connection refused".into()), "Network error: connection refused"),
        (GovernanceError::PolicyViolation("not allowed".into()), "Policy violation: not allowed"),
        (GovernanceError::RateLimitExceeded("too many".into()), "Rate limit exceeded: too many"),
        (GovernanceError::Auth("expired token".into()), "Authentication error: expired token"),
        (GovernanceError::NotFound("pkg v1".into()), "Resource not found: pkg v1"),
        (GovernanceError::NotAllowed("write denied".into()), "Operation not allowed: write denied"),
        (GovernanceError::Sync("drift detected".into()), "Sync error: drift detected"),
        (GovernanceError::Internal("panic".into()), "Internal error: panic"),
        (GovernanceError::Rubric("invalid json".into()), "Rubric error: invalid json"),
    ];
    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected, "mismatch for {}", expected);
    }
}

#[test]
fn error_invalid_channel_transition_display() {
    let err = GovernanceError::InvalidChannelTransition {
        from: "canary".into(),
        to: "alpha".into(),
    };
    assert_eq!(
        err.to_string(),
        "Invalid channel transition from canary to alpha"
    );
}

#[test]
fn error_is_policy_error() {
    assert!(GovernanceError::PolicyViolation("x".into()).is_policy_error());
    assert!(GovernanceError::NotAllowed("x".into()).is_policy_error());
    assert!(!GovernanceError::Config("x".into()).is_policy_error());
    assert!(!GovernanceError::Database("x".into()).is_policy_error());
    assert!(!GovernanceError::Network("x".into()).is_policy_error());
    assert!(!GovernanceError::RateLimitExceeded("x".into()).is_policy_error());
    assert!(!GovernanceError::Auth("x".into()).is_policy_error());
    assert!(!GovernanceError::NotFound("x".into()).is_policy_error());
    assert!(!GovernanceError::Sync("x".into()).is_policy_error());
    assert!(!GovernanceError::Internal("x".into()).is_policy_error());
    assert!(!GovernanceError::Rubric("x".into()).is_policy_error());
}

#[test]
fn error_is_rate_limit_error() {
    assert!(GovernanceError::RateLimitExceeded("x".into()).is_rate_limit_error());
    assert!(!GovernanceError::PolicyViolation("x".into()).is_rate_limit_error());
    assert!(!GovernanceError::Config("x".into()).is_rate_limit_error());
    assert!(!GovernanceError::Internal("x".into()).is_rate_limit_error());
}

#[test]
fn error_status_code_all_variants() {
    assert_eq!(GovernanceError::Config("x".into()).status_code(), 400);
    assert_eq!(GovernanceError::Database("x".into()).status_code(), 500);
    assert_eq!(GovernanceError::Network("x".into()).status_code(), 503);
    assert_eq!(GovernanceError::PolicyViolation("x".into()).status_code(), 403);
    assert_eq!(GovernanceError::RateLimitExceeded("x".into()).status_code(), 429);
    assert_eq!(GovernanceError::Auth("x".into()).status_code(), 401);
    assert_eq!(
        GovernanceError::InvalidChannelTransition { from: "a".into(), to: "b".into() }
            .status_code(),
        400
    );
    assert_eq!(GovernanceError::NotFound("x".into()).status_code(), 404);
    assert_eq!(GovernanceError::NotAllowed("x".into()).status_code(), 403);
    assert_eq!(GovernanceError::Sync("x".into()).status_code(), 500);
    assert_eq!(GovernanceError::Internal("x".into()).status_code(), 500);
    assert_eq!(GovernanceError::Rubric("x".into()).status_code(), 422);
}

#[test]
fn error_from_io_error() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let gov_err: GovernanceError = io_err.into();
    assert!(matches!(gov_err, GovernanceError::Internal(_)));
    assert!(gov_err.to_string().contains("access denied"));
}

#[test]
fn error_from_serde_json_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("{bad json}").unwrap_err();
    let gov_err: GovernanceError = json_err.into();
    assert!(matches!(gov_err, GovernanceError::Config(_)));
}

#[test]
fn error_from_toml_error() {
    let toml_err = toml::from_str::<toml::Value>("key = [unclosed").unwrap_err();
    let gov_err: GovernanceError = toml_err.into();
    assert!(matches!(gov_err, GovernanceError::Config(_)));
}

#[test]
fn error_from_rusqlite_error() {
    let sql_err = rusqlite::Error::InvalidParameterName("bad_param".into());
    let gov_err: GovernanceError = sql_err.into();
    assert!(matches!(gov_err, GovernanceError::Database(_)));
    assert!(gov_err.to_string().contains("bad_param"));
}

#[test]
fn error_is_debug() {
    let err = GovernanceError::Internal("test".into());
    let debug = format!("{:?}", err);
    assert!(debug.contains("Internal"));
}

#[test]
fn error_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GovernanceError>();
}

#[test]
fn error_variant_equality_via_display() {
    let e1 = GovernanceError::Config("test".into());
    let e2 = GovernanceError::Config("test".into());
    assert_eq!(e1.to_string(), e2.to_string());
}

#[test]
fn error_not_found_is_not_policy_error() {
    assert!(!GovernanceError::NotFound("item".into()).is_policy_error());
}

#[test]
fn error_auth_is_not_rate_limit_error() {
    assert!(!GovernanceError::Auth("token".into()).is_rate_limit_error());
}
