use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    pub fn new(slug: &str, name: &str, description: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: slug.into(),
            name: name.into(),
            description: description.into(),
            created_at: now,
            updated_at: now,
        }
    }
}
