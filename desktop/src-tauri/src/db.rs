use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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

/// Open the CLI's .agileplus/agileplus.db (read-only safe, WAL mode for writes).
pub fn open_project_db(project_root: &Path) -> Result<Connection, String> {
    let db_path = project_root.join(".agileplus").join("agileplus.db");
    if !db_path.exists() {
        return Err(format!(
            "No .agileplus/agileplus.db found in {}",
            project_root.display()
        ));
    }
    let conn =
        Connection::open(&db_path).map_err(|e| format!("Failed to open database: {e}"))?;

    // Enable WAL mode for concurrent reads
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("WAL pragma failed: {e}"))?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("FK pragma failed: {e}"))?;

    Ok(conn)
}
