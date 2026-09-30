// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for the `agileplus queue` batch-ingest paths:
//! `queue import FILE` and `queue add --from-file`.
//!
//! Both are driven through `run_queue` with the real in-memory SQLite
//! adapter, so assertions are about rows the command persisted.

use std::path::PathBuf;

use agileplus_cli::commands::queue::{QueueAction, QueueArgs, run_queue};
use agileplus_domain::domain::backlog::{BacklogPriority, Intent};
use agileplus_domain::ports::ContentStoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

mod support;
use support::queue::{AddSpec, add_item, filters, titles, write_ndjson};
#[tokio::test]
async fn queue_import_persists_every_record_and_applies_defaults() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let file = write_ndjson(
        tmp.path(),
        "backlog.ndjson",
        r#"{"title":"first"}
{"title":"second","type":"feature","priority":"low","tags":["x"],"description":"from record"}
"#,
    );

    run_queue(
        QueueArgs {
            action: QueueAction::Import {
                file: file.clone(),
                description: "cli default description".to_string(),
                r#type: Some("bug".to_string()),
                priority: None,
                tags: vec!["cli-tag".to_string()],
                source: "importer".to_string(),
                feature_slug: Some("beta".to_string()),
            },
        },
        &storage,
    )
    .await
    .expect("import should succeed");

    let mut items = storage.list_backlog_items(&filters()).await.unwrap();
    items.sort_by(|a, b| a.title.cmp(&b.title));
    assert_eq!(items.len(), 2, "both records imported");

    let first = &items[0];
    assert_eq!(first.title, "first");
    assert_eq!(
        first.intent,
        Intent::Bug,
        "record had no type so the CLI default applied"
    );
    assert_eq!(
        first.description, "cli default description",
        "empty record description falls back to the CLI default"
    );
    assert_eq!(first.tags, vec!["cli-tag".to_string()]);
    assert_eq!(first.source, "importer");
    assert_eq!(first.feature_slug.as_deref(), Some("beta"));

    let second = &items[1];
    assert_eq!(second.intent, Intent::Feature, "record type wins");
    assert_eq!(
        second.priority,
        BacklogPriority::Low,
        "record priority wins"
    );
    assert_eq!(second.description, "from record", "record description wins");
    assert_eq!(second.tags, vec!["x".to_string()], "record tags win");
}

#[tokio::test]
async fn queue_add_from_file_ingests_a_newline_delimited_file() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let file = write_ndjson(
        tmp.path(),
        "bulk.ndjson",
        "{\"title\":\"alpha\"}\n{\"title\":\"beta\"}\n\n{\"title\":\"gamma\"}\n",
    );

    add_item(
        &storage,
        AddSpec {
            from_file: Some(file),
            ..Default::default()
        },
    )
    .await
    .expect("add --from-file should succeed");

    assert_eq!(
        titles(&storage).await,
        vec!["alpha", "beta", "gamma"],
        "blank lines are skipped and every record is persisted"
    );
}

#[tokio::test]
async fn queue_import_rejects_a_malformed_record() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let file = write_ndjson(tmp.path(), "broken.ndjson", "{\"title\": oops\n");

    let err = run_queue(
        QueueArgs {
            action: QueueAction::Import {
                file,
                description: String::new(),
                r#type: None,
                priority: None,
                tags: Vec::new(),
                source: "cli".to_string(),
                feature_slug: None,
            },
        },
        &storage,
    )
    .await
    .expect_err("malformed NDJSON must fail");
    assert!(
        err.to_string().contains("Invalid backlog import record"),
        "got: {err}"
    );
    assert!(titles(&storage).await.is_empty(), "nothing persisted");
}

#[tokio::test]
async fn queue_import_missing_file_errors() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    let err = run_queue(
        QueueArgs {
            action: QueueAction::Import {
                file: PathBuf::from("/nonexistent/does-not-exist.ndjson"),
                description: String::new(),
                r#type: None,
                priority: None,
                tags: Vec::new(),
                source: "cli".to_string(),
                feature_slug: None,
            },
        },
        &storage,
    )
    .await
    .expect_err("a missing file must fail");
    assert!(!err.to_string().is_empty());
}
