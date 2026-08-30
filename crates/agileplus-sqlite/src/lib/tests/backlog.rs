use agileplus_domain::{
    domain::backlog::{BacklogItem, BacklogPriority, BacklogStatus, Intent},
    ports::ContentStoragePort,
};

use crate::SqliteStorageAdapter;

fn make_adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().expect("in-memory adapter")
}

#[tokio::test]
async fn updating_an_unknown_backlog_item_returns_an_error() {
    let db = make_adapter();

    let result = ContentStoragePort::update_backlog_status(&db, 999, BacklogStatus::Triaged).await;

    assert!(
        result.is_err(),
        "unknown backlog IDs must not report a successful update"
    );
    let priority_result =
        ContentStoragePort::update_backlog_priority(&db, 999, BacklogPriority::Critical).await;
    assert!(
        priority_result.is_err(),
        "unknown backlog IDs must not report a successful priority update"
    );
}

#[tokio::test]
async fn create_and_get_backlog_item_round_trips_queue_metadata() {
    let db = make_adapter();
    let item = BacklogItem::from_triage(
        "Persist queue contract".to_string(),
        "Keep title, body, source, tags, and feature scope.".to_string(),
        Intent::Feature,
        "sqlite-test".to_string(),
    )
    .with_tags(vec!["grpc".to_string(), "queue".to_string()])
    .with_feature_slug(Some("canonical-grpc".to_string()));

    let id = ContentStoragePort::create_backlog_item(&db, &item)
        .await
        .expect("create backlog item");
    let stored = ContentStoragePort::get_backlog_item(&db, id)
        .await
        .expect("get backlog item")
        .expect("created backlog item");

    assert_eq!(stored.id, Some(id));
    assert_eq!(stored.title, item.title);
    assert_eq!(stored.description, item.description);
    assert_eq!(stored.intent, Intent::Feature);
    assert_eq!(stored.priority, BacklogPriority::Medium);
    assert_eq!(stored.status, BacklogStatus::New);
    assert_eq!(stored.source, "sqlite-test");
    assert_eq!(stored.feature_slug.as_deref(), Some("canonical-grpc"));
    assert_eq!(stored.tags, ["grpc", "queue"]);
}

#[tokio::test]
async fn backlog_status_and_priority_updates_persist() {
    let db = make_adapter();
    let item = BacklogItem::from_triage(
        "Escalate queue item".to_string(),
        "Persist both state changes.".to_string(),
        Intent::Idea,
        "sqlite-test".to_string(),
    );
    let id = ContentStoragePort::create_backlog_item(&db, &item)
        .await
        .expect("create backlog item");

    ContentStoragePort::update_backlog_status(&db, id, BacklogStatus::InProgress)
        .await
        .expect("update backlog status");
    ContentStoragePort::update_backlog_priority(&db, id, BacklogPriority::Critical)
        .await
        .expect("update backlog priority");

    let stored = ContentStoragePort::get_backlog_item(&db, id)
        .await
        .expect("get updated backlog item")
        .expect("updated backlog item");
    assert_eq!(stored.status, BacklogStatus::InProgress);
    assert_eq!(stored.priority, BacklogPriority::Critical);
}

#[tokio::test]
async fn pop_next_backlog_item_marks_the_item_triaged_then_empties_the_queue() {
    let db = make_adapter();
    let item = BacklogItem::from_triage(
        "Claim queue item".to_string(),
        "Pop should atomically triage this item.".to_string(),
        Intent::Bug,
        "sqlite-test".to_string(),
    );
    let id = ContentStoragePort::create_backlog_item(&db, &item)
        .await
        .expect("create backlog item");

    let popped = ContentStoragePort::pop_next_backlog_item(&db)
        .await
        .expect("pop backlog item")
        .expect("queued backlog item");
    assert_eq!(popped.id, Some(id));
    assert_eq!(popped.status, BacklogStatus::Triaged);
    assert_eq!(
        ContentStoragePort::get_backlog_item(&db, id)
            .await
            .expect("get popped backlog item")
            .expect("popped backlog item")
            .status,
        BacklogStatus::Triaged
    );
    assert!(
        ContentStoragePort::pop_next_backlog_item(&db)
            .await
            .expect("pop empty backlog")
            .is_none()
    );
}
