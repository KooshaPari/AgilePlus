// SPDX-License-Identifier: MIT OR Apache-2.0
//! TransitionStory use-case tests: allowed and rejected status transitions,
//! persistence, and port-failure propagation.

use std::sync::Arc;

use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::events::DomainEvent;
use agileplus_domain::ports::story::StoryRepository;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::{create_story::CreateStory, transition_story::TransitionStory};

#[tokio::test]
async fn transition_story_happy_path() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let out = create_uc
        .execute(CreateStoryCmd {
            epic_id: 2,
            project_id: 20,
            title: "Login flow".to_string(),
            points: None,
        })
        .await
        .unwrap();

    trans_uc
        .execute(TransitionStoryCmd {
            story_id: out.id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap();

    let story = repo.get_by_id(out.id).await.unwrap().unwrap();
    assert_eq!(story.status, StoryStatus::InProgress);

    let events = pub_.emitted();
    assert_eq!(events.len(), 2);
    assert!(
        matches!(&events[1], DomainEvent::StoryStatusChanged { from, to, .. }
        if from == "todo" && to == "in_progress")
    );
}

#[tokio::test]
async fn transition_story_invalid_transition_rejected() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let out = create_uc
        .execute(CreateStoryCmd {
            epic_id: 2,
            project_id: 20,
            title: "Story skip".to_string(),
            points: None,
        })
        .await
        .unwrap();

    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: out.id,
            target_status: StoryStatus::Done,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(_)));
}

#[tokio::test]
async fn transition_story_not_found() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = TransitionStory::new(repo, pub_);

    let err = uc
        .execute(TransitionStoryCmd {
            story_id: 999,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::NotFound(_)));
}

// --- TransitionStory: multi-step flows and port failure ---

/// Walking `todo -> in_progress -> review -> done` persists each hop and
/// publishes one event per hop with the right from/to.
#[tokio::test]
async fn transition_story_walks_todo_to_done() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Ship it".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    let hops = [
        ("todo", StoryStatus::InProgress),
        ("in_progress", StoryStatus::Review),
        ("review", StoryStatus::Done),
    ];

    for (hop, (from, to)) in hops.into_iter().enumerate() {
        trans_uc
            .execute(TransitionStoryCmd {
                story_id: id,
                target_status: to,
            })
            .await
            .unwrap_or_else(|e| panic!("{from} -> {to} should be allowed: {e:?}"));

        let stored = repo.get_by_id(id).await.unwrap().unwrap();
        assert_eq!(stored.status, to);

        let events = pub_.emitted();
        assert_eq!(
            events.len(),
            1 + hop + 1,
            "one StoryCreated plus one event per completed hop"
        );
        assert!(
            matches!(
                events.last().unwrap(),
                DomainEvent::StoryStatusChanged { id: event_id, from: ev_from, to: ev_to }
                    if *event_id == id && ev_from == from && ev_to == &to.to_string()
            ),
            "event for {from} -> {to} missing or wrong: {:?}",
            events.last()
        );
    }

    assert_eq!(
        repo.get_by_id(id).await.unwrap().unwrap().status,
        StoryStatus::Done
    );
}

/// `done` is terminal: no further status change is allowed.
#[tokio::test]
async fn transition_story_from_done_is_rejected() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Terminal".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    for to in [
        StoryStatus::InProgress,
        StoryStatus::Review,
        StoryStatus::Done,
    ] {
        trans_uc
            .execute(TransitionStoryCmd {
                story_id: id,
                target_status: to,
            })
            .await
            .unwrap();
    }

    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(
        err,
        AppError::Domain(DomainError::InvalidTransition { .. })
    ));
    assert_eq!(
        repo.get_by_id(id).await.unwrap().unwrap().status,
        StoryStatus::Done
    );
}

/// Blocking and unblocking is a legal cycle, and the emitted events carry the
/// exact status strings.
#[tokio::test]
async fn transition_story_block_and_unblock() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 2,
            project_id: 20,
            title: "Blocked work".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap();
    trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::Blocked,
        })
        .await
        .unwrap();
    trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap();

    let events = pub_.emitted();
    assert_eq!(events.len(), 4, "create + three transitions");
    assert!(matches!(
        &events[2],
        DomainEvent::StoryStatusChanged { from, to, .. } if from == "in_progress" && to == "blocked"
    ));
    assert!(matches!(
        &events[3],
        DomainEvent::StoryStatusChanged { from, to, .. } if from == "blocked" && to == "in_progress"
    ));
}

/// Cancelling from `todo` is allowed and terminal.
#[tokio::test]
async fn transition_story_todo_can_be_cancelled() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 5,
            project_id: 50,
            title: "Dropped".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::Cancelled,
        })
        .await
        .unwrap();
    assert_eq!(
        repo.get_by_id(id).await.unwrap().unwrap().status,
        StoryStatus::Cancelled
    );

    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::Domain(DomainError::InvalidTransition { .. })
    ));
}

/// The rejection carries the transition it refused, not just a generic error.
#[tokio::test]
async fn transition_story_reports_refused_transition_payload() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Skipper".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::Review,
        })
        .await
        .unwrap_err();

    match err {
        AppError::Domain(DomainError::InvalidTransition { from, to, reason }) => {
            assert_eq!((from.as_str(), to.as_str()), ("todo", "review"));
            assert!(!reason.is_empty());
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
    assert_eq!(pub_.emitted().len(), 1, "only StoryCreated was published");
}

/// A read failure while loading the story is reported and nothing is
/// published.
#[tokio::test]
async fn transition_story_storage_read_error_propagates() {
    let repo = Arc::new(InMemoryStoryRepo::default().failing_read());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = TransitionStory::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(TransitionStoryCmd {
            story_id: 1,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert!(pub_.emitted().is_empty());
}

/// A write failure after a legal transition reports the error, publishes
/// nothing, and leaves the stored status unchanged.
#[tokio::test]
async fn transition_story_status_write_error_does_not_publish() {
    let repo = Arc::new(InMemoryStoryRepo::default().failing_status_write());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), pub_.clone());
    let trans_uc = TransitionStory::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Write fail".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert_eq!(pub_.emitted().len(), 1, "no status-change event");
    assert_eq!(
        repo.get_by_id(id).await.unwrap().unwrap().status,
        StoryStatus::Todo,
        "failed write leaves the stored status untouched"
    );
}

/// The status change is committed before the event is published.
#[tokio::test]
async fn transition_story_publish_error_leaves_status_changed() {
    let repo = Arc::new(InMemoryStoryRepo::default());
    let spy = Arc::new(SpyPublisher::default());
    let create_uc = CreateStory::new(repo.clone(), spy.clone());

    let id = create_uc
        .execute(CreateStoryCmd {
            epic_id: 1,
            project_id: 10,
            title: "Publish fail".to_string(),
            points: None,
        })
        .await
        .unwrap()
        .id;

    let trans_uc = TransitionStory::new(repo.clone(), Arc::new(FailingPublisher));
    let err = trans_uc
        .execute(TransitionStoryCmd {
            story_id: id,
            target_status: StoryStatus::InProgress,
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert_eq!(
        repo.get_by_id(id).await.unwrap().unwrap().status,
        StoryStatus::InProgress
    );
    assert_eq!(spy.emitted().len(), 1);
}
