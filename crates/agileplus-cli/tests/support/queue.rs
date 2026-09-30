// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared fixtures for the `agileplus queue` integration tests.
//!
//! Test-side only; nothing here is compiled into the production binary.
use std::path::PathBuf;

use agileplus_cli::commands::queue::{QueueAction, QueueArgs, run_queue};
use agileplus_domain::domain::backlog::{BacklogFilters, BacklogSort, BacklogStatus};
use agileplus_domain::ports::ContentStoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

// ── helpers ──────────────────────────────────────────────────────────────────

pub fn filters() -> BacklogFilters {
    BacklogFilters {
        intent: None,
        status: None,
        priority: None,
        feature_slug: None,
        source: None,
        sort: BacklogSort::Priority,
        limit: None,
    }
}

#[derive(Default)]
pub struct AddSpec<'a> {
    pub text: Option<&'a str>,
    pub description: &'a str,
    pub intent: Option<&'a str>,
    pub priority: Option<&'a str>,
    pub tags: Vec<&'a str>,
    pub source: &'a str,
    pub feature_slug: Option<&'a str>,
    pub from_file: Option<PathBuf>,
}

impl<'a> AddSpec<'a> {
    pub fn text(self, text: &'a str) -> Self {
        Self {
            text: Some(text),
            ..self
        }
    }

    pub fn build(self) -> QueueArgs {
        QueueArgs {
            action: QueueAction::Add {
                text: self.text.map(str::to_string),
                description: self.description.to_string(),
                r#type: self.intent.map(str::to_string),
                priority: self.priority.map(str::to_string),
                tags: self.tags.iter().map(|s| s.to_string()).collect(),
                source: if self.source.is_empty() {
                    "cli".to_string()
                } else {
                    self.source.to_string()
                },
                feature_slug: self.feature_slug.map(str::to_string),
                from_file: self.from_file,
            },
        }
    }
}

pub struct ListSpec<'a> {
    pub intent: Option<&'a str>,
    pub status: Option<&'a str>,
    pub priority: Option<&'a str>,
    pub feature_slug: Option<&'a str>,
    pub source: Option<&'a str>,
    pub sort: &'a str,
    pub output: &'a str,
    pub limit: usize,
}

/// Mirrors the clap `default_value`s on `QueueAction::List` so a spec that
/// does not care about a flag still exercises the same defaults the binary
/// would pass in.
impl Default for ListSpec<'_> {
    fn default() -> Self {
        Self {
            intent: None,
            status: None,
            priority: None,
            feature_slug: None,
            source: None,
            sort: "priority",
            output: "table",
            limit: 20,
        }
    }
}

impl<'a> ListSpec<'a> {
    pub fn build(self) -> QueueArgs {
        QueueArgs {
            action: QueueAction::List {
                r#type: self.intent.map(str::to_string),
                status: self.status.map(str::to_string),
                priority: self.priority.map(str::to_string),
                feature_slug: self.feature_slug.map(str::to_string),
                source: self.source.map(str::to_string),
                sort: self.sort.to_string(),
                output: self.output.to_string(),
                limit: self.limit,
            },
        }
    }
}

pub fn show(id: i64, output: &str) -> QueueArgs {
    QueueArgs {
        action: QueueAction::Show {
            id,
            output: output.to_string(),
        },
    }
}

pub fn pop(count: usize, output: &str) -> QueueArgs {
    QueueArgs {
        action: QueueAction::Pop {
            count,
            output: output.to_string(),
        },
    }
}

pub async fn add_item(storage: &SqliteStorageAdapter, spec: AddSpec<'_>) -> anyhow::Result<()> {
    run_queue(spec.build(), storage).await
}

pub async fn titles(storage: &SqliteStorageAdapter) -> Vec<String> {
    let mut items = storage.list_backlog_items(&filters()).await.unwrap();
    items.sort_by(|a, b| a.title.cmp(&b.title));
    items.iter().map(|i| i.title.clone()).collect()
}

/// Titles of the rows currently in `status`.
pub async fn titles_with(storage: &SqliteStorageAdapter, status: BacklogStatus) -> Vec<String> {
    let mut items = storage
        .list_backlog_items(&BacklogFilters {
            status: Some(status),
            ..filters()
        })
        .await
        .unwrap();
    items.sort_by(|a, b| a.title.cmp(&b.title));
    items.iter().map(|i| i.title.clone()).collect()
}

pub fn write_ndjson(dir: &std::path::Path, name: &str, contents: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, contents).expect("write ndjson fixture");
    path
}

// ── add ──────────────────────────────────────────────────────────────────────
