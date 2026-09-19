// SPDX-License-Identifier: MIT OR Apache-2.0
//! CreateFeature use-case tests: defaults, id assignment, and
//! port-failure propagation.

use std::sync::Arc;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::events::DomainEvent;
use agileplus_domain::ports::StoragePort;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::create_feature::CreateFeature;

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

// --- CreateFeature: port failure and persistence ---

/// A storage failure on `create_feature` surfaces as `AppError::Domain` —
/// ports speak `DomainError`, so the application layer must not invent a
/// second error type — and no event is published.
#[tokio::test]
async fn create_feature_storage_error_propagates_and_publishes_nothing() {
    let repo = Arc::new(InMemoryFeatureRepo::default().failing_create());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(CreateFeatureCmd {
            slug: "auth".to_string(),
            friendly_name: "Authentication".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
    assert!(pub_.emitted().is_empty(), "no event on failed persist");
    assert!(
        repo.get_feature_by_slug("auth").await.unwrap().is_none(),
        "nothing should be stored"
    );
}

/// The row is committed *before* the event is published, so a failing
/// publisher leaves the feature persisted and reports the publish error.
#[tokio::test]
async fn create_feature_publish_error_leaves_row_persisted() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let uc = CreateFeature::new(repo.clone(), Arc::new(FailingPublisher));

    let err = uc
        .execute(CreateFeatureCmd {
            slug: "auth".to_string(),
            friendly_name: "Authentication".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );

    let stored = repo
        .get_feature_by_slug("auth")
        .await
        .unwrap()
        .expect("create is committed before publish");
    assert_eq!(stored.id, 1);
    assert_eq!(stored.state, FeatureState::Created);
}

/// Ids come from the repository (not from the use case), each create is a
/// separate row, and the output aggregate mirrors the persisted id.
#[tokio::test]
async fn create_feature_assigns_sequential_ids_and_returns_persisted_aggregate() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());

    let first = uc
        .execute(CreateFeatureCmd {
            slug: "first".to_string(),
            friendly_name: "First".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();
    let second = uc
        .execute(CreateFeatureCmd {
            slug: "second".to_string(),
            friendly_name: "Second".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();

    assert_eq!((first.id, second.id), (1, 2));
    assert_eq!(
        first.feature.id, first.id,
        "returned aggregate carries the persisted id"
    );
    assert_eq!(second.feature.id, second.id);
    assert_eq!(repo.list_all_features().await.unwrap().len(), 2);

    let events = pub_.emitted();
    assert_eq!(events.len(), 2);
    assert!(matches!(&events[0], DomainEvent::FeatureCreated { id: 1, slug } if slug == "first"));
    assert!(matches!(&events[1], DomainEvent::FeatureCreated { id: 2, slug } if slug == "second"));
}

// --- CreateFeature: target branch semantics ---

/// Only a missing branch falls back to the default; an explicit empty string
/// is stored as given rather than being silently replaced by `main`.
#[tokio::test]
async fn create_feature_empty_target_branch_is_stored_verbatim() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = CreateFeature::new(repo.clone(), pub_.clone());

    let defaulted = uc
        .execute(CreateFeatureCmd {
            slug: "no-branch".to_string(),
            friendly_name: "No branch".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap();
    let explicit = uc
        .execute(CreateFeatureCmd {
            slug: "empty-branch".to_string(),
            friendly_name: "Empty branch".to_string(),
            spec_hash: None,
            target_branch: Some(String::new()),
        })
        .await
        .unwrap();

    assert_eq!(defaulted.feature.target_branch, "main");
    assert_eq!(explicit.feature.target_branch, "");
    assert_eq!(
        repo.get_feature_by_slug("empty-branch")
            .await
            .unwrap()
            .expect("row was persisted")
            .target_branch,
        "",
        "the persisted row keeps the caller's explicit choice"
    );
    assert_eq!(pub_.emitted().len(), 2);
}
