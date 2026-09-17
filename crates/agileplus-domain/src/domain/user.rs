//! User aggregate — a person who interacts with the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DomainError;

/// Role of a user within the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Member,
    Viewer,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserRole::Admin => "admin",
            UserRole::Member => "member",
            UserRole::Viewer => "viewer",
        };
        write!(f, "{s}")
    }
}

impl FromStr for UserRole {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "member" => Ok(UserRole::Member),
            "viewer" => Ok(UserRole::Viewer),
            _ => Err(DomainError::Validation(format!("unknown UserRole: {s}"))),
        }
    }
}

/// Account status of a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
            UserStatus::Suspended => "suspended",
        };
        write!(f, "{s}")
    }
}

impl FromStr for UserStatus {
    type Err = DomainError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            "suspended" => Ok(UserStatus::Suspended),
            _ => Err(DomainError::Validation(format!("unknown UserStatus: {s}"))),
        }
    }
}

impl UserStatus {
    /// Returns `true` if a transition from `self` to `target` is allowed.
    pub fn can_transition_to(self, target: UserStatus) -> bool {
        use UserStatus::*;
        matches!(
            (self, target),
            (Active, Inactive) | (Active, Suspended) | (Inactive, Active) | (Suspended, Active)
        )
    }
}

/// A user who interacts with the platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    /// Display name — must be non-empty.
    pub display_name: String,
    /// Email address — must contain '@'.
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub avatar_url: Option<String>,
    pub github_login: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Construct a new `User`, enforcing non-empty display name and basic email
    /// validity (must contain `@`).
    pub fn new(display_name: &str, email: &str, role: UserRole) -> Result<Self, DomainError> {
        let display_name = display_name.trim();
        if display_name.is_empty() {
            return Err(DomainError::Validation(
                "display_name must not be empty".to_string(),
            ));
        }
        let email = email.trim();
        if !email.contains('@') {
            return Err(DomainError::Validation(
                "email must contain '@'".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: 0,
            display_name: display_name.to_string(),
            email: email.to_string(),
            role,
            status: UserStatus::Active,
            avatar_url: None,
            github_login: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Attempt a status transition. Returns `Err(DomainError::InvalidTransition)`
    /// if the transition is not permitted.
    pub fn transition_status(&mut self, target: UserStatus) -> Result<(), DomainError> {
        if !self.status.can_transition_to(target) {
            return Err(DomainError::InvalidTransition {
                from: self.status.to_string(),
                to: target.to_string(),
                reason: "not an allowed user status transition".to_string(),
            });
        }
        self.status = target;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_user_construction() {
        let u = User::new("Alice", "alice@example.com", UserRole::Member).unwrap();
        assert_eq!(u.display_name, "Alice");
        assert_eq!(u.email, "alice@example.com");
        assert_eq!(u.status, UserStatus::Active);
        assert_eq!(u.role, UserRole::Member);
    }

    #[test]
    fn rejects_empty_display_name() {
        let err = User::new("  ", "a@b.com", UserRole::Viewer).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn rejects_invalid_email() {
        let err = User::new("Bob", "notanemail", UserRole::Member).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn valid_status_transition() {
        let mut u = User::new("Carol", "carol@x.com", UserRole::Admin).unwrap();
        u.transition_status(UserStatus::Inactive).unwrap();
        assert_eq!(u.status, UserStatus::Inactive);
    }

    #[test]
    fn invalid_status_transition_rejected() {
        let mut u = User::new("Dave", "dave@x.com", UserRole::Member).unwrap();
        // Active -> Suspended is allowed; Suspended -> Inactive is NOT
        u.transition_status(UserStatus::Suspended).unwrap();
        let err = u.transition_status(UserStatus::Inactive).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition { .. }));
    }

    // --- Additional coverage ---

    #[test]
    fn user_role_serde_roundtrip() {
        for role in [UserRole::Admin, UserRole::Member, UserRole::Viewer] {
            let json = serde_json::to_string(&role).unwrap();
            let back: UserRole = serde_json::from_str(&json).unwrap();
            assert_eq!(back, role);
        }
    }

    #[test]
    fn user_status_serde_roundtrip() {
        for s in [UserStatus::Active, UserStatus::Inactive, UserStatus::Suspended] {
            let json = serde_json::to_string(&s).unwrap();
            let back: UserStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, s);
        }
    }

    #[test]
    fn user_serde_roundtrip() {
        let u = User::new("Alice", "a@b.com", UserRole::Admin).unwrap();
        let json = serde_json::to_string(&u).unwrap();
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back.display_name, "Alice");
        assert_eq!(back.email, "a@b.com");
    }

    #[test]
    fn user_role_display_all_variants() {
        assert_eq!(UserRole::Admin.to_string(), "admin");
        assert_eq!(UserRole::Member.to_string(), "member");
        assert_eq!(UserRole::Viewer.to_string(), "viewer");
    }

    #[test]
    fn user_role_from_str_invalid() {
        assert!("superuser".parse::<UserRole>().is_err());
    }

    #[test]
    fn user_status_from_str_invalid() {
        assert!("deleted".parse::<UserStatus>().is_err());
    }

    #[test]
    fn user_status_self_transition_rejected() {
        let mut u = User::new("A", "a@b.com", UserRole::Member).unwrap();
        assert!(u.transition_status(UserStatus::Active).is_err());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    const ROLES: [UserRole; 3] = [UserRole::Admin, UserRole::Member, UserRole::Viewer];
    const STATUSES: [UserStatus; 3] = [
        UserStatus::Active,
        UserStatus::Inactive,
        UserStatus::Suspended,
    ];

    fn allowed(a: UserStatus, b: UserStatus) -> bool {
        matches!(
            (a, b),
            (UserStatus::Active, UserStatus::Inactive)
                | (UserStatus::Active, UserStatus::Suspended)
                | (UserStatus::Inactive, UserStatus::Active)
                | (UserStatus::Suspended, UserStatus::Active)
        )
    }

    #[test]
    fn can_transition_to_full_matrix() {
        for from in STATUSES {
            for to in STATUSES {
                assert_eq!(
                    from.can_transition_to(to),
                    allowed(from, to),
                    "{from:?} -> {to:?}"
                );
            }
        }
    }

    #[test]
    fn transition_status_matrix() {
        for from in STATUSES {
            for to in STATUSES {
                let mut u = User::new("N", "n@x.com", UserRole::Member).unwrap();
                u.status = from;
                let r = u.transition_status(to);
                if allowed(from, to) {
                    assert!(r.is_ok());
                    assert_eq!(u.status, to);
                } else {
                    assert!(r.is_err());
                    assert_eq!(u.status, from);
                    match r.unwrap_err() {
                        DomainError::InvalidTransition { from: f, to: t, .. } => {
                            assert_eq!(f, from.to_string());
                            assert_eq!(t, to.to_string());
                        }
                        other => panic!("wrong error: {other:?}"),
                    }
                }
            }
        }
    }

    #[test]
    fn new_trims_and_defaults() {
        let u = User::new("  Alice  ", " a@b.com ", UserRole::Admin).unwrap();
        assert_eq!(u.display_name, "Alice");
        assert_eq!(u.email, "a@b.com");
        assert_eq!(u.status, UserStatus::Active);
        assert_eq!(u.role, UserRole::Admin);
        assert_eq!(u.id, 0);
        assert!(u.avatar_url.is_none());
        assert!(u.github_login.is_none());
        assert_eq!(u.created_at, u.updated_at);
    }

    #[test]
    fn new_rejects_empty_display_name() {
        for name in ["", "   ", "\t\n"] {
            assert!(matches!(
                User::new(name, "a@b.com", UserRole::Member),
                Err(DomainError::Validation(_))
            ));
        }
    }

    #[test]
    fn email_validation_requires_at_sign() {
        assert!(User::new("N", "a@b.com", UserRole::Admin).is_ok());
        assert!(User::new("N", "@", UserRole::Admin).is_ok());
        assert!(User::new("N", "no-at-sign", UserRole::Admin).is_err());
        assert!(User::new("N", "", UserRole::Admin).is_err());
    }

    #[test]
    fn role_display_roundtrip() {
        for r in ROLES {
            assert_eq!(r.to_string().parse::<UserRole>().unwrap(), r);
        }
    }

    #[test]
    fn status_display_roundtrip() {
        for s in STATUSES {
            assert_eq!(s.to_string().parse::<UserStatus>().unwrap(), s);
        }
    }

    #[test]
    fn from_str_rejects_unknown_and_is_case_sensitive() {
        assert!("SUPERUSER".parse::<UserRole>().is_err());
        assert!("Admin".parse::<UserRole>().is_err());
        assert!("DELETED".parse::<UserStatus>().is_err());
        assert!("Active".parse::<UserStatus>().is_err());
    }

    #[test]
    fn serde_roundtrip_user_and_enums() {
        for r in ROLES {
            let back: UserRole = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
            assert_eq!(back, r);
        }
        for s in STATUSES {
            let back: UserStatus =
                serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
            assert_eq!(back, s);
        }
        let mut u = User::new("N", "n@x.com", UserRole::Viewer).unwrap();
        u.avatar_url = Some("http://a".into());
        u.github_login = Some("gh".into());
        let back: User = serde_json::from_str(&serde_json::to_string(&u).unwrap()).unwrap();
        assert_eq!(back.avatar_url.as_deref(), Some("http://a"));
        assert_eq!(back.github_login.as_deref(), Some("gh"));
    }

    #[test]
    fn wire_strings() {
        assert_eq!(serde_json::to_string(&UserRole::Admin).unwrap(), "\"admin\"");
        assert_eq!(
            serde_json::to_string(&UserStatus::Active).unwrap(),
            "\"active\""
        );
    }

    #[test]
    fn user_clone_and_debug() {
        let u = User::new("N", "n@x.com", UserRole::Member).unwrap();
        let c = u.clone();
        assert_eq!(c.email, u.email);
        assert!(format!("{u:?}").contains("User"));
    }
}
