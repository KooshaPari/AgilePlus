// SPDX-License-Identifier: MIT OR Apache-2.0
//! Integration tests for `agileplus queue`
//! (`commands/queue/{mod,import,parsing,output}.rs`).
//!
//! `run_queue` is generic over `ContentStoragePort`. These tests drive it with
//! the real in-memory SQLite adapter — the same wiring `main.rs` uses for the
//! `Queue` subcommand (`SqliteStorageAdapter::new(db)` then `run_queue`) — so
//! every assertion is about rows the command itself persisted, not about a
//! test double's canned answers.
//!
//! `run_queue` prints its results rather than returning them, so the rendering
//! paths are asserted at the `Result` level (accept/reject) while the
//! observable effects — what was written, what was popped, what is now
//! missing — are asserted through storage. The rendered *shape* of `List`,
//! `Show` and `Pop` output is therefore not covered here.

use agileplus_cli::commands::queue::run_queue;
use agileplus_domain::domain::backlog::{BacklogFilters, BacklogPriority, BacklogStatus, Intent};
use agileplus_domain::ports::ContentStoragePort;
use agileplus_sqlite::SqliteStorageAdapter;

mod support;
use support::queue::{AddSpec, ListSpec, add_item, filters, pop, show, titles, titles_with};
#[tokio::test]
async fn queue_add_persists_item_with_explicit_type_and_priority() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    add_item(
        &storage,
        AddSpec {
            source: "manual",
            feature_slug: Some("alpha"),
            tags: vec!["auth", "ui"],
            intent: Some("bug"),
            priority: Some("critical"),
            ..Default::default()
        }
        .text("Fix the login redirect loop"),
    )
    .await
    .expect("add should succeed");

    let items = storage.list_backlog_items(&filters()).await.unwrap();
    assert_eq!(items.len(), 1, "exactly one row persisted");
    let item = &items[0];
    assert_eq!(item.title, "Fix the login redirect loop");
    assert_eq!(item.intent, Intent::Bug);
    assert_eq!(item.priority, BacklogPriority::Critical);
    assert_eq!(item.status, BacklogStatus::New);
    assert_eq!(item.source, "manual");
    assert_eq!(item.feature_slug.as_deref(), Some("alpha"));
    assert_eq!(item.tags, vec!["auth".to_string(), "ui".to_string()]);
    assert!(item.id.is_some(), "row was assigned an id");
}

#[tokio::test]
async fn queue_add_classifies_when_type_is_omitted() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    add_item(
        &storage,
        AddSpec::default().text("Automatically triaged item"),
    )
    .await
    .expect("add should succeed");

    let items = storage.list_backlog_items(&filters()).await.unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    // No `--type` and no `--priority` were supplied: the classifier picks the
    // intent and the item takes that intent's default priority. An explicit
    // override would have produced a different value (see the test above).
    assert_eq!(item.priority, item.intent.default_priority());
    assert_eq!(item.status, BacklogStatus::New);
    assert_eq!(item.source, "cli", "default source applied");
}

#[tokio::test]
async fn queue_add_with_no_text_is_rejected() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    let err = run_queue(AddSpec::default().build(), &storage)
        .await
        .expect_err("no text and no --from-file must fail");
    assert!(
        err.to_string().contains("No item text provided"),
        "got: {err}"
    );
    assert!(titles(&storage).await.is_empty(), "nothing persisted");
}

#[tokio::test]
async fn queue_add_rejects_invalid_priority_and_persists_nothing() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    let err = add_item(
        &storage,
        AddSpec {
            priority: Some("not-a-priority"),
            ..Default::default()
        }
        .text("Something worth doing"),
    )
    .await
    .expect_err("unknown priority must fail");
    assert!(!err.to_string().is_empty());
    assert!(titles(&storage).await.is_empty(), "no partial write");
}

#[tokio::test]
async fn queue_add_rejects_invalid_type_and_persists_nothing() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    let err = add_item(
        &storage,
        AddSpec {
            intent: Some("not-an-intent"),
            ..Default::default()
        }
        .text("Something worth doing"),
    )
    .await
    .expect_err("unknown type must fail");
    assert!(!err.to_string().is_empty());
    assert!(titles(&storage).await.is_empty(), "no partial write");
}

// ── list ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn queue_list_wires_every_filter_parser_and_both_output_formats() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    add_item(&storage, AddSpec::default().text("seeded item"))
        .await
        .unwrap();

    // Each of these is parsed inside `run_queue`'s `List` arm before the
    // storage query, so a rejected value proves the command wiring, not just
    // the parser.
    for spec in [
        ListSpec {
            intent: Some("nope"),
            ..Default::default()
        },
        ListSpec {
            status: Some("nope"),
            ..Default::default()
        },
        ListSpec {
            priority: Some("nope"),
            ..Default::default()
        },
        ListSpec {
            sort: "nope",
            ..Default::default()
        },
    ] {
        let err = run_queue(spec.build(), &storage)
            .await
            .expect_err("unknown filter value must fail");
        assert!(!err.to_string().is_empty());
    }

    // Accepted paths: valid filters, both output modes, and the non-default
    // sort names.
    for spec in [
        ListSpec {
            intent: Some("bug"),
            output: "json",
            ..Default::default()
        },
        ListSpec {
            status: Some("new"),
            ..Default::default()
        },
        ListSpec {
            priority: Some("high"),
            sort: "age",
            ..Default::default()
        },
        ListSpec {
            sort: "impact",
            limit: 1,
            ..Default::default()
        },
        ListSpec {
            sort: "priority",
            output: "table",
            limit: 20,
            ..Default::default()
        },
    ] {
        run_queue(spec.build(), &storage)
            .await
            .expect("valid List arguments must succeed");
    }
}

#[tokio::test]
async fn queue_list_filters_select_the_expected_rows() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    add_item(
        &storage,
        AddSpec {
            intent: Some("bug"),
            ..Default::default()
        }
        .text("a bug"),
    )
    .await
    .unwrap();
    add_item(
        &storage,
        AddSpec {
            intent: Some("feature"),
            ..Default::default()
        }
        .text("a feature"),
    )
    .await
    .unwrap();

    // Same `BacklogFilters` the `List` arm hands to storage.
    let bugs = storage
        .list_backlog_items(&BacklogFilters {
            intent: Some(Intent::Bug),
            ..filters()
        })
        .await
        .unwrap();
    assert_eq!(bugs.len(), 1);
    assert_eq!(bugs[0].title, "a bug");
    assert_eq!(bugs[0].intent, Intent::Bug);

    let all = storage.list_backlog_items(&filters()).await.unwrap();
    assert_eq!(all.len(), 2, "no filter sees both rows");
}

// ── show ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn queue_show_reads_back_the_item_that_was_added() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    add_item(
        &storage,
        AddSpec {
            description: "steps to reproduce",
            intent: Some("bug"),
            ..Default::default()
        }
        .text("Crash on empty input"),
    )
    .await
    .unwrap();

    let items = storage.list_backlog_items(&filters()).await.unwrap();
    let id = items[0].id.expect("assigned id");

    run_queue(show(id, "table"), &storage)
        .await
        .expect("table output succeeds");
    run_queue(show(id, "json"), &storage)
        .await
        .expect("json output succeeds");
}

#[tokio::test]
async fn queue_show_unknown_id_errors() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    let err = run_queue(show(999_999, "table"), &storage)
        .await
        .expect_err("unknown id must fail");
    assert!(err.to_string().contains("not found"), "got: {err}",);
}

// ── pop ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn queue_pop_marks_highest_priority_item_triaged_first() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    // Titles are deliberately in alphabetical order that differs from the
    // priority order, so the assertion fails if `pop` ever orders by title or
    // insertion order instead of priority.
    for (title, priority) in [
        ("aaa low item", "low"),
        ("bbb critical item", "critical"),
        ("ccc high item", "high"),
    ] {
        add_item(
            &storage,
            AddSpec {
                priority: Some(priority),
                ..Default::default()
            }
            .text(title),
        )
        .await
        .unwrap();
    }
    assert_eq!(titles_with(&storage, BacklogStatus::New).await.len(), 3);

    run_queue(pop(1, "table"), &storage)
        .await
        .expect("pop succeeds");
    assert_eq!(
        titles_with(&storage, BacklogStatus::Triaged).await,
        vec!["bbb critical item"],
        "the critical item was popped first, ahead of the alphabetically earlier low item"
    );
    assert_eq!(
        titles_with(&storage, BacklogStatus::New).await,
        vec!["aaa low item", "ccc high item"]
    );

    run_queue(pop(1, "json"), &storage)
        .await
        .expect("json pop succeeds");
    assert_eq!(
        titles_with(&storage, BacklogStatus::Triaged).await,
        vec!["bbb critical item", "ccc high item"],
        "the high item was popped second"
    );
    assert_eq!(
        titles_with(&storage, BacklogStatus::New).await,
        vec!["aaa low item"],
        "the low item is still queued"
    );
}

#[tokio::test]
async fn queue_pop_transitions_status_and_the_row_remains_readable() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    add_item(&storage, AddSpec::default().text("queued item"))
        .await
        .unwrap();
    let items = storage.list_backlog_items(&filters()).await.unwrap();
    let id = items[0].id.expect("assigned id");

    run_queue(show(id, "table"), &storage)
        .await
        .expect("show works while the row is new");

    // `pop` is a state transition, not a delete: it selects a `new` row,
    // writes `triaged`, and hands the updated row back (repository/backlog.rs).
    run_queue(pop(1, "table"), &storage)
        .await
        .expect("pop succeeds");

    let popped = storage
        .get_backlog_item(id)
        .await
        .unwrap()
        .expect("the row still exists after being popped");
    assert_eq!(popped.status, BacklogStatus::Triaged);
    assert_eq!(popped.title, "queued item");

    run_queue(show(id, "table"), &storage)
        .await
        .expect("a popped item is still showable");
}

#[tokio::test]
async fn queue_pop_on_empty_queue_succeeds_without_side_effects() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();

    run_queue(pop(3, "table"), &storage)
        .await
        .expect("popping an empty queue is not an error");
    assert!(titles(&storage).await.is_empty());
}

#[tokio::test]
async fn queue_pop_stops_once_the_queue_is_exhausted() {
    let storage = SqliteStorageAdapter::in_memory().unwrap();
    add_item(&storage, AddSpec::default().text("only item"))
        .await
        .unwrap();

    run_queue(pop(5, "table"), &storage)
        .await
        .expect("asking for more than available is not an error");

    // Exactly one row moved to `triaged` even though five were requested:
    // the loop breaks as soon as `pop_next_backlog_item` returns `None`.
    assert_eq!(
        titles_with(&storage, BacklogStatus::Triaged).await,
        vec!["only item"]
    );
    assert!(
        titles_with(&storage, BacklogStatus::New).await.is_empty(),
        "no `new` rows remain"
    );
    assert_eq!(
        titles(&storage).await.len(),
        1,
        "popping does not delete rows"
    );

    run_queue(pop(1, "table"), &storage)
        .await
        .expect("popping an already-exhausted queue stays a no-op");
    assert_eq!(
        titles_with(&storage, BacklogStatus::Triaged).await,
        vec!["only item"],
        "nothing else transitioned"
    );
}

// ── import / add --from-file ─────────────────────────────────────────────────
