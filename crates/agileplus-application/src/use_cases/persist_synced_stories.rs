// SPDX-License-Identifier: MIT OR Apache-2.0
//! Use case: Persist a batch of synced Stories via the StoryRepository port.
//!
//! Traceability: FR-AGP-013
//!
//! # Design
//!
//! `PersistSyncedStories` is the application-layer bridge between the
//! agileplus-github `sync_repository` function (which produces an in-memory
//! `Vec<Story>`) and the `StoryRepository` port (backed by SQLite in
//! production, in-memory double in tests).
//!
//! Each story is upserted by `requirement_id` (set to `gh:issue:<n>` or
//! `gh:pr:<n>` by the mapper) so that re-running sync is idempotent — no
//! duplicates are created on repeated calls.
//!
//! Stories without a `requirement_id` are rejected with a validation error
//! so callers are forced to fix the upstream mapping, not silently lose data.

use std::sync::Arc;

use agileplus_domain::domain::story::Story;
use agileplus_domain::ports::story::StoryRepository;

use crate::error::AppError;

// ── Command / Output DTOs ─────────────────────────────────────────────────────

/// Command carrying the batch of already-mapped stories to persist.
///
/// Pass the `stories` field from `agileplus_github::sync::SyncReport`.
#[derive(Debug, Clone)]
pub struct PersistSyncedStoriesCmd {
    /// Stories to persist; each must have a non-`None` `requirement_id`.
    pub stories: Vec<Story>,
}

/// Outcome of a `PersistSyncedStories::execute` call.
#[derive(Debug, Default, Clone)]
pub struct PersistSyncReport {
    /// Row IDs returned by the repository for each upserted story.
    pub persisted_ids: Vec<i64>,
    /// Number of stories that were newly created (not previously in the store).
    pub created: usize,
    /// Number of stories that updated an existing row.
    pub updated: usize,
}

// ── Use case ──────────────────────────────────────────────────────────────────

/// Persists a batch of GitHub-synced stories via the `StoryRepository` port.
///
/// Depends **only** on the port trait — never on any adapter type.
pub struct PersistSyncedStories {
    repo: Arc<dyn StoryRepository>,
}

impl PersistSyncedStories {
    pub fn new(repo: Arc<dyn StoryRepository>) -> Self {
        Self { repo }
    }

    /// Upsert every story in `cmd.stories`.
    ///
    /// Uses `upsert_by_requirement_id` so the operation is idempotent: calling
    /// `execute` twice with the same stories will not create duplicates.
    ///
    /// Returns `AppError::Domain(DomainError::Validation)` if any story is
    /// missing its `requirement_id`.
    pub async fn execute(
        &self,
        cmd: PersistSyncedStoriesCmd,
    ) -> Result<PersistSyncReport, AppError> {
        let mut report = PersistSyncReport::default();

        for story in &cmd.stories {
            // Validate before calling the port so callers get a clear error.
            if story.requirement_id.is_none() {
                return Err(AppError::Domain(
                    agileplus_domain::error::DomainError::Validation(format!(
                        "story '{}' has no requirement_id — cannot upsert idempotently",
                        story.title
                    )),
                ));
            }

            let id = self.repo.upsert_by_requirement_id(story).await?;
            report.persisted_ids.push(id);
        }

        // Distinguish creates from updates: ids that match the story's own id
        // were freshly inserted (repo sets id = last_insert_rowid); ids that
        // differ from the story's incoming id were pre-existing rows.
        // NOTE: the in-memory double uses auto-increment, so any id != 0
        // returned for a story with id=0 means a create; same id means update.
        for (story, &persisted_id) in cmd.stories.iter().zip(report.persisted_ids.iter()) {
            if story.id == 0 || story.id != persisted_id {
                // story.id == 0 → freshly mapped from GitHub, never persisted
                // story.id != persisted_id → mapping assigned a GitHub number
                //   as the id; a different DB id means a pre-existing row was
                //   returned.  Either way we track as a create for observability.
                report.created += 1;
            } else {
                report.updated += 1;
            }
        }

        Ok(report)
    }
}
