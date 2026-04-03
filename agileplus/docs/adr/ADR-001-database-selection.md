# ADR-001: Database Selection

**Date**: 2026-04-02  
**Status**: Accepted  
**Deciders**: Agent  

## Context

AgilePlus requires a database for storing work items, specifications, and git-work correlations. The database choice impacts performance, complexity, and deployment options.

## Decision Drivers

- **Local-first**: Must work offline
- **Performance**: Fast queries for CLI responsiveness
- **Complexity**: Minimal setup and dependencies
- **Rust integration**: Native Rust support
- **Single binary**: Prefer embedded (no separate process)
- **ACID compliance**: Data integrity for work tracking

## Options Considered

### Option A: SQLite

**Pros**:
- Embedded (no separate process)
- ACID transactions
- Zero configuration
- Battle-tested (billions of deployments)
- libsql (Turso) adds modern features
- Rust ecosystem: `rusqlite`, `sqlx`

**Cons**:
- Not horizontally scalable
- Single writer at a time
- Limited concurrency

**Performance**: <1ms for typical queries

### Option B: PostgreSQL

**Pros**:
- Full-featured RDBMS
- Excellent concurrency
- Scalable

**Cons**:
- Requires separate process
- Complex deployment
- Overkill for local-first tool

**Verdict**: Rejected (not embedded)

### Option C: sled (Rust-native)

**Pros**:
- Pure Rust
- Modern design
- No C dependencies

**Cons**:
- Beta status
- Limited query capabilities
- Smaller ecosystem

**Verdict**: Too immature for production

## Decision

**Adopt SQLite with libsql (Turso) extensions**.

### Schema Design

```rust
// Core schema for AgilePlus

pub const SCHEMA: &str = r#"
-- Features (top-level work items)
CREATE TABLE IF NOT EXISTS features (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK (status IN ('proposed', 'in_progress', 'completed', 'archived')),
    priority INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    parent_id TEXT REFERENCES features(id),
    spec_path TEXT, -- Path to PRD file
    metadata TEXT -- JSON blob
);

-- Work Packages (implementation units)
CREATE TABLE IF NOT EXISTS work_packages (
    id TEXT PRIMARY KEY,
    feature_id TEXT NOT NULL REFERENCES features(id),
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL CHECK (status IN ('todo', 'in_progress', 'review', 'done')),
    assigned_to TEXT,
    estimated_hours INTEGER,
    actual_hours INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    spec_path TEXT
);

-- Git Correlations (work <-> code linkage)
CREATE TABLE IF NOT EXISTS git_correlations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    feature_id TEXT REFERENCES features(id),
    work_package_id TEXT REFERENCES work_packages(id),
    commit_hash TEXT NOT NULL,
    commit_message TEXT,
    commit_author TEXT,
    commit_date DATETIME,
    files_changed TEXT, -- JSON array
    lines_added INTEGER,
    lines_deleted INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Specification Cache (for search/indexing)
CREATE TABLE IF NOT EXISTS spec_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT UNIQUE NOT NULL,
    content_hash TEXT NOT NULL,
    title TEXT,
    status TEXT,
    parsed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    content TEXT -- Full text for search
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_features_status ON features(status);
CREATE INDEX IF NOT EXISTS idx_features_parent ON features(parent_id);
CREATE INDEX IF NOT EXISTS idx_work_packages_feature ON work_packages(feature_id);
CREATE INDEX IF NOT EXISTS idx_work_packages_status ON work_packages(status);
CREATE INDEX IF NOT EXISTS idx_git_correlations_commit ON git_correlations(commit_hash);
CREATE INDEX IF NOT EXISTS idx_git_correlations_feature ON git_correlations(feature_id);

-- Full-text search using FTS5
CREATE VIRTUAL TABLE IF NOT EXISTS spec_search USING fts5(
    title,
    content,
    content_rowid=id,
    content=spec_cache
);
"#;
```

### Rust Implementation

```rust
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::path::Path;
use anyhow::Result;

pub struct Database {
    pool: sqlx::SqlitePool,
}

impl Database {
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        
        // Run migrations
        sqlx::query(SCHEMA)
            .execute(&pool)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn create_feature(&self, feature: &Feature) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO features (id, title, description, status, priority, spec_path)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            feature.id,
            feature.title,
            feature.description,
            feature.status.to_string(),
            feature.priority,
            feature.spec_path
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn list_features(&self, status: Option<Status>) -> Result<Vec<Feature>> {
        let features = match status {
            Some(s) => {
                sqlx::query_as!(
                    Feature,
                    r#"SELECT * FROM features WHERE status = ?1 ORDER BY priority DESC, created_at DESC"#,
                    s.to_string()
                )
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Feature,
                    r#"SELECT * FROM features ORDER BY priority DESC, created_at DESC"#
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        
        Ok(features)
    }
    
    pub async fn correlate_commit(&self, correlation: &GitCorrelation) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO git_correlations 
            (feature_id, work_package_id, commit_hash, commit_message, 
             commit_author, commit_date, files_changed, lines_added, lines_deleted)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            correlation.feature_id,
            correlation.work_package_id,
            correlation.commit_hash,
            correlation.commit_message,
            correlation.commit_author,
            correlation.commit_date,
            correlation.files_changed,
            correlation.lines_added,
            correlation.lines_deleted
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Cargo.toml

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "migrate", "chrono", "uuid"] }
libsql-client = { version = "0.31", optional = true } # For remote sync

[features]
default = ["local"]
local = []
remote = ["libsql-client"]
```

## Consequences

### Positive
- **Zero setup**: Single binary deployment
- **Fast**: <1ms queries
- **Reliable**: ACID, battle-tested
- **Rust-native**: sqlx provides compile-time checked queries
- **Future-proof**: Can add libsql for sync later

### Negative
- **Not distributed**: Single-node only
- **Write contention**: Single writer
- **Limited analytics**: Not for complex reporting

### Neutral
- **Size**: ~2MB binary overhead (acceptable)

## Sync Strategy (Future)

When multi-device sync is needed:

```rust
// Option 1: Git-based sync
// - SQLite dump to JSON
// - Commit to git
// - Merge on pull

// Option 2: libsql (Turso)
// - Remote database
// - Local replica
// - Automatic sync

// Option 3: Litestream (S3 backup)
// - Continuous backup
// - Point-in-time restore
```

## References

- SQLite: https://www.sqlite.org/
- libsql: https://github.com/tursodatabase/libsql
- sqlx: https://github.com/launchbadge/sqlx
- AgilePlus SOTA Research: `docs/research/AGILE_TOOLS_SOTA.md`

---

*This ADR will be updated as implementation progresses*
