//! Integration tests for `domain::feature` — slug derivation, the feature state
//! machine, and the JSON shape of `spec_hash` and the optional fields.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;

fn feature() -> Feature {
    Feature::new("auth", "Authentication", [0xAB; 32], None)
}

#[test]
fn new_sets_created_state_and_defaults() {
    let f = Feature::new("my-slug", "My Name", [7; 32], None);
    assert_eq!(f.id, 0);
    assert_eq!(f.slug, "my-slug");
    assert_eq!(f.friendly_name, "My Name");
    assert_eq!(f.state, FeatureState::Created);
    assert_eq!(f.spec_hash, [7; 32]);
    assert_eq!(f.target_branch, "main");
    assert!(f.plane_issue_id.is_none());
    assert!(f.plane_state_id.is_none());
    assert!(f.labels.is_empty());
    assert!(f.module_id.is_none());
    assert!(f.project_id.is_none());
    assert!(f.created_at_commit.is_none());
    assert!(f.last_modified_commit.is_none());
    assert_eq!(f.created_at, f.updated_at);

    let custom = Feature::new("s", "S", [0; 32], Some("develop"));
    assert_eq!(custom.target_branch, "develop");
}

#[test]
fn happy_path_walks_every_state_in_order() {
    let mut f = feature();
    let chain = [
        FeatureState::Specified,
        FeatureState::Researched,
        FeatureState::Planned,
        FeatureState::Implementing,
        FeatureState::Validated,
        FeatureState::Shipped,
        FeatureState::Retrospected,
    ];
    for target in chain {
        f.transition(target).expect("allowed transition");
        assert_eq!(f.state, target);
    }
}

#[test]
fn illegal_and_skip_ahead_transitions_report_both_endpoints() {
    let mut f = feature();
    let error = f.transition(FeatureState::Shipped).unwrap_err();
    assert_eq!(error, "invalid transition Created -> Shipped");
    assert_eq!(
        f.state,
        FeatureState::Created,
        "state must not move on error"
    );

    // Backwards transitions are rejected as well.
    f.transition(FeatureState::Specified).unwrap();
    assert!(f.transition(FeatureState::Created).is_err());
}

#[test]
fn retrospected_is_terminal() {
    let mut f = feature();
    f.state = FeatureState::Retrospected;
    for target in [
        FeatureState::Created,
        FeatureState::Specified,
        FeatureState::Retrospected,
        FeatureState::Shipped,
    ] {
        assert!(f.transition(target).is_err(), "{target:?} must be rejected");
    }
    assert_eq!(f.state, FeatureState::Retrospected);
}

#[test]
fn slug_from_name_collapses_separators_and_lowercases() {
    let cases = [
        ("Hello World", "hello-world"),
        ("v2 Release", "v2-release"),
        ("foo@bar!baz#qux", "foo-bar-baz-qux"),
        ("a\tb\nc", "a-b-c"),
        ("--x--y--", "x-y"),
        (" test ", "test"),
        ("CamelCase", "camelcase"),
        ("", ""),
        ("---", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(Feature::slug_from_name(input), expected, "input {input:?}");
    }
}

#[test]
fn slug_from_name_keeps_unicode_alphanumerics() {
    assert_eq!(Feature::slug_from_name("功能 Design"), "功能-design");
}

#[test]
fn json_round_trip_preserves_state_and_optional_metadata() {
    let mut f = feature();
    f.labels = vec!["auth".to_string(), "security".to_string()];
    f.module_id = Some(5);
    f.project_id = Some(1);
    f.plane_issue_id = Some("plane-1".to_string());
    f.created_at_commit = Some("abc1234".to_string());
    f.transition(FeatureState::Specified).unwrap();

    let back: Feature = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
    assert_eq!(back.state, FeatureState::Specified);
    assert_eq!(back.spec_hash, [0xAB; 32]);
    assert_eq!(back.labels, vec!["auth", "security"]);
    assert_eq!(back.module_id, Some(5));
    assert_eq!(back.project_id, Some(1));
    assert_eq!(back.plane_issue_id.as_deref(), Some("plane-1"));
    assert_eq!(back.created_at_commit.as_deref(), Some("abc1234"));
}

#[test]
fn spec_hash_serializes_as_a_thirty_two_element_array() {
    let value = serde_json::to_value(feature()).unwrap();
    let array = value["spec_hash"].as_array().expect("array");
    assert_eq!(array.len(), 32);
    assert!(array.iter().all(|byte| byte.as_u64() == Some(0xAB)));

    // A short array is not a valid 32-byte hash.
    let mut short = serde_json::to_value(feature()).unwrap();
    short["spec_hash"] = serde_json::json!([1, 2, 3]);
    assert!(serde_json::from_value::<Feature>(short).is_err());
}
