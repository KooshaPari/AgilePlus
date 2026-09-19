//! Public-boundary tests for the domain use cases in `agileplus-application`.
//!
//! Covers `CreateStory`, `TransitionStory`, `CreateEpic`, and
//! `PersistSyncedStories` through the same surface a downstream caller uses:
//! `use_cases::*`, the `dto::*` command types, and the domain repository /
//! event-publisher ports. The in-crate unit tests rely on `crate::test_mocks`
//! (cfg(test)-only), so this file wires its own doubles and asserts what
//! actually happened: rows persisted, event payloads, and error propagation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use agileplus_application::dto::{CreateEpicCmd, CreateStoryCmd, TransitionStoryCmd};
use agileplus_application::error::AppError;
use agileplus_application::events::{DomainEvent, DomainEventPublisher};
use agileplus_application::use_cases::create_epic::CreateEpic;
use agileplus_application::use_cases::create_story::CreateStory;
use agileplus_application::use_cases::persist_synced_stories::{
    PersistSyncedStories, PersistSyncedStoriesCmd,
};
use agileplus_application::use_cases::transition_story::TransitionStory;

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::epic::EpicRepository;
use agileplus_domain::ports::story::StoryRepository;

// ── Story repository double ──────────────────────────────────────────────────

#[derive(Default)]
struct StoryStore {
    stories: Mutex<Vec<Story>>,
    next_id: Mutex<i64>,
    fail_create: AtomicBool,
}

impl StoryStore {
    fn with_story(self, story: Story) -> Self {
        *self.next_id.lock().unwrap() = story.id;
        self.stories.lock().unwrap().push(story);
        self
    }

    fn failing_create(self) -> Self {
        self.fail_create.store(true, Ordering::SeqCst);
        self
    }

    fn count(&self) -> usize {
        self.stories.lock().unwrap().len()
    }

    fn status_of(&self, id: i64) -> Option<StoryStatus> {
        self.stories
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.status)
    }
}

#[async_trait]
impl StoryRepository for StoryStore {
    async fn create(&self, story: &Story) -> Result<i64, DomainError> {
        if self.fail_create.load(Ordering::SeqCst) {
            return Err(DomainError::Storage("story store offline".into()));
        }
        let mut next = self.next_id.lock().unwrap();
        *next += 1;
        let id = *next;
        let mut stored = story.clone();
        stored.id = id;
        self.stories.lock().unwrap().push(stored);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError> {
        Ok(self
            .stories
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == id)
            .cloned())
    }

    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        let mut stories = self.stories.lock().unwrap();
        match stories.iter_mut().find(|s| s.id == id) {
            Some(s) => {
                s.status = status;
                Ok(())
            }
            None => Err(DomainError::NotFound(id.to_string())),
        }
    }

    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(self
            .stories
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.epic_id == epic_id)
            .cloned()
            .collect())
    }
}

// ── Epic repository double ───────────────────────────────────────────────────

#[derive(Default)]
struct EpicStore {
    epics: Mutex<Vec<Epic>>,
    next_id: Mutex<i64>,
}

#[async_trait]
impl EpicRepository for EpicStore {
    async fn create(&self, epic: &Epic) -> Result<i64, DomainError> {
        let mut next = self.next_id.lock().unwrap();
        *next += 1;
        let id = *next;
        let mut stored = epic.clone();
        stored.id = id;
        self.epics.lock().unwrap().push(stored);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError> {
        Ok(self
            .epics
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.id == id)
            .cloned())
    }

    async fn update_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError> {
        let mut epics = self.epics.lock().unwrap();
        match epics.iter_mut().find(|e| e.id == id) {
            Some(e) => {
                e.status = status;
                Ok(())
            }
            None => Err(DomainError::NotFound(id.to_string())),
        }
    }

    async fn list_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(self
            .epics
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.project_id == project_id)
            .cloned()
            .collect())
    }
}

// ── Publisher double ─────────────────────────────────────────────────────────

#[derive(Default)]
struct Recorder {
    events: Mutex<Vec<DomainEvent>>,
}

impl Recorder {
    fn events(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl DomainEventPublisher for Recorder {
    fn publish(&self, event: DomainEvent) -> Result<(), DomainError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

fn story_with_requirement(epic: i64, req: &str) -> Story {
    let mut story = Story::new(epic, 1, "Synced story", Some(3)).unwrap();
    story.requirement_id = Some(req.to_string());
    story
}

/// A story that already has a persisted row id (as if loaded from the store).
fn persisted_story(id: i64) -> Story {
    let mut story = Story::new(1, 1, "Login", Some(3)).unwrap();
    story.id = id;
    story
}

// ── CreateStory ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_story_persists_then_publishes_story_created() {
    let repo = Arc::new(StoryStore::default());
    let publisher = Arc::new(Recorder::default());
    let uc = CreateStory::new(repo.clone(), publisher.clone());

    let out = uc
        .execute(CreateStoryCmd {
            epic_id: 4,
            project_id: 9,
            title: "  Login works  ".into(),
            points: Some(5),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);
    assert_eq!(
        out.story.title, "Login works",
        "title is trimmed by the domain"
    );
    assert_eq!(out.story.status, StoryStatus::Todo);
    assert_eq!(repo.count(), 1);

    let events = publisher.events();
    assert_eq!(events.len(), 1, "exactly one event is published");
    assert!(
        matches!(
            &events[0],
            DomainEvent::StoryCreated { id: 1, epic_id: 4, title } if title == "Login works"
        ),
        "unexpected event: {events:?}"
    );
}

#[tokio::test]
async fn create_story_rejects_an_empty_title_without_persisting() {
    let repo = Arc::new(StoryStore::default());
    let publisher = Arc::new(Recorder::default());
    let uc = CreateStory::new(repo.clone(), publisher.clone());

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 1,
            title: "   ".into(),
            points: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert_eq!(repo.count(), 0);
    assert!(publisher.events().is_empty());
}

#[tokio::test]
async fn create_story_surfaces_a_storage_failure_without_publishing() {
    let repo = Arc::new(StoryStore::default().failing_create());
    let publisher = Arc::new(Recorder::default());
    let uc = CreateStory::new(repo, publisher.clone());

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 1,
            title: "Login".into(),
            points: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert!(publisher.events().is_empty(), "no event on failed persist");
}

// ── TransitionStory ──────────────────────────────────────────────────────────

#[tokio::test]
async fn transition_story_applies_an_allowed_transition_and_publishes_the_change() {
    let repo = Arc::new(StoryStore::default().with_story(persisted_story(1)));
    let publisher = Arc::new(Recorder::default());
    let uc = TransitionStory::new(repo.clone(), publisher.clone());

    uc.execute(TransitionStoryCmd {
        story_id: 1,
        target_status: StoryStatus::InProgress,
    })
    .await
    .unwrap();

    assert_eq!(repo.status_of(1), Some(StoryStatus::InProgress));
    let events = publisher.events();
    assert_eq!(events.len(), 1, "exactly one event is published");
    assert!(
        matches!(
            &events[0],
            DomainEvent::StoryStatusChanged { id: 1, from, to }
                if from == "todo" && to == "in_progress"
        ),
        "unexpected event: {events:?}"
    );
}

#[tokio::test]
async fn transition_story_reports_not_found_for_an_unknown_id() {
    let repo = Arc::new(StoryStore::default());
    let publisher = Arc::new(Recorder::default());
    let uc = TransitionStory::new(repo, publisher.clone());

    let err = uc
        .execute(TransitionStoryCmd {
            story_id: 99,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::NotFound(ref m) if m.contains("99")));
    assert!(publisher.events().is_empty());
}

#[tokio::test]
async fn transition_story_refuses_a_disallowed_transition_without_writing() {
    // Todo -> Done is not an allowed edge in the story state machine.
    let repo = Arc::new(StoryStore::default().with_story(persisted_story(1)));
    let publisher = Arc::new(Recorder::default());
    let uc = TransitionStory::new(repo.clone(), publisher.clone());

    let err = uc
        .execute(TransitionStoryCmd {
            story_id: 1,
            target_status: StoryStatus::Done,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        AppError::Domain(DomainError::InvalidTransition { .. })
    ));
    assert_eq!(repo.status_of(1), Some(StoryStatus::Todo));
    assert!(publisher.events().is_empty());
}

// ── CreateEpic ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_epic_persists_then_publishes_epic_created() {
    let repo = Arc::new(EpicStore::default());
    let publisher = Arc::new(Recorder::default());
    let uc = CreateEpic::new(repo.clone(), publisher.clone());

    let out = uc
        .execute(CreateEpicCmd {
            project_id: 3,
            title: "  Auth epic ".into(),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);
    let stored = repo.get_by_id(1).await.unwrap().unwrap();
    assert_eq!(stored.title, "Auth epic");
    assert_eq!(stored.status, EpicStatus::Backlog);
    let events = publisher.events();
    assert_eq!(events.len(), 1, "exactly one event is published");
    assert!(
        matches!(
            &events[0],
            DomainEvent::EpicCreated { id: 1, project_id: 3, title } if title == "Auth epic"
        ),
        "unexpected event: {events:?}"
    );
}

// ── PersistSyncedStories ─────────────────────────────────────────────────────

#[tokio::test]
async fn persist_synced_stories_upserts_by_requirement_id_without_duplicating() {
    let repo = Arc::new(StoryStore::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let batch = || PersistSyncedStoriesCmd {
        stories: vec![
            story_with_requirement(1, "gh:issue:1"),
            story_with_requirement(1, "gh:issue:2"),
        ],
    };

    let first = uc.execute(batch()).await.unwrap();
    assert_eq!(first.persisted_ids, vec![1, 2]);
    assert_eq!(repo.count(), 2);

    // Re-running sync must update the same rows, never insert duplicates.
    let second = uc.execute(batch()).await.unwrap();
    assert_eq!(second.persisted_ids, vec![1, 2]);
    assert_eq!(repo.count(), 2, "requirement_id is the idempotency key");
}

#[tokio::test]
async fn persist_synced_stories_counts_a_matching_row_id_as_updated() {
    let mut existing = story_with_requirement(1, "gh:issue:7");
    existing.id = 7;
    existing.status = StoryStatus::Todo;
    let repo = Arc::new(StoryStore::default().with_story(existing));
    let uc = PersistSyncedStories::new(repo.clone());

    let mut incoming = story_with_requirement(1, "gh:issue:7");
    incoming.id = 7;
    incoming.status = StoryStatus::InProgress;

    let report = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![incoming],
        })
        .await
        .unwrap();

    assert_eq!(report.persisted_ids, vec![7]);
    assert_eq!(report.updated, 1);
    assert_eq!(report.created, 0);
    assert_eq!(repo.status_of(7), Some(StoryStatus::InProgress));
    assert_eq!(repo.count(), 1);
}

#[tokio::test]
async fn persist_synced_stories_rejects_a_story_without_a_requirement_id() {
    let repo = Arc::new(StoryStore::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let untagged = Story::new(1, 1, "Untagged", None).unwrap();
    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![untagged],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert_eq!(repo.count(), 0, "validation must precede persistence");
}

#[tokio::test]
async fn persist_synced_stories_stops_the_batch_at_the_first_invalid_story() {
    let repo = Arc::new(StoryStore::default());
    let uc = PersistSyncedStories::new(repo.clone());

    let mut valid_first = story_with_requirement(1, "gh:issue:1");
    valid_first.id = 0;
    let invalid_second = Story::new(1, 1, "Untagged", None).unwrap();

    let err = uc
        .execute(PersistSyncedStoriesCmd {
            stories: vec![valid_first, invalid_second],
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert_eq!(
        repo.count(),
        1,
        "the first story is persisted before the batch is abandoned"
    );
}
