use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Desktop clients may project local state directly, but they must not mutate
/// the AgilePlus database outside the canonical application/service boundary.
pub const DIRECT_MUTATION_DISABLED: &str =
    "Direct desktop database mutation is disabled. Use the AgilePlus CLI, MCP, or device service so governance, evidence, audit, and acceptance rules are enforced.";

pub fn direct_mutation_disabled(operation: &str) -> String {
    format!("{DIRECT_MUTATION_DISABLED} Operation: {operation}.")
}

pub struct DatabaseState(pub Mutex<Option<Connection>>);

impl Default for DatabaseState {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

/// Application state combining the database connection and the active repo path.
pub struct AppState {
    pub db: DatabaseState,
    pub repo_path: Mutex<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: DatabaseState::default(),
            repo_path: Mutex::new(String::new()),
        }
    }
}

impl AppState {
    pub fn new(db: DatabaseState, repo_path: String) -> Self {
        Self {
            db,
            repo_path: Mutex::new(repo_path),
        }
    }

    /// Return a clone of the current repo path.
    pub fn repo_path(&self) -> String {
        self.repo_path.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Set the repo path at runtime (e.g. when user selects a new repo).
    pub fn set_repo_path(&self, path: String) {
        if let Ok(mut guard) = self.repo_path.lock() {
            *guard = path;
        }
    }

    /// Acquire a lock on the database connection.
    pub fn db_connection(&self) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, String> {
        self.db.0.lock().map_err(|e| e.to_string())
    }
}

/// Walk up from `start` looking for `.agileplus/agileplus.db`.
/// Returns the path to the project root if found.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let db_candidate = current.join(".agileplus").join("agileplus.db");
        if db_candidate.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Open the CLI's `.agileplus/agileplus.db` as a query-only projection.
///
/// All mutations must flow through the AgilePlus application/service boundary.
/// Read-only open flags plus SQLite's `query_only` pragma make accidental writes
/// fail closed even if a future desktop command attempts raw SQL.
pub fn open_project_db(project_root: &Path) -> Result<Connection, String> {
    let db_path = project_root.join(".agileplus").join("agileplus.db");
    if !db_path.exists() {
        return Err(format!(
            "No .agileplus/agileplus.db found in {}",
            project_root.display()
        ));
    }

    let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open database read-only: {e}"))?;

    conn.execute_batch("PRAGMA query_only=ON; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to configure query-only database projection: {e}"))?;

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "agileplus-desktop-db-test-{}-{suffix}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(path.join(".agileplus")).expect("create project database dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn direct_mutation_error_names_the_authoritative_paths() {
        let error = direct_mutation_disabled("create_evidence");
        assert!(error.contains("Direct desktop database mutation is disabled"));
        assert!(error.contains("CLI, MCP, or device service"));
        assert!(error.contains("create_evidence"));
    }

    #[test]
    fn project_database_connection_is_query_only() {
        let scratch = Scratch::new();
        let db_path = scratch.path().join(".agileplus").join("agileplus.db");
        let setup = Connection::open(&db_path).expect("create fixture database");
        setup
            .execute_batch(
                "CREATE TABLE sentinel (value TEXT NOT NULL);\n                 INSERT INTO sentinel (value) VALUES ('original');",
            )
            .expect("seed fixture database");
        drop(setup);

        let conn = open_project_db(scratch.path()).expect("open query-only projection");
        let query_only: i64 = conn
            .query_row("PRAGMA query_only", [], |row| row.get(0))
            .expect("read query_only pragma");
        assert_eq!(query_only, 1);

        assert!(
            conn.execute("INSERT INTO sentinel (value) VALUES ('blocked')", [])
                .is_err(),
            "desktop projection must reject raw writes"
        );
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sentinel", [], |row| row.get(0))
            .expect("read fixture row count");
        assert_eq!(count, 1, "failed write must leave the database unchanged");
    }
}
