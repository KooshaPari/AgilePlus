//! End-to-end tests for the crate's standalone seeding binaries.
//!
//! These run the real `seed_db` / `seed_requirements` executables against a
//! throwaway database file in the system temp directory, then assert the
//! observable result: a migrated database holding one epic per initiative plus
//! its catalog stories. No operator data files are ever touched.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Create a temp directory unique to this process and label.
fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "agileplus-sqlite-{label}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

fn run_binary(bin: &str, args: &[&Path]) -> std::process::Output {
    let mut cmd = Command::new(bin);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.output().expect("spawn binary")
}

fn stdout_of(output: &std::process::Output) -> String {
    assert!(
        output.status.success(),
        "binary failed: status={:?} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap_or_else(|e| panic!("count {table}: {e}"))
}

#[test]
fn seed_db_binary_migrates_and_seeds_four_initiatives() {
    let dir = unique_temp_dir("seed-db-bin");
    let db = dir.join("agileplus.db");

    let stdout = stdout_of(&run_binary(env!("CARGO_BIN_EXE_seed_db"), &[&db]));
    assert!(
        stdout.contains("Migrations applied."),
        "seeding must apply migrations first: {stdout}"
    );
    assert!(
        stdout.contains("Done: 4 epic(s)"),
        "seed_db seeds exactly four initiatives: {stdout}"
    );

    let conn = rusqlite::Connection::open(&db).expect("open seeded db");
    assert_eq!(count(&conn, "epics"), 4, "one epic per initiative");
    let stories = count(&conn, "stories");
    assert!(stories > 0, "catalog FR/NFR entries must become stories");
    assert_eq!(
        count(&conn, "projects"),
        4,
        "each initiative gets one project row"
    );

    // Re-running against the same file must be idempotent: same rows, no dupes.
    let second = stdout_of(&run_binary(env!("CARGO_BIN_EXE_seed_db"), &[&db]));
    assert!(second.contains("Done: 4 epic(s)"), "second run: {second}");
    assert_eq!(count(&conn, "epics"), 4, "re-seed must not duplicate epics");
    assert_eq!(
        count(&conn, "stories"),
        stories,
        "re-seed must not duplicate stories"
    );

    drop(conn);
    cleanup(&dir);
}

#[test]
fn seed_requirements_binary_honours_db_flag_and_seeds_six_initiatives() {
    let dir = unique_temp_dir("seed-req-bin");
    let db = dir.join("requirements.db");
    // The --db flag is honoured even when other arguments precede it.
    let flag = PathBuf::from("--db");

    let stdout = stdout_of(&run_binary(
        env!("CARGO_BIN_EXE_seed_requirements"),
        &[PathBuf::from("--verbose").as_path(), &flag, &db],
    ));
    assert!(
        stdout.contains("Opening database:"),
        "the binary must report the database it opens: {stdout}"
    );
    assert!(
        stdout.contains("Seeded 6 epic(s)"),
        "seed_requirements ingests six catalogs: {stdout}"
    );
    assert!(
        stdout.contains("epic_id=") && stdout.contains("stories="),
        "per-initiative summary lines must be printed: {stdout}"
    );

    let conn = rusqlite::Connection::open(&db).expect("open seeded db");
    assert_eq!(count(&conn, "epics"), 6, "one epic per catalog");
    assert!(count(&conn, "stories") > 0, "catalogs must yield stories");
    assert_eq!(count(&conn, "projects"), 6);

    drop(conn);
    cleanup(&dir);
}
