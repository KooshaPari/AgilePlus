use agileplus_domain::{
    domain::backlog::{BacklogPriority, BacklogStatus},
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
