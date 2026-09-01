use agileplus_grpc::runtime::CoreConfig;

#[test]
fn defaults_to_loopback_and_a_stable_database_path() {
    let config = CoreConfig::from_values(None, None).expect("default config");

    assert!(config.bind.ip().is_loopback());
    assert_eq!(config.bind.port(), 50051);
    assert_eq!(config.database.to_string_lossy(), ".agileplus/agileplus.db");
}

#[test]
fn rejects_non_loopback_bindings() {
    let error = CoreConfig::from_values(Some("0.0.0.0:50051"), None)
        .expect_err("public plaintext bind must be rejected");

    assert!(error.to_string().contains("loopback"));
}

#[test]
fn rejects_empty_or_whitespace_database_paths() {
    for database in ["", "  \t "] {
        let error = CoreConfig::from_values(None, Some(database))
            .expect_err("empty database path must be rejected");
        assert!(error.contains("must not be empty"));
    }
}

#[test]
fn canonical_database_environment_value_takes_precedence() {
    let config = CoreConfig::from_env_values(
        None,
        None,
        Some("/canonical/agileplus.db"),
        Some("/legacy/agileplus.db"),
    )
    .expect("canonical database config");

    assert_eq!(config.database.to_string_lossy(), "/canonical/agileplus.db");
}

#[test]
fn legacy_database_environment_value_remains_a_fallback() {
    let config = CoreConfig::from_env_values(None, None, None, Some("/legacy/agileplus.db"))
        .expect("legacy database config");

    assert_eq!(config.database.to_string_lossy(), "/legacy/agileplus.db");
}

#[test]
fn resolved_port_binds_to_loopback_unless_an_explicit_bind_is_set() {
    let resolved =
        CoreConfig::from_env_values(None, Some("51234"), None, None).expect("resolved port config");
    assert_eq!(resolved.bind, "127.0.0.1:51234".parse().unwrap());

    let explicit = CoreConfig::from_env_values(Some("[::1]:52345"), Some("51234"), None, None)
        .expect("explicit bind config");
    assert_eq!(explicit.bind, "[::1]:52345".parse().unwrap());
}
