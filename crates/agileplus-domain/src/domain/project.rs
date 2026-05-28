//! Project aggregate — top-level organisational unit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A project that owns modules, cycles, and features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
