use agileplus_grpc::runtime::CoreConfig;

#[test]
fn rejects_startup_without_an_explicit_project_root() {
    let error = CoreConfig::from_values(None, None, None).expect_err("project root is required");

    assert!(error.contains("project root is required"));
}

#[test]
fn rejects_non_loopback_bindings() {
    let error = CoreConfig::from_values(Some("0.0.0.0:50051"), Some("."), None)
        .expect_err("public plaintext bind must be rejected");

    assert!(error.to_string().contains("loopback"));
}

#[test]
fn rejects_empty_or_whitespace_project_roots() {
    for project_root in ["", "  \t "] {
        let error = CoreConfig::from_values(None, Some(project_root), None)
            .expect_err("project root is required");
        assert!(error.contains("project root is required"));
    }
}

#[test]
fn rejects_canonical_database_environment_value() {
    let config = CoreConfig::from_env_values(
        None,
        Some("."),
        Some("/canonical/agileplus.db"),
        Some("/legacy/agileplus.db"),
    )
    .expect_err("database configuration must not select core state");

    assert!(config.contains("database path is not supported"));
}

#[test]
fn rejects_legacy_database_environment_value() {
    let config = CoreConfig::from_env_values(None, Some("."), None, Some("/legacy/agileplus.db"))
        .expect_err("legacy database configuration must not select core state");

    assert!(config.contains("database path is not supported"));
}
