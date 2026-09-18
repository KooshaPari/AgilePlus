// SPDX-License-Identifier: MIT OR Apache-2.0
//! Apply pending AgilePlus SQLite migrations to a database file.
//!
//! Usage:
//!   cargo run -p agileplus-sqlite --bin migrate -- [.agileplus/agileplus.db]
//!
//! Opening via `SqliteStorageAdapter::new` also applies migrations; this binary
//! exists so stale DBs (created by older CLIs that skipped 016+) can be healed
//! without running an unrelated subcommand.

use std::path::PathBuf;

use agileplus_sqlite::migrations::MigrationRunner;

fn main() -> anyhow::Result<()> {
    let db_path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".agileplus/agileplus.db"));

    if let Some(parent) = db_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    println!("Migrating database at: {}", db_path.display());

    let conn = rusqlite::Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    let before = applied_names(&conn).unwrap_or_default();
    let runner = MigrationRunner::new(&conn);
    runner
        .run_all()
        .map_err(|e| anyhow::anyhow!("migration failed: {e}"))?;
    let after = applied_names(&conn)?;

    let newly: Vec<_> = after
        .iter()
        .filter(|name| !before.contains(name))
        .cloned()
        .collect();

    if newly.is_empty() {
        println!("Already up to date ({} migrations applied).", after.len());
    } else {
        println!("Applied {} migration(s):", newly.len());
        for name in &newly {
            println!("  + {name}");
        }
        println!("Total applied: {}", after.len());
    }

    let has_backlog: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'backlog_items'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !has_backlog {
        anyhow::bail!(
            "backlog_items table still missing after migrate — 016_create_backlog_items did not apply"
        );
    }

    println!("OK: backlog_items present");
    Ok(())
}

fn applied_names(conn: &rusqlite::Connection) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM _migrations ORDER BY id")?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applied_names_lists_migrations_in_application_order() {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory");
        MigrationRunner::new(&conn).run_all().expect("migrate");

        let names = applied_names(&conn).expect("applied names");
        assert_eq!(
            names.first().map(String::as_str),
            Some("001_create_features")
        );
        assert!(
            names.contains(&"016_create_backlog_items".to_string()),
            "the backlog migration must stay applied: {names:?}"
        );
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), names.len(), "migration names must be unique");
        let has_backlog: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'backlog_items'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert!(has_backlog, "backlog_items must exist after run_all");
    }

    #[test]
    fn applied_names_errors_before_migrations_run() {
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory");
        assert!(
            applied_names(&conn).is_err(),
            "the _migrations table does not exist yet"
        );
    }
}
