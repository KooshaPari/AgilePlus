// SPDX-License-Identifier: MIT OR Apache-2.0
//! Service-probe behaviour of `/detailed-health`, and the env-backed
//! `SharedSecretVerifier`.
//!
//! Both subjects read process environment variables, which are process-global
//! state. This file is therefore its own integration-test binary (its own
//! process), so a mutation here can never perturb
//! `tests/api_integration.rs`, and within it every test that touches the
//! environment holds [`ENV_MUTEX`] for its whole body via [`EnvSandbox`].
//!
//! What was previously untested: the handlers only ever saw *absent* probe
//! variables. Every test below pins the configured branches — TCP connect to a
//! live listener, connection refused, scheme stripping, the multi-key fallback
//! for Dragonfly/Redis, and the overall-status derivation that follows.
//!
//! Traceability: WP11-T070, FR-AGP-012

#![allow(dead_code)]

#[path = "api_integration/support/mod.rs"]
mod support;

use std::ffi::OsString;
use std::sync::{Mutex, MutexGuard};

use agileplus_api::middleware::token_verifier::{SharedSecretVerifier, TokenVerifier};

use crate::support::{MockStorage, setup_test_server_with_storage};

/// Serializes every environment read/write in this binary.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Variables the health probe consults.
const PROBE_KEYS: [&str; 5] = [
    "NATS_URL",
    "DRAGONFLY_URL",
    "REDIS_URL",
    "NEO4J_URI",
    "S3_ENDPOINT",
];

/// Holds [`ENV_MUTEX`], clears the given keys, and restores their previous
/// values on drop.
struct EnvSandbox {
    _guard: MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<OsString>)>,
}

impl EnvSandbox {
    fn cleared(keys: &[&'static str]) -> Self {
        // `unwrap_or_else(into_inner)` keeps the suite working even if an
        // earlier test panicked while holding the lock.
        let guard = ENV_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let saved = keys
            .iter()
            .map(|key| (*key, std::env::var_os(key)))
            .collect();
        for key in keys {
            // SAFETY: `set_var`/`remove_var` are unsafe because concurrent
            // environment access is data-racy. Every test in this binary that
            // reads these variables holds `ENV_MUTEX` for its whole body, and
            // the guard is held here for as long as the mutation is live.
            unsafe { std::env::remove_var(key) };
        }
        Self {
            _guard: guard,
            saved,
        }
    }

    fn probes() -> Self {
        Self::cleared(&PROBE_KEYS)
    }

    fn set(&self, key: &str, value: &str) {
        // SAFETY: see `cleared` — the sandbox holds `ENV_MUTEX`.
        unsafe { std::env::set_var(key, value) };
    }
}

impl Drop for EnvSandbox {
    fn drop(&mut self) {
        for (key, previous) in &self.saved {
            // SAFETY: see `cleared` — the sandbox still holds `ENV_MUTEX`.
            unsafe {
                match previous {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Bind an ephemeral port and keep the listener alive so the port stays open.
async fn live_listener() -> (tokio::net::TcpListener, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind an ephemeral port");
    let addr = listener.local_addr().expect("listener address").to_string();
    (listener, addr)
}

/// Reserve a port and immediately release it so connects are refused.
async fn closed_port() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind an ephemeral port");
    let addr = listener.local_addr().expect("listener address").to_string();
    drop(listener);
    addr
}

async fn detailed_health() -> serde_json::Value {
    let server = setup_test_server_with_storage(MockStorage::with_test_data()).await;
    let resp = server.get("/detailed-health").await;
    resp.assert_status_ok();
    resp.json()
}

// ── Probes ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_default_deployment_reports_every_probe_not_configured_and_stays_healthy() {
    let _env = EnvSandbox::probes();
    let body = detailed_health().await;

    for key in ["nats", "dragonfly", "neo4j", "minio"] {
        assert_eq!(
            body["services"][key]["status"], "not_configured",
            "{key} is unset, so it is not configured, got: {body}"
        );
        assert!(
            body["services"][key]["error"]
                .as_str()
                .is_some_and(|e| e.contains("not configured")),
            "{key} should explain itself, got: {body}"
        );
    }
    assert_eq!(body["services"]["sqlite"]["status"], "healthy");
    assert_eq!(
        body["status"], "healthy",
        "unconfigured services must not degrade a deployment, got: {body}"
    );
}

#[tokio::test]
async fn nats_url_is_probed_and_its_scheme_stripped() {
    let env = EnvSandbox::probes();
    let (_listener, addr) = live_listener().await;
    env.set("NATS_URL", &format!("nats://{addr}/stream"));

    let body = detailed_health().await;
    assert_eq!(
        body["services"]["nats"]["status"], "healthy",
        "a reachable NATS_URL must probe healthy (scheme and path stripped), got: {body}"
    );
    assert!(
        body["services"]["nats"]["latency_ms"].is_u64(),
        "a successful probe reports latency, got: {body}"
    );
    assert!(
        body["services"]["nats"].get("error").is_none(),
        "a healthy service carries no error, got: {body}"
    );
    assert_eq!(
        body["services"]["minio"]["status"], "not_configured",
        "other services must be unaffected, got: {body}"
    );
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn dragonfly_probes_its_primary_env_var() {
    let env = EnvSandbox::probes();
    let (_listener, addr) = live_listener().await;
    env.set("DRAGONFLY_URL", &addr);

    let body = detailed_health().await;
    assert_eq!(
        body["services"]["dragonfly"]["status"], "healthy",
        "DRAGONFLY_URL was set and reachable, got: {body}"
    );
}

/// `DRAGONFLY_URL` is tried first, `REDIS_URL` second: with only the fallback
/// set, the probe must still resolve.
#[tokio::test]
async fn dragonfly_falls_back_to_redis_url_when_the_primary_is_unset() {
    let env = EnvSandbox::probes();
    let (_listener, addr) = live_listener().await;
    env.set("REDIS_URL", &addr);

    let body = detailed_health().await;
    assert_eq!(
        body["services"]["dragonfly"]["status"], "healthy",
        "the REDIS_URL fallback must be consulted, got: {body}"
    );
}

#[tokio::test]
async fn minio_probes_s3_endpoint_with_an_http_scheme() {
    let env = EnvSandbox::probes();
    let (_listener, addr) = live_listener().await;
    env.set("S3_ENDPOINT", &format!("http://{addr}/bucket"));

    let body = detailed_health().await;
    assert_eq!(
        body["services"]["minio"]["status"], "healthy",
        "an http:// S3_ENDPOINT must have its scheme stripped before probing, got: {body}"
    );
}

/// A configured-but-unreachable service is exactly what the probe exists to
/// catch: it must name the address and flip the overall status to
/// `unavailable`, while leaving the HTTP API itself healthy.
#[tokio::test]
async fn a_refused_service_probe_makes_the_service_unavailable_and_the_deployment_degrade() {
    let env = EnvSandbox::probes();
    let addr = closed_port().await;
    env.set("NEO4J_URI", &format!("bolt://{addr}"));

    let body = detailed_health().await;
    assert_eq!(
        body["services"]["neo4j"]["status"], "unavailable",
        "a refused connection must be reported as unavailable, got: {body}"
    );
    let error = body["services"]["neo4j"]["error"]
        .as_str()
        .expect("an unavailable service carries the reason");
    assert!(
        error.starts_with(&addr),
        "the reason should identify the probed address ({addr}), got: {error}"
    );
    assert_eq!(
        body["status"], "unavailable",
        "an unavailable dependency dominates the overall status, got: {body}"
    );
    assert_eq!(
        body["api"]["status"], "healthy",
        "the API process itself is still serving, got: {body}"
    );
    assert_eq!(
        body["services"]["dragonfly"]["status"], "not_configured",
        "only the configured service is affected, got: {body}"
    );
}

// ── SharedSecretVerifier::from_env ───────────────────────────────────────────

#[tokio::test]
async fn verifier_from_env_accepts_each_comma_separated_key() {
    let env = EnvSandbox::cleared(&["AGILEPLUS_API_KEY"]);
    env.set("AGILEPLUS_API_KEY", " alpha , beta ,, gamma ");

    let verifier = SharedSecretVerifier::from_env();
    for key in ["alpha", "beta", "gamma"] {
        assert!(
            verifier.verify(key).expect("verification cannot fail"),
            "`{key}` came from AGILEPLUS_API_KEY and must verify"
        );
    }
    assert!(
        !verifier.verify("delta").expect("verification cannot fail"),
        "an unknown key must not verify"
    );
    assert!(
        !verifier.verify("alpha ").expect("verification cannot fail"),
        "keys are trimmed at parse time, so the padded form is not a key"
    );
    assert!(
        !verifier.verify("").expect("verification cannot fail"),
        "the blank entry between the doubled separators must be dropped"
    );
}

#[tokio::test]
async fn verifier_from_env_rejects_everything_when_the_variable_is_unset() {
    let _env = EnvSandbox::cleared(&["AGILEPLUS_API_KEY"]);

    let verifier = SharedSecretVerifier::from_env();
    assert!(
        !verifier.verify("").expect("verification cannot fail"),
        "with no configured keys nothing can verify"
    );
    assert!(
        !verifier
            .verify("any-token")
            .expect("verification cannot fail"),
        "with no configured keys nothing can verify"
    );
}
