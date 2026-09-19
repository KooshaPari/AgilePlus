//! Integration tests for `domain::user` — account construction validation, the
//! account status state machine, and role/status serialization.

use agileplus_domain::domain::user::{User, UserRole, UserStatus};
use agileplus_domain::error::DomainError;

const STATUSES: [UserStatus; 3] = [
    UserStatus::Active,
    UserStatus::Inactive,
    UserStatus::Suspended,
];
const ROLES: [UserRole; 3] = [UserRole::Admin, UserRole::Member, UserRole::Viewer];

fn user() -> User {
    User::new("Alice", "alice@example.com", UserRole::Member).expect("valid user")
}

#[test]
fn new_trims_inputs_and_stamps_defaults() {
    let u = User::new("  Alice  ", " alice@example.com ", UserRole::Admin).unwrap();
    assert_eq!(u.display_name, "Alice");
    assert_eq!(u.email, "alice@example.com");
    assert_eq!(u.role, UserRole::Admin);
    assert_eq!(u.status, UserStatus::Active);
    assert_eq!(u.id, 0);
    assert!(u.avatar_url.is_none());
    assert!(u.github_login.is_none());
    assert_eq!(u.created_at, u.updated_at);
}

#[test]
fn new_rejects_blank_names_and_emails_without_an_at_sign() {
    for name in ["", "   ", "\t\n"] {
        assert!(
            matches!(
                User::new(name, "a@b.com", UserRole::Member),
                Err(DomainError::Validation(_))
            ),
            "name {name:?} must be rejected"
        );
    }
    for email in ["", "   ", "no-at-sign"] {
        assert!(
            matches!(
                User::new("N", email, UserRole::Member),
                Err(DomainError::Validation(_))
            ),
            "email {email:?} must be rejected"
        );
    }
}

#[test]
fn email_validation_only_requires_an_at_sign() {
    for email in ["@", "a@b@c", "@domain", "local@"] {
        assert!(
            User::new("N", email, UserRole::Member).is_ok(),
            "email {email:?} should be accepted"
        );
    }
}

#[test]
fn display_name_keeps_unicode_characters() {
    let u = User::new("Åsa Öberg 用户", "asa@example.com", UserRole::Viewer).unwrap();
    assert_eq!(u.display_name, "Åsa Öberg 用户");
}

#[test]
fn status_matrix_matches_can_transition_to_and_keeps_state_on_error() {
    let allowed = [
        (UserStatus::Active, UserStatus::Inactive),
        (UserStatus::Active, UserStatus::Suspended),
        (UserStatus::Inactive, UserStatus::Active),
        (UserStatus::Suspended, UserStatus::Active),
    ];
    for from in STATUSES {
        for to in STATUSES {
            let permitted = allowed.contains(&(from, to));
            assert_eq!(from.can_transition_to(to), permitted, "{from:?} -> {to:?}");

            let mut u = user();
            u.status = from;
            let result = u.transition_status(to);
            assert_eq!(result.is_ok(), permitted, "{from:?} -> {to:?}");
            assert_eq!(u.status, if permitted { to } else { from });
        }
    }
}

#[test]
fn rejected_transition_error_reports_the_account_endpoints() {
    let mut u = user();
    u.transition_status(UserStatus::Suspended).unwrap();
    let error = u.transition_status(UserStatus::Inactive).unwrap_err();
    match error {
        DomainError::InvalidTransition { from, to, reason } => {
            assert_eq!(from, "suspended");
            assert_eq!(to, "inactive");
            assert!(reason.contains("user"), "reason was {reason:?}");
        }
        other => panic!("expected InvalidTransition, got {other:?}"),
    }
    assert_eq!(u.status, UserStatus::Suspended);
}

#[test]
fn json_round_trip_preserves_a_transitioned_status_and_profile_fields() {
    let mut u = user();
    u.avatar_url = Some("https://example.com/avatar.png".to_string());
    u.github_login = Some("alice".to_string());
    u.transition_status(UserStatus::Inactive).unwrap();

    let back: User = serde_json::from_str(&serde_json::to_string(&u).unwrap()).unwrap();
    assert_eq!(back.status, UserStatus::Inactive);
    assert_eq!(
        back.avatar_url.as_deref(),
        Some("https://example.com/avatar.png")
    );
    assert_eq!(back.github_login.as_deref(), Some("alice"));
    assert_eq!(back.display_name, "Alice");
}

#[test]
fn role_and_status_wire_names_are_lowercase_and_strict() {
    for role in ROLES {
        let wire = serde_json::to_string(&role).unwrap();
        assert_eq!(wire, format!("\"{role}\""));
        assert_eq!(serde_json::from_str::<UserRole>(&wire).unwrap(), role);
        assert_eq!(role.to_string().parse::<UserRole>().unwrap(), role);
    }
    for status in STATUSES {
        let wire = serde_json::to_string(&status).unwrap();
        assert_eq!(wire, format!("\"{status}\""));
        assert_eq!(serde_json::from_str::<UserStatus>(&wire).unwrap(), status);
        assert_eq!(status.to_string().parse::<UserStatus>().unwrap(), status);
    }
    assert!(serde_json::from_str::<UserRole>("\"Superuser\"").is_err());
    assert!(serde_json::from_str::<UserStatus>("\"Deleted\"").is_err());
    assert!("Admin".parse::<UserRole>().is_err());
    assert!("Active".parse::<UserStatus>().is_err());
}
