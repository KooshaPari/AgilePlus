// SPDX-License-Identifier: MIT OR Apache-2.0
//! Newtype ID wrappers for compile-time type safety.
//!
//! Wrapping primitive IDs in distinct types prevents the most common
//! domain-bug class — accidentally passing a `UserId` where a `FeatureId`
//! is expected (or vice versa). Each wrapper implements:
//! - `Debug`/`Clone`/`PartialEq`/`Eq`/`Hash` for normal usage
//! - `Serialize`/`Deserialize` (transparent — wire format unchanged)
//! - `Display`/`FromStr` so existing string-based APIs still work
//! - `AsRef<str>` for ergonomic borrow handling
//!
//! All wrappers take the inner string at construction; use `::new()`
//! for unchecked construction and `::parse()` if you want validation.
//!
//! Migration is incremental — start using `UserId` for new code and
//! convert existing call sites as you touch them. Plain `i64` / `String`
//! call sites continue to work unchanged.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::DomainError;

macro_rules! newtype_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            /// Construct without validation.
            #[inline]
            pub fn new(inner: impl Into<String>) -> Self {
                Self(inner.into())
            }

            /// Borrow the inner string.
            #[inline]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Take the inner String, consuming self.
            #[inline]
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = DomainError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.is_empty() {
                    Err(DomainError::Validation(format!(
                        "{} must not be empty",
                        stringify!($name)
                    )))
                } else {
                    Ok(Self(s.to_string()))
                }
            }
        }

        impl AsRef<str> for $name {
            #[inline]
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            #[inline]
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            #[inline]
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

// --- PM aggregates ---
newtype_id!(ProjectId, "Project identifier (UUID or short slug).");
newtype_id!(ModuleId, "Module / sub-project identifier.");
newtype_id!(FeatureId, "Feature identifier.");
newtype_id!(EpicId, "Epic identifier.");
newtype_id!(StoryId, "Story identifier.");
newtype_id!(CycleId, "Cycle / sprint identifier.");
newtype_id!(WorkPackageId, "Work package identifier.");
newtype_id!(BacklogId, "Backlog identifier.");
newtype_id!(IntentId, "Intent (strategic goal) identifier.");

// --- People & RBAC ---
newtype_id!(UserId, "User identifier.");
newtype_id!(ApiKeyId, "API key identifier (prefix only).");
newtype_id!(AuditId, "Audit-log entry identifier.");

// --- External / federated ---
newtype_id!(PlaneIssueId, "Upstream Plane.so issue identifier.");
newtype_id!(PlaneStateId, "Upstream Plane.so state identifier.");
newtype_id!(GithubLogin, "GitHub login (owner/repo/actor).");
newtype_id!(BranchName, "Git branch name.");
newtype_id!(CommitSha, "Git commit SHA (40-char hex).");

impl CommitSha {
    /// Validate `s` is a 7- to 40-character hex string (full or short SHA).
    pub fn parse_sha(s: &str) -> Result<Self, DomainError> {
        if !(7..=40).contains(&s.len()) {
            return Err(DomainError::Validation(format!(
                "CommitSha must be 7-40 chars, got {}",
                s.len()
            )));
        }
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DomainError::Validation(
                "CommitSha must be hex characters only".to_string(),
            ));
        }
        Ok(Self(s.to_ascii_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newtype_round_trip() {
        let id = FeatureId::new("F-123");
        assert_eq!(id.to_string(), "F-123");
        let parsed: FeatureId = "F-123".parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn empty_rejected() {
        assert!("".parse::<FeatureId>().is_err());
    }

    #[test]
    fn serde_transparent() {
        let v = serde_json::to_string(&FeatureId::new("F-1")).unwrap();
        assert_eq!(v, "\"F-1\"");
        let back: FeatureId = serde_json::from_str(&v).unwrap();
        assert_eq!(back, FeatureId::new("F-1"));
    }

    #[test]
    fn commit_sha_validation() {
        assert!(CommitSha::parse_sha("0123abc").is_ok());
        assert!(CommitSha::parse_sha("deadbeef1234567890abcdef1234567890abcdef").is_ok());
        assert!(CommitSha::parse_sha("short").is_err()); // too short
        assert!(CommitSha::parse_sha("not a sha!").is_err()); // invalid chars
    }

    #[test]
    fn as_ref_str() {
        let id = ModuleId::new("payments");
        assert_eq!(id.as_ref(), "payments");
        assert_eq!(id.as_str(), "payments");
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::str::FromStr;

    macro_rules! id_tests {
        ($($fn_name:ident : $ty:ty),+ $(,)?) => {
            $(
                #[test]
                fn $fn_name() {
                    let t = <$ty>::new("sample-1");
                    assert_eq!(t.as_str(), "sample-1");
                    assert_eq!(t.to_string(), "sample-1");
                    assert_eq!(t.clone().into_inner(), "sample-1");
                    assert_eq!(<$ty>::from("sample-1"), t);
                    assert_eq!(<$ty>::from("sample-1".to_string()), t);
                    assert_eq!(AsRef::<str>::as_ref(&t), "sample-1");
                    assert_eq!(<$ty>::from_str("sample-1").unwrap(), t);
                    assert!(<$ty>::from_str("").is_err());
                    let json = serde_json::to_string(&t).unwrap();
                    assert_eq!(json, "\"sample-1\"");
                    let back: $ty = serde_json::from_str(&json).unwrap();
                    assert_eq!(back, t);
                    assert!(!format!("{t:?}").is_empty());
                }
            )+
        };
    }

    id_tests! {
        project_id_roundtrip: ProjectId,
        module_id_roundtrip: ModuleId,
        feature_id_roundtrip: FeatureId,
        epic_id_roundtrip: EpicId,
        story_id_roundtrip: StoryId,
        cycle_id_roundtrip: CycleId,
        work_package_id_roundtrip: WorkPackageId,
        backlog_id_roundtrip: BacklogId,
        intent_id_roundtrip: IntentId,
        user_id_roundtrip: UserId,
        api_key_id_roundtrip: ApiKeyId,
        audit_id_roundtrip: AuditId,
        plane_issue_id_roundtrip: PlaneIssueId,
        plane_state_id_roundtrip: PlaneStateId,
        github_login_roundtrip: GithubLogin,
        branch_name_roundtrip: BranchName,
        commit_sha_roundtrip: CommitSha,
    }

    #[test]
    fn empty_parse_error_mentions_type_name() {
        let err = FeatureId::from_str("").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("FeatureId"), "got: {msg}");
        assert!(matches!(err, DomainError::Validation(_)));

        let err = UserId::from_str("").unwrap_err();
        assert!(err.to_string().contains("UserId"));
    }

    #[test]
    fn whitespace_only_is_accepted_by_newtype_parse() {
        // Only empty is rejected; whitespace is a valid (unchecked) id.
        assert!(FeatureId::from_str(" ").is_ok());
    }

    #[test]
    fn ids_are_hashable_across_types() {
        // Same inner value, different newtype wrappers: each is independently
        // hashable in its own typed set.
        let mut feature_set = std::collections::HashSet::new();
        assert!(feature_set.insert(FeatureId::new("x")));
        assert_eq!(feature_set.len(), 1);

        let mut epic_set = std::collections::HashSet::new();
        assert!(epic_set.insert(EpicId::new("x")));
        assert_eq!(epic_set.len(), 1);
    }

    #[test]
    fn identical_ids_are_eq_and_dedup_in_set() {
        let a = FeatureId::new("dup");
        let mut set = std::collections::HashSet::new();
        assert!(set.insert(a.clone()));
        assert!(!set.insert(a));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn commit_sha_parse_valid_short_and_full() {
        assert!(CommitSha::parse_sha("0123abc").is_ok());
        assert!(CommitSha::parse_sha("deadbeef1234567890abcdef1234567890abcdef").is_ok());
        let short = CommitSha::parse_sha("abcdef1").unwrap();
        assert_eq!(short.as_str(), "abcdef1");
        let full = CommitSha::parse_sha("DEADBEEF1234567890abcdef1234567890abcdef").unwrap();
        assert_eq!(full.as_str(), "deadbeef1234567890abcdef1234567890abcdef");
    }

    #[test]
    fn commit_sha_parse_rejects_too_short_and_too_long() {
        assert!(CommitSha::parse_sha("").is_err());
        assert!(CommitSha::parse_sha("abcdef").is_err()); // 6
        let long = "a".repeat(41);
        assert!(CommitSha::parse_sha(&long).is_err());
    }

    #[test]
    fn commit_sha_parse_rejects_non_hex() {
        assert!(CommitSha::parse_sha("not a sha!").is_err());
        assert!(CommitSha::parse_sha("zzzzzzz").is_err());
        assert!(CommitSha::parse_sha("abc-def1").is_err());
    }

    #[test]
    fn commit_sha_parse_boundary_lengths() {
        assert!(CommitSha::parse_sha(&"0".repeat(7)).is_ok());
        assert!(CommitSha::parse_sha(&"0".repeat(40)).is_ok());
        assert!(CommitSha::parse_sha(&"0".repeat(6)).is_err());
        assert!(CommitSha::parse_sha(&"0".repeat(41)).is_err());
    }

    #[test]
    fn commit_sha_error_messages_are_descriptive() {
        let err = CommitSha::parse_sha("abc").unwrap_err();
        assert!(err.to_string().contains("7-40"), "got: {err}");
        let err = CommitSha::parse_sha("ggggggg").unwrap_err();
        assert!(err.to_string().contains("hex"), "got: {err}");
    }

    #[test]
    fn newtype_new_accepts_string_and_str() {
        let from_str = ModuleId::new("m");
        let from_owned = ModuleId::new(String::from("m"));
        assert_eq!(from_str, from_owned);
    }

    #[test]
    fn newtype_into_inner_transfers_ownership() {
        let id = BacklogId::new("b-1");
        let inner: String = id.into_inner();
        assert_eq!(inner, "b-1");
    }
}
