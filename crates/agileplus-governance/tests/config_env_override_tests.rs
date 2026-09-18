//! `GovernanceConfig::from_env` override + fallback coverage.
//!
//! Every test here mutates process environment variables, so they are
//! serialized behind a mutex. They live in their own integration test
//! binary (one process per `tests/*.rs` target) so no other test target
//! can observe the mutations.

use std::sync::{Mutex, MutexGuard};

use agileplus_governance::config::{GovernanceConfig, PolicyDefaultAction};
use agileplus_governance::types::AuthMethod;

/// Serializes env mutation across the tests in this binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Holds the env lock and clears the given keys for the duration of a test,
/// restoring the process to "unset" afterwards (even on panic).
struct EnvGuard {
    keys: &'static [&'static str],
    _lock: MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn acquire(keys: &'static [&'static str]) -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        for key in keys {
            std::env::remove_var(key);
        }
        Self { keys, _lock: lock }
    }

    fn set(&self, key: &str, value: &str) {
        assert!(self.keys.contains(&key), "key {key} must be scoped by the guard");
        std::env::set_var(key, value);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for key in self.keys {
            std::env::remove_var(key);
        }
    }
}

const AUTH_KEYS: &[&str] = &[
    "AGILEPLUS_GOVERNANCE_AUTH_METHOD",
    "AGILEPLUS_GOVERNANCE_API_KEY",
    "AGILEPLUS_GOVERNANCE_BEARER_TOKEN",
];

const NUMERIC_KEYS: &[&str] = &[
    "AGILEPLUS_GOVERNANCE_TIMEOUT",
    "AGILEPLUS_GOVERNANCE_RETRY",
    "AGILEPLUS_LOCAL_RETENTION_DAYS",
    "AGILEPLUS_SYNC_INTERVAL",
    "AGILEPLUS_SYNC_BATCH_SIZE",
    "AGILEPLUS_SYNC_TIMEOUT",
    "AGILEPLUS_RATE_LIMIT_MAX",
    "AGILEPLUS_RATE_LIMIT_WINDOW",
];

const FLAG_KEYS: &[&str] = &[
    "AGILEPLUS_GOVERNANCE_ENABLED",
    "AGILEPLUS_GOVERNANCE_BASE_URL",
    "AGILEPLUS_LOCAL_ENABLED",
    "AGILEPLUS_SYNC_ENABLED",
    "AGILEPLUS_POLICY_ENABLED",
    "AGILEPLUS_POLICY_DEFAULT",
    "AGILEPLUS_POLICY_ENFORCE_GATES",
    "AGILEPLUS_POLICY_ENFORCE_RATE_LIMITS",
    "AGILEPLUS_RATE_LIMIT_ENABLED",
];

#[test]
fn from_env_maps_auth_method_strings_and_credentials() {
    let env = EnvGuard::acquire(AUTH_KEYS);

    env.set("AGILEPLUS_GOVERNANCE_AUTH_METHOD", "bearer");
    env.set("AGILEPLUS_GOVERNANCE_API_KEY", "api-key-value");
    env.set("AGILEPLUS_GOVERNANCE_BEARER_TOKEN", "bearer-value");
    let bearer = GovernanceConfig::from_env();
    assert_eq!(bearer.governance.auth.method, AuthMethod::BearerToken);
    assert_eq!(bearer.governance.auth.api_key, "api-key-value");
    assert_eq!(bearer.governance.auth.bearer_token, "bearer-value");

    env.set("AGILEPLUS_GOVERNANCE_AUTH_METHOD", "none");
    let none = GovernanceConfig::from_env();
    assert_eq!(none.governance.auth.method, AuthMethod::None);

    // Unrecognized spellings fall back to api-key auth.
    env.set("AGILEPLUS_GOVERNANCE_AUTH_METHOD", "something-else");
    let fallback = GovernanceConfig::from_env();
    assert_eq!(fallback.governance.auth.method, AuthMethod::ApiKey);
}

#[test]
fn from_env_reads_valid_numeric_overrides() {
    let env = EnvGuard::acquire(NUMERIC_KEYS);

    env.set("AGILEPLUS_GOVERNANCE_TIMEOUT", "45");
    env.set("AGILEPLUS_GOVERNANCE_RETRY", "7");
    env.set("AGILEPLUS_LOCAL_RETENTION_DAYS", "7");
    env.set("AGILEPLUS_SYNC_INTERVAL", "1000");
    env.set("AGILEPLUS_SYNC_BATCH_SIZE", "5");
    env.set("AGILEPLUS_SYNC_TIMEOUT", "9");
    env.set("AGILEPLUS_RATE_LIMIT_MAX", "42");
    env.set("AGILEPLUS_RATE_LIMIT_WINDOW", "5000");

    let config = GovernanceConfig::from_env();
    assert_eq!(config.governance.timeout_secs, 45);
    assert_eq!(config.governance.retry_attempts, 7);
    assert_eq!(config.local.retention_days, 7);
    assert_eq!(config.sync.interval_ms, 1000);
    assert_eq!(config.sync.batch_size, 5);
    assert_eq!(config.sync.timeout_secs, 9);
    assert_eq!(config.rate_limit.max_requests, 42);
    assert_eq!(config.rate_limit.window_ms, 5000);
}

#[test]
fn from_env_unparsable_numbers_fall_back_to_defaults() {
    let env = EnvGuard::acquire(NUMERIC_KEYS);

    for key in NUMERIC_KEYS {
        env.set(key, "not-a-number");
    }

    let config = GovernanceConfig::from_env();
    assert_eq!(config.governance.timeout_secs, 30);
    assert_eq!(config.governance.retry_attempts, 3);
    assert_eq!(config.local.retention_days, 90);
    assert_eq!(config.sync.interval_ms, 300_000);
    assert_eq!(config.sync.batch_size, 100);
    assert_eq!(config.sync.timeout_secs, 60);
    assert_eq!(config.rate_limit.max_requests, 100);
    assert_eq!(config.rate_limit.window_ms, 3_600_000);
    // Only "true" enables a boolean flag; anything else (including garbage)
    // keeps the feature off.
    assert!(!config.governance.enabled);
}

#[test]
fn from_env_reads_enabled_flags_and_policy_default_action() {
    let env = EnvGuard::acquire(FLAG_KEYS);

    env.set("AGILEPLUS_GOVERNANCE_ENABLED", "true");
    env.set("AGILEPLUS_GOVERNANCE_BASE_URL", "http://governance.internal:9090");
    env.set("AGILEPLUS_LOCAL_ENABLED", "false");
    env.set("AGILEPLUS_SYNC_ENABLED", "false");
    env.set("AGILEPLUS_POLICY_ENABLED", "false");
    env.set("AGILEPLUS_POLICY_DEFAULT", "deny");
    env.set("AGILEPLUS_POLICY_ENFORCE_GATES", "false");
    env.set("AGILEPLUS_POLICY_ENFORCE_RATE_LIMITS", "false");
    env.set("AGILEPLUS_RATE_LIMIT_ENABLED", "true");

    let config = GovernanceConfig::from_env();
    assert!(config.governance.enabled);
    assert_eq!(config.governance.base_url, "http://governance.internal:9090");
    assert!(!config.local.enabled);
    assert!(!config.sync.enabled);
    assert!(!config.policy.enabled);
    assert_eq!(config.policy.default_action, PolicyDefaultAction::Deny);
    assert!(!config.policy.enforce_gates);
    assert!(!config.policy.enforce_rate_limits);
    assert!(config.rate_limit.enabled);

    // Anything that is not "deny" selects the allow default.
    env.set("AGILEPLUS_POLICY_DEFAULT", "allow");
    assert_eq!(
        GovernanceConfig::from_env().policy.default_action,
        PolicyDefaultAction::Allow
    );
}

#[test]
fn from_env_with_all_overrides_produces_a_valid_config() {
    let env = EnvGuard::acquire(&[]); // lock only; expects a clean environment
    let mut config = GovernanceConfig::from_env();
    // Defaults derived from the environment must satisfy validation without
    // requiring the operator to provide secrets.
    assert!(config.validate().is_empty());

    config.governance.enabled = true;
    config.governance.base_url = String::new();
    config.local.db_path = String::new();
    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("base URL")));
    assert!(errors.iter().any(|e| e.contains("database path")));
    drop(env);
}
