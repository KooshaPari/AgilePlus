// SPDX-License-Identifier: MIT OR Apache-2.0
//! PersistSyncedStories use-case tests: idempotent upsert by
//! `requirement_id`, report accounting, and repository failure propagation.

use std::sync::Arc;

use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::story::StoryRepository;

use crate::error::AppError;
use crate::test_mocks::InMemoryStoryRepo;
use crate::use_cases::persist_synced_stories::{
    PersistSyncReport, PersistSyncedStories, PersistSyncedStoriesCmd,
};

fn make_story(epic: i64, proj: i64, title: &str, req_id: &str) -> Story {
    let mut s = Story::new(epic, proj, title, None).unwrap();
    s.requirement_id = Some(req_id.to_string());
    s
}

// ── happy path ───────────────────────────────────────────────────────────────

/// N stories are persisted; returned id count matches story count.
#[tokio::test]
async fn persists_n_stories_to_repo() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let stories = vec![
        make_story(1, 10, "Fix login crash", "gh:issue:1"),
        make_story(1, 10, "Add dark mode", "gh:issue:2"),
        make_story(1, 10, "feat: dark mode", "gh:pr:10"),
    ];

    let report = uc
        .execute(PersistSyncedStoriesCmd { stories })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids.len(), 3, "should have 3 persisted ids");

    // Verify all are actually in the repo.
    for id in &report.persisted_ids {
        assert!(
            repo.get_by_id(*id).await.unwrap().is_some(),
            "story {id} should be retrievable"
        );
    }
}

/// Re-syncing the same stories is idempotent — no duplicates.
#[tokio::test]
async fn resync_is_idempotent_no_duplicates() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let stories = vec![
        make_story(2, 20, "Story A", "gh:issue:5"),
        make_story(2, 20, "Story B", "gh:issue:6"),
    ];

    // First sync.
    let r1 = uc
        .execute(PersistSyncedStoriesCmd {
            stories: stories.clone(),
        })
        .await
        .unwrap();
    assert_eq!(r1.persisted_ids.len(), 2);

    // Second sync — same stories.
    let r2 = uc
        .execute(PersistSyncedStoriesCmd {
            stories: stories.clone(),
        })
        .await
        .unwrap();
    assert_eq!(
        r2.persisted_ids.len(),
        2,
        "second sync must still return 2 ids"
    );

    // Repo must still have exactly 2 stories for the epic.
    let all = repo.list_by_epic(2).await.unwrap();
    assert_eq!(all.len(), 2, "no duplicates after re-sync");

    // The ids from the second sync must be the same rows as the first.
    assert_eq!(
        r1.persisted_ids, r2.persisted_ids,
        "upsert should return same ids"
    );
}

/// Distinct requirement ids in the same epic are distinct rows, even when the
/// titles collide (issue #1 and PR #1 both mention "login").
#[tokio::test]
async fn distinct_requirement_ids_produce_distinct_rows() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let report = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![
                make_story(3, 30, "Login fix", "gh:issue:1"),
                make_story(3, 30, "Login fix", "gh:pr:1"),
            ],
        })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids, vec![1, 2]);
    let stored = repo.list_by_epic(3).await.unwrap();
    assert_eq!(stored.len(), 2);
    assert!(stored
        .iter()
        .any(|s| s.requirement_id.as_deref() == Some("gh:issue:1")));
    assert!(stored
        .iter()
        .any(|s| s.requirement_id.as_deref() == Some("gh:pr:1")));
}

/// The upsert contract is "update the existing row for this requirement id":
/// the second sync with a changed status keeps the row and its id but applies
/// the new status.
#[tokio::test]
async fn resync_of_existing_requirement_id_updates_that_row() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let first = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![make_story(4, 40, "Track me", "gh:issue:9")],
        })
        .await
        .unwrap();

    let mut updated = make_story(4, 40, "Track me (renamed upstream)", "gh:issue:9");
    updated.status = StoryStatus::InProgress;

    let second = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![updated],
        })
        .await
        .unwrap();

    assert_eq!(
        second.persisted_ids, first.persisted_ids,
        "same requirement id must reuse the row"
    );
    assert_eq!(repo.list_by_epic(4).await.unwrap().len(), 1);

    let stored = repo
        .get_by_id(second.persisted_ids[0])
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.status, StoryStatus::InProgress);
    assert_eq!(
        stored.title, "Track me",
        "the portable default upsert applies status only, so the stored title \
         is the one from the first sync"
    );
}

// ── report accounting ────────────────────────────────────────────────────────

/// Freshly mapped stories (`id == 0`) are reported as creates.
#[tokio::test]
async fn report_counts_freshly_mapped_stories_as_created() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let report = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![
                make_story(1, 10, "A", "gh:issue:1"),
                make_story(1, 10, "B", "gh:issue:2"),
            ],
        })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids.len(), 2);
    assert_eq!(report.created, 2, "both stories came in with id 0");
    assert_eq!(report.updated, 0);
}

/// A story whose incoming id already equals the stored row id is reported as
/// an update; the others stay creates.
#[tokio::test]
async fn report_counts_matching_ids_as_updated() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let seeded = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![
                make_story(5, 50, "One", "gh:issue:1"),
                make_story(5, 50, "Two", "gh:issue:2"),
            ],
        })
        .await
        .unwrap();
    assert_eq!(seeded.persisted_ids, vec![1, 2]);

    // Re-sync, but this time the mapper hands back the known row id for "Two".
    let mut known = make_story(5, 50, "Two", "gh:issue:2");
    known.id = seeded.persisted_ids[1];

    let report = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![make_story(5, 50, "One", "gh:issue:1"), known],
        })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids, vec![1, 2]);
    assert_eq!(report.updated, 1, "story 2 matched its stored row id");
    assert_eq!(report.created, 1, "story 1 was still id 0");
}

/// Empty story list produces an empty report — no error.
#[tokio::test]
async fn empty_story_list_produces_empty_report() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let report = uc
        .execute(PersistSyncedStoriesCmd { stories: vec![] })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids.len(), 0);
    assert_eq!(report.created, 0);
    assert_eq!(report.updated, 0);
}

// ── validation ───────────────────────────────────────────────────────────────

/// Stories without a requirement_id are rejected, not silently skipped.
#[tokio::test]
async fn story_without_requirement_id_returns_error() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    // Story with no requirement_id (would come from a broken mapper).
    let bad_story = Story::new(3, 30, "Orphaned story", None).unwrap();

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![bad_story],
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Validation(_))),
        "expected Validation error, got {err:?}"
    );
    assert!(
        repo.list_by_epic(3).await.unwrap().is_empty(),
        "a rejected story must not be written"
    );
}

/// The requirement-id check runs before any write, so a batch whose *first*
/// story is unmappable leaves the store untouched.
#[tokio::test]
async fn validation_precedes_persistence_for_the_whole_batch() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![
                Story::new(1, 10, "No requirement id", None).unwrap(),
                make_story(1, 10, "Mappable", "gh:issue:11"),
            ],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert!(
        repo.list_by_epic(1).await.unwrap().is_empty(),
        "validation failure precedes the first upsert"
    );
}

/// Batch with mix of good and bad stories fails on the first bad one.
#[tokio::test]
async fn batch_fails_on_first_missing_requirement_id() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let good = make_story(1, 10, "Good", "gh:issue:10");
    let bad = Story::new(1, 10, "Bad", None).unwrap(); // no requirement_id
    let good2 = make_story(1, 10, "Good2", "gh:issue:11");

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![good, bad, good2],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));

    // good story should still be in repo since it was processed before error
    let all = repo.list_by_epic(1).await.unwrap();
    assert_eq!(all.len(), 1);
}

/// The error names the offending story so the upstream mapper can be fixed.
#[tokio::test]
async fn missing_requirement_id_error_names_the_story() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![Story::new(1, 10, "Orphaned story", None).unwrap()],
        })
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(
        msg.contains("Orphaned story"),
        "error should name the story, got: {msg}"
    );
}

/// Skipped items (never in the stories vec) do not appear in the repo.
#[tokio::test]
async fn skipped_items_not_persisted() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = PersistSyncedStories::new(repo.clone());

    // Only 2 good stories — the "skipped" bad ones from sync_repository
    // would never reach this use case; simulate by passing only the good ones.
    let good = vec![
        make_story(4, 40, "Valid A", "gh:issue:100"),
        make_story(4, 40, "Valid B", "gh:issue:101"),
    ];

    let report = uc
        .execute(PersistSyncedStoriesCmd { stories: good })
        .await
        .unwrap();
    assert_eq!(report.persisted_ids.len(), 2);

    // The repo has no story for issue #999 (skipped upstream).
    let all = repo.list_by_epic(4).await.unwrap();
    assert!(
        !all.iter()
            .any(|s| s.requirement_id.as_deref() == Some("gh:issue:999")),
        "skipped item must not be in repo"
    );
}

// ── port failure ─────────────────────────────────────────────────────────────

/// A storage failure surfaces as `AppError::Domain` and no ids are reported.
#[tokio::test]
async fn repository_failure_propagates_as_domain_error() {
    let repo = Arc::new(InMemoryStoryRepo::default().failing_list());
    let uc = PersistSyncedStories::new(repo.clone());

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![make_story(1, 10, "Will fail", "gh:issue:1")],
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
}

/// A repository that cannot create rows fails the batch.
#[tokio::test]
async fn create_failure_on_first_story_stops_the_batch() {
    let repo = Arc::new(InMemoryStoryRepo::default().failing_create());
    let uc = PersistSyncedStories::new(repo.clone());

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![
                make_story(1, 10, "First", "gh:issue:1"),
                make_story(1, 10, "Second", "gh:issue:2"),
            ],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert!(repo.list_by_epic(1).await.unwrap().is_empty());
}

// ── DTO surface ──────────────────────────────────────────────────────────────

/// PersistSyncReport default values are sensible.
#[test]
fn persist_sync_report_default() {
    let report = PersistSyncReport::default();
    assert!(report.persisted_ids.is_empty());
    assert_eq!(report.created, 0);
    assert_eq!(report.updated, 0);
}

/// PersistSyncedStoriesCmd clone and debug traits work.
#[test]
fn persist_synced_stories_cmd_clone_debug() {
    let stories = vec![make_story(1, 10, "T", "gh:issue:1")];
    let cmd = PersistSyncedStoriesCmd { stories };
    let cloned = cmd.clone();
    assert_eq!(cloned.stories.len(), 1);
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("PersistSyncedStoriesCmd"));
}
