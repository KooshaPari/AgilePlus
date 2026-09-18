//! Port-contract tests for `agileplus_domain::ports`.
//!
//! These are integration tests on purpose: they exercise the port traits the
//! way an adapter crate does, through the public API only.
//!
//! Every test here targets behavior that no unit test reached before:
//! the `StoragePort` / `ContentStoragePort` trait defaults (which adapters
//! inherit unless they opt in), the blanket `StoryRepository` /
//! `EpicRepository` impls that forward to `StoragePort`, and the
//! `TriageTicket` / `TriageError` conversion surface.
//!
//! Traceability: FR-STORE-* / WP05-T025

mod support;

use agileplus_domain::domain::backlog::{BacklogItem, BacklogPriority, BacklogStatus, Intent};
use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::domain::user::{UserRole, UserStatus};
use agileplus_domain::domain::work_package::WorkPackage;
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::{
    EpicRepository, StoragePort, StoryRepository, TriageError, TriageOutcome, TriageTicket,
};

use support::{RecordingContentStorage, RecordingStorage};

fn sample_wp() -> WorkPackage {
    WorkPackage::new(1, "WP", 1, "criteria")
}

fn sample_story() -> Story {
    Story::new(1, 1, "Story", None).unwrap()
}

fn sample_feature() -> Feature {
    Feature::new("feat", "Feature", [0u8; 32], None)
}

fn sample_epic() -> Epic {
    Epic::new(1, "Epic").unwrap()
}

/// Every default on `StoragePort` must fail closed with
/// `DomainError::NotImplemented` rather than silently pretending success, and
/// must not delegate to a required method.
#[tokio::test]
async fn storage_port_defaults_all_report_not_implemented() {
    let storage = RecordingStorage::new(11);
    let port: &dyn StoragePort = &storage;
    let wp = sample_wp();
    let story = sample_story();
    let feature = sample_feature();

    let results: Vec<(&str, Result<(), DomainError>)> = vec![
        (
            "create_work_package_for_story",
            port.create_work_package_for_story(1, &wp).await.map(drop),
        ),
        (
            "list_wps_by_story",
            port.list_wps_by_story(1).await.map(drop),
        ),
        (
            "list_all_work_packages",
            port.list_all_work_packages().await.map(drop),
        ),
        (
            "get_next_ready_wps",
            port.get_next_ready_wps(Some(7)).await.map(drop),
        ),
        (
            "add_story_to_cycle",
            port.add_story_to_cycle(1, 2).await.map(drop),
        ),
        (
            "get_project_by_id",
            port.get_project_by_id(1).await.map(drop),
        ),
        ("delete_project", port.delete_project(1).await),
        ("delete_epic", port.delete_epic(1).await),
        (
            "list_stories_by_project",
            port.list_stories_by_project(1).await.map(drop),
        ),
        ("delete_story", port.delete_story(1).await),
        (
            "upsert_story_by_requirement_id",
            port.upsert_story_by_requirement_id(&story).await.map(drop),
        ),
        ("update_feature", port.update_feature(&feature).await),
        (
            "list_features_by_label",
            port.list_features_by_label("ui").await.map(drop),
        ),
        (
            "update_user_status",
            port.update_user_status(1, UserStatus::Active).await,
        ),
        (
            "update_user_role",
            port.update_user_role(1, UserRole::Admin).await,
        ),
        ("delete_user", port.delete_user(1).await),
    ];

    for (name, result) in results {
        assert!(
            matches!(result, Err(DomainError::NotImplemented)),
            "{name} should fail closed with NotImplemented"
        );
    }
    assert!(
        storage.calls().is_empty(),
        "defaults must not reach a required storage method, got {:?}",
        storage.calls()
    );
}

/// Required methods still dispatch to the adapter itself (guards against the
/// defaults above being confused with the required surface).
#[tokio::test]
async fn storage_port_required_methods_reach_the_adapter() {
    let storage = RecordingStorage::new(42);
    let port: &dyn StoragePort = &storage;

    assert_eq!(port.create_feature(&sample_feature()).await.unwrap(), 42);
    assert!(port.list_all_features().await.unwrap().is_empty());
    assert!(port.get_story(9).await.unwrap().is_none());
    port.update_story_status(9, StoryStatus::Done)
        .await
        .unwrap();
    port.delete_module(3).await.unwrap();

    assert_eq!(
        storage.calls(),
        vec![
            "create_feature",
            "list_all_features",
            "get_story",
            "update_story_status",
            "delete_module",
        ]
    );
}

/// `StoryRepository` is blanket-implemented for every `StoragePort`; each
/// method must forward to its exact storage counterpart.
#[tokio::test]
async fn story_repository_blanket_impl_forwards_to_storage() {
    let storage = RecordingStorage::new(5);
    let repo: &dyn StoryRepository = &storage;
    let story = sample_story();

    assert_eq!(repo.create(&story).await.unwrap(), 5);
    assert!(repo.get_by_id(3).await.unwrap().is_none());
    repo.update_status(3, StoryStatus::InProgress)
        .await
        .unwrap();
    assert!(repo.list_by_epic(2).await.unwrap().is_empty());

    assert_eq!(
        storage.calls(),
        vec![
            "create_story",
            "get_story",
            "update_story_status",
            "list_stories_by_epic",
        ]
    );
}

/// Same contract for `EpicRepository`.
#[tokio::test]
async fn epic_repository_blanket_impl_forwards_to_storage() {
    let storage = RecordingStorage::new(6);
    let repo: &dyn EpicRepository = &storage;
    let epic = sample_epic();

    assert_eq!(repo.create(&epic).await.unwrap(), 6);
    assert!(repo.get_by_id(4).await.unwrap().is_none());
    repo.update_status(4, EpicStatus::Active).await.unwrap();
    assert!(repo.list_by_project(1).await.unwrap().is_empty());

    assert_eq!(
        storage.calls(),
        vec![
            "create_epic",
            "get_epic",
            "update_epic_status",
            "list_epics_by_project",
        ]
    );
}

/// `ContentStoragePort` has exactly one default: label filtering is not part of
/// the minimum contract, so it must fail closed.
#[tokio::test]
async fn content_storage_port_default_label_filter_fails_closed() {
    let storage = RecordingContentStorage::new(3);
    let port: &dyn agileplus_domain::ports::ContentStoragePort = &storage;

    assert!(matches!(
        port.list_features_by_label("ui").await,
        Err(DomainError::NotImplemented)
    ));
    assert!(storage.calls().is_empty());

    // The required surface is unaffected.
    assert_eq!(port.create_feature(&sample_feature()).await.unwrap(), 3);
    assert_eq!(storage.calls(), vec!["create_feature"]);
}

fn backlog_item(id: Option<i64>) -> BacklogItem {
    let mut item = BacklogItem::from_triage(
        "Crash on login".to_string(),
        "Steps to reproduce".to_string(),
        Intent::Bug,
        "github".to_string(),
    );
    item.id = id;
    item.feature_slug = Some("auth".to_string());
    item.tags = vec!["p1".to_string()];
    item
}

#[test]
fn triage_ticket_preserves_every_backlog_field() {
    let item = backlog_item(Some(77));
    let expected = (
        item.intent,
        item.priority,
        item.status,
        item.feature_slug.clone(),
        item.tags.clone(),
    );

    let ticket = TriageTicket::from(item);

    assert_eq!(ticket.id, "77");
    assert_eq!(ticket.title, "Crash on login");
    assert_eq!(ticket.description, "Steps to reproduce");
    assert_eq!(ticket.source, "github");
    assert_eq!(
        (ticket.intent, ticket.priority, ticket.status),
        (expected.0, expected.1, expected.2)
    );
    assert_eq!(ticket.feature_slug, expected.3);
    assert_eq!(ticket.tags, expected.4);
    assert_eq!(expected.1, BacklogPriority::High);
    assert_eq!(expected.2, BacklogStatus::New);
}

#[test]
fn triage_ticket_uses_zero_for_an_unsaved_backlog_item() {
    let ticket = TriageTicket::from(backlog_item(None));
    assert_eq!(ticket.id, "0");
}

#[test]
fn triage_ticket_serializes_with_snake_case_enums() {
    let ticket = TriageTicket::from(backlog_item(Some(1)));
    let json = serde_json::to_value(&ticket).unwrap();
    assert_eq!(json["intent"], "bug");
    assert_eq!(json["priority"], "high");
    assert_eq!(json["status"], "new");
    assert_eq!(json["feature_slug"], "auth");
    assert_eq!(json["tags"], serde_json::json!(["p1"]));
    assert_eq!(
        serde_json::from_value::<TriageTicket>(json).unwrap(),
        ticket
    );
}

#[test]
fn triage_error_maps_not_found_and_storage_variants() {
    assert_eq!(
        TriageError::from(DomainError::NotFound("ticket 4".to_string())),
        TriageError::TicketNotFound("ticket 4".to_string())
    );
    assert_eq!(
        TriageError::from(DomainError::Storage("db down".to_string())),
        TriageError::Storage("db down".to_string())
    );
}

#[test]
fn triage_error_maps_every_other_domain_error_to_storage_with_its_message() {
    let cases = [
        DomainError::Validation("bad input".to_string()),
        DomainError::NotImplemented,
        DomainError::Conflict("dup".to_string()),
        DomainError::FeatureNotFound("f".to_string()),
        DomainError::NoOpTransition,
    ];
    for error in cases {
        let expected = format!("triage storage error: {error}");
        let mapped = TriageError::from(error);
        assert_eq!(mapped.to_string(), expected);
        assert!(matches!(mapped, TriageError::Storage(_)));
    }
}

#[test]
fn triage_error_display_messages_are_stable() {
    assert_eq!(
        TriageError::NoTicketAvailable.to_string(),
        "no triage ticket available"
    );
    assert_eq!(
        TriageError::InvalidTicketId("x".to_string()).to_string(),
        "invalid triage ticket id: x"
    );
    assert_eq!(
        TriageError::TicketNotFound("x".to_string()).to_string(),
        "triage ticket not found: x"
    );
    assert_eq!(
        TriageError::Storage("x".to_string()).to_string(),
        "triage storage error: x"
    );
}

#[test]
fn triage_outcome_wire_format_is_snake_case() {
    let accepted = serde_json::to_string(&TriageOutcome::Accepted).unwrap();
    let dismissed = serde_json::to_string(&TriageOutcome::Dismissed).unwrap();
    assert_eq!(accepted, "\"accepted\"");
    assert_eq!(dismissed, "\"dismissed\"");
}
