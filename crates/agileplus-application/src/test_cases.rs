// SPDX-License-Identifier: MIT OR Apache-2.0
//! Use-case tests for the application layer.

use std::sync::Arc;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::story::StoryStatus;
use agileplus_domain::ports::StoragePort;
use agileplus_domain::ports::epic::EpicRepository;
use agileplus_domain::ports::events::DomainEvent;
use agileplus_domain::ports::story::StoryRepository;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::{
    advance_feature::AdvanceFeature, create_epic::CreateEpic, create_feature::CreateFeature,
    create_story::CreateStory, persist_synced_stories::{PersistSyncedStories, PersistSyncedStoriesCmd},
    transition_story::TransitionStory,
};

// --- CreateFeature ---

#[tokio::test]
async fn create_feature_happy_path() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateFeatureCmd {
            slug: "auth".to_string(),
            friendly_name: "Authentication".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();

    assert_eq!(out.id, 1);
    assert_eq!(out.feature.slug, "auth");

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], DomainEvent::FeatureCreated { slug, .. } if slug == "auth"));
}

#[tokio::test]
async fn create_feature_defaults_target_branch_and_spec_hash() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());

    let out = uc
        .execute(CreateFeatureCmd {
            slug: "spec-default".to_string(),
            friendly_name: "Spec-driven".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();

    assert_eq!(out.feature.target_branch, "main");
    assert_eq!(out.feature.spec_hash, [0u8; 32]);

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::FeatureCreated { slug, .. } if slug == "spec-default"
    ));
}

#[tokio::test]
async fn create_feature_preserves_target_branch_and_spec_hash() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());
    let spec_hash = [11u8; 32];

    let out = uc
        .execute(CreateFeatureCmd {
            slug: "branched-feature".to_string(),
            friendly_name: "Branch-aware feature".to_string(),
            spec_hash: Some(spec_hash),
            target_branch: Some("feature/login".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(out.feature.target_branch, "feature/login");
    assert_eq!(out.feature.spec_hash, spec_hash);

    let events = pub_.emitted();
    assert_eq!(events.len(), 1);
    assert!(matches!(
        &events[0],
        DomainEvent::FeatureCreated { slug, .. } if slug == "branched-feature"
    ));
}

// --- AdvanceFeature ---

#[tokio::test]
async fn advance_feature_valid_transition() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let out = create_uc
        .execute(CreateFeatureCmd {
            slug: "feat-a".to_string(),
            friendly_name: "Feature A".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();

    advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: out.id,
            target_state: "specified".to_string(),
        })
        .await
        .unwrap();

    let feature = repo.get_feature_by_id(out.id).await.unwrap().unwrap();
    assert_eq!(feature.state, FeatureState::Specified);

    let events = pub_.emitted();
    assert_eq!(events.len(), 2);
    assert!(
        matches!(&events[1], DomainEvent::FeatureStateAdvanced { from, to, .. }
        if from == "created" && to == "specified")
    );
}

#[tokio::test]
async fn advance_feature_invalid_transition_rejected() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let out = create_uc
        .execute(CreateFeatureCmd {
            slug: "feat-b".to_string(),
            friendly_name: "Feature B".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();

    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: out.id,
            target_state: "shipped".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(_)));
}

#[tokio::test]
async fn advance_feature_not_found() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(AdvanceFeatureCmd {
            feature_id: 999,
            target_state: "specified".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::NotFound(_)));
}

// --- CreateStory ---

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

// --- TransitionStory ---

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

// --- CreateEpic ---

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
