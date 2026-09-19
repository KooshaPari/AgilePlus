// SPDX-License-Identifier: MIT OR Apache-2.0
//! Startup contract of the `agileplus-api` binary.
//!
//! `src/main.rs` is a binary target, so no library test can reach it: the
//! argument handling, the `DATABASE_URL` translation, and the operator-key
//! requirement were all unexercised. These tests spawn the real artifact
//! (`CARGO_BIN_EXE_agileplus-api`) under an isolated `HOME`, so the operator's
//! own `~/.agileplus/config.toml` can neither satisfy nor break the run.
//!
//! Every invocation below either exits before any storage is opened
//! (`--dump-openapi`, bad `DATABASE_URL`) or confines its writes to a
//! per-test temporary directory.
//!
//! Traceability: WP11-T064, WP11-T069, #118

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use utoipa::OpenApi;

const BIN: &str = env!("CARGO_BIN_EXE_agileplus-api");

/// Environment variables the binary reads that could leak in from the
/// developer's shell and change the outcome.
const RUNTIME_VARS: [&str; 11] = [
    "DATABASE_URL",
    "AGILEPLUS_API_KEY",
    "AGILEPLUS_CREDENTIAL_KEY",
    "API_HOST",
    "AGILEPLUS_API_HOST",
    "API_PORT",
    "AGILEPLUS_HTTP_PORT",
    "AGILEPLUS_API_PORT",
    "AGILEPLUS_GRPC_PORT",
    "AGILEPLUS_TELEMETRY_LOG_LEVEL",
    "AGILEPLUS_CORE_DB_PATH",
];

struct Sandbox {
    home: PathBuf,
}

impl Sandbox {
    fn new(name: &str) -> Self {
        let home = std::env::temp_dir().join(format!(
            "agileplus-api-binary-contract-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join(".agileplus")).expect("create the sandbox home");
        Self { home }
    }

    /// A child process with a clean environment and `HOME` pointed at the
    /// sandbox, so `~/.agileplus/config.toml` resolves inside it.
    fn command(&self) -> Command {
        let mut command = Command::new(BIN);
        for key in RUNTIME_VARS {
            command.env_remove(key);
        }
        command.env("HOME", &self.home);
        command
    }

    fn credentials_file(&self) -> PathBuf {
        self.home.join(".agileplus").join("credentials.enc")
    }

    fn database_path(&self) -> PathBuf {
        self.home.join("agileplus.db")
    }

    /// Write a `config.toml` that selects the file credential backend, so the
    /// child never reaches for the OS keychain.
    fn write_file_backend_config(&self) {
        let config = format!(
            "[credentials]\nbackend = \"file\"\nfile_path = \"{}\"\n",
            self.credentials_file().display()
        );
        std::fs::write(self.home.join(".agileplus").join("config.toml"), config)
            .expect("write the sandbox config");
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ── `--dump-openapi` ─────────────────────────────────────────────────────────

#[test]
fn dump_openapi_prints_exactly_the_generated_document() {
    let sandbox = Sandbox::new("dump-openapi");
    let output = sandbox
        .command()
        .arg("--dump-openapi")
        .output()
        .expect("the built binary runs");

    assert!(
        output.status.success(),
        "--dump-openapi must exit 0, stderr: {}",
        stderr_of(&output)
    );

    let stdout = String::from_utf8(output.stdout).expect("the OpenAPI document is UTF-8");
    let expected = serde_yaml::to_string(&agileplus_api::openapi::ApiDoc::openapi())
        .expect("the library document serializes");
    assert_eq!(
        stdout, expected,
        "the binary must dump the document the library generates, byte for byte"
    );
}

#[test]
fn dumped_openapi_declares_the_committed_contract() {
    let sandbox = Sandbox::new("openapi-contract");
    let output = sandbox
        .command()
        .arg("--dump-openapi")
        .output()
        .expect("the built binary runs");
    assert!(output.status.success(), "stderr: {}", stderr_of(&output));

    let document: serde_yaml::Value =
        serde_yaml::from_slice(&output.stdout).expect("stdout parses as YAML");

    assert_eq!(document["info"]["title"], "AgilePlus REST API");
    assert!(
        document["info"]["version"].is_string(),
        "the document carries a version"
    );
    assert_eq!(document["info"]["license"]["name"], "MIT");

    let paths = document["paths"]
        .as_mapping()
        .expect("paths is a mapping")
        .keys()
        .filter_map(|key| key.as_str())
        .collect::<Vec<_>>();
    for path in [
        "/api/v1/features",
        "/api/v1/work-packages/{id}",
        "/api/v1/events",
        "/api/v1/features/{slug}/audit",
        "/api/v1/features/{slug}/governance",
    ] {
        assert!(
            paths.contains(&path),
            "the dumped contract is missing {path}, got: {paths:?}"
        );
    }

    for schema in [
        "FeatureResponse",
        "CreateFeatureRequest",
        "WorkPackageResponse",
        "EventResponse",
        "AuditEntryResponse",
        "GovernanceResponse",
    ] {
        assert!(
            document["components"]["schemas"][schema].is_mapping(),
            "the dumped contract is missing schema {schema}"
        );
    }

    assert_eq!(
        document["components"]["securitySchemes"]["api_key"]["name"], "X-API-Key",
        "the only documented credential is the X-API-Key header"
    );
    assert_eq!(
        document["components"]["securitySchemes"]["api_key"]["in"],
        "header"
    );
}

// ── DATABASE_URL translation ─────────────────────────────────────────────────

#[test]
fn rejects_a_database_url_that_is_not_sqlite() {
    let sandbox = Sandbox::new("non-sqlite-url");
    let output = sandbox
        .command()
        .env("DATABASE_URL", "postgres://localhost:5432/agileplus")
        .output()
        .expect("the built binary runs");

    assert!(
        !output.status.success(),
        "a non-sqlite DATABASE_URL must abort startup"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("must use the sqlite: scheme"),
        "startup should name the required scheme, got: {stderr}"
    );
    assert!(
        !sandbox.database_path().exists(),
        "startup must abort before touching the database"
    );
}

#[test]
fn rejects_a_sqlite_database_url_without_a_path() {
    let sandbox = Sandbox::new("empty-sqlite-path");
    let output = sandbox
        .command()
        .env("DATABASE_URL", "sqlite:")
        .output()
        .expect("the built binary runs");

    assert!(
        !output.status.success(),
        "`sqlite:` with no path must abort startup"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("must include a filesystem path after sqlite:"),
        "startup should explain the missing path, got: {stderr}"
    );
    assert!(
        !sandbox.database_path().exists(),
        "startup must abort before touching the database"
    );
}

// ── Operator key requirement ─────────────────────────────────────────────────

/// File-backed credentials need a passphrase to derive their key from, so a
/// deployment that selects the file backend without one must not start.
#[test]
fn requires_a_credential_encryption_key_for_file_backed_credentials() {
    let sandbox = Sandbox::new("missing-credential-key");
    sandbox.write_file_backend_config();

    let output = sandbox
        .command()
        .env("DATABASE_URL", format!("sqlite:{}", sandbox.database_path().display()))
        .output()
        .expect("the built binary runs");

    assert!(
        !output.status.success(),
        "startup must fail without a credential encryption key"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("credential encryption key is required"),
        "startup should name the credential-key requirement, got: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "the refusal must be a clean error, got: {stderr}"
    );
}

/// The API key may only come from the operator's environment. With the
/// credential store reachable and a database path prepared, a missing
/// `AGILEPLUS_API_KEY` must still stop the process rather than let it serve
/// unauthenticated traffic.
#[test]
fn requires_an_operator_supplied_api_key_to_start() {
    let sandbox = Sandbox::new("missing-api-key");
    sandbox.write_file_backend_config();

    let output = sandbox
        .command()
        .env("DATABASE_URL", format!("sqlite:{}", sandbox.database_path().display()))
        .env("AGILEPLUS_CREDENTIAL_KEY", "operator-passphrase-for-tests")
        .output()
        .expect("the built binary runs");

    assert!(
        !output.status.success(),
        "startup must fail without an operator-managed API key"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("AGILEPLUS_API_KEY is required"),
        "startup should name the missing variable, got: {stderr}"
    );
    assert!(
        !stderr.contains("panicked"),
        "the refusal must be a clean error, got: {stderr}"
    );
}

#[test]
fn binary_path_used_by_these_tests_exists() {
    assert!(
        Path::new(BIN).exists(),
        "cargo must build the binary for {BIN}"
    );
}
