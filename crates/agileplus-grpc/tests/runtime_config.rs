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
