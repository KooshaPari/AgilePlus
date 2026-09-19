//! Integration tests for `domain::backlog` — backlog item triage defaults,
//! builder helpers, query filters, and the wire format of its enums.

use agileplus_domain::domain::backlog::{
    BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent,
};

#[test]
fn from_triage_derives_priority_and_status_from_the_intent() {
    let cases = [
        (Intent::Bug, BacklogPriority::High),
        (Intent::Feature, BacklogPriority::Medium),
        (Intent::Task, BacklogPriority::Medium),
        (Intent::Idea, BacklogPriority::Low),
        (Intent::Docs, BacklogPriority::Low),
    ];
    for (intent, expected) in cases {
        let item = BacklogItem::from_triage(
            "title".to_string(),
            "description".to_string(),
            intent,
            "github".to_string(),
        );
        assert_eq!(item.priority, expected, "{intent:?}");
        assert_eq!(item.status, BacklogStatus::New);
        assert_eq!(item.intent, intent);
        assert_eq!(item.source, "github");
        assert!(item.id.is_none());
        assert!(item.feature_slug.is_none());
        assert!(item.tags.is_empty());
        assert_eq!(item.created_at, item.updated_at);
    }
}

#[test]
fn builders_attach_tags_and_a_feature_slug() {
    let item = BacklogItem::from_triage(
        "crash".to_string(),
        "app crashes".to_string(),
        Intent::Bug,
        "cli".to_string(),
    )
    .with_tags(vec!["regression".to_string(), "p1".to_string()])
    .with_feature_slug(Some("login".to_string()));

    assert_eq!(item.tags, vec!["regression", "p1"]);
    assert_eq!(item.feature_slug.as_deref(), Some("login"));
}

#[test]
fn filters_partial_json_defaults_optional_fields_to_none() {
    let filters: BacklogFilters = serde_json::from_str(r#"{"sort":"priority"}"#).unwrap();
    assert_eq!(filters.sort, BacklogSort::Priority);
    assert!(filters.intent.is_none());
    assert!(filters.status.is_none());
    assert!(filters.priority.is_none());
    assert!(filters.feature_slug.is_none());
    assert!(filters.source.is_none());
    assert!(filters.limit.is_none());
}

#[test]
fn filters_json_requires_a_sort_mode() {
    // `sort` is not an `Option`, so an absent value is a deserialization error.
    assert!(serde_json::from_str::<BacklogFilters>("{}").is_err());
}

#[test]
fn filters_round_trip_every_field() {
    let filters = BacklogFilters {
        intent: Some(Intent::Bug),
        status: Some(BacklogStatus::Triaged),
        priority: Some(BacklogPriority::Critical),
        feature_slug: Some("auth".to_string()),
        source: Some("github".to_string()),
        sort: BacklogSort::Impact,
        limit: Some(25),
    };
    let back: BacklogFilters =
        serde_json::from_str(&serde_json::to_string(&filters).unwrap()).unwrap();
    assert_eq!(back.intent, Some(Intent::Bug));
    assert_eq!(back.status, Some(BacklogStatus::Triaged));
    assert_eq!(back.priority, Some(BacklogPriority::Critical));
    assert_eq!(back.feature_slug.as_deref(), Some("auth"));
    assert_eq!(back.source.as_deref(), Some("github"));
    assert_eq!(back.sort, BacklogSort::Impact);
    assert_eq!(back.limit, Some(25));
}

#[test]
fn enum_wire_names_are_lowercase_or_snake_case() {
    assert_eq!(serde_json::to_string(&Intent::Task).unwrap(), "\"task\"");
    assert_eq!(
        serde_json::to_string(&BacklogPriority::Medium).unwrap(),
        "\"medium\""
    );
    assert_eq!(
        serde_json::to_string(&BacklogStatus::InProgress).unwrap(),
        "\"in_progress\""
    );
    assert_eq!(serde_json::to_string(&BacklogSort::Age).unwrap(), "\"age\"");
}

#[test]
fn enum_deserialization_rejects_unknown_and_wrong_case_wire_values() {
    assert!(serde_json::from_str::<Intent>("\"BUG\"").is_err());
    assert!(serde_json::from_str::<Intent>("\"wip\"").is_err());
    assert!(serde_json::from_str::<BacklogPriority>("\"urgent\"").is_err());
    assert!(serde_json::from_str::<BacklogStatus>("\"archived\"").is_err());
    assert!(serde_json::from_str::<BacklogSort>("\"date\"").is_err());
    // ... but accepts the canonical spellings.
    assert_eq!(
        serde_json::from_str::<BacklogStatus>("\"in_progress\"").unwrap(),
        BacklogStatus::InProgress
    );
}

#[test]
fn from_str_parsers_accept_documented_spellings_only() {
    assert_eq!("BUG".parse::<Intent>().unwrap(), Intent::Bug);
    assert!("wip".parse::<Intent>().is_err());
    assert_eq!(
        "CRITICAL".parse::<BacklogPriority>().unwrap(),
        BacklogPriority::Critical
    );
    assert!("urgent".parse::<BacklogPriority>().is_err());
    assert_eq!(
        "Done".parse::<BacklogStatus>().unwrap(),
        BacklogStatus::Done
    );
    assert!("archived".parse::<BacklogStatus>().is_err());
    assert_eq!(
        "Impact".parse::<BacklogSort>().unwrap(),
        BacklogSort::Impact
    );
    assert_eq!(BacklogSort::default(), BacklogSort::Age);
}

#[test]
fn backlog_item_round_trips_through_json() {
    let item = BacklogItem::from_triage(
        "docs".to_string(),
        "write docs".to_string(),
        Intent::Docs,
        "manual".to_string(),
    )
    .with_feature_slug(Some("guide".to_string()))
    .with_tags(vec!["docs".to_string()]);

    let back: BacklogItem = serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
    assert_eq!(back.title, "docs");
    assert_eq!(back.intent, Intent::Docs);
    assert_eq!(back.priority, BacklogPriority::Low);
    assert_eq!(back.feature_slug.as_deref(), Some("guide"));
    assert_eq!(back.tags, vec!["docs"]);
}
