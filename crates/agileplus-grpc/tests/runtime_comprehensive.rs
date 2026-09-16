//! Comprehensive tests for CoreConfig validation and construction.
//!
//! Complements the inline unit tests in `runtime.rs` and the existing
//! `runtime_config.rs` tests.
//!
//! Traceability: WP14-T079

use std::path::PathBuf;

use agileplus_grpc::runtime::CoreConfig;

// ---------------------------------------------------------------------------
// CoreConfig::from_values — valid inputs
// ---------------------------------------------------------------------------

#[test]
fn from_values_localhost_default_port() {
    let config = CoreConfig::from_values(None, Some("/tmp/proj"), None).unwrap();
    assert_eq!(config.bind.port(), 50051);
    assert_eq!(config.bind.ip().to_string(), "127.0.0.1");
}

#[test]
fn from_values_custom_port() {
    let config = CoreConfig::from_values(Some("127.0.0.1:8080"), Some("/tmp/proj"), None).unwrap();
    assert_eq!(config.bind.port(), 8080);
}

#[test]
fn from_values_ipv6_loopback() {
    let config = CoreConfig::from_values(Some("[::1]:50051"), Some("/tmp/proj"), None).unwrap();
    assert!(config.bind.ip().is_loopback());
    assert_eq!(config.bind.port(), 50051);
}

#[test]
fn from_values_project_root_preserved() {
    let config = CoreConfig::from_values(None, Some("/home/user/project"), None).unwrap();
    assert_eq!(config.project_root, PathBuf::from("/home/user/project"));
}

#[test]
fn from_values_project_root_trimmed() {
    let config = CoreConfig::from_values(None, Some("  /tmp/proj  "), None).unwrap();
    assert_eq!(config.project_root, PathBuf::from("/tmp/proj"));
}

#[test]
fn from_values_dot_project_root() {
    let config = CoreConfig::from_values(None, Some("."), None).unwrap();
    assert_eq!(config.project_root, PathBuf::from("."));
}

// ---------------------------------------------------------------------------
// CoreConfig::from_values — error cases
// ---------------------------------------------------------------------------

#[test]
fn from_values_missing_project_root() {
    let err = CoreConfig::from_values(Some("127.0.0.1:50051"), None, None).unwrap_err();
    assert!(err.contains("project root is required"));
}

#[test]
fn from_values_empty_project_root() {
    let err = CoreConfig::from_values(Some("127.0.0.1:50051"), Some(""), None).unwrap_err();
    assert!(err.contains("project root is required"));
}

#[test]
fn from_values_whitespace_only_project_root() {
    let err = CoreConfig::from_values(Some("127.0.0.1:50051"), Some("   "), None).unwrap_err();
    assert!(err.contains("project root is required"));
}

#[test]
fn from_values_invalid_bind_address() {
    let err = CoreConfig::from_values(Some("not-a-url"), Some("/tmp"), None).unwrap_err();
    assert!(err.contains("invalid core bind address"));
}

#[test]
fn from_values_non_loopback_0_0_0_0() {
    let err = CoreConfig::from_values(Some("0.0.0.0:50051"), Some("/tmp"), None).unwrap_err();
    assert!(err.contains("loopback"));
}

#[test]
fn from_values_non_loopback_public_ip() {
    let err = CoreConfig::from_values(Some("192.168.1.1:50051"), Some("/tmp"), None).unwrap_err();
    assert!(err.contains("loopback"));
}

#[test]
fn from_values_rejects_database_path() {
    let err = CoreConfig::from_values(Some("127.0.0.1:50051"), Some("/tmp"), Some("db.sqlite"))
        .unwrap_err();
    assert!(err.contains("database path is not supported"));
}

// ---------------------------------------------------------------------------
// CoreConfig::from_env_values — database precedence
// ---------------------------------------------------------------------------

#[test]
fn from_env_values_both_databases_rejects() {
    let err = CoreConfig::from_env_values(
        Some("127.0.0.1:50051"),
        Some("/tmp"),
        Some("core.db"),
        Some("legacy.db"),
    )
    .unwrap_err();
    assert!(err.contains("database path is not supported"));
}

#[test]
fn from_env_values_core_database_only_rejects() {
    let err = CoreConfig::from_env_values(
        Some("127.0.0.1:50051"),
        Some("/tmp"),
        Some("core.db"),
        None,
    )
    .unwrap_err();
    assert!(err.contains("database path is not supported"));
}

#[test]
fn from_env_values_legacy_database_only_rejects() {
    let err = CoreConfig::from_env_values(
        Some("127.0.0.1:50051"),
        Some("/tmp"),
        None,
        Some("legacy.db"),
    )
    .unwrap_err();
    assert!(err.contains("database path is not supported"));
}

#[test]
fn from_env_values_no_database_succeeds() {
    let config =
        CoreConfig::from_env_values(Some("127.0.0.1:50051"), Some("/tmp/proj"), None, None)
            .unwrap();
    assert_eq!(config.project_root, PathBuf::from("/tmp/proj"));
}

// ---------------------------------------------------------------------------
// CoreConfig — equality and cloning
// ---------------------------------------------------------------------------

#[test]
fn config_equality() {
    let a = CoreConfig::from_values(Some("127.0.0.1:9999"), Some("/proj"), None).unwrap();
    let b = CoreConfig::from_values(Some("127.0.0.1:9999"), Some("/proj"), None).unwrap();
    assert_eq!(a, b);
}

#[test]
fn config_inequality_different_port() {
    let a = CoreConfig::from_values(Some("127.0.0.1:8080"), Some("/proj"), None).unwrap();
    let b = CoreConfig::from_values(Some("127.0.0.1:9090"), Some("/proj"), None).unwrap();
    assert_ne!(a, b);
}

#[test]
fn config_inequality_different_root() {
    let a = CoreConfig::from_values(None, Some("/proj-a"), None).unwrap();
    let b = CoreConfig::from_values(None, Some("/proj-b"), None).unwrap();
    assert_ne!(a, b);
}

#[test]
fn config_clone_independent() {
    let a = CoreConfig::from_values(None, Some("/proj"), None).unwrap();
    let mut b = a.clone();
    b.project_root = PathBuf::from("/changed");
    assert_eq!(a.project_root, PathBuf::from("/proj"));
    assert_eq!(b.project_root, PathBuf::from("/changed"));
}

#[test]
fn config_debug_output() {
    let config = CoreConfig::from_values(None, Some("/proj"), None).unwrap();
    let debug = format!("{:?}", config);
    assert!(debug.contains("CoreConfig"));
    assert!(debug.contains("/proj"));
}
