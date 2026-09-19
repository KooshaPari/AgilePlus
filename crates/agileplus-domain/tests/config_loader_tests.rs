//! Integration tests for `config::loader` — config path resolution, load-time
//! validation, env-var overrides, and `init_default`.
//!
//! `AppConfig::load*` and `AppConfig::init_default` resolve
//! `~/.agileplus/config.toml` through `dirs_next::home_dir()`, so these tests
//! repoint `HOME`. The variable is process-global; every test that depends on it
//! runs behind one mutex and restores the previous environment on drop.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use agileplus_domain::config::{AppConfig, ConfigError};

/// Serialises every test in this binary that rewrites the process environment.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Variables `load_with_env_overrides` reads. Cleared for the duration of a test
/// so the ambient developer environment cannot leak in.
const OVERRIDE_VARS: [&str; 7] = [
    "API_PORT",
    "AGILEPLUS_HTTP_PORT",
    "AGILEPLUS_API_PORT",
    "AGILEPLUS_GRPC_PORT",
    "AGILEPLUS_TELEMETRY_LOG_LEVEL",
    "AGILEPLUS_CORE_DB_PATH",
    "AGILEPLUS_CORE_SPECS_DIR",
];

/// Points `HOME` at a fresh temporary directory, clears the override variables,
/// and restores everything on drop.
struct HomeSandbox {
    previous_home: Option<std::ffi::OsString>,
    previous_vars: Vec<(&'static str, Option<std::ffi::OsString>)>,
    home: PathBuf,
    // Kept alive for the duration of the test so the temporary home is removed
    // on drop.
    _dir: tempfile::TempDir,
    // Declared last; the explicit `Drop` impl runs first, while it is still held.
    _guard: MutexGuard<'static, ()>,
}

impl HomeSandbox {
    fn new() -> Self {
        let guard = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous_home = std::env::var_os("HOME");
        let previous_vars = OVERRIDE_VARS
            .iter()
            .map(|name| (*name, std::env::var_os(name)))
            .collect();
        for name in OVERRIDE_VARS {
            // SAFETY: process-global; ENV_LOCK serialises every test that uses it.
            unsafe { std::env::remove_var(name) };
        }
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        // SAFETY: as above; restored in `Drop`.
        unsafe { std::env::set_var("HOME", &home) };
        Self {
            previous_home,
            previous_vars,
            home,
            _dir: dir,
            _guard: guard,
        }
    }

    fn home(&self) -> &Path {
        &self.home
    }

    fn set(&self, name: &str, value: &str) {
        assert!(OVERRIDE_VARS.contains(&name), "unexpected var {name}");
        // SAFETY: ENV_LOCK is held for the whole test.
        unsafe { std::env::set_var(name, value) };
    }

    fn config_file(&self) -> PathBuf {
        self.home.join(".agileplus").join("config.toml")
    }

    fn write_config(&self, contents: &str) {
        let path = self.config_file();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, contents).unwrap();
    }
}

impl Drop for HomeSandbox {
    fn drop(&mut self) {
        for (name, value) in self.previous_vars.drain(..) {
            // SAFETY: ENV_LOCK is still held here.
            unsafe {
                match value {
                    Some(value) => std::env::set_var(name, value),
                    None => std::env::remove_var(name),
                }
            }
        }
        // SAFETY: as above.
        unsafe {
            match self.previous_home.take() {
                Some(value) => std::env::set_var("HOME", value),
                None => std::env::remove_var("HOME"),
            }
        }
    }
}

#[test]
fn config_path_resolves_under_the_home_directory() {
    let sandbox = HomeSandbox::new();
    assert_eq!(AppConfig::config_path(), sandbox.config_file());
}

#[test]
fn load_without_a_file_returns_defaults_and_writes_nothing() {
    let sandbox = HomeSandbox::new();
    let config = AppConfig::load().expect("defaults load");

    assert_eq!(config.api.port, AppConfig::default().api.port);
    assert_eq!(config.api.grpc_port, AppConfig::default().api.grpc_port);
    assert!(!sandbox.config_file().exists());
    assert!(!sandbox.home().join(".agileplus").exists());
}

#[test]
fn load_reads_partial_toml_and_keeps_defaults_for_omitted_sections() {
    let sandbox = HomeSandbox::new();
    sandbox.write_config("[telemetry]\nlog_level = \"warn\"\n");

    let config = AppConfig::load().expect("partial config loads");
    assert_eq!(config.telemetry.log_level, "warn");
    // Sections that were omitted fall back to their defaults.
    let defaults = AppConfig::default();
    assert_eq!(config.api.port, defaults.api.port);
    assert_eq!(config.api.grpc_port, defaults.api.grpc_port);
    assert_eq!(config.agents.default_agent, defaults.agents.default_agent);
    assert_eq!(config.agents.max_subagents, defaults.agents.max_subagents);
    assert_eq!(config.core.database_path, defaults.core.database_path);
    assert_eq!(config.core.specs_dir, defaults.core.specs_dir);
}

#[test]
fn load_reports_a_toml_parse_error_for_malformed_content() {
    let sandbox = HomeSandbox::new();
    sandbox.write_config("this is not = toml =\n");

    let error = AppConfig::load().expect_err("malformed toml must fail");
    assert!(
        matches!(error, ConfigError::TomlParse(_)),
        "expected TomlParse, got {error:?}"
    );
    assert!(error.to_string().starts_with("TOML parse error:"));
}

#[test]
fn load_reports_an_io_error_when_the_config_path_is_a_directory() {
    let sandbox = HomeSandbox::new();
    // `path.exists()` is true, but reading a directory as a file fails.
    std::fs::create_dir_all(sandbox.config_file()).unwrap();

    let error = AppConfig::load().expect_err("directory is not a config file");
    assert!(
        matches!(error, ConfigError::Io(_)),
        "expected Io, got {error:?}"
    );
    assert!(error.to_string().starts_with("IO error:"));
}

#[test]
fn load_rejects_a_zero_api_port_from_file() {
    let sandbox = HomeSandbox::new();
    sandbox.write_config("[api]\nport = 0\n");

    let error = AppConfig::load().expect_err("port 0 must fail validation");
    assert_eq!(
        error.to_string(),
        "invalid config value: api.port must be > 0"
    );
}

#[test]
fn init_default_creates_the_parent_directory_and_defaults_are_loadable() {
    let sandbox = HomeSandbox::new();
    // `.agileplus` deliberately does not exist yet.
    assert!(!sandbox.home().join(".agileplus").exists());

    let written = AppConfig::init_default().expect("init_default writes defaults");
    assert_eq!(written, sandbox.config_file());
    assert!(written.is_file());

    let loaded = AppConfig::load().expect("written file is valid");
    assert_eq!(loaded.api.port, AppConfig::default().api.port);
}

#[test]
fn init_default_leaves_an_existing_file_untouched() {
    let sandbox = HomeSandbox::new();
    sandbox.write_config("[api]\nport = 4711\n");

    let written = AppConfig::init_default().expect("no-op init");
    assert_eq!(written, sandbox.config_file());
    assert_eq!(AppConfig::load().unwrap().api.port, 4711);
}

#[test]
fn env_overrides_apply_in_order_and_the_last_source_wins() {
    let sandbox = HomeSandbox::new();
    sandbox.write_config("[api]\nport = 1\ngrpc_port = 2\n");
    sandbox.set("API_PORT", "5001");
    sandbox.set("AGILEPLUS_HTTP_PORT", "5002");
    sandbox.set("AGILEPLUS_API_PORT", "5003");
    sandbox.set("AGILEPLUS_GRPC_PORT", "5004");
    sandbox.set("AGILEPLUS_TELEMETRY_LOG_LEVEL", "debug");
    sandbox.set("AGILEPLUS_CORE_DB_PATH", "/tmp/override.db");
    sandbox.set("AGILEPLUS_CORE_SPECS_DIR", "specs-from-env");

    let config = AppConfig::load_with_env_overrides().expect("overrides apply");
    assert_eq!(
        config.api.port, 5003,
        "AGILEPLUS_API_PORT wins over API_PORT"
    );
    assert_eq!(config.api.grpc_port, 5004);
    assert_eq!(config.telemetry.log_level, "debug");
    assert_eq!(config.core.database_path, PathBuf::from("/tmp/override.db"));
    assert_eq!(config.core.specs_dir, "specs-from-env");
}

#[test]
fn env_override_parse_error_names_the_variable() {
    let sandbox = HomeSandbox::new();
    sandbox.set("AGILEPLUS_GRPC_PORT", "not-a-port");

    let error = AppConfig::load_with_env_overrides().expect_err("bad port must fail");
    assert!(
        matches!(error, ConfigError::EnvParse(ref message) if message == "AGILEPLUS_GRPC_PORT=not-a-port"),
        "got {error:?}"
    );
    assert!(
        error
            .to_string()
            .contains("environment variable parse error: AGILEPLUS_GRPC_PORT=not-a-port")
    );
}

#[test]
fn env_override_is_validated_after_being_applied() {
    let sandbox = HomeSandbox::new();
    sandbox.set("AGILEPLUS_TELEMETRY_LOG_LEVEL", "verbose");

    let error = AppConfig::load_with_env_overrides().expect_err("bad log level must fail");
    assert!(
        matches!(
            error,
            ConfigError::Validation(ref message) if message.contains("invalid log_level 'verbose'")
        ),
        "got {error:?}"
    );
}

#[test]
fn config_error_display_messages_are_stable() {
    let io = ConfigError::Io(std::io::Error::new(ErrorKind::Other, "disk gone"));
    assert_eq!(io.to_string(), "IO error: disk gone");

    assert_eq!(
        ConfigError::Validation("bad value".to_string()).to_string(),
        "invalid config value: bad value"
    );
    assert_eq!(
        ConfigError::EnvParse("API_PORT=abc".to_string()).to_string(),
        "environment variable parse error: API_PORT=abc"
    );

    let parse: ConfigError = toml::from_str::<i32>("nope").unwrap_err().into();
    assert!(parse.to_string().starts_with("TOML parse error:"));
}
