//! Integration tests: backlog queue, epics, and stories repositories.
//!
//! Exercises every public function in `repository::backlog`, `repository::epics`
//! and `repository::stories`, including SQL filter/sort paths and error paths.

use agileplus_domain::domain::{
    backlog::{BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent},
    epic::{Epic, EpicStatus},
    story::{Story, StoryStatus},
};
use agileplus_sqlite::{
    repository::{backlog, epics, stories},
    SqliteStorageAdapter,
};
use rusqlite::Connection;

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn seed_project(conn: &Connection, id: i64) {
    conn.execute(
        "INSERT INTO projects (id, slug, name, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', ?4, ?4)",
        rusqlite::params![
            id,
            format!("proj-{id}"),
            format!("Project {id}"),
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .unwrap();
}

fn item(intent: Intent, title: &str) -> BacklogItem {
    BacklogItem::from_triage(
        title.to_string(),
        format!("desc {title}"),
        intent,
        "test".to_string(),
    )
}

// ---------------------------------------------------------------------------
// Backlog CRUD
// ---------------------------------------------------------------------------

#[test]
fn backlog_create_returns_positive_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "b1")).unwrap();
    assert!(id > 0);
}

#[test]
fn backlog_get_roundtrips_all_fields() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut it = item(Intent::Feature, "feature work");
    it.tags = vec!["x".into(), "y".into()];
    it.feature_slug = Some("auth".into());
    let id = backlog::create_backlog_item(&conn, &it).unwrap();

    let got = backlog::get_backlog_item(&conn, id).unwrap().unwrap();
    assert_eq!(got.id, Some(id));
    assert_eq!(got.title, "feature work");
    assert_eq!(got.description, "desc feature work");
    assert_eq!(got.intent, Intent::Feature);
    assert_eq!(got.priority, BacklogPriority::Medium);
    assert_eq!(got.status, BacklogStatus::New);
    assert_eq!(got.source, "test");
    assert_eq!(got.feature_slug.as_deref(), Some("auth"));
    assert_eq!(got.tags, vec!["x", "y"]);
}

#[test]
fn backlog_get_nonexistent_is_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(backlog::get_backlog_item(&conn, 424242).unwrap().is_none());
}

#[test]
fn backlog_list_default_sort_is_by_age() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Idea, "first")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    backlog::create_backlog_item(&conn, &item(Intent::Bug, "second")).unwrap();

    let all = backlog::list_backlog_items(&conn, &BacklogFilters::default()).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].title, "first");
    assert_eq!(all[1].title, "second");
}

#[test]
fn backlog_list_filters_by_feature_slug() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut tagged = item(Intent::Task, "tagged");
    tagged.feature_slug = Some("alpha".into());
    backlog::create_backlog_item(&conn, &tagged).unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Task, "untagged")).unwrap();

    let filters = BacklogFilters {
        feature_slug: Some("alpha".into()),
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].title, "tagged");
}

#[test]
fn backlog_list_filters_by_source() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut github = item(Intent::Bug, "from gh");
    github.source = "github".into();
    backlog::create_backlog_item(&conn, &github).unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Bug, "from cli")).unwrap();

    let filters = BacklogFilters {
        source: Some("github".into()),
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].title, "from gh");
}

#[test]
fn backlog_list_filters_by_priority() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Bug, "critical-ish")).unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Idea, "low")).unwrap();

    let filters = BacklogFilters {
        priority: Some(BacklogPriority::Low),
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].title, "low");
}

#[test]
fn backlog_list_combined_filters() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut target = item(Intent::Bug, "match");
    target.status = BacklogStatus::Triaged;
    target.priority = BacklogPriority::High;
    backlog::create_backlog_item(&conn, &target).unwrap();
    backlog::create_backlog_item(&conn, &item(Intent::Bug, "not-triaged")).unwrap();

    let filters = BacklogFilters {
        intent: Some(Intent::Bug),
        status: Some(BacklogStatus::Triaged),
        priority: Some(BacklogPriority::High),
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].title, "match");
}

#[test]
fn backlog_list_sort_priority_orders_critical_first() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut low = item(Intent::Idea, "low");
    low.priority = BacklogPriority::Low;
    let mut crit = item(Intent::Bug, "crit");
    crit.priority = BacklogPriority::Critical;
    let mut med = item(Intent::Task, "med");
    med.priority = BacklogPriority::Medium;
    backlog::create_backlog_item(&conn, &low).unwrap();
    backlog::create_backlog_item(&conn, &crit).unwrap();
    backlog::create_backlog_item(&conn, &med).unwrap();

    let filters = BacklogFilters {
        sort: BacklogSort::Priority,
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(
        got.iter().map(|i| i.title.as_str()).collect::<Vec<_>>(),
        vec!["crit", "med", "low"]
    );
}

#[test]
fn backlog_list_sort_impact_matches_priority_ordering() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut high = item(Intent::Bug, "high");
    high.priority = BacklogPriority::High;
    let mut low = item(Intent::Docs, "low");
    low.priority = BacklogPriority::Low;
    backlog::create_backlog_item(&conn, &low).unwrap();
    backlog::create_backlog_item(&conn, &high).unwrap();

    let filters = BacklogFilters {
        sort: BacklogSort::Impact,
        ..BacklogFilters::default()
    };
    let got = backlog::list_backlog_items(&conn, &filters).unwrap();
    assert_eq!(got[0].title, "high");
}

#[test]
fn backlog_list_limit_is_respected() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for i in 0..5 {
        backlog::create_backlog_item(&conn, &item(Intent::Task, &format!("t{i}"))).unwrap();
    }
    let filters = BacklogFilters {
        limit: Some(2),
        ..BacklogFilters::default()
    };
    assert_eq!(backlog::list_backlog_items(&conn, &filters).unwrap().len(), 2);
}

#[test]
fn backlog_list_empty_table_returns_empty() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(backlog::list_backlog_items(&conn, &BacklogFilters::default())
        .unwrap()
        .is_empty());
}

#[test]
fn backlog_update_status_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "b")).unwrap();
    backlog::update_backlog_status(&conn, id, BacklogStatus::Done).unwrap();
    let got = backlog::get_backlog_item(&conn, id).unwrap().unwrap();
    assert_eq!(got.status, BacklogStatus::Done);
}

#[test]
fn backlog_update_priority_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "b")).unwrap();
    backlog::update_backlog_priority(&conn, id, BacklogPriority::Critical).unwrap();
    let got = backlog::get_backlog_item(&conn, id).unwrap().unwrap();
    assert_eq!(got.priority, BacklogPriority::Critical);
}

#[test]
fn backlog_update_status_unknown_id_is_ok_noop() {
    // SQLite UPDATE affecting zero rows is not an error for this repository.
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    backlog::update_backlog_status(&conn, 9999, BacklogStatus::Done).unwrap();
}

#[test]
fn backlog_all_status_variants_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for s in [
        BacklogStatus::New,
        BacklogStatus::Triaged,
        BacklogStatus::InProgress,
        BacklogStatus::Done,
        BacklogStatus::Dismissed,
    ] {
        let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "x")).unwrap();
        backlog::update_backlog_status(&conn, id, s).unwrap();
        assert_eq!(backlog::get_backlog_item(&conn, id).unwrap().unwrap().status, s);
    }
}

#[test]
fn backlog_all_intents_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for intent in [Intent::Bug, Intent::Feature, Intent::Idea, Intent::Task, Intent::Docs] {
        let id = backlog::create_backlog_item(&conn, &item(intent, "x")).unwrap();
        assert_eq!(backlog::get_backlog_item(&conn, id).unwrap().unwrap().intent, intent);
    }
}

#[test]
fn backlog_pop_next_marks_item_triaged() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "top")).unwrap();
    let popped = backlog::pop_next_backlog_item(&conn).unwrap().unwrap();
    assert_eq!(popped.id, Some(id));
    assert_eq!(popped.status, BacklogStatus::Triaged);
    // Persisted too.
    assert_eq!(
        backlog::get_backlog_item(&conn, id).unwrap().unwrap().status,
        BacklogStatus::Triaged
    );
}

#[test]
fn backlog_pop_next_none_when_only_done_items() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Bug, "done")).unwrap();
    backlog::update_backlog_status(&conn, id, BacklogStatus::Done).unwrap();
    assert!(backlog::pop_next_backlog_item(&conn).unwrap().is_none());
}

#[test]
fn backlog_pop_next_is_fifo_within_same_priority() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    // Same intent => same default priority.
    backlog::create_backlog_item(&conn, &item(Intent::Task, "first")).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    backlog::create_backlog_item(&conn, &item(Intent::Task, "second")).unwrap();
    let first = backlog::pop_next_backlog_item(&conn).unwrap().unwrap();
    assert_eq!(first.title, "first");
    let second = backlog::pop_next_backlog_item(&conn).unwrap().unwrap();
    assert_eq!(second.title, "second");
}

#[test]
fn backlog_row_with_invalid_intent_is_storage_error() {
    // Error path: corrupt intent text must surface as DomainError::Storage.
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    conn.execute(
        "INSERT INTO backlog_items
         (title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at)
         VALUES ('bad','','wip','high','new','t',NULL,'[]',?1,?1)",
        rusqlite::params![chrono::Utc::now().to_rfc3339()],
    )
    .unwrap();
    let err = backlog::list_backlog_items(&conn, &BacklogFilters::default()).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::Storage(_)));
}

#[test]
fn backlog_description_defaults_empty_on_raw_insert() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = backlog::create_backlog_item(&conn, &item(Intent::Idea, "x")).unwrap();
    let got = backlog::get_backlog_item(&conn, id).unwrap().unwrap();
    assert!(!got.description.is_empty());
}

// ---------------------------------------------------------------------------
// Epics
// ---------------------------------------------------------------------------

#[test]
fn epic_create_returns_positive_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let e = Epic::new(1, "Epic One").unwrap();
    assert!(epics::create_epic(&conn, &e).unwrap() > 0);
}

#[test]
fn epic_get_by_id_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let mut e = Epic::new(1, "Epic One").unwrap();
    e.description = Some("desc".into());
    e.owner_id = None;
    let id = epics::create_epic(&conn, &e).unwrap();
    let got = epics::get_epic_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.title, "Epic One");
    assert_eq!(got.project_id, 1);
    assert_eq!(got.description.as_deref(), Some("desc"));
    assert_eq!(got.status, EpicStatus::Backlog);
}

#[test]
fn epic_get_by_id_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(epics::get_epic_by_id(&conn, 7).unwrap().is_none());
}

#[test]
fn epic_update_status_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let id = epics::create_epic(&conn, &Epic::new(1, "E").unwrap()).unwrap();
    epics::update_epic_status(&conn, id, EpicStatus::Active).unwrap();
    assert_eq!(
        epics::get_epic_by_id(&conn, id).unwrap().unwrap().status,
        EpicStatus::Active
    );
}

#[test]
fn epic_update_status_unknown_id_is_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = epics::update_epic_status(&conn, 123, EpicStatus::Active).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

#[test]
fn epic_list_by_project_filters() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    seed_project(&conn, 2);
    epics::create_epic(&conn, &Epic::new(1, "A").unwrap()).unwrap();
    epics::create_epic(&conn, &Epic::new(1, "B").unwrap()).unwrap();
    epics::create_epic(&conn, &Epic::new(2, "C").unwrap()).unwrap();

    let p1 = epics::list_epics_by_project(&conn, 1).unwrap();
    assert_eq!(p1.len(), 2);
    let p2 = epics::list_epics_by_project(&conn, 2).unwrap();
    assert_eq!(p2.len(), 1);
    assert!(epics::list_epics_by_project(&conn, 99).unwrap().is_empty());
}

#[test]
fn epic_delete_then_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let id = epics::create_epic(&conn, &Epic::new(1, "E").unwrap()).unwrap();
    epics::delete_epic(&conn, id).unwrap();
    assert!(epics::get_epic_by_id(&conn, id).unwrap().is_none());
    let err = epics::delete_epic(&conn, id).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

#[test]
fn epic_delete_with_story_fails_fk() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let epic_id = epics::create_epic(&conn, &Epic::new(1, "E").unwrap()).unwrap();
    stories::create_story(&conn, &Story::new(epic_id, 1, "S", None).unwrap()).unwrap();
    assert!(epics::delete_epic(&conn, epic_id).is_err());
}

#[test]
fn epic_create_requires_existing_project_fk() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(epics::create_epic(&conn, &Epic::new(12345, "orphan").unwrap()).is_err());
}

#[test]
fn epic_upsert_without_requirement_id_creates() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let id = epics::upsert_epic_by_requirement_id(&conn, &Epic::new(1, "E").unwrap()).unwrap();
    assert!(id > 0);
}

#[test]
fn epic_upsert_with_requirement_id_is_findable_and_idempotent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    let mut e = Epic::new(1, "Traceable").unwrap();
    e.requirement_id = Some("FR-EPIC-1".into());

    let id1 = epics::upsert_epic_by_requirement_id(&conn, &e).unwrap();
    let id2 = epics::upsert_epic_by_requirement_id(&conn, &e).unwrap();
    assert_eq!(id1, id2, "second upsert must reuse the existing row");

    let found = epics::get_epic_by_requirement_id(&conn, "FR-EPIC-1")
        .unwrap()
        .unwrap();
    assert_eq!(found.id, id1);
    assert_eq!(found.requirement_id.as_deref(), Some("FR-EPIC-1"));
}

#[test]
fn epic_get_by_requirement_id_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(epics::get_epic_by_requirement_id(&conn, "nope").unwrap().is_none());
}

// ---------------------------------------------------------------------------
// Stories
// ---------------------------------------------------------------------------

fn seed_epic(conn: &Connection, project: i64) -> i64 {
    seed_project(conn, project);
    epics::create_epic(conn, &Epic::new(project, "E").unwrap()).unwrap()
}

#[test]
fn story_create_and_get_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let mut s = Story::new(epic_id, 1, "As a user", Some(5)).unwrap();
    s.description = Some("narrative".into());
    let id = stories::create_story(&conn, &s).unwrap();
    let got = stories::get_story_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.title, "As a user");
    assert_eq!(got.points, Some(5));
    assert_eq!(got.epic_id, epic_id);
    assert_eq!(got.project_id, 1);
    assert_eq!(got.description.as_deref(), Some("narrative"));
    assert_eq!(got.status, StoryStatus::Todo);
}

#[test]
fn story_points_none_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let id = stories::create_story(&conn, &Story::new(epic_id, 1, "S", None).unwrap()).unwrap();
    assert_eq!(stories::get_story_by_id(&conn, id).unwrap().unwrap().points, None);
}

#[test]
fn story_get_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(stories::get_story_by_id(&conn, 88).unwrap().is_none());
}

#[test]
fn story_update_status_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let id = stories::create_story(&conn, &Story::new(epic_id, 1, "S", None).unwrap()).unwrap();
    stories::update_story_status(&conn, id, StoryStatus::InProgress).unwrap();
    assert_eq!(
        stories::get_story_by_id(&conn, id).unwrap().unwrap().status,
        StoryStatus::InProgress
    );
}

#[test]
fn story_update_status_unknown_id_is_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = stories::update_story_status(&conn, 555, StoryStatus::Done).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

#[test]
fn story_all_statuses_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    for status in [
        StoryStatus::Todo,
        StoryStatus::InProgress,
        StoryStatus::Review,
        StoryStatus::Done,
        StoryStatus::Blocked,
        StoryStatus::Cancelled,
    ] {
        let id =
            stories::create_story(&conn, &Story::new(epic_id, 1, "S", None).unwrap()).unwrap();
        stories::update_story_status(&conn, id, status).unwrap();
        assert_eq!(
            stories::get_story_by_id(&conn, id).unwrap().unwrap().status,
            status
        );
    }
}

#[test]
fn story_list_by_epic_and_project() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_a = seed_epic(&conn, 1);
    seed_project(&conn, 2);
    let epic_b = epics::create_epic(&conn, &Epic::new(2, "B").unwrap()).unwrap();

    stories::create_story(&conn, &Story::new(epic_a, 1, "s1", None).unwrap()).unwrap();
    stories::create_story(&conn, &Story::new(epic_a, 1, "s2", None).unwrap()).unwrap();
    stories::create_story(&conn, &Story::new(epic_b, 2, "s3", None).unwrap()).unwrap();

    assert_eq!(stories::list_stories_by_epic(&conn, epic_a).unwrap().len(), 2);
    assert_eq!(stories::list_stories_by_epic(&conn, epic_b).unwrap().len(), 1);
    assert_eq!(stories::list_stories_by_project(&conn, 1).unwrap().len(), 2);
    assert_eq!(stories::list_stories_by_project(&conn, 2).unwrap().len(), 1);
    assert!(stories::list_stories_by_epic(&conn, 999).unwrap().is_empty());
}

#[test]
fn story_delete_then_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let id = stories::create_story(&conn, &Story::new(epic_id, 1, "S", None).unwrap()).unwrap();
    stories::delete_story(&conn, id).unwrap();
    assert!(stories::get_story_by_id(&conn, id).unwrap().is_none());
    assert!(stories::delete_story(&conn, id).is_err());
}

#[test]
fn story_upsert_with_requirement_id_is_idempotent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let mut s = Story::new(epic_id, 1, "Traceable", Some(3)).unwrap();
    s.requirement_id = Some("FR-STORY-1".into());

    let id1 = stories::upsert_story_by_requirement_id(&conn, &s).unwrap();
    let id2 = stories::upsert_story_by_requirement_id(&conn, &s).unwrap();
    assert_eq!(id1, id2);
    let found = stories::get_story_by_requirement_id(&conn, "FR-STORY-1")
        .unwrap()
        .unwrap();
    assert_eq!(found.id, id1);
    assert_eq!(found.requirement_id.as_deref(), Some("FR-STORY-1"));
    assert_eq!(found.points, Some(3));
}

#[test]
fn story_upsert_without_requirement_id_creates() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let epic_id = seed_epic(&conn, 1);
    let id =
        stories::upsert_story_by_requirement_id(&conn, &Story::new(epic_id, 1, "S", None).unwrap())
            .unwrap();
    assert!(id > 0);
}

#[test]
fn story_get_by_requirement_id_none() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(stories::get_story_by_requirement_id(&conn, "missing").unwrap().is_none());
}

#[test]
fn story_create_requires_existing_epic_fk() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    seed_project(&conn, 1);
    // epic_id 999 does not exist.
    assert!(stories::create_story(&conn, &Story::new(999, 1, "S", None).unwrap()).is_err());
}
