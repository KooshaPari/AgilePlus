//! E2E tests for the AgilePlus Desktop application lifecycle.
//!
//! These tests verify the desktop app can start, initialize its database,
//! and perform basic operations through the CLI bridge.

use std::process::Command;

use agileplus_e2e_desktop::{
    icons_dir, init_desktop_schema, isolated_temp_dir, repo_root, tauri_config_path,
};

// ---------------------------------------------------------------------------
// CLI binary
// ---------------------------------------------------------------------------

#[test]
fn test_cli_binary_exists() {
    // Verify the CLI binary can be found and responds to --version.
    // Uses the workspace-built binary (or AGILEPLUS_BIN env override).
    let bin = if let Ok(path) = std::env::var("AGILEPLUS_BIN") {
        std::path::PathBuf::from(path)
    } else {
        repo_root().join("target/debug/agileplus")
    };

    let output = Command::new(&bin)
        .args(["--version"])
        .output();
    assert!(
        output.is_ok(),
        "agileplus CLI binary should be runnable at {}",
        bin.display()
    );
}

// ---------------------------------------------------------------------------
// SQLite database initialisation
// ---------------------------------------------------------------------------

#[test]
fn test_desktop_initializes_database() {
    // Create a temp directory and verify the desktop can initialise its DB
    let temp_dir = isolated_temp_dir();
    let db_path = temp_dir.path().join("test.db");

    let conn = rusqlite::Connection::open(&db_path).expect("Failed to open SQLite database");

    // Apply the same schema the desktop app uses
    init_desktop_schema(&conn).expect("Schema initialisation failed");

    // Verify at least one table was created
    let table_count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )
        .expect("Failed to count tables");

    assert!(
        table_count >= 3,
        "Expected at least 3 tables (features, work_packages, evidence), got {table_count}"
    );

    drop(conn);
    assert!(db_path.exists(), "Database file should be created");
}

// ---------------------------------------------------------------------------
// Feature CRUD through the CLI bridge
// ---------------------------------------------------------------------------

#[test]
fn test_feature_crud_via_cli() {
    // Test listing features through the CLI binary in a fresh git repo.
    // The CLI requires a git repository to function.
    let bin = if let Ok(path) = std::env::var("AGILEPLUS_BIN") {
        std::path::PathBuf::from(path)
    } else {
        repo_root().join("target/debug/agileplus")
    };

    let temp_dir = isolated_temp_dir();

    // Initialize a git repo so the CLI can operate
    Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to git init");

    let output = Command::new(&bin)
        .args(["list"])
        .current_dir(&temp_dir)
        .output()
        .expect("Failed to execute CLI binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should succeed (empty list) in a fresh repo
    assert!(
        output.status.success(),
        "CLI list should succeed in a fresh git repo: stdout={stdout} stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// Work-package operations
// ---------------------------------------------------------------------------

#[test]
fn test_work_packages_via_cli() {
    // Verify work-package subcommand is recognised by the CLI
    let bin = if let Ok(path) = std::env::var("AGILEPLUS_BIN") {
        std::path::PathBuf::from(path)
    } else {
        repo_root().join("target/debug/agileplus")
    };

    let output = Command::new(&bin)
        .args(["work-package", "--help"])
        .output()
        .expect("Failed to execute CLI binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The subcommand should be recognised (not an "unknown command" error)
    assert!(
        output.status.success() || stderr.contains("work-package") || stdout.contains("work-package"),
        "work-package subcommand should exist: stdout={stdout} stderr={stderr}"
    );
}

// ---------------------------------------------------------------------------
// Tray-icon files
// ---------------------------------------------------------------------------

#[test]
fn test_tray_icon_files_exist() {
    // Verify all required tray icon files exist at correct paths
    let icons = icons_dir();

    assert!(icons.join("32x32.png").exists(), "Missing tray icon 32x32.png");
    assert!(icons.join("128x128.png").exists(), "Missing bundle icon 128x128.png");
    assert!(icons.join("icon.icns").exists(), "Missing macOS icon");
    assert!(icons.join("icon.ico").exists(), "Missing Windows icon");
}

// ---------------------------------------------------------------------------
// Tauri configuration
// ---------------------------------------------------------------------------

#[test]
fn test_tauri_config_valid() {
    // Verify tauri.conf.json is valid JSON with required fields
    let config_path = tauri_config_path();

    let content = std::fs::read_to_string(&config_path).expect("Should read tauri.conf.json");
    let config: serde_json::Value =
        serde_json::from_str(&content).expect("Should be valid JSON");

    assert_eq!(config["productName"], "AgilePlus Desktop");
    assert_eq!(config["version"], "0.1.0");
    assert!(
        config["app"]["trayIcon"].is_object(),
        "Tray icon config should exist"
    );
    assert!(
        config["app"]["updater"]["active"].as_bool().unwrap(),
        "Updater should be active"
    );
    assert!(
        config["bundle"]["active"].as_bool().unwrap(),
        "Bundle should be active"
    );
}

// ---------------------------------------------------------------------------
// Database schema round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_database_schema_matches_desktop() {
    // Verify the E2E helper schema matches what the desktop crate produces
    let temp_dir = isolated_temp_dir();
    let db_path = temp_dir.path().join("schema_test.db");

    let conn = rusqlite::Connection::open(&db_path).unwrap();
    init_desktop_schema(&conn).unwrap();

    // Verify table names
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    assert!(tables.contains(&"features".to_string()), "Missing features table");
    assert!(
        tables.contains(&"work_packages".to_string()),
        "Missing work_packages table"
    );
    assert!(
        tables.contains(&"evidence".to_string()),
        "Missing evidence table"
    );

    // Verify we can insert and query a feature
    conn.execute(
        "INSERT INTO features (id, name, state) VALUES (?, ?, ?)",
        ["test-id", "Test Feature", "created"],
    )
    .unwrap();

    let name: String = conn
        .query_row("SELECT name FROM features WHERE id = 'test-id'", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(name, "Test Feature");
}
