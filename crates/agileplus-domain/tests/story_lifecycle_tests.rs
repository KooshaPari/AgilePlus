//! Integration tests for `domain::story` — construction validation, the status
//! state machine, and serialization of a story as a consumer sees it.

use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;

fn story() -> Story {
    Story::new(7, 42, "User can log in", Some(3)).expect("valid story")
}

#[test]
fn new_stamps_defaults_and_keeps_internal_whitespace() {
    let s = Story::new(1, 2, "  Two  words  ", None).unwrap();
    assert_eq!(s.title, "Two  words");
    assert_eq!(s.status, StoryStatus::Todo);
    assert_eq!(s.id, 0);
    assert_eq!(s.epic_id, 1);
    assert_eq!(s.project_id, 2);
    assert_eq!(s.created_at, s.updated_at);
    assert!(s.description.is_none());
    assert!(s.assignee_id.is_none());
    assert!(s.requirement_id.is_none());
    assert!(s.points.is_none());
}

#[test]
fn new_rejects_empty_titles_and_zero_points() {
    for title in ["", "   ", "\t\n"] {
        assert!(matches!(
            Story::new(1, 1, title, None),
            Err(DomainError::Validation(_))
        ));
    }
    assert!(matches!(
        Story::new(1, 1, "zero", Some(0)),
        Err(DomainError::Validation(_))
    ));
    // Boundary: the largest representable estimate is accepted.
    assert_eq!(
        Story::new(1, 1, "max", Some(u32::MAX)).unwrap().points,
        Some(u32::MAX)
    );
}

#[test]
fn happy_path_lifecycle_advances_and_touches_updated_at() {
    let mut s = story();
    let created = s.created_at;

    s.transition_status(StoryStatus::InProgress).unwrap();
    assert_eq!(s.status, StoryStatus::InProgress);
    s.transition_status(StoryStatus::Review).unwrap();
    assert_eq!(s.status, StoryStatus::Review);
    s.transition_status(StoryStatus::Done).unwrap();
    assert_eq!(s.status, StoryStatus::Done);
    assert!(s.updated_at >= created);
}

#[test]
fn cancellation_is_reachable_from_todo_and_in_progress() {
    let mut from_todo = story();
    from_todo.transition_status(StoryStatus::Cancelled).unwrap();
    assert_eq!(from_todo.status, StoryStatus::Cancelled);

    let mut from_progress = story();
    from_progress
        .transition_status(StoryStatus::InProgress)
        .unwrap();
    from_progress
        .transition_status(StoryStatus::Cancelled)
        .unwrap();
    assert_eq!(from_progress.status, StoryStatus::Cancelled);
}

#[test]
fn blocked_stories_can_return_to_in_progress() {
    let mut s = story();
    s.transition_status(StoryStatus::InProgress).unwrap();
    s.transition_status(StoryStatus::Blocked).unwrap();
    s.transition_status(StoryStatus::InProgress).unwrap();
    assert_eq!(s.status, StoryStatus::InProgress);
}

#[test]
fn review_can_bounce_back_to_in_progress_or_advance_to_done() {
    let mut bounce = story();
    bounce.transition_status(StoryStatus::InProgress).unwrap();
    bounce.transition_status(StoryStatus::Review).unwrap();
    bounce.transition_status(StoryStatus::InProgress).unwrap();
    assert_eq!(bounce.status, StoryStatus::InProgress);

    let mut advance = story();
    advance.transition_status(StoryStatus::InProgress).unwrap();
    advance.transition_status(StoryStatus::Review).unwrap();
    advance.transition_status(StoryStatus::Done).unwrap();
    assert_eq!(advance.status, StoryStatus::Done);
}

#[test]
fn rejected_transitions_leave_the_story_untouched_and_report_endpoints() {
    let mut s = story();
    let before = s.updated_at;

    let error = s.transition_status(StoryStatus::Done).unwrap_err();
    match error {
        DomainError::InvalidTransition { from, to, reason } => {
            assert_eq!(from, "todo");
            assert_eq!(to, "done");
            assert!(!reason.is_empty());
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
    assert_eq!(s.status, StoryStatus::Todo);
    assert_eq!(s.updated_at, before);
}

#[test]
fn terminal_states_have_no_outgoing_transitions() {
    for terminal in [StoryStatus::Done, StoryStatus::Cancelled] {
        for target in [
            StoryStatus::Todo,
            StoryStatus::InProgress,
            StoryStatus::Review,
            StoryStatus::Blocked,
            StoryStatus::Done,
            StoryStatus::Cancelled,
        ] {
            let mut s = story();
            s.status = terminal;
            assert!(
                s.transition_status(target).is_err(),
                "{terminal:?} -> {target:?} must be rejected"
            );
        }
    }
}

#[test]
fn json_round_trip_preserves_a_transitioned_state_and_traceability_key() {
    let mut s = story();
    s.description = Some("narrative".to_string());
    s.assignee_id = Some(99);
    s.requirement_id = Some("FR-001".to_string());
    s.transition_status(StoryStatus::InProgress).unwrap();

    let back: Story = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
    assert_eq!(back.status, StoryStatus::InProgress);
    assert_eq!(back.description.as_deref(), Some("narrative"));
    assert_eq!(back.assignee_id, Some(99));
    assert_eq!(back.requirement_id.as_deref(), Some("FR-001"));
    assert_eq!(back.points, Some(3));
}

#[test]
fn status_wire_names_are_snake_case_and_strict() {
    for status in [
        StoryStatus::Todo,
        StoryStatus::InProgress,
        StoryStatus::Review,
        StoryStatus::Done,
        StoryStatus::Blocked,
        StoryStatus::Cancelled,
    ] {
        let wire = serde_json::to_string(&status).unwrap();
        assert_eq!(serde_json::from_str::<StoryStatus>(&wire).unwrap(), status);
        assert_eq!(status.to_string().parse::<StoryStatus>().unwrap(), status);
    }
    assert!(serde_json::from_str::<StoryStatus>("\"TODO\"").is_err());
    assert!(serde_json::from_str::<StoryStatus>("\"unknown\"").is_err());
    assert!("unknown".parse::<StoryStatus>().is_err());
}
