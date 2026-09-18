// SPDX-License-Identifier: MIT OR Apache-2.0
//! CreateStory use-case tests: validation, persistence, and
//! port-failure propagation.

use std::sync::Arc;

use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::events::DomainEvent;
use agileplus_domain::ports::story::StoryRepository;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::create_story::CreateStory;

#[tokio::test]
async fn create_story_happy_path() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateStory::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "User can log in".to_string(),
            points: Some(3),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);
    assert_eq!(out.story.title, "User can log in");

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::StoryCreated { epic_id: 1, .. }
    ));
}

#[tokio::test]
async fn create_story_rejects_empty_title() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateStory::new(repo, pub_);

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "".to_string(),
            points: None,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        AppError::Domain(agileplus_domain::error::DomainError::Validation(_))
    ));
}

#[tokio::test]
async fn create_story_rejects_zero_points() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateStory::new(repo, pub_);

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Some story".to_string(),
            points: Some(0),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(_)));
}

// --- CreateStory: port failure and persistence ---

/// A storage failure on `create` surfaces as `AppError::Domain` and no event
/// is published.
#[tokio::test]
async fn create_story_storage_error_propagates_and_publishes_nothing() {
    let repo = Arc::new(InMemoryStoryRepo::default().failing_create());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateStory::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "User can log in".to_string(),
            points: Some(3),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
    assert!(pub_.emitted().is_empty());
    assert!(repo.list_by_epic(1).await.unwrap().is_empty());
}

/// The story is committed before the event is published, so a failing
/// publisher leaves the row behind and reports the error.
#[tokio::test]
async fn create_story_publish_error_leaves_row_persisted() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let uc = CreateStory::new(repo.clone(), Arc::new(FailingPublisher));

    let err = uc
        .execute(CreateStoryCmd {
            epic_id: 7,
            project_id: 70,
            title: "Dark mode".to_string(),
            points: Some(5),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));

    let stored = repo.list_by_epic(7).await.unwrap();
    assert_eq!(stored.len(), 1, "create is committed before publish");
    assert_eq!(stored[0].title, "Dark mode");
    assert_eq!(stored[0].points, Some(5));
}

/// Domain defaults are applied to the persisted aggregate: a new story is
/// `Todo` with no assignee/requirement, and the output mirrors the stored id.
#[tokio::test]
async fn create_story_persists_domain_defaults_and_stored_id() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateStory::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateStoryCmd {
            epic_id: 3,
            project_id: 30,
            title: "  Trim me  ".to_string(),
            points: None,
        })
        .await
        .unwrap();

    assert_eq!(out.story.id, out.id, "output carries the persisted id");
    assert_eq!(out.story.title, "Trim me", "title is trimmed");
    assert_eq!(out.story.status, StoryStatus::Todo);

    let stored = repo.get_by_id(out.id).await.unwrap().unwrap();
    assert_eq!(stored.status, StoryStatus::Todo);
    assert_eq!(stored.assignee_id, None);
    assert_eq!(stored.requirement_id, None);
    assert_eq!(stored.points, None);

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::StoryCreated { id, epic_id: 3, title }
            if *id == out.id && title == "Trim me"
    ));

    // A second create under the same epic gets the next repo id.
    let second = uc
        .execute(CreateStoryCmd {
            epic_id: 3,
            project_id: 30,
            title: "Second".to_string(),
            points: None,
        })
        .await
        .unwrap();
    assert_eq!((out.id, second.id), (1, 2));
    assert_eq!(repo.list_by_epic(3).await.unwrap().len(), 2);
    assert!(repo.list_by_epic(999).await.unwrap().is_empty());
}
