use rusqlite::Connection;
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
    repo_path: Mutex<String>,
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
    /// Returns a MutexGuard that the caller uses to query via `as_ref()`.
    pub fn db_connection(&self) -> Result<std::sync::MutexGuard<'_, Option<Connection>>, String> {
        self.db.0.lock().map_err(|e| e.to_string())
    }
}

pub fn initialize_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS features (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'created',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS work_packages (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            state TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features(id)
        );

        CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            feature_id TEXT NOT NULL,
            work_package_id TEXT,
            evidence_type TEXT NOT NULL,
            content TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (feature_id) REFERENCES features(id),
            FOREIGN KEY (work_package_id) REFERENCES work_packages(id)
        );
        ",
    )?;
    Ok(())
}
