use rusqlite::Connection;
use std::sync::Mutex;

pub struct DatabaseState(pub Mutex<Option<Connection>>);

impl Default for DatabaseState {
    fn default() -> Self {
        Self(Mutex::new(None))
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
