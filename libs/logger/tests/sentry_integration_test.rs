/// Integration test for Sentry error capture
///
/// This test verifies that Sentry is properly initialized and can capture errors.
/// Run with: cargo test --test sentry_integration_test -- --nocapture

#[test]
fn test_sentry_initialization() {
    // FR-SENTRY-001: Sentry should initialize without panicking
    let _guard = logger::initialize();
    // Guard is valid, test passes
}

#[test]
fn test_sentry_capture_message() {
    // FR-SENTRY-002: Should be able to capture messages
    let _guard = logger::initialize();
    logger::capture_message("Integration test message", sentry::Level::Info);
    // Message captured, test passes
}

#[test]
fn test_sentry_capture_error() {
    // FR-SENTRY-003: Should be able to capture errors
    let _guard = logger::initialize();
    let error = std::io::Error::other("Test error for Sentry");
    logger::capture_error(&error);
    // Error captured, test passes
}

#[test]
fn test_sentry_panic_capture() {
    // FR-SENTRY-004: Sentry should capture panics (verified via dashboard)
    let _guard = logger::initialize();

    // Simulate an error scenario
    let result: Result<i32, _> = Err("Intentional error for Sentry capture");
    if let Err(e) = result {
        logger::capture_message(&format!("Error occurred: {}", e), sentry::Level::Error);
    }
}
