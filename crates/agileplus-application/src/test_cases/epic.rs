// SPDX-License-Identifier: MIT OR Apache-2.0
//! CreateEpic use-case tests.

use std::sync::Arc;

use agileplus_domain::domain::epic::EpicStatus;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::epic::EpicRepository;
use agileplus_domain::ports::events::DomainEvent;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::create_epic::CreateEpic;

#[tokio::test]
async fn create_epic_happy_path() {
    let repo = Arc::new(InMemoryEpicRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateEpicCmd {
            project_id: 5,
            title: "Auth Epic".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::EpicCreated { project_id: 5, .. }
    ));
}

#[tokio::test]
async fn create_epic_rejects_empty_title() {
    let repo = Arc::new(InMemoryEpicRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo, pub_);

    let err = uc
        .execute(CreateEpicCmd {
            project_id: 5,
            title: "   ".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(_)));
}

#[tokio::test]
async fn create_epic_trims_title_and_emits_expected_event() {
    let repo = Arc::new(InMemoryEpicRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateEpicCmd {
            project_id: 42,
            title: "  API hardening  ".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);

    let stored = repo.get_by_id(1).await.unwrap().unwrap();
    assert_eq!(stored.title, "API hardening");

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::EpicCreated {
            project_id: 42,
            title,
            ..
        } if title == "API hardening"
    ));
}

// --- CreateEpic: port failure and persistence ---

/// A storage failure on `create` surfaces as `AppError::Domain` and no event
/// is published.
#[tokio::test]
async fn create_epic_storage_error_propagates_and_publishes_nothing() {
    let repo = Arc::new(InMemoryEpicRepo::default().failing_create());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(CreateEpicCmd {
            project_id: 5,
            title: "Auth Epic".to_string(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
    assert!(pub_.emitted().is_empty());
    assert!(repo.list_by_project(5).await.unwrap().is_empty());
}

/// The epic is committed before the event is published, so a failing
/// publisher leaves the row behind and reports the error.
#[tokio::test]
async fn create_epic_publish_error_leaves_row_persisted() {
    let repo = Arc::new(InMemoryEpicRepo::default());
    let uc = CreateEpic::new(repo.clone(), Arc::new(FailingPublisher));

    let err = uc
        .execute(CreateEpicCmd {
            project_id: 9,
            title: "  Hardening  ".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));

    let stored = repo
        .get_by_id(1)
        .await
        .unwrap()
        .expect("committed before publish");
    assert_eq!(
        stored.title, "Hardening",
        "title is trimmed before persisting"
    );
    assert_eq!(stored.status, EpicStatus::Backlog);
}

/// Domain defaults and repo-assigned ids are observable from the outside.
#[tokio::test]
async fn create_epic_persists_domain_defaults_and_sequential_ids() {
    let repo = Arc::new(InMemoryEpicRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo.clone(), pub_.clone());

    let first = uc
        .execute(CreateEpicCmd {
            project_id: 11,
            title: "First epic".to_string(),
        })
        .await
        .unwrap();
    let second = uc
        .execute(CreateEpicCmd {
            project_id: 11,
            title: "Second epic".to_string(),
        })
        .await
        .unwrap();

    assert_eq!((first.id, second.id), (1, 2));
    assert_eq!(repo.list_by_project(11).await.unwrap().len(), 2);
    assert!(repo.list_by_project(12).await.unwrap().is_empty());

    let stored = repo.get_by_id(first.id).await.unwrap().unwrap();
    assert_eq!(stored.project_id, 11);
    assert_eq!(stored.status, EpicStatus::Backlog);
    assert_eq!(stored.owner_id, None);
    assert_eq!(stored.requirement_id, None);

    let events = pub_.emitted();
    assert_eq!(events.len(), 2);
    assert!(matches!(
        &events[0],
        DomainEvent::EpicCreated { id: 1, project_id: 11, title } if title == "First epic"
    ));
    assert!(matches!(
        &events[1],
        DomainEvent::EpicCreated { id: 2, project_id: 11, title } if title == "Second epic"
    ));
}

/// Creating an epic writes without reading: a read fault on the repository
/// does not prevent the create from succeeding.
#[tokio::test]
async fn create_epic_does_not_read_before_writing() {
    let repo = Arc::new(InMemoryEpicRepo::default().failing_read());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateEpic::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateEpicCmd {
            project_id: 5,
            title: "Stored but unreadable".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1, "create itself is not affected by the read fault");
    assert!(matches!(
        repo.get_by_id(out.id).await.unwrap_err(),
        DomainError::Storage(_)
    ));
}
