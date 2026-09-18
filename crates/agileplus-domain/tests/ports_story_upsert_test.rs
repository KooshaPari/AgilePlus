//! Contract tests for `StoryRepository::upsert_by_requirement_id`.
//!
//! The trait provides this upsert as a *default* implementation, so it is the
//! behavior every adapter inherits unless it opts into a native SQL upsert.
//! Nothing exercised it before: the previously covered adapters all override
//! it, so the portable get-then-insert/update path was dead in tests.
//!
//! Traceability: FR-STORE-* / WP05-T025

use std::sync::Mutex;

use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::StoryRepository;
use async_trait::async_trait;

/// Minimal in-memory `StoryRepository` that implements only the four required
/// methods, so every assertion below runs through the trait default.
#[derive(Default)]
struct InMemoryStoryRepo {
    stories: Mutex<Vec<Story>>,
    created: Mutex<Vec<Story>>,
    status_updates: Mutex<Vec<(i64, StoryStatus)>>,
    next_id: i64,
}

impl InMemoryStoryRepo {
    fn with_stories(stories: Vec<Story>) -> Self {
        Self {
            stories: Mutex::new(stories),
            next_id: 900,
            ..Self::default()
        }
    }

    fn created_titles(&self) -> Vec<String> {
        self.created
            .lock()
            .unwrap()
            .iter()
            .map(|story| story.title.clone())
            .collect()
    }

    fn status_updates(&self) -> Vec<(i64, StoryStatus)> {
        self.status_updates.lock().unwrap().clone()
    }
}

#[async_trait]
impl StoryRepository for InMemoryStoryRepo {
    async fn create(&self, story: &Story) -> Result<i64, DomainError> {
        self.created.lock().unwrap().push(story.clone());
        Ok(self.next_id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError> {
        Ok(self
            .stories
            .lock()
            .unwrap()
            .iter()
            .find(|story| story.id == id)
            .cloned())
    }

    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        self.status_updates.lock().unwrap().push((id, status));
        Ok(())
    }

    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        Ok(self
            .stories
            .lock()
            .unwrap()
            .iter()
            .filter(|story| story.epic_id == epic_id)
            .cloned()
            .collect())
    }
}

fn story(epic_id: i64, title: &str, requirement_id: Option<&str>, status: StoryStatus) -> Story {
    let mut story = Story::new(epic_id, 1, title, None).unwrap();
    story.requirement_id = requirement_id.map(str::to_string);
    story.status = status;
    story
}

#[tokio::test]
async fn upsert_requires_a_requirement_id_and_stays_fail_closed() {
    let repo = InMemoryStoryRepo::default();
    let unsaved = story(1, "Ad-hoc", None, StoryStatus::Todo);

    let error = repo.upsert_by_requirement_id(&unsaved).await.unwrap_err();

    assert!(matches!(
        error,
        DomainError::Validation(ref message)
            if message == "upsert_by_requirement_id requires story.requirement_id to be set"
    ));
    assert!(repo.created_titles().is_empty());
    assert!(repo.status_updates().is_empty());
}

#[tokio::test]
async fn upsert_inserts_when_the_requirement_id_is_unknown() {
    let repo = InMemoryStoryRepo::with_stories(Vec::new());
    let incoming = story(1, "Fresh", Some("FR-001"), StoryStatus::Todo);

    let id = repo.upsert_by_requirement_id(&incoming).await.unwrap();

    assert_eq!(id, 900);
    assert_eq!(repo.created_titles(), vec!["Fresh".to_string()]);
    assert!(repo.status_updates().is_empty());
}

#[tokio::test]
async fn upsert_updates_status_of_the_existing_match_and_reuses_its_id() {
    let mut existing = story(1, "Original", Some("FR-001"), StoryStatus::Todo);
    existing.id = 42;
    let repo = InMemoryStoryRepo::with_stories(vec![existing]);
    let incoming = story(1, "Renamed upstream", Some("FR-001"), StoryStatus::Done);

    let id = repo.upsert_by_requirement_id(&incoming).await.unwrap();

    assert_eq!(id, 42, "the existing row id must be preserved");
    assert_eq!(repo.status_updates(), vec![(42, StoryStatus::Done)]);
    assert!(
        repo.created_titles().is_empty(),
        "an existing requirement must never be inserted twice"
    );
}

#[tokio::test]
async fn upsert_matches_within_the_storys_epic_only() {
    let mut other_epic = story(7, "Same FR, other epic", Some("FR-002"), StoryStatus::Todo);
    other_epic.id = 11;
    let repo = InMemoryStoryRepo::with_stories(vec![other_epic]);
    let incoming = story(1, "Mine", Some("FR-002"), StoryStatus::Done);

    let id = repo.upsert_by_requirement_id(&incoming).await.unwrap();

    assert_eq!(id, 900);
    assert_eq!(repo.created_titles(), vec!["Mine".to_string()]);
    assert!(repo.status_updates().is_empty());
}

#[tokio::test]
async fn upsert_creates_when_the_epic_holds_different_requirement_ids() {
    let mut sibling = story(1, "Sibling", Some("FR-010"), StoryStatus::Todo);
    sibling.id = 3;
    let repo = InMemoryStoryRepo::with_stories(vec![sibling]);
    let incoming = story(1, "New work", Some("FR-011"), StoryStatus::Review);

    let id = repo.upsert_by_requirement_id(&incoming).await.unwrap();

    assert_eq!(id, 900);
    assert_eq!(repo.created_titles(), vec!["New work".to_string()]);
    assert!(repo.status_updates().is_empty());
}

#[tokio::test]
async fn upsert_ignores_candidate_stories_without_a_requirement_id() {
    let mut ad_hoc = story(1, "Ad-hoc", None, StoryStatus::Todo);
    ad_hoc.id = 5;
    let repo = InMemoryStoryRepo::with_stories(vec![ad_hoc]);
    let incoming = story(1, "Tracked", Some("FR-003"), StoryStatus::Todo);

    let id = repo.upsert_by_requirement_id(&incoming).await.unwrap();

    assert_eq!(id, 900);
    assert_eq!(repo.created_titles(), vec!["Tracked".to_string()]);
    assert!(repo.status_updates().is_empty());
}
