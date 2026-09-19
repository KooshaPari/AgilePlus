//! Integration tests for the dashboard subcommand.
//!
//! Covers: URL construction, port configuration, health checks, error handling,
//! CLI argument parsing, subcommand dispatch, config persistence failures, and
//! browser-launch error propagation.

#![cfg(feature = "dashboard")]

use agileplus_subcmds::dashboard::open_browser;
use agileplus_subcmds::{
    DashboardArgs, DashboardOpenArgs, DashboardPortArgs, DashboardSubcommand, api_reachable,
    configured_port, dashboard_url, run_dashboard, run_dashboard_open, run_dashboard_port,
};
use clap::Parser;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

// ---------------------------------------------------------------------------
// Process-global state serialization
// ---------------------------------------------------------------------------

const ENV_PORT: &str = "AGILEPLUS_DASHBOARD_PORT";
const ENV_PATH: &str = "PATH";

/// Serializes every test that touches process-global state (environment,
/// working directory) or that spawns a process.
///
/// Lock poisoning is recovered instead of unwrapped so that one failing test
/// cannot cascade into unrelated failures.
fn process_state_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Restores an environment variable on drop, even if the test panics.
struct EnvGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvGuard {
    /// Capture the current value of `key`, restoring it when the guard drops.
    /// Caller must hold [`process_state_lock`] for the whole guard lifetime.
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: std::env::var_os(key),
        }
    }

    /// Capture and set `key`.
    fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
        let guard = Self::capture(key);
        // SAFETY: the caller holds `process_state_lock`, so no other thread in
        // this test binary reads or writes the process environment.
        unsafe { std::env::set_var(key, value) };
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match self.original.take() {
            // SAFETY: as above; the lock is still held by the caller's guard.
            Some(value) => unsafe { std::env::set_var(self.key, value) },
            None => unsafe { std::env::remove_var(self.key) },
        }
    }
}

/// Restores the working directory on drop, even if the test panics.
struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    /// Change into `dir`. Caller must hold [`process_state_lock`].
    fn enter(dir: &Path) -> Self {
        let original = std::env::current_dir().expect("read current dir");
        std::env::set_current_dir(dir).expect("enter test dir");
        Self { original }
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A loopback port that was bound and then released, so nothing is listening.
fn unused_loopback_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("read local addr").port();
    drop(listener);
    port
}

/// Two different free ports, so an assertion on one cannot accidentally match
/// the other.
fn two_distinct_unused_ports() -> (u16, u16) {
    for _ in 0..16 {
        let a = unused_loopback_port();
        let b = unused_loopback_port();
        if a != b {
            return (a, b);
        }
    }
    panic!("kernel kept handing back the same ephemeral port");
}

/// A listener on `127.0.0.1` standing in for a running API server.
fn stub_listener() -> (std::net::TcpListener, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind stub listener");
    let port = listener.local_addr().expect("read local addr").port();
    (listener, port)
}

/// Executable stand-in for the platform browser launcher. Records the first
/// argument it is handed, then exits 0. It never opens a browser.
#[cfg(unix)]
fn install_launcher_stub(dir: &Path, record_path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/bin/sh\nprintf '%s' \"$1\" > {}\n",
        record_path.display()
    );
    let names: &[&str] = if cfg!(target_os = "macos") {
        &["open"]
    } else {
        &["xdg-open"]
    };
    for name in names {
        let path = dir.join(name);
        std::fs::write(&path, &script).expect("write launcher stub");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("mark launcher stub executable");
    }
}

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
    let _guard = process_state_lock();
    let _restore = EnvGuard::capture(ENV_PORT);

    // SAFETY: every mutation below happens under `process_state_lock`.
    unsafe { std::env::remove_var(ENV_PORT) };
    assert_eq!(configured_port(), 8080, "default should be 8080");

    unsafe { std::env::set_var(ENV_PORT, "9999") };
    assert_eq!(configured_port(), 9999, "env override should be 9999");

    unsafe { std::env::set_var(ENV_PORT, "not-a-number") };
    assert_eq!(
        configured_port(),
        8080,
        "invalid env should fall back to 8080"
    );

    unsafe { std::env::set_var(ENV_PORT, "-1") };
    assert_eq!(
        configured_port(),
        8080,
        "negative env should fall back to 8080"
    );

    unsafe { std::env::set_var(ENV_PORT, "70000") };
    assert_eq!(
        configured_port(),
        8080,
        "out-of-range env should fall back to 8080"
    );

    unsafe { std::env::set_var(ENV_PORT, "") };
    assert_eq!(
        configured_port(),
        8080,
        "empty env should fall back to 8080"
    );

    unsafe { std::env::set_var(ENV_PORT, "65535") };
    assert_eq!(configured_port(), 65535, "max u16 must be accepted");
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

#[test]
fn api_reachable_is_true_for_a_bound_loopback_listener() {
    let (_listener, port) = stub_listener();
    assert!(
        api_reachable(port),
        "a socket listening on loopback port {port} must probe as reachable"
    );
}

#[test]
fn api_reachable_is_false_after_the_listener_is_closed() {
    let (listener, port) = stub_listener();
    assert!(api_reachable(port));
    drop(listener);
    assert!(
        !api_reachable(port),
        "probing port {port} must fail once the listener is gone"
    );
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

#[test]
fn run_dashboard_open_error_names_the_probed_port() {
    let _guard = process_state_lock();
    let port = unused_loopback_port();
    let err = run_dashboard_open(port).unwrap_err().to_string();
    assert!(
        err.contains(&port.to_string()),
        "error must name the probed port {port}, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Browser launch: success path via a launcher stub (no real browser)
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn run_dashboard_open_hands_the_dashboard_url_to_the_launcher() {
    let _guard = process_state_lock();
    let (_listener, port) = stub_listener();

    let stub_dir = tempfile::tempdir().unwrap();
    let record = stub_dir.path().join("launcher-args.txt");
    install_launcher_stub(stub_dir.path(), &record);
    let _path = EnvGuard::set(ENV_PATH, stub_dir.path());

    run_dashboard_open(port).expect("reachable API plus a working launcher must succeed");

    let recorded = std::fs::read_to_string(&record).expect("launcher stub must record its argv");
    assert_eq!(
        recorded,
        dashboard_url(port),
        "launcher must receive the dashboard URL for port {port}"
    );
}

#[cfg(unix)]
#[test]
fn run_dashboard_open_succeeds_with_manual_url_when_launcher_exits_nonzero() {
    use std::os::unix::fs::PermissionsExt;

    let _guard = process_state_lock();
    let (_listener, port) = stub_listener();

    let stub_dir = tempfile::tempdir().unwrap();
    let script = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let stub = stub_dir.path().join(script);
    std::fs::write(&stub, "#!/bin/sh\nexit 3\n").unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let _path = EnvGuard::set(ENV_PATH, stub_dir.path());

    assert!(
        run_dashboard_open(port).is_ok(),
        "a failing browser launcher must not be fatal once the API is reachable"
    );
}

#[test]
fn open_browser_reports_error_when_launcher_is_missing() {
    let _guard = process_state_lock();
    let empty_dir = tempfile::tempdir().unwrap();
    let _path = EnvGuard::set(ENV_PATH, empty_dir.path());

    let err = open_browser("http://localhost:1/dashboard")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Failed to launch browser"),
        "missing launcher binary must be reported, got: {err}"
    );
}

#[cfg(target_os = "macos")]
#[test]
fn open_browser_reports_nonzero_launcher_exit() {
    let _guard = process_state_lock();
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("no-such-dashboard-target");

    let err = open_browser(missing.to_str().unwrap())
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Browser launcher exited with"),
        "nonzero launcher exit must be reported, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// run_dashboard_open is a no-op for an unreachable API
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn run_dashboard_open_does_not_spawn_a_launcher_when_api_is_down() {
    let _guard = process_state_lock();
    let port = unused_loopback_port();

    let stub_dir = tempfile::tempdir().unwrap();
    let marker = stub_dir.path().join("launcher-ran.txt");
    install_launcher_stub(stub_dir.path(), &marker);
    let _path = EnvGuard::set(ENV_PATH, stub_dir.path());

    assert!(run_dashboard_open(port).is_err());
    assert!(
        !marker.exists(),
        "the browser launcher must not run when the API probe fails"
    );
}

// ---------------------------------------------------------------------------
// run_dashboard dispatch and port precedence
// ---------------------------------------------------------------------------

#[test]
fn run_dashboard_uses_env_port_when_no_port_is_given() {
    let _guard = process_state_lock();
    let port = unused_loopback_port();
    let _env = EnvGuard::set(ENV_PORT, port.to_string());

    let err = run_dashboard(DashboardArgs {
        subcommand: None,
        port: None,
    })
    .unwrap_err()
    .to_string();

    assert!(
        err.contains(&port.to_string()),
        "dispatcher must fall back to the configured port {port}, got: {err}"
    );
}

#[test]
fn run_dashboard_uses_top_level_port_for_a_bare_open() {
    let _guard = process_state_lock();
    let port = unused_loopback_port();

    let err = run_dashboard(DashboardArgs {
        subcommand: None,
        port: Some(port),
    })
    .unwrap_err()
    .to_string();

    assert!(
        err.contains(&port.to_string()),
        "top-level --port {port} must be used, got: {err}"
    );
}

#[test]
fn run_dashboard_uses_top_level_port_when_open_has_no_port() {
    let _guard = process_state_lock();
    let port = unused_loopback_port();

    let err = run_dashboard(DashboardArgs {
        subcommand: Some(DashboardSubcommand::Open(DashboardOpenArgs { port: None })),
        port: Some(port),
    })
    .unwrap_err()
    .to_string();

    assert!(
        err.contains(&port.to_string()),
        "`dashboard open` must inherit top-level --port {port}, got: {err}"
    );
}

#[test]
fn run_dashboard_prefers_the_open_subcommand_port_over_the_top_level_port() {
    let _guard = process_state_lock();
    let (sub_port, top_port) = two_distinct_unused_ports();

    let err = run_dashboard(DashboardArgs {
        subcommand: Some(DashboardSubcommand::Open(DashboardOpenArgs {
            port: Some(sub_port),
        })),
        port: Some(top_port),
    })
    .unwrap_err()
    .to_string();

    assert!(
        err.contains(&sub_port.to_string()),
        "`open --port {sub_port}` must win over top-level --port, got: {err}"
    );
    assert!(
        !err.contains(&top_port.to_string()),
        "top-level port {top_port} must not be probed, got: {err}"
    );
}

#[test]
fn run_dashboard_dispatches_the_port_subcommand_to_the_config_write() {
    let _guard = process_state_lock();
    let tmp = tempfile::tempdir().unwrap();
    let agileplus_dir = tmp.path().join(".agileplus");
    std::fs::create_dir(&agileplus_dir).unwrap();
    std::fs::write(
        agileplus_dir.join("config.toml"),
        "[dashboard]\nport = 1111\n",
    )
    .unwrap();

    let _cwd = CwdGuard::enter(tmp.path());
    run_dashboard(DashboardArgs {
        subcommand: Some(DashboardSubcommand::Port(DashboardPortArgs { port: 4242 })),
        // A top-level port must be ignored by the `port` subcommand.
        port: Some(9999),
    })
    .expect("`dashboard port` must succeed in a writable directory");

    let content = std::fs::read_to_string(agileplus_dir.join("config.toml")).unwrap();
    assert_eq!(content, "[dashboard]\nport = 4242\n");
}

// ---------------------------------------------------------------------------
// Config persistence: creation, failure, and fallback
// ---------------------------------------------------------------------------

#[test]
fn run_dashboard_port_creates_the_agileplus_config_directory() {
    let _guard = process_state_lock();
    let tmp = tempfile::tempdir().unwrap();

    let _cwd = CwdGuard::enter(tmp.path());
    assert!(!tmp.path().join(".agileplus").exists());

    run_dashboard_port(4711).expect("writable cwd must persist the port");

    let config_path = tmp.path().join(".agileplus").join("config.toml");
    assert!(config_path.is_file(), "config file must be created");
    assert_eq!(
        std::fs::read_to_string(&config_path).unwrap(),
        "[dashboard]\nport = 4711\n"
    );
}

#[test]
fn run_dashboard_port_propagates_write_failures() {
    let _guard = process_state_lock();
    let tmp = tempfile::tempdir().unwrap();
    // `.agileplus` exists but is a file, so the config write must fail and the
    // error must reach the caller.
    let blocker = tmp.path().join(".agileplus");
    std::fs::write(&blocker, "not a directory").unwrap();

    let _cwd = CwdGuard::enter(tmp.path());
    let err = run_dashboard_port(5150).unwrap_err().to_string();

    assert!(
        err.contains("Failed to write config"),
        "write failure must be reported, got: {err}"
    );
    assert!(
        blocker.is_file(),
        "the blocking file must be left untouched"
    );
    assert_eq!(
        std::fs::read_to_string(&blocker).unwrap(),
        "not a directory"
    );
}

#[test]
fn run_dashboard_port_falls_back_to_an_env_hint_when_the_config_dir_is_unusable() {
    let _guard = process_state_lock();
    let tmp = tempfile::tempdir().unwrap();
    let dir_path = tmp.path().to_path_buf();

    let _cwd = CwdGuard::enter(&dir_path);
    // Removing the (now-current) directory makes `create_dir_all(".agileplus")`
    // fail while `.agileplus` still does not exist: the documented fallback.
    drop(tmp);

    assert!(
        !Path::new(".agileplus").exists(),
        "precondition: no config dir relative to the dead cwd"
    );
    run_dashboard_port(6161).expect("the env-hint fallback is not an error");
    assert!(
        !Path::new(".agileplus").exists(),
        "the fallback must not leave a config dir behind"
    );
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Parser)]
#[command(name = "dashboard-cli-test")]
struct DashboardCli {
    #[command(flatten)]
    args: DashboardArgs,
}

#[test]
fn cli_parses_a_bare_dashboard_command() {
    let cli = DashboardCli::try_parse_from(["dashboard"]).unwrap();
    assert!(cli.args.subcommand.is_none());
    assert!(cli.args.port.is_none());
}

#[test]
fn cli_parses_the_top_level_port_option() {
    let cli = DashboardCli::try_parse_from(["dashboard", "--port", "3000"]).unwrap();
    assert_eq!(cli.args.port, Some(3000));
    assert!(cli.args.subcommand.is_none());
}

#[test]
fn cli_parses_open_without_and_with_port() {
    let bare = DashboardCli::try_parse_from(["dashboard", "open"]).unwrap();
    match bare.args.subcommand {
        Some(DashboardSubcommand::Open(args)) => assert_eq!(args.port, None),
        other => panic!("expected `open`, got {other:?}"),
    }

    let with_port = DashboardCli::try_parse_from(["dashboard", "open", "--port", "5000"]).unwrap();
    match with_port.args.subcommand {
        Some(DashboardSubcommand::Open(args)) => assert_eq!(args.port, Some(5000)),
        other => panic!("expected `open`, got {other:?}"),
    }
}

#[test]
fn cli_parses_the_port_subcommand_positional() {
    let cli = DashboardCli::try_parse_from(["dashboard", "port", "9001"]).unwrap();
    match cli.args.subcommand {
        Some(DashboardSubcommand::Port(args)) => assert_eq!(args.port, 9001),
        other => panic!("expected `port`, got {other:?}"),
    }
}

#[test]
fn cli_accepts_the_maximum_port_value() {
    let cli = DashboardCli::try_parse_from(["dashboard", "port", "65535"]).unwrap();
    match cli.args.subcommand {
        Some(DashboardSubcommand::Port(args)) => assert_eq!(args.port, 65535),
        other => panic!("expected `port`, got {other:?}"),
    }
}

#[test]
fn cli_rejects_a_port_above_u16_range() {
    let err = DashboardCli::try_parse_from(["dashboard", "port", "70000"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("70000"), "error must quote the input: {err}");

    let err = DashboardCli::try_parse_from(["dashboard", "--port", "70000"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("70000"), "error must quote the input: {err}");
}

#[test]
fn cli_rejects_a_non_numeric_port() {
    let err = DashboardCli::try_parse_from(["dashboard", "port", "abc"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("abc"), "error must quote the input: {err}");
}

#[test]
fn cli_rejects_a_negative_port() {
    let err = DashboardCli::try_parse_from(["dashboard", "port", "-1"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("-1"), "error must quote the input: {err}");
}

#[test]
fn cli_requires_the_port_argument() {
    let err = DashboardCli::try_parse_from(["dashboard", "port"])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("<PORT>"),
        "error must mention the missing PORT argument: {err}"
    );
}

#[test]
fn cli_rejects_an_unknown_subcommand() {
    let err = DashboardCli::try_parse_from(["dashboard", "explode"])
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("explode"),
        "error must quote the unknown subcommand: {err}"
    );
}
