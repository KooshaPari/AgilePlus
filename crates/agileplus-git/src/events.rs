//! NATS event publishing for git observer events.

use crate::observer::GitEvent;
use serde::Serialize;

/// Subject prefix for all git events.
pub const GIT_SUBJECT_PREFIX: &str = "agileplus.git";

#[derive(Debug, Clone, Serialize)]
pub struct GitEventEnvelope {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub repo_root: String,
    pub payload: serde_json::Value,
}

impl GitEventEnvelope {
    pub fn from_git_event(event: &GitEvent, repo_root: &str) -> Self {
        let (event_type, payload) = match event {
            GitEvent::RefChanged {
                ref_name,
                old_oid,
                new_oid,
            } => (
                "ref_changed".to_string(),
                serde_json::json!({ "ref": ref_name, "old": old_oid, "new": new_oid }),
            ),
            GitEvent::Checkout { branch } => (
                "checkout".to_string(),
                serde_json::json!({ "branch": branch }),
            ),
            GitEvent::Merge { source, target } => (
                "merge".to_string(),
                serde_json::json!({ "source": source, "target": target }),
            ),
            GitEvent::Rebase { branch } => (
                "rebase".to_string(),
                serde_json::json!({ "branch": branch }),
            ),
            GitEvent::WorktreeAdded { path } => (
                "worktree_added".to_string(),
                serde_json::json!({ "path": path.display().to_string() }),
            ),
            GitEvent::WorktreeRemoved { path } => (
                "worktree_removed".to_string(),
                serde_json::json!({ "path": path.display().to_string() }),
            ),
        };

        Self {
            event_type,
            timestamp: chrono::Utc::now(),
            repo_root: repo_root.to_string(),
            payload,
        }
    }

    pub fn nats_subject(&self) -> String {
        format!("{}.{}", GIT_SUBJECT_PREFIX, self.event_type)
    }
}

/// Publishes a git event to NATS. Caller provides the NATS client.
pub async fn publish_git_event(
    nats: &async_nats::Client,
    event: &GitEvent,
    repo_root: &str,
) -> Result<(), async_nats::PublishError> {
    let envelope = GitEventEnvelope::from_git_event(event, repo_root);
    let subject = envelope.nats_subject();
    let payload = serde_json::to_vec(&envelope).unwrap_or_default();
    nats.publish(subject, payload.into()).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn envelope(event: &GitEvent) -> GitEventEnvelope {
        GitEventEnvelope::from_git_event(event, "/repo/root")
    }

    #[test]
    fn ref_changed_maps_to_ref_changed_type() {
        let e = envelope(&GitEvent::RefChanged {
            ref_name: "refs/heads/main".into(),
            old_oid: Some("aaa".into()),
            new_oid: "bbb".into(),
        });
        assert_eq!(e.event_type, "ref_changed");
        assert_eq!(e.payload["ref"], "refs/heads/main");
        assert_eq!(e.payload["old"], "aaa");
        assert_eq!(e.payload["new"], "bbb");
    }

    #[test]
    fn ref_changed_null_old_oid() {
        let e = envelope(&GitEvent::RefChanged {
            ref_name: "refs/heads/new".into(),
            old_oid: None,
            new_oid: "bbb".into(),
        });
        assert!(e.payload["old"].is_null());
    }

    #[test]
    fn checkout_maps_to_checkout_type() {
        let e = envelope(&GitEvent::Checkout {
            branch: "feature/x".into(),
        });
        assert_eq!(e.event_type, "checkout");
        assert_eq!(e.payload["branch"], "feature/x");
    }

    #[test]
    fn merge_payload_has_source_and_target() {
        let e = envelope(&GitEvent::Merge {
            source: "feature/x".into(),
            target: "main".into(),
        });
        assert_eq!(e.event_type, "merge");
        assert_eq!(e.payload["source"], "feature/x");
        assert_eq!(e.payload["target"], "main");
    }

    #[test]
    fn rebase_payload_has_branch() {
        let e = envelope(&GitEvent::Rebase {
            branch: "topic".into(),
        });
        assert_eq!(e.event_type, "rebase");
        assert_eq!(e.payload["branch"], "topic");
    }

    #[test]
    fn worktree_added_payload_has_path_string() {
        let e = envelope(&GitEvent::WorktreeAdded {
            path: PathBuf::from("/repo/.worktrees/f-WP1"),
        });
        assert_eq!(e.event_type, "worktree_added");
        assert_eq!(e.payload["path"], "/repo/.worktrees/f-WP1");
    }

    #[test]
    fn worktree_removed_payload_has_path_string() {
        let e = envelope(&GitEvent::WorktreeRemoved {
            path: PathBuf::from("/repo/.worktrees/f-WP1"),
        });
        assert_eq!(e.event_type, "worktree_removed");
        assert_eq!(e.payload["path"], "/repo/.worktrees/f-WP1");
    }

    #[test]
    fn nats_subject_prefixes_event_type() {
        let e = envelope(&GitEvent::Checkout {
            branch: "main".into(),
        });
        assert_eq!(e.nats_subject(), "agileplus.git.checkout");
        assert!(e.nats_subject().starts_with(GIT_SUBJECT_PREFIX));
    }

    #[test]
    fn envelope_records_repo_root() {
        let e = envelope(&GitEvent::Checkout {
            branch: "main".into(),
        });
        assert_eq!(e.repo_root, "/repo/root");
    }

    #[test]
    fn envelope_timestamp_is_recent() {
        let before = chrono::Utc::now();
        let e = envelope(&GitEvent::Checkout {
            branch: "main".into(),
        });
        let after = chrono::Utc::now();
        assert!(e.timestamp >= before && e.timestamp <= after);
    }

    #[test]
    fn envelope_serde_roundtrip() {
        let e = envelope(&GitEvent::Merge {
            source: "a".into(),
            target: "b".into(),
        });
        let json = serde_json::to_string(&e).unwrap();
        let back: GitEventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.event_type, "merge");
        assert_eq!(back.repo_root, "/repo/root");
        assert_eq!(back.payload["source"], "a");
    }

    #[test]
    fn subject_prefix_constant_value() {
        assert_eq!(GIT_SUBJECT_PREFIX, "agileplus.git");
    }
}
