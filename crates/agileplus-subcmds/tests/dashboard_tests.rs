//! Integration tests for the dashboard subcommand.
//!
//! Covers: URL construction, port configuration, health checks, error handling.

#![cfg(feature = "dashboard")]

use agileplus_subcmds::{
    DashboardArgs, DashboardSubcommand, api_reachable, configured_port, dashboard_url,
    run_dashboard_open,
};

// ---------------------------------------------------------------------------
// URL construction
// ---------------------------------------------------------------------------

#[test]
fn dashboard_url_standard_port() {
    assert_eq!(dashboard_url(8080), "http://localhost:8080/dashboard");
}

#[test]
fn dashboard_url_custom_port() {
    assert_eq!(dashboard_url(3000), "http://localhost:3000/dashboard");
}

#[test]
fn dashboard_url_high_port() {
    assert_eq!(dashboard_url(65535), "http://localhost:65535/dashboard");
}

#[test]
fn dashboard_url_port_one() {
    assert_eq!(dashboard_url(1), "http://localhost:1/dashboard");
}

// ---------------------------------------------------------------------------
// Port configuration
// ---------------------------------------------------------------------------

#[test]
fn configured_port_env_var_overrides_default() {
    // SAFETY: consolidated into single test to avoid parallel env-var races.
    // No other thread reads this var during this test.
    unsafe {
        std::env::remove_var("AGILEPLUS_DASHBOARD_PORT");
    }
    assert_eq!(configured_port(), 8080, "default should be 8080");

    unsafe {
        std::env::set_var("AGILEPLUS_DASHBOARD_PORT", "9999");
    }
    assert_eq!(configured_port(), 9999, "env override should be 9999");

    unsafe {
        std::env::set_var("AGILEPLUS_DASHBOARD_PORT", "not-a-number");
    }
    assert_eq!(
        configured_port(),
        8080,
        "invalid env should fall back to 8080"
    );

    unsafe {
        std::env::set_var("AGILEPLUS_DASHBOARD_PORT", "");
    }
    assert_eq!(
        configured_port(),
        8080,
        "empty env should fall back to 8080"
    );

    unsafe {
        std::env::remove_var("AGILEPLUS_DASHBOARD_PORT");
    }
}

// ---------------------------------------------------------------------------
// Health check (TCP probe)
// ---------------------------------------------------------------------------

#[test]
fn api_reachable_returns_false_for_unused_port() {
    // Port 19999 should be unused in CI and local environments.
    assert!(!api_reachable(19999));
}

#[test]
fn api_reachable_returns_false_for_loopback_unlikely_port() {
    assert!(!api_reachable(59123));
}

// ---------------------------------------------------------------------------
// Error handling: open fails when API is down
// ---------------------------------------------------------------------------

#[test]
fn run_dashboard_open_errors_when_api_not_running() {
    // Port 19998 should have no API running.
    let result = run_dashboard_open(19998);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not running"),
        "Expected 'not running' in error, got: {err_msg}"
    );
}

// ---------------------------------------------------------------------------
// DashboardArgs construction
// ---------------------------------------------------------------------------

#[test]
fn dashboard_args_no_subcommand() {
    let args = DashboardArgs {
        subcommand: None,
        port: None,
    };
    assert!(args.subcommand.is_none());
    assert!(args.port.is_none());
}

#[test]
fn dashboard_args_with_port() {
    let args = DashboardArgs {
        subcommand: None,
        port: Some(4000),
    };
    assert_eq!(args.port, Some(4000));
}

#[test]
fn dashboard_args_open_subcommand() {
    let args = DashboardArgs {
        subcommand: Some(DashboardSubcommand::Open(
            agileplus_subcmds::DashboardOpenArgs { port: Some(5000) },
        )),
        port: None,
    };
    match args.subcommand.unwrap() {
        DashboardSubcommand::Open(a) => assert_eq!(a.port, Some(5000)),
        _ => panic!("Expected Open subcommand"),
    }
}

#[test]
fn dashboard_args_port_subcommand() {
    let args = DashboardArgs {
        subcommand: Some(DashboardSubcommand::Port(
            agileplus_subcmds::DashboardPortArgs { port: 7777 },
        )),
        port: None,
    };
    match args.subcommand.unwrap() {
        DashboardSubcommand::Port(a) => assert_eq!(a.port, 7777),
        _ => panic!("Expected Port subcommand"),
    }
}
