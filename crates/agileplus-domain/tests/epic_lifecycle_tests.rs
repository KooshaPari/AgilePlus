//! Integration tests for `domain::epic` — construction, lifecycle transitions,
//! and how an epic serializes for downstream consumers.

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::error::DomainError;

const ALL: [EpicStatus; 5] = [
    EpicStatus::Backlog,
    EpicStatus::Active,
    EpicStatus::Review,
    EpicStatus::Done,
    EpicStatus::Cancelled,
];

fn epic() -> Epic {
    Epic::new(11, "Authentication overhaul").expect("valid epic")
}

#[test]
fn new_trims_the_title_and_stamps_defaults() {
    let e = Epic::new(3, "  Big Epic  ").unwrap();
    assert_eq!(e.title, "Big Epic");
    assert_eq!(e.project_id, 3);
    assert_eq!(e.id, 0);
    assert_eq!(e.status, EpicStatus::Backlog);
    assert!(e.description.is_none());
    assert!(e.owner_id.is_none());
    assert!(e.requirement_id.is_none());
    assert_eq!(e.created_at, e.updated_at);
}

#[test]
fn new_rejects_empty_and_whitespace_titles() {
    for title in ["", "   ", "\t\n"] {
        assert!(
            matches!(Epic::new(1, title), Err(DomainError::Validation(_))),
            "title {title:?} must be rejected"
        );
    }
}

#[test]
fn happy_path_advances_backlog_active_review_done() {
    let mut e = epic();
    let created = e.created_at;

    e.transition_status(EpicStatus::Active).unwrap();
    assert_eq!(e.status, EpicStatus::Active);
    e.transition_status(EpicStatus::Review).unwrap();
    assert_eq!(e.status, EpicStatus::Review);
    e.transition_status(EpicStatus::Done).unwrap();
    assert_eq!(e.status, EpicStatus::Done);
    assert!(e.updated_at >= created);
}

#[test]
fn review_can_return_to_active_and_active_can_be_cancelled() {
    let mut bounce = epic();
    bounce.transition_status(EpicStatus::Active).unwrap();
    bounce.transition_status(EpicStatus::Review).unwrap();
    bounce.transition_status(EpicStatus::Active).unwrap();
    assert_eq!(bounce.status, EpicStatus::Active);

    let mut cancelled = epic();
    cancelled.transition_status(EpicStatus::Active).unwrap();
    cancelled.transition_status(EpicStatus::Cancelled).unwrap();
    assert_eq!(cancelled.status, EpicStatus::Cancelled);
}

#[test]
fn transition_matrix_matches_can_transition_to() {
    let allowed = [
        (EpicStatus::Backlog, EpicStatus::Active),
        (EpicStatus::Active, EpicStatus::Review),
        (EpicStatus::Active, EpicStatus::Cancelled),
        (EpicStatus::Review, EpicStatus::Done),
        (EpicStatus::Review, EpicStatus::Active),
    ];
    for from in ALL {
        for to in ALL {
            let permitted = allowed.contains(&(from, to));
            assert_eq!(from.can_transition_to(to), permitted, "{from:?} -> {to:?}");

            let mut e = epic();
            e.status = from;
            let result = e.transition_status(to);
            assert_eq!(result.is_ok(), permitted, "{from:?} -> {to:?}");
            assert_eq!(e.status, if permitted { to } else { from });
        }
    }
}

#[test]
fn invalid_transition_names_the_epic_reason_and_keeps_state() {
    let mut e = epic();
    let error = e.transition_status(EpicStatus::Done).unwrap_err();
    match error {
        DomainError::InvalidTransition { from, to, reason } => {
            assert_eq!(from, "backlog");
            assert_eq!(to, "done");
            assert!(reason.contains("epic"), "reason was {reason:?}");
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
    assert_eq!(e.status, EpicStatus::Backlog);
}

#[test]
fn json_round_trip_preserves_a_transitioned_state_and_fields() {
    let mut e = epic();
    e.description = Some("description".to_string());
    e.owner_id = Some(5);
    e.requirement_id = Some("EP-1".to_string());
    e.transition_status(EpicStatus::Active).unwrap();

    let back: Epic = serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
    assert_eq!(back.status, EpicStatus::Active);
    assert_eq!(back.description.as_deref(), Some("description"));
    assert_eq!(back.owner_id, Some(5));
    assert_eq!(back.requirement_id.as_deref(), Some("EP-1"));
    assert_eq!(back.title, "Authentication overhaul");
}

#[test]
fn status_wire_names_are_lowercase_and_case_sensitive() {
    for status in ALL {
        assert_eq!(
            serde_json::to_string(&status).unwrap(),
            format!("\"{status}\"")
        );
        assert_eq!(
            serde_json::from_str::<EpicStatus>(&format!("\"{status}\"")).unwrap(),
            status
        );
        assert_eq!(status.to_string().parse::<EpicStatus>().unwrap(), status);
    }
    assert!(serde_json::from_str::<EpicStatus>("\"Backlog\"").is_err());
    assert!(serde_json::from_str::<EpicStatus>("\"bogus\"").is_err());
    assert!("Backlog".parse::<EpicStatus>().is_err());
}
