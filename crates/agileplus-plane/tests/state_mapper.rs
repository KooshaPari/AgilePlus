//! Integration tests for state_mapper module.
//!
//! Covers: bidirectional mapping, PlaneStateGroup parsing, overrides, edge cases.

use std::collections::HashMap;

use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_plane::state_mapper::{
    PlaneStateGroup, PlaneStateMapper, PlaneStateMapperConfig, StateOverride,
};

// ── PlaneStateGroup::from_str ──────────────────────────────

#[test]
fn parse_backlog() {
    assert!(matches!(
        "backlog".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Backlog
    ));
}

#[test]
fn parse_backlog_case_insensitive() {
    assert!(matches!(
        "Backlog".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Backlog
    ));
    assert!(matches!(
        "BACKLOG".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Backlog
    ));
}

#[test]
fn parse_unstarted() {
    assert!(matches!(
        "unstarted".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Unstarted
    ));
}

#[test]
fn parse_todo_as_unstarted() {
    assert!(matches!(
        "todo".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Unstarted
    ));
}

#[test]
fn parse_started() {
    assert!(matches!(
        "started".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    ));
}

#[test]
fn parse_in_progress_as_started() {
    assert!(matches!(
        "in_progress".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    ));
}

#[test]
fn parse_in_progress_space_as_started() {
    assert!(matches!(
        "in progress".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Started
    ));
}

#[test]
fn parse_completed() {
    assert!(matches!(
        "completed".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Completed
    ));
}

#[test]
fn parse_done_as_completed() {
    assert!(matches!(
        "done".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Completed
    ));
}

#[test]
fn parse_cancelled() {
    assert!(matches!(
        "cancelled".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Cancelled
    ));
}

#[test]
fn parse_canceled_american() {
    assert!(matches!(
        "canceled".parse::<PlaneStateGroup>().unwrap(),
        PlaneStateGroup::Cancelled
    ));
}

#[test]
fn parse_unknown_becomes_unknown_variant() {
    let result = "random_string".parse::<PlaneStateGroup>().unwrap();
    assert!(matches!(result, PlaneStateGroup::Unknown(ref s) if s == "random_string"));
}

// ── PlaneStateGroup::as_str ────────────────────────────────

#[test]
fn as_str_roundtrip_all_known() {
    let cases = [
        (PlaneStateGroup::Backlog, "backlog"),
        (PlaneStateGroup::Unstarted, "unstarted"),
        (PlaneStateGroup::Started, "started"),
        (PlaneStateGroup::Completed, "completed"),
        (PlaneStateGroup::Cancelled, "cancelled"),
    ];
    for (group, expected) in cases {
        assert_eq!(group.as_str(), expected);
        let parsed: PlaneStateGroup = expected.parse().unwrap();
        assert_eq!(parsed.as_str(), expected);
    }
}

#[test]
fn as_str_unknown_preserves_value() {
    let unknown = PlaneStateGroup::Unknown("custom_state".to_string());
    assert_eq!(unknown.as_str(), "custom_state");
}

// ── PlaneStateMapper: map_plane_state (Plane -> FeatureState) ─

#[test]
fn map_backlog_to_created() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("backlog", "Backlog"),
        FeatureState::Created
    );
}

#[test]
fn map_unstarted_to_specified() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("unstarted", "Todo"),
        FeatureState::Specified
    );
}

#[test]
fn map_todo_to_specified() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("todo", "Todo"),
        FeatureState::Specified
    );
}

#[test]
fn map_started_to_implementing() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("started", "In Progress"),
        FeatureState::Implementing
    );
}

#[test]
fn map_completed_to_validated() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("completed", "Done"),
        FeatureState::Validated
    );
}

#[test]
fn map_done_to_validated() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("done", "Done"),
        FeatureState::Validated
    );
}

#[test]
fn map_cancelled_to_validated_with_fallback() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("cancelled", "WontFix"),
        FeatureState::Validated
    );
}

#[test]
fn map_unknown_group_to_created_with_fallback() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("mystery_group", "SomeState"),
        FeatureState::Created
    );
}

#[test]
fn map_case_insensitive_state_group() {
    let mapper = PlaneStateMapper::new();
    assert_eq!(
        mapper.map_plane_state("BACKLOG", "Backlog"),
        FeatureState::Created
    );
    assert_eq!(
        mapper.map_plane_state("Started", "Coding"),
        FeatureState::Implementing
    );
}

// ── PlaneStateMapper: to_plane (FeatureState -> Plane) ──────

#[test]
fn to_plane_created_gives_backlog() {
    let mapper = PlaneStateMapper::new();
    let (group, id) = mapper.to_plane(FeatureState::Created);
    assert_eq!(group, "backlog");
    assert!(id.is_empty());
}

#[test]
fn to_plane_specified_gives_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Specified);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_researched_gives_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Researched);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_planned_gives_unstarted() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Planned);
    assert_eq!(group, "unstarted");
}

#[test]
fn to_plane_implementing_gives_started() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Implementing);
    assert_eq!(group, "started");
}

#[test]
fn to_plane_validated_gives_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Validated);
    assert_eq!(group, "completed");
}

#[test]
fn to_plane_shipped_gives_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Shipped);
    assert_eq!(group, "completed");
}

#[test]
fn to_plane_retrospected_gives_completed() {
    let mapper = PlaneStateMapper::new();
    let (group, _) = mapper.to_plane(FeatureState::Retrospected);
    assert_eq!(group, "completed");
}

// ── PlaneStateMapper: config overrides ──────────────────────

#[test]
fn override_with_group_and_name_takes_precedence() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "started".into(),
            plane_name: Some("review".into()),
            feature_state: FeatureState::Validated,
        }],
        state_id_map: HashMap::new(),
    };
    let mapper = PlaneStateMapper::with_config(config);
    assert_eq!(
        mapper.map_plane_state("started", "review"),
        FeatureState::Validated
    );
}

#[test]
fn override_group_only_falls_back_for_unmatched_name() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "started".into(),
            plane_name: None,
            feature_state: FeatureState::Shipped,
        }],
        state_id_map: HashMap::new(),
    };
    let mapper = PlaneStateMapper::with_config(config);
    assert_eq!(
        mapper.map_plane_state("started", "anything"),
        FeatureState::Shipped
    );
}

#[test]
fn override_group_and_name_beats_group_only() {
    let config = PlaneStateMapperConfig {
        overrides: vec![
            StateOverride {
                plane_group: "started".into(),
                plane_name: None,
                feature_state: FeatureState::Shipped,
            },
            StateOverride {
                plane_group: "started".into(),
                plane_name: Some("review".into()),
                feature_state: FeatureState::Validated,
            },
        ],
        state_id_map: HashMap::new(),
    };
    let mapper = PlaneStateMapper::with_config(config);
    assert_eq!(
        mapper.map_plane_state("started", "review"),
        FeatureState::Validated
    );
    assert_eq!(
        mapper.map_plane_state("started", "coding"),
        FeatureState::Shipped
    );
}

#[test]
fn override_case_insensitive_match() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "BACKLOG".into(),
            plane_name: Some("NEW".into()),
            feature_state: FeatureState::Retrospected,
        }],
        state_id_map: HashMap::new(),
    };
    let mapper = PlaneStateMapper::with_config(config);
    assert_eq!(
        mapper.map_plane_state("backlog", "New"),
        FeatureState::Retrospected
    );
}

#[test]
fn no_override_falls_back_to_default() {
    let config = PlaneStateMapperConfig {
        overrides: vec![StateOverride {
            plane_group: "unknown_group".into(),
            plane_name: None,
            feature_state: FeatureState::Shipped,
        }],
        state_id_map: HashMap::new(),
    };
    let mapper = PlaneStateMapper::with_config(config);
    assert_eq!(
        mapper.map_plane_state("backlog", "Backlog"),
        FeatureState::Created
    );
}

#[test]
fn to_plane_with_config_returns_custom_id() {
    let mut config = PlaneStateMapperConfig::default();
    config.state_id_map.insert(
        FeatureState::Implementing,
        ("started".into(), "uuid-state-123".into()),
    );
    let mapper = PlaneStateMapper::with_config(config);
    let (group, id) = mapper.to_plane(FeatureState::Implementing);
    assert_eq!(group, "started");
    assert_eq!(id, "uuid-state-123");
}

#[test]
fn to_plane_without_config_returns_empty_id() {
    let mapper = PlaneStateMapper::new();
    let (_, id) = mapper.to_plane(FeatureState::Implementing);
    assert!(id.is_empty());
}

#[test]
fn default_config_is_empty() {
    let config = PlaneStateMapperConfig::default();
    assert!(config.overrides.is_empty());
    assert!(config.state_id_map.is_empty());
}

#[test]
fn mapper_default_trait() {
    let mapper = PlaneStateMapper::default();
    assert_eq!(
        mapper.map_plane_state("backlog", "Backlog"),
        FeatureState::Created
    );
}

#[test]
fn mapper_debug_format() {
    let mapper = PlaneStateMapper::new();
    let debug = format!("{:?}", mapper);
    assert!(debug.contains("PlaneStateMapper"));
}
