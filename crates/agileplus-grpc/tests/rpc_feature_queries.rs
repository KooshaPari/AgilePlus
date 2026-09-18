//! RPC handler tests for feature and work-package queries.
//!
//! Drives the real `AgilePlusCoreService` implementations in `server::mod`
//! against an in-memory store, asserting the observable proto responses and
//! the gRPC status codes for rejection paths.
//!
//! Traceability: WP14-T079, T080

mod support;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{DependencyType, WpState};
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreService;
use agileplus_proto::agileplus::v1::{
    GetFeatureRequest, GetFeatureStateRequest, GetWorkPackageStatusRequest, ListFeaturesRequest,
    ListWorkPackagesRequest,
};
use support::{Harness, foreign_scope, scope};
use tonic::{Code, Request};

// ---------------------------------------------------------------------------
// GetFeature
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_feature_returns_persisted_feature_with_wp_counts() {
    let harness = Harness::new().await;
    let feature = harness
        .seed_feature("checkout", FeatureState::Specified)
        .await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness.seed_wp(feature.id, 2, WpState::Doing).await;

    let response = harness
        .server
        .get_feature(Request::new(GetFeatureRequest {
            slug: "checkout".into(),
            project_scope: scope(),
        }))
        .await
        .expect("existing feature should be returned")
        .into_inner();
    let proto = response.feature.expect("feature payload should be present");

    assert_eq!(proto.id, feature.id);
    assert_eq!(proto.slug, "checkout");
    assert_eq!(proto.friendly_name, "Feature checkout");
    assert_eq!(proto.state, "specified");
    assert_eq!(proto.target_branch, "main");
    // Counts must reflect the persisted work packages, not the request.
    assert_eq!(proto.wp_count, 2);
    assert_eq!(proto.wp_done, 1);
}

#[tokio::test]
async fn get_feature_unknown_slug_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .get_feature(Request::new(GetFeatureRequest {
            slug: "ghost".into(),
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

#[tokio::test]
async fn get_feature_without_project_scope_is_invalid_argument() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .get_feature(Request::new(GetFeatureRequest {
            slug: "checkout".into(),
            project_scope: None,
        }))
        .await
        .expect_err("a scope-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}

#[tokio::test]
async fn get_feature_with_foreign_project_scope_is_permission_denied() {
    let harness = Harness::new().await;
    harness
        .seed_feature("checkout", FeatureState::Created)
        .await;

    let status = harness
        .server
        .get_feature(Request::new(GetFeatureRequest {
            slug: "checkout".into(),
            project_scope: foreign_scope(),
        }))
        .await
        .expect_err("a foreign scope must be denied");

    assert_eq!(status.code(), Code::PermissionDenied);
    assert_eq!(
        status.message(),
        "project scope does not match this core server repository"
    );
}

// ---------------------------------------------------------------------------
// ListFeatures
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_features_returns_every_feature_with_wp_counts() {
    let harness = Harness::new().await;
    let alpha = harness.seed_feature("alpha", FeatureState::Created).await;
    harness.seed_wp(alpha.id, 1, WpState::Done).await;
    let beta = harness.seed_feature("beta", FeatureState::Planned).await;
    harness.seed_wp(beta.id, 1, WpState::Planned).await;
    harness.seed_wp(beta.id, 2, WpState::Done).await;

    let mut features = harness
        .server
        .list_features(Request::new(ListFeaturesRequest {
            state_filter: String::new(),
            project_scope: scope(),
        }))
        .await
        .expect("an unfiltered listing should succeed")
        .into_inner()
        .features;
    features.sort_by(|a, b| a.slug.cmp(&b.slug));

    assert_eq!(features.len(), 2);
    assert_eq!(features[0].slug, "alpha");
    assert_eq!((features[0].wp_count, features[0].wp_done), (1, 1));
    assert_eq!(features[1].slug, "beta");
    assert_eq!(features[1].state, "planned");
    assert_eq!((features[1].wp_count, features[1].wp_done), (2, 1));
}

#[tokio::test]
async fn list_features_filters_by_state() {
    let harness = Harness::new().await;
    harness
        .seed_feature("draft-one", FeatureState::Created)
        .await;
    harness
        .seed_feature("draft-two", FeatureState::Created)
        .await;
    harness
        .seed_feature("shipped-one", FeatureState::Shipped)
        .await;

    let filtered = harness
        .server
        .list_features(Request::new(ListFeaturesRequest {
            state_filter: "created".into(),
            project_scope: scope(),
        }))
        .await
        .expect("a known state filter should succeed")
        .into_inner()
        .features;

    let mut slugs: Vec<_> = filtered.iter().map(|f| f.slug.clone()).collect();
    slugs.sort();
    assert_eq!(slugs, vec!["draft-one", "draft-two"]);
    assert!(filtered.iter().all(|f| f.state == "created"));

    let shipped = harness
        .server
        .list_features(Request::new(ListFeaturesRequest {
            state_filter: "shipped".into(),
            project_scope: scope(),
        }))
        .await
        .expect("a known state filter should succeed")
        .into_inner()
        .features;
    assert_eq!(shipped.len(), 1);
    assert_eq!(shipped[0].slug, "shipped-one");
}

#[tokio::test]
async fn list_features_rejects_unknown_state_filter() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let status = harness
        .server
        .list_features(Request::new(ListFeaturesRequest {
            state_filter: "not-a-state".into(),
            project_scope: scope(),
        }))
        .await
        .expect_err("an unknown state must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("not-a-state"));
}

#[tokio::test]
async fn list_features_on_empty_store_is_empty() {
    let harness = Harness::new().await;

    let response = harness
        .server
        .list_features(Request::new(ListFeaturesRequest {
            state_filter: String::new(),
            project_scope: scope(),
        }))
        .await
        .expect("listing an empty store should succeed")
        .into_inner();

    assert!(response.features.is_empty());
}

// ---------------------------------------------------------------------------
// GetFeatureState
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_feature_state_maps_every_state_to_its_next_command() {
    let harness = Harness::new().await;
    let expected = [
        (FeatureState::Created, "created", "specify"),
        (FeatureState::Specified, "specified", "research"),
        (FeatureState::Researched, "researched", "plan"),
        (FeatureState::Planned, "planned", "implement"),
        (FeatureState::Implementing, "implementing", "validate"),
        (FeatureState::Validated, "validated", "ship"),
        (FeatureState::Shipped, "shipped", "retrospective"),
        (FeatureState::Retrospected, "retrospected", ""),
    ];

    for (state, state_str, next_command) in expected {
        let slug = format!("state-{state_str}");
        harness.seed_feature(&slug, state).await;

        let proto = harness
            .server
            .get_feature_state(Request::new(GetFeatureStateRequest {
                slug: slug.clone(),
                project_scope: scope(),
            }))
            .await
            .expect("existing feature should report its state")
            .into_inner()
            .feature_state
            .expect("feature state payload should be present");

        assert_eq!(proto.state, state_str, "state name for {slug}");
        assert_eq!(proto.next_command, next_command, "next command for {slug}");
        assert!(proto.blockers.is_empty());
        assert!(proto.governance.is_none());
    }
}

#[tokio::test]
async fn get_feature_state_unknown_slug_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .get_feature_state(Request::new(GetFeatureStateRequest {
            slug: "ghost".into(),
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

#[tokio::test]
async fn get_feature_state_without_project_scope_is_invalid_argument() {
    let harness = Harness::new().await;
    harness.seed_feature("alpha", FeatureState::Created).await;

    let status = harness
        .server
        .get_feature_state(Request::new(GetFeatureStateRequest {
            slug: "alpha".into(),
            project_scope: None,
        }))
        .await
        .expect_err("a scope-less request must be rejected");

    assert_eq!(status.code(), Code::InvalidArgument);
}

// ---------------------------------------------------------------------------
// ListWorkPackages
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_work_packages_returns_packages_with_dependencies() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Planned).await;
    let first = harness.seed_wp(feature.id, 1, WpState::Done).await;
    let second = harness.seed_wp(feature.id, 2, WpState::Doing).await;
    harness
        .seed_dependency(second.id, first.id, DependencyType::Explicit)
        .await;

    let packages = harness
        .server
        .list_work_packages(Request::new(ListWorkPackagesRequest {
            feature_slug: "alpha".into(),
            state_filter: String::new(),
            project_scope: scope(),
        }))
        .await
        .expect("listing work packages should succeed")
        .into_inner()
        .packages;

    assert_eq!(packages.len(), 2);
    assert_eq!(packages[0].sequence, 1);
    assert_eq!(packages[0].state, "done");
    assert!(packages[0].depends_on.is_empty());
    assert_eq!(packages[1].sequence, 2);
    assert_eq!(packages[1].state, "doing");
    assert_eq!(packages[1].depends_on, vec![first.id]);
    assert_eq!(packages[1].title, "WP 2");
}

#[tokio::test]
async fn list_work_packages_state_filter_is_case_insensitive() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Planned).await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;
    harness.seed_wp(feature.id, 2, WpState::Blocked).await;

    let matching = harness
        .server
        .list_work_packages(Request::new(ListWorkPackagesRequest {
            feature_slug: "alpha".into(),
            state_filter: "DONE".into(),
            project_scope: scope(),
        }))
        .await
        .expect("filtering should succeed")
        .into_inner()
        .packages;
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].sequence, 1);

    let unmatched = harness
        .server
        .list_work_packages(Request::new(ListWorkPackagesRequest {
            feature_slug: "alpha".into(),
            state_filter: "review".into(),
            project_scope: scope(),
        }))
        .await
        .expect("filtering should succeed")
        .into_inner()
        .packages;
    assert!(unmatched.is_empty());
}

#[tokio::test]
async fn list_work_packages_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .list_work_packages(Request::new(ListWorkPackagesRequest {
            feature_slug: "ghost".into(),
            state_filter: String::new(),
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}

// ---------------------------------------------------------------------------
// GetWorkPackageStatus
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_work_package_status_returns_requested_sequence() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Planned).await;
    let first = harness.seed_wp(feature.id, 1, WpState::Done).await;
    let second = harness.seed_wp(feature.id, 2, WpState::Review).await;
    harness
        .seed_dependency(second.id, first.id, DependencyType::FileOverlap)
        .await;

    let proto = harness
        .server
        .get_work_package_status(Request::new(GetWorkPackageStatusRequest {
            feature_slug: "alpha".into(),
            wp_sequence: 2,
            project_scope: scope(),
        }))
        .await
        .expect("an existing sequence should be returned")
        .into_inner()
        .work_package_status
        .expect("work package payload should be present");

    assert_eq!(proto.id, second.id);
    assert_eq!(proto.sequence, 2);
    assert_eq!(proto.state, "review");
    assert_eq!(proto.depends_on, vec![first.id]);
}

#[tokio::test]
async fn get_work_package_status_unknown_sequence_is_not_found() {
    let harness = Harness::new().await;
    let feature = harness.seed_feature("alpha", FeatureState::Planned).await;
    harness.seed_wp(feature.id, 1, WpState::Done).await;

    let status = harness
        .server
        .get_work_package_status(Request::new(GetWorkPackageStatusRequest {
            feature_slug: "alpha".into(),
            wp_sequence: 99,
            project_scope: scope(),
        }))
        .await
        .expect_err("a missing sequence must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("99"));
}

#[tokio::test]
async fn get_work_package_status_unknown_feature_is_not_found() {
    let harness = Harness::new().await;

    let status = harness
        .server
        .get_work_package_status(Request::new(GetWorkPackageStatusRequest {
            feature_slug: "ghost".into(),
            wp_sequence: 1,
            project_scope: scope(),
        }))
        .await
        .expect_err("missing feature must fail");

    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("ghost"));
}
