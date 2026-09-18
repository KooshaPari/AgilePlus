//! Plane.so sync logic with idempotency and conflict detection.
//!
//! Traceability: WP18-T105, T106, T107

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::client::{PlaneClient, PlaneIssue};

/// Sync state for tracking idempotent operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub feature_slug: String,
    pub plane_issue_id: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub content_hash: Option<String>,
    /// Maps WP ID â†’ Plane sub-issue ID
    pub wp_mappings: HashMap<String, String>,
}

impl SyncState {
    pub fn new(feature_slug: String) -> Self {
        Self {
            feature_slug,
            plane_issue_id: None,
            last_synced_at: None,
            content_hash: None,
            wp_mappings: HashMap::new(),
        }
    }
}

/// Plane.so sync adapter.
#[derive(Debug)]
pub struct PlaneSyncAdapter {
    client: PlaneClient,
}

/// Outcome of a sync operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncOutcome {
    Created(String),
    Updated(String),
    Skipped,
    Conflict(String),
}

impl PlaneSyncAdapter {
    pub fn new(client: PlaneClient) -> Self {
        Self { client }
    }

    /// Sync a feature to Plane.so as an issue.
    pub async fn sync_feature(
        &self,
        state: &mut SyncState,
        title: &str,
        description: &str,
    ) -> Result<SyncOutcome> {
        let content_hash = hash_content(&format!("{title}\n{description}"));

        // Check if already synced and unchanged
        if state
            .content_hash
            .as_ref()
            .is_some_and(|existing_hash| existing_hash == &content_hash)
        {
            tracing::debug!("Feature {} unchanged, skipping sync", state.feature_slug);
            return Ok(SyncOutcome::Skipped);
        }

        let issue = PlaneIssue {
            id: None,
            name: title.to_string(),
            description_html: Some(format!("<p>{description}</p>")),
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec!["agileplus".to_string(), "feature".to_string()],
        };

        let outcome =
            if let Some(ref issue_id) = state.plane_issue_id {
                // Check for conflicts before update
                if let Some((remote_name, remote_desc)) = self
                    .client
                    .get_issue(issue_id)
                    .await
                    .ok()
                    .and_then(|remote| {
                        remote
                            .description_html
                            .map(|description| (remote.name, description))
                    })
                {
                    let remote_hash = hash_content(&format!("{remote_name}\n{remote_desc}"));
                    if state.content_hash.as_ref().is_some_and(|our_hash| {
                        remote_hash != *our_hash && content_hash != remote_hash
                    }) {
                        tracing::warn!(
                            "Conflict detected on Plane issue {}: remote was modified",
                            issue_id
                        );
                        return Ok(SyncOutcome::Conflict(issue_id.clone()));
                    }
                }

                let resp = self.client.update_issue(issue_id, &issue).await?;
                SyncOutcome::Updated(resp.id)
            } else {
                let resp = self.client.create_issue(&issue).await?;
                state.plane_issue_id = Some(resp.id.clone());
                SyncOutcome::Created(resp.id)
            };

        state.content_hash = Some(content_hash);
        state.last_synced_at = Some(Utc::now());

        Ok(outcome)
    }

    /// Sync a work package as a sub-issue under the feature issue.
    pub async fn sync_work_package(
        &self,
        state: &mut SyncState,
        wp_id: &str,
        title: &str,
        description: &str,
    ) -> Result<SyncOutcome> {
        let parent_id = state
            .plane_issue_id
            .as_deref()
            .context("cannot sync WP before parent feature is synced")?;

        let issue = PlaneIssue {
            id: None,
            name: format!("[{wp_id}] {title}"),
            description_html: Some(format!("<p>{description}</p>")),
            state: None,
            priority: Some(3),
            parent: Some(parent_id.to_string()),
            labels: vec!["agileplus".to_string(), "work-package".to_string()],
        };

        if let Some(existing_id) = state.wp_mappings.get(wp_id) {
            let resp = self.client.update_issue(existing_id, &issue).await?;
            Ok(SyncOutcome::Updated(resp.id))
        } else {
            let resp = self.client.create_issue(&issue).await?;
            state.wp_mappings.insert(wp_id.to_string(), resp.id.clone());
            Ok(SyncOutcome::Created(resp.id))
        }
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_state_new() {
        let state = SyncState::new("test-feature".to_string());
        assert!(state.plane_issue_id.is_none());
        assert!(state.wp_mappings.is_empty());
    }

    #[test]
    fn hash_deterministic() {
        let h1 = hash_content("hello world");
        let h2 = hash_content("hello world");
        assert_eq!(h1, h2);
        assert_ne!(h1, hash_content("different"));
    }

    #[test]
    fn sync_state_serialization() {
        let mut state = SyncState::new("feat".to_string());
        state.plane_issue_id = Some("issue-123".to_string());
        state
            .wp_mappings
            .insert("WP01".to_string(), "sub-456".to_string());

        let json = serde_json::to_string(&state).unwrap();
        let restored: SyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.plane_issue_id.unwrap(), "issue-123");
        assert_eq!(restored.wp_mappings["WP01"], "sub-456");
    }

    #[test]
    fn sync_state_feature_slug_preserved() {
        let state = SyncState::new("my-feature-slug".to_string());
        assert_eq!(state.feature_slug, "my-feature-slug");
    }

    #[test]
    fn sync_state_default_hash_is_none() {
        let state = SyncState::new("feat".to_string());
        assert!(state.content_hash.is_none());
        assert!(state.last_synced_at.is_none());
    }

    #[test]
    fn hash_empty_string() {
        let h = hash_content("");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn hash_special_chars() {
        let h = hash_content("hello\x00world");
        assert_eq!(h.len(), 64);
    }

    #[test]
    fn sync_outcome_variants() {
        let created = SyncOutcome::Created("id-1".to_string());
        let updated = SyncOutcome::Updated("id-2".to_string());
        let skipped = SyncOutcome::Skipped;
        let conflict = SyncOutcome::Conflict("id-3".to_string());

        assert_ne!(format!("{:?}", created), format!("{:?}", updated));
        assert_ne!(format!("{:?}", skipped), format!("{:?}", conflict));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn hash_content_is_64_hex_chars() {
        let h = hash_content("anything");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_content_distinguishes_concatenations() {
        assert_ne!(hash_content("ab"), hash_content("a b"));
    }

    #[test]
    fn hash_content_handles_unicode() {
        assert_eq!(hash_content("λ"), hash_content("λ"));
        assert_ne!(hash_content("λ"), hash_content("L"));
    }

    #[test]
    fn hash_content_empty_vs_newline() {
        assert_ne!(hash_content(""), hash_content("\n"));
    }

    #[test]
    fn sync_state_clone_is_independent() {
        let mut original = SyncState::new("feat".into());
        original.wp_mappings.insert("WP01".into(), "sub".into());
        let mut clone = original.clone();
        clone.wp_mappings.insert("WP02".into(), "sub2".into());
        assert_eq!(original.wp_mappings.len(), 1);
        assert_eq!(clone.wp_mappings.len(), 2);
    }

    #[test]
    fn sync_state_debug_contains_slug() {
        let state = SyncState::new("my-slug".into());
        assert!(format!("{state:?}").contains("my-slug"));
    }

    #[test]
    fn sync_outcome_clone_and_eq() {
        let outcome = SyncOutcome::Conflict("id".into());
        assert_eq!(outcome.clone(), outcome);
        assert_eq!(format!("{outcome:?}").contains("Conflict"), true);
    }
}

/// End-to-end behaviour of [`PlaneSyncAdapter`] against a stubbed Plane.so.
///
/// The two contract changes worth pinning are: (a) an unchanged content hash
/// short-circuits before any I/O, and (b) a remotely-edited issue whose content
/// matches neither the stored hash nor the new local content is reported as a
/// conflict instead of silently overwriting the remote edit.
#[cfg(test)]
mod adapter_tests {
    use super::*;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const DEAD_ENDPOINT: &str = "http://127.0.0.1:1";
    const WORK_ITEMS_PATH: &str = "/api/v1/workspaces/ws/projects/proj/work-items/";

    fn item_path(id: &str) -> String {
        format!("/api/v1/workspaces/ws/projects/proj/work-items/{id}/")
    }

    fn client(server: &MockServer) -> PlaneClient {
        PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into())
    }

    /// A client whose every request fails, used to prove a code path performs
    /// no I/O at all.
    fn unreachable_client() -> PlaneClient {
        PlaneClient::new(DEAD_ENDPOINT.into(), "key".into(), "ws".into(), "proj".into())
    }

    fn issue_body(id: &str, name: &str, description_html: Option<&str>) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": name,
            "description_html": description_html,
            "state": "backlog",
            "updated_at": "2026-01-01T00:00:00Z"
        })
    }

    async fn mount_json_ok(server: &MockServer, verb: &str, route: &str, body: serde_json::Value) {
        Mock::given(method(verb))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(server)
            .await;
    }

    // -- sync_feature: create / skip / update / conflict --

    #[tokio::test]
    async fn sync_feature_creates_issue_and_records_state() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(WORK_ITEMS_PATH))
            .and(body_partial_json(serde_json::json!({
                "name": "Title",
                "description_html": "<p>Desc</p>"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_body("plane-1", "Title", None)))
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Created("plane-1".into()));
        assert_eq!(state.plane_issue_id.as_deref(), Some("plane-1"));
        assert_eq!(
            state.content_hash.as_deref(),
            Some(hash_content("Title\nDesc").as_str())
        );
        assert!(state.last_synced_at.is_some());
    }

    #[tokio::test]
    async fn sync_feature_skips_unchanged_content_without_io() {
        let adapter = PlaneSyncAdapter::new(unreachable_client());
        let mut state = SyncState::new("feat".into());
        state.content_hash = Some(hash_content("Title\nDesc"));

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Skipped);
        assert!(state.plane_issue_id.is_none());
        assert!(
            state.last_synced_at.is_none(),
            "a skipped sync must not look like a successful one"
        );
    }

    #[tokio::test]
    async fn sync_feature_updates_issue_when_remote_content_matches() {
        let server = MockServer::start().await;
        // The remote copy is byte-identical to what we last synced, so this is
        // an ordinary update rather than a conflict.
        mount_json_ok(
            &server,
            "GET",
            &item_path("plane-1"),
            issue_body("plane-1", "Title", Some("<p>Desc</p>")),
        )
        .await;
        Mock::given(method("PATCH"))
            .and(path(item_path("plane-1")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(issue_body("plane-1", "Title", None)),
            )
            .expect(1)
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("plane-1".into());
        state.content_hash = Some(hash_content("Title\n<p>Desc</p>"));

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Updated("plane-1".into()));
        assert_eq!(
            state.content_hash.as_deref(),
            Some(hash_content("Title\nDesc").as_str()),
            "the stored hash must track the newly pushed content"
        );
    }

    #[tokio::test]
    async fn sync_feature_reports_conflict_when_remote_diverged() {
        let server = MockServer::start().await;
        mount_json_ok(
            &server,
            "GET",
            &item_path("plane-1"),
            issue_body("plane-1", "Title", Some("<p>edited remotely</p>")),
        )
        .await;
        // Nothing may be pushed while a conflict is unresolved.
        Mock::given(method("PATCH"))
            .respond_with(ResponseTemplate::new(200).set_body_json(issue_body("plane-1", "T", None)))
            .expect(0)
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("plane-1".into());
        state.content_hash = Some("stale-local-hash".into());

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Conflict("plane-1".into()));
        assert_eq!(
            state.content_hash.as_deref(),
            Some("stale-local-hash"),
            "a conflict must leave the recorded hash untouched"
        );
        assert!(state.last_synced_at.is_none());
    }

    #[tokio::test]
    async fn sync_feature_ignores_remote_without_description() {
        let server = MockServer::start().await;
        // No description_html means the remote content cannot be hashed, so the
        // conflict check is skipped and the update proceeds.
        mount_json_ok(
            &server,
            "GET",
            &item_path("plane-1"),
            issue_body("plane-1", "Title", None),
        )
        .await;
        Mock::given(method("PATCH"))
            .and(path(item_path("plane-1")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(issue_body("plane-1", "Title", None)),
            )
            .expect(1)
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("plane-1".into());
        state.content_hash = Some("stale-local-hash".into());

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Updated("plane-1".into()));
    }

    #[tokio::test]
    async fn sync_feature_ignores_unreadable_remote_issue() {
        let server = MockServer::start().await;
        // wiremock answers unmatched GETs with 404; the failed probe must not
        // block the update.
        Mock::given(method("PATCH"))
            .and(path(item_path("plane-1")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(issue_body("plane-1", "Title", None)),
            )
            .expect(1)
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("plane-1".into());
        state.content_hash = Some("stale-local-hash".into());

        let outcome = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap();

        assert_eq!(outcome, SyncOutcome::Updated("plane-1".into()));
    }

    #[tokio::test]
    async fn sync_feature_create_failure_leaves_state_untouched() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(WORK_ITEMS_PATH))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());

        let err = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap_err();

        assert!(err.to_string().contains("500"), "unexpected error: {err}");
        assert!(state.plane_issue_id.is_none());
        assert!(state.content_hash.is_none());
    }

    #[tokio::test]
    async fn sync_feature_update_failure_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path(item_path("plane-1")))
            .respond_with(ResponseTemplate::new(503).set_body_string("unavailable"))
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("plane-1".into());

        let err = adapter.sync_feature(&mut state, "Title", "Desc").await.unwrap_err();

        assert!(err.to_string().contains("503"), "unexpected error: {err}");
    }

    // -- sync_work_package: parent precondition / create / update --

    #[tokio::test]
    async fn sync_work_package_requires_a_synced_parent() {
        let adapter = PlaneSyncAdapter::new(unreachable_client());
        let mut state = SyncState::new("feat".into());

        let err = adapter
            .sync_work_package(&mut state, "WP01", "Do it", "Details")
            .await
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("cannot sync WP before parent feature is synced"),
            "unexpected error: {err}"
        );
        assert!(state.wp_mappings.is_empty());
    }

    #[tokio::test]
    async fn sync_work_package_creates_sub_issue_and_records_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(WORK_ITEMS_PATH))
            .and(body_partial_json(serde_json::json!({
                "name": "[WP01] Do it",
                "parent": "parent-1",
                "description_html": "<p>Details</p>"
            })))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(issue_body("sub-1", "[WP01] Do it", None)),
            )
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("parent-1".into());

        let outcome = adapter
            .sync_work_package(&mut state, "WP01", "Do it", "Details")
            .await
            .unwrap();

        assert_eq!(outcome, SyncOutcome::Created("sub-1".into()));
        assert_eq!(state.wp_mappings.get("WP01").map(String::as_str), Some("sub-1"));
    }

    #[tokio::test]
    async fn sync_work_package_updates_when_mapping_exists() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path(item_path("sub-1")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(issue_body("sub-1", "[WP01] Do it", None)),
            )
            .expect(1)
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("parent-1".into());
        state.wp_mappings.insert("WP01".into(), "sub-1".into());

        let outcome = adapter
            .sync_work_package(&mut state, "WP01", "Do it", "Details")
            .await
            .unwrap();

        assert_eq!(outcome, SyncOutcome::Updated("sub-1".into()));
        assert_eq!(state.wp_mappings.get("WP01").map(String::as_str), Some("sub-1"));
    }

    #[tokio::test]
    async fn sync_work_package_failure_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(WORK_ITEMS_PATH))
            .respond_with(ResponseTemplate::new(422).set_body_string("rejected"))
            .mount(&server)
            .await;

        let adapter = PlaneSyncAdapter::new(client(&server));
        let mut state = SyncState::new("feat".into());
        state.plane_issue_id = Some("parent-1".into());

        let err = adapter
            .sync_work_package(&mut state, "WP01", "Do it", "Details")
            .await
            .unwrap_err();

        assert!(err.to_string().contains("422"), "unexpected error: {err}");
        assert!(
            state.wp_mappings.is_empty(),
            "a failed create must not record a mapping"
        );
    }
}
