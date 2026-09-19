// SPDX-License-Identifier: MIT OR Apache-2.0
//! AdvanceFeature use-case tests: the full feature lifecycle, invalid
//! transitions, and port-failure propagation.

use std::sync::Arc;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::events::DomainEvent;
use agileplus_domain::ports::StoragePort;

use crate::dto::*;
use crate::error::AppError;
use crate::test_mocks::*;
use crate::use_cases::{advance_feature::AdvanceFeature, create_feature::CreateFeature};

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

// --- AdvanceFeature: lifecycle coverage ---

/// Every legal hop of the feature lifecycle is accepted, each hop persists
/// the new state and publishes exactly one event carrying from/to.
#[tokio::test]
async fn advance_feature_walks_entire_lifecycle() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "lifecycle".to_string(),
            friendly_name: "Lifecycle".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    let hops = [
        "specified",
        "researched",
        "planned",
        "implementing",
        "validated",
        "shipped",
        "retrospected",
    ];

    let mut from = "created";
    for hop in hops {
        advance_uc
            .execute(AdvanceFeatureCmd {
                feature_id: id,
                target_state: hop.to_string(),
            })
            .await
            .unwrap_or_else(|e| panic!("{from} -> {hop} should be allowed: {e:?}"));

        let stored = repo.get_feature_by_id(id).await.unwrap().unwrap();
        assert_eq!(stored.state.to_string(), hop);

        let events = pub_.emitted();
        assert!(
            matches!(
                events.last().unwrap(),
                DomainEvent::FeatureStateAdvanced { id: event_id, from: ev_from, to }
                    if *event_id == id && ev_from == from && to == hop
            ),
            "event for {from} -> {hop} missing or wrong: {:?}",
            events.last()
        );
        from = hop;
    }

    assert_eq!(
        pub_.emitted().len(),
        1 + hops.len(),
        "one FeatureCreated plus one event per hop"
    );

    // `retrospected` is terminal.
    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "shipped".to_string(),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert_eq!(
        pub_.emitted().len(),
        1 + hops.len(),
        "no event on rejection"
    );
}

/// An unparseable `target_state` is a validation error — not a panic and
/// not a silent no-op — and leaves the feature and the event bus untouched.
#[tokio::test]
async fn advance_feature_rejects_unknown_target_state() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "bad-target".to_string(),
            friendly_name: "Bad target".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "in-review".to_string(),
        })
        .await
        .unwrap_err();

    match err {
        AppError::Domain(DomainError::Validation(msg)) => {
            assert!(
                !msg.is_empty(),
                "validation message should explain the parse failure"
            );
        }
        other => panic!("expected Domain(Validation), got {other:?}"),
    }

    assert_eq!(
        repo.get_feature_by_id(id).await.unwrap().unwrap().state,
        FeatureState::Created
    );
    assert_eq!(pub_.emitted().len(), 1, "only FeatureCreated was published");
}

/// Re-advancing to the current state is not a legal transition.
#[tokio::test]
async fn advance_feature_rejects_no_op_transition() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "noop".to_string(),
            friendly_name: "No-op".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "created".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Validation(_))));
    assert_eq!(pub_.emitted().len(), 1);
}

/// A read failure while loading the feature is reported and nothing is
/// published or written.
#[tokio::test]
async fn advance_feature_storage_read_error_propagates() {
    let repo = Arc::new(InMemoryFeatureRepo::default().failing_read());
    let pub_ = Arc::new(SpyPublisher::default());
    let uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let err = uc
        .execute(AdvanceFeatureCmd {
            feature_id: 1,
            target_state: "specified".to_string(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
    assert!(pub_.emitted().is_empty());
}

/// A write failure after a legal transition reports the storage error, does
/// not publish the advance event, and leaves the stored state unchanged.
#[tokio::test]
async fn advance_feature_state_write_error_does_not_publish() {
    let repo = Arc::new(InMemoryFeatureRepo::default().failing_state_write());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "write-fail".to_string(),
            friendly_name: "Write fail".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "specified".to_string(),
        })
        .await
        .unwrap_err();

    assert!(
        matches!(err, AppError::Domain(DomainError::Storage(_))),
        "expected Domain(Storage), got {err:?}"
    );
    assert_eq!(
        pub_.emitted().len(),
        1,
        "advance event must not be published"
    );
    assert_eq!(
        repo.get_feature_by_id(id).await.unwrap().unwrap().state,
        FeatureState::Created,
        "failed write must leave the stored state untouched"
    );
}

/// The state change is committed before the event is published: a failing
/// publisher leaves the new state persisted and reports the error.
#[tokio::test]
async fn advance_feature_publish_error_leaves_state_advanced() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let spy = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), spy.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "publish-fail".to_string(),
            friendly_name: "Publish fail".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    let advance_uc = AdvanceFeature::new(repo.clone(), Arc::new(FailingPublisher));
    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "specified".to_string(),
        })
        .await
        .unwrap_err();

    assert!(matches!(err, AppError::Domain(DomainError::Storage(_))));
    assert_eq!(
        repo.get_feature_by_id(id).await.unwrap().unwrap().state,
        FeatureState::Specified,
        "state advance is committed before publish"
    );
    assert_eq!(
        spy.emitted().len(),
        1,
        "only FeatureCreated reached the spy"
    );
}

/// The target state is parsed case-sensitively: a real state spelled in the
/// wrong case is refused (and named in the error) rather than silently
/// normalised into a transition.
#[tokio::test]
async fn advance_feature_rejects_valid_state_in_wrong_case() {
    let repo = Arc::new(InMemoryFeatureRepo::default());
    let pub_ = Arc::new(SpyPublisher::default());
    let create_uc = CreateFeature::new(repo.clone(), pub_.clone());
    let advance_uc = AdvanceFeature::new(repo.clone(), pub_.clone());

    let id = create_uc
        .execute(CreateFeatureCmd {
            slug: "wrong-case".to_string(),
            friendly_name: "Wrong case".to_string(),
            spec_hash: None,
            target_branch: None,
        })
        .await
        .unwrap()
        .id;

    // `specified` is the next legal state, but only in lower case.
    let err = advance_uc
        .execute(AdvanceFeatureCmd {
            feature_id: id,
            target_state: "Specified".to_string(),
        })
        .await
        .unwrap_err();

    match err {
        AppError::Domain(DomainError::Validation(msg)) => {
            assert!(
                msg.contains("Specified"),
                "the refusal should name the offending input, got {msg:?}"
            );
        }
        other => panic!("expected Domain(Validation), got {other:?}"),
    }

    assert_eq!(
        repo.get_feature_by_id(id).await.unwrap().unwrap().state,
        FeatureState::Created,
        "a refused parse must not advance the feature"
    );
    assert_eq!(pub_.emitted().len(), 1, "only FeatureCreated was published");
}
