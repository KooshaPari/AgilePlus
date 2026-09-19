//! T048: Outbound Sync — push features and work packages to Plane.so.
//!
//! Traceability: WP08-T048, WP06-T031, WP06-T033

pub mod module_cycle;

use agileplus_domain::domain::cycle::Cycle;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::ports::StoragePort;
use anyhow::{Context, Result};
use chrono::Utc;

use crate::client::{PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneIssue};
use crate::state_mapper::PlaneStateMapper;

/// Outbound sync adapter for pushing AgilePlus entities to Plane.so.
#[derive(Debug)]
pub struct OutboundSync {
    client: PlaneClient,
    mapper: PlaneStateMapper,
}

impl OutboundSync {
    pub fn new(client: PlaneClient, mapper: PlaneStateMapper) -> Self {
        Self { client, mapper }
    }

    /// Push a Feature to Plane.so.
    ///
    /// - Creates a new issue if `feature.plane_issue_id` is None.
    /// - Updates the existing issue via PATCH if it exists.
    ///
    /// Returns the Plane.so issue ID.
    pub async fn push_feature(&self, feature: &Feature) -> Result<String> {
        let (_group, state_id) = self.mapper.to_plane(feature.state);
        let state_opt = if state_id.is_empty() {
            None
        } else {
            Some(state_id)
        };

        let issue = PlaneIssue {
            id: None,
            name: feature.friendly_name.clone(),
            description_html: None, // description not on Feature; extend if needed
            state: state_opt,
            priority: Some(2),
            parent: None,
            labels: feature.labels.clone(),
        };

        let issue_id = if let Some(ref existing_id) = feature.plane_issue_id {
            let resp = self
                .client
                .update_issue(existing_id, &issue)
                .await
                .with_context(|| format!("updating Plane issue {existing_id}"))?;
            tracing::info!(
                feature_slug = feature.slug,
                plane_issue_id = resp.id,
                "updated Plane.so issue"
            );
            resp.id
        } else {
            let resp =
                self.client.create_issue(&issue).await.with_context(|| {
                    format!("creating Plane issue for feature {}", feature.slug)
                })?;
            tracing::info!(
                feature_slug = feature.slug,
                plane_issue_id = resp.id,
                "created Plane.so issue"
            );
            resp.id
        };

        Ok(issue_id)
    }

    /// Push a work package as a sub-issue under a parent Plane.so issue.
    ///
    /// `parent_plane_id` is the Plane.so issue ID of the parent feature.
    /// `wp_plane_id` is the existing Plane sub-issue ID, if any.
    /// Returns the Plane.so sub-issue ID.
    pub async fn push_work_package(
        &self,
        wp_id: &str,
        title: &str,
        description: Option<&str>,
        labels: &[String],
        parent_plane_id: &str,
        wp_plane_id: Option<&str>,
    ) -> Result<String> {
        let desc_html = description.map(|d| format!("<p>{d}</p>"));
        let issue = PlaneIssue {
            id: None,
            name: format!("[{wp_id}] {title}"),
            description_html: desc_html,
            state: None,
            priority: Some(3),
            parent: Some(parent_plane_id.to_string()),
            labels: labels.to_vec(),
        };

        let issue_id = if let Some(existing_id) = wp_plane_id {
            let resp = self
                .client
                .update_issue(existing_id, &issue)
                .await
                .with_context(|| format!("updating Plane sub-issue {existing_id}"))?;
            tracing::info!(
                wp_id,
                plane_issue_id = resp.id,
                "updated Plane.so sub-issue"
            );
            resp.id
        } else {
            let resp = self
                .client
                .create_issue(&issue)
                .await
                .with_context(|| format!("creating Plane sub-issue for WP {wp_id}"))?;
            tracing::info!(
                wp_id,
                plane_issue_id = resp.id,
                "created Plane.so sub-issue"
            );
            resp.id
        };

        Ok(issue_id)
    }
}

// -- Module & Cycle outbound push (WP06-T031) --

/// Push a newly created or updated Module to Plane.so.
/// Stores a sync_mappings row with entity_type = "module".
pub async fn push_module<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    module: &Module,
) -> Result<()> {
    let existing = storage
        .get_sync_mapping("module", module.id)
        .await
        .context("looking up module sync mapping")?;

    let req = PlaneCreateModuleRequest {
        name: module.friendly_name.clone(),
        description: module.description.clone(),
    };

    if let Some(mapping) = existing {
        client
            .update_module(&mapping.plane_issue_id, &req)
            .await
            .with_context(|| format!("updating Plane module {}", mapping.plane_issue_id))?;
        // Update last_synced_at
        let updated = SyncMapping {
            last_synced_at: Utc::now(),
            ..mapping
        };
        storage
            .upsert_sync_mapping(&updated)
            .await
            .context("updating sync mapping timestamp")?;
    } else {
        let resp = client
            .create_module(&req)
            .await
            .context("creating Plane module")?;
        let mapping = SyncMapping::new("module", module.id, &resp.id, "");
        storage
            .upsert_sync_mapping(&mapping)
            .await
            .context("storing module sync mapping")?;
        tracing::info!(
            module_id = module.id,
            plane_module_id = resp.id,
            "synced module to Plane.so"
        );
    }

    Ok(())
}

/// Push a newly created or updated Cycle to Plane.so.
/// Stores a sync_mappings row with entity_type = "cycle".
pub async fn push_cycle<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    cycle: &Cycle,
) -> Result<()> {
    let existing = storage
        .get_sync_mapping("cycle", cycle.id)
        .await
        .context("looking up cycle sync mapping")?;

    let req = PlaneCreateCycleRequest {
        name: cycle.name.clone(),
        description: cycle.description.clone(),
        start_date: cycle.start_date.to_string(),
        end_date: cycle.end_date.to_string(),
    };

    if let Some(mapping) = existing {
        client
            .update_cycle(&mapping.plane_issue_id, &req)
            .await
            .with_context(|| format!("updating Plane cycle {}", mapping.plane_issue_id))?;
        let updated = SyncMapping {
            last_synced_at: Utc::now(),
            ..mapping
        };
        storage
            .upsert_sync_mapping(&updated)
            .await
            .context("updating sync mapping timestamp")?;
    } else {
        let resp = client
            .create_cycle(&req)
            .await
            .context("creating Plane cycle")?;
        let mapping = SyncMapping::new("cycle", cycle.id, &resp.id, "");
        storage
            .upsert_sync_mapping(&mapping)
            .await
            .context("storing cycle sync mapping")?;
        tracing::info!(
            cycle_id = cycle.id,
            plane_cycle_id = resp.id,
            "synced cycle to Plane.so"
        );
    }

    Ok(())
}

/// Delete a Module from Plane.so and remove its sync mapping.
pub async fn push_module_delete<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    module_id: i64,
) -> Result<()> {
    let mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping for delete")?;

    if let Some(m) = mapping {
        client
            .delete_module(&m.plane_issue_id)
            .await
            .with_context(|| format!("deleting Plane module {}", m.plane_issue_id))?;
        storage
            .delete_sync_mapping("module", module_id)
            .await
            .context("removing module sync mapping")?;
        tracing::info!(
            module_id,
            plane_module_id = m.plane_issue_id,
            "deleted Plane module"
        );
    }

    Ok(())
}

/// Delete a Cycle from Plane.so and remove its sync mapping.
pub async fn push_cycle_delete<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    cycle_id: i64,
) -> Result<()> {
    let mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping for delete")?;

    if let Some(m) = mapping {
        client
            .delete_cycle(&m.plane_issue_id)
            .await
            .with_context(|| format!("deleting Plane cycle {}", m.plane_issue_id))?;
        storage
            .delete_sync_mapping("cycle", cycle_id)
            .await
            .context("removing cycle sync mapping")?;
        tracing::info!(
            cycle_id,
            plane_cycle_id = m.plane_issue_id,
            "deleted Plane cycle"
        );
    }

    Ok(())
}

// -- Assignment sync (WP06-T033) --

/// When a Feature is assigned to a Module, sync the Plane issue-to-module link.
pub async fn push_feature_module_assignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let module_mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping")?;

    match (feature_mapping, module_mapping) {
        (Some(fm), Some(mm)) => {
            client
                .add_issue_to_module(&mm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane issue {} to module {}",
                        fm.plane_issue_id, mm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                module_id,
                "synced feature-to-module assignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                module_id,
                "skipping feature-module assignment sync: one or both sides not mapped"
            );
        }
    }

    Ok(())
}

/// When a Feature is assigned to a Cycle, sync the Plane issue-to-cycle link.
pub async fn push_feature_cycle_assignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let cycle_mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping")?;

    match (feature_mapping, cycle_mapping) {
        (Some(fm), Some(cm)) => {
            client
                .add_issue_to_cycle(&cm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane issue {} to cycle {}",
                        fm.plane_issue_id, cm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                cycle_id,
                "synced feature-to-cycle assignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                cycle_id,
                "skipping feature-cycle assignment sync: one or both sides not mapped"
            );
        }
    }

    Ok(())
}

/// When a Feature is removed from a Module, remove the Plane work-item link.
pub async fn push_feature_module_unassignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let module_mapping = storage
        .get_sync_mapping("module", module_id)
        .await
        .context("looking up module sync mapping")?;

    match (feature_mapping, module_mapping) {
        (Some(fm), Some(mm)) => {
            client
                .delete_work_item_from_module(&mm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "removing Plane work item {} from module {}",
                        fm.plane_issue_id, mm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                module_id,
                "synced feature-to-module unassignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                module_id,
                "skipping feature-module unassignment sync: one or both sides not mapped"
            );
        }
    }

    Ok(())
}

/// When a Feature is removed from a Cycle, remove the Plane work-item link.
pub async fn push_feature_cycle_unassignment<S: StoragePort>(
    client: &PlaneClient,
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> Result<()> {
    let feature_mapping = storage
        .get_sync_mapping("feature", feature_id)
        .await
        .context("looking up feature sync mapping")?;
    let cycle_mapping = storage
        .get_sync_mapping("cycle", cycle_id)
        .await
        .context("looking up cycle sync mapping")?;

    match (feature_mapping, cycle_mapping) {
        (Some(fm), Some(cm)) => {
            client
                .delete_work_item_from_cycle(&cm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "removing Plane work item {} from cycle {}",
                        fm.plane_issue_id, cm.plane_issue_id
                    )
                })?;
            tracing::info!(
                feature_id,
                cycle_id,
                "synced feature-to-cycle unassignment to Plane"
            );
        }
        _ => {
            tracing::debug!(
                feature_id,
                cycle_id,
                "skipping feature-cycle unassignment sync: one or both sides not mapped"
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mapper::PlaneStateMapper;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn outbound_sync_constructs() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "slug".into(),
            "project".into(),
        );
        let mapper = PlaneStateMapper::new();
        let _sync = OutboundSync::new(client, mapper);
    }

    fn sample_feature() -> Feature {
        let mut f = Feature::new("test-feature", "Test Feature", [0u8; 32], None);
        f.id = 1;
        f.labels = vec!["bug".to_string()];
        f
    }

    #[tokio::test]
    async fn push_feature_creates_new_issue() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-uuid-1",
                "name": "Test Feature"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());
        let feature = sample_feature();

        let issue_id = sync.push_feature(&feature).await.unwrap();
        assert_eq!(issue_id, "plane-uuid-1");
    }

    #[tokio::test]
    async fn push_feature_updates_existing_issue() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/existing-id/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "existing-id",
                "name": "Test Feature"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());
        let mut feature = sample_feature();
        feature.plane_issue_id = Some("existing-id".to_string());

        let issue_id = sync.push_feature(&feature).await.unwrap();
        assert_eq!(issue_id, "existing-id");
    }

    #[tokio::test]
    async fn push_feature_create_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());
        let feature = sample_feature();

        let result = sync.push_feature(&feature).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn push_feature_update_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/existing-id/"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());
        let mut feature = sample_feature();
        feature.plane_issue_id = Some("existing-id".to_string());

        let result = sync.push_feature(&feature).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn push_work_package_creates_new_sub_issue() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "wp-uuid-1",
                "name": "[WP01] Test WP"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());

        let id = sync
            .push_work_package(
                "WP01",
                "Test WP",
                Some("description"),
                &["label".to_string()],
                "parent-plane-id",
                None,
            )
            .await
            .unwrap();
        assert_eq!(id, "wp-uuid-1");
    }

    #[tokio::test]
    async fn push_work_package_updates_existing() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wp-existing/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "wp-existing",
                "name": "[WP02] Updated"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());

        let id = sync
            .push_work_package(
                "WP02",
                "Updated",
                None,
                &[],
                "parent-id",
                Some("wp-existing"),
            )
            .await
            .unwrap();
        assert_eq!(id, "wp-existing");
    }

    #[tokio::test]
    async fn push_work_package_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .respond_with(ResponseTemplate::new(422).set_body_string("unprocessable"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());

        let result = sync
            .push_work_package("WP03", "Fail", None, &[], "parent", None)
            .await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::mock_storage::MockStoragePort;
    use crate::state_mapper::{PlaneStateMapper, PlaneStateMapperConfig};
    use std::collections::HashMap;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn feature() -> Feature {
        let mut f = Feature::new("feat", "Feature", [0u8; 32], None);
        f.id = 1;
        f.labels = vec!["bug".to_string()];
        f
    }

    // -- assignment happy paths --

    #[tokio::test]
    async fn module_assignment_links_when_both_mapped() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/m1/module-issues/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("module", 2, "m1");
        push_feature_module_assignment(&client, &storage, 1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn module_assignment_skips_when_feature_unmapped() {
        let client = PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 2, "m1");
        push_feature_module_assignment(&client, &storage, 1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn module_assignment_skips_when_module_unmapped() {
        let client = PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("feature", 1, "plane-feat");
        push_feature_module_assignment(&client, &storage, 1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn module_assignment_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/m1/module-issues/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("err"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("module", 2, "m1");
        assert!(push_feature_module_assignment(&client, &storage, 1, 2).await.is_err());
    }

    #[tokio::test]
    async fn module_unassignment_removes_link() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/m1/module-issues/plane-feat/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("module", 2, "m1");
        push_feature_module_unassignment(&client, &storage, 1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn module_unassignment_skips_when_missing() {
        let client = PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        push_feature_module_unassignment(&client, &storage, 1, 2).await.unwrap();
    }

    #[tokio::test]
    async fn module_unassignment_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/m1/module-issues/plane-feat/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("err"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("module", 2, "m1");
        assert!(push_feature_module_unassignment(&client, &storage, 1, 2).await.is_err());
    }

    #[tokio::test]
    async fn cycle_assignment_links_when_both_mapped() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c1/cycle-issues/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("cycle", 3, "c1");
        push_feature_cycle_assignment(&client, &storage, 1, 3).await.unwrap();
    }

    #[tokio::test]
    async fn cycle_assignment_skips_when_missing() {
        let client = PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        push_feature_cycle_assignment(&client, &storage, 1, 3).await.unwrap();
    }

    #[tokio::test]
    async fn cycle_assignment_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c1/cycle-issues/"))
            .respond_with(ResponseTemplate::new(409).set_body_string("conflict"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("cycle", 3, "c1");
        assert!(push_feature_cycle_assignment(&client, &storage, 1, 3).await.is_err());
    }

    #[tokio::test]
    async fn cycle_unassignment_removes_link() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c1/cycle-issues/plane-feat/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("cycle", 3, "c1");
        push_feature_cycle_unassignment(&client, &storage, 1, 3).await.unwrap();
    }

    #[tokio::test]
    async fn cycle_unassignment_skips_when_missing() {
        let client = PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        push_feature_cycle_unassignment(&client, &storage, 1, 3).await.unwrap();
    }

    #[tokio::test]
    async fn cycle_unassignment_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/c1/cycle-issues/plane-feat/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("err"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "plane-feat")
            .with_sync_mapping("cycle", 3, "c1");
        assert!(push_feature_cycle_unassignment(&client, &storage, 1, 3).await.is_err());
    }

    // -- mapper-driven payload shaping --

    #[tokio::test]
    async fn push_feature_sends_configured_state_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
            .and(wiremock::matchers::body_partial_json(serde_json::json!({
                "state": "state-uuid"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-1", "name": "Feature"
            })))
            .mount(&server)
            .await;

        let mut config = PlaneStateMapperConfig::default();
        config.state_id_map = HashMap::new();
        config.state_id_map.insert(
            agileplus_domain::domain::state_machine::FeatureState::Created,
            ("backlog".into(), "state-uuid".into()),
        );
        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::with_config(config));
        assert_eq!(sync.push_feature(&feature()).await.unwrap(), "plane-1");
    }

    #[tokio::test]
    async fn outbound_sync_debug_impl() {
        let client = PlaneClient::new("http://x".into(), "k".into(), "w".into(), "p".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());
        assert!(format!("{sync:?}").contains("OutboundSync"));
    }

    // -- module / cycle push (the copies re-exported as the live API) --
    //
    // `outbound::push_module` and friends are what `runtime::maybe_*_from_env`
    // and the `pub use` in `lib.rs` resolve to; the identically-named
    // functions in `outbound::module_cycle` are a separate copy with their own
    // tests, so the branches below were previously reached only from
    // `runtime`'s no-Plane and create-failure paths.

    fn module(id: i64, name: &str) -> Module {
        let mut module = Module::new(name, None);
        module.id = id;
        module
    }

    fn cycle(id: i64, name: &str) -> Cycle {
        use agileplus_domain::domain::cycle::CycleState;
        use chrono::NaiveDate;

        Cycle {
            id,
            name: name.to_string(),
            description: None,
            state: CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn push_module_creates_module_and_records_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-mod-1",
                "name": "Auth"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();

        push_module(&client, &storage, &module(7, "Auth")).await.unwrap();

        let mapping = storage.get_sync_mapping("module", 7).await.unwrap().unwrap();
        assert_eq!(mapping.entity_type, "module");
        assert_eq!(mapping.entity_id, 7);
        assert_eq!(mapping.plane_issue_id, "plane-mod-1");
    }

    #[tokio::test]
    async fn push_module_patches_mapped_module_and_keeps_the_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/plane-mod-9/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-mod-9",
                "name": "Auth v2"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 7, "plane-mod-9");
        let before = storage
            .get_sync_mapping("module", 7)
            .await
            .unwrap()
            .unwrap()
            .last_synced_at;

        push_module(&client, &storage, &module(7, "Auth v2")).await.unwrap();

        let after = storage.get_sync_mapping("module", 7).await.unwrap().unwrap();
        assert_eq!(
            after.plane_issue_id, "plane-mod-9",
            "an update must reuse the existing Plane module, not create a second one"
        );
        assert!(after.last_synced_at >= before);
    }

    #[tokio::test]
    async fn push_module_create_failure_is_contextual() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();

        let err = push_module(&client, &storage, &module(7, "Auth"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("creating Plane module"),
            "unexpected error: {err}"
        );
        assert!(
            storage.get_sync_mapping("module", 7).await.unwrap().is_none(),
            "a failed push must not record a mapping"
        );
    }

    #[tokio::test]
    async fn push_cycle_creates_cycle_and_records_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-cyc-1",
                "name": "Sprint 1"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();

        push_cycle(&client, &storage, &cycle(3, "Sprint 1")).await.unwrap();

        let mapping = storage.get_sync_mapping("cycle", 3).await.unwrap().unwrap();
        assert_eq!(mapping.entity_type, "cycle");
        assert_eq!(mapping.plane_issue_id, "plane-cyc-1");
    }

    #[tokio::test]
    async fn push_cycle_update_failure_is_contextual() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc-9/"))
            .respond_with(ResponseTemplate::new(503).set_body_string("unavailable"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 3, "plane-cyc-9");

        let err = push_cycle(&client, &storage, &cycle(3, "Sprint 1"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("updating Plane cycle plane-cyc-9"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn push_module_delete_removes_plane_module_and_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/plane-mod-5/"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 5, "plane-mod-5");

        push_module_delete(&client, &storage, 5).await.unwrap();

        assert!(storage.get_sync_mapping("module", 5).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn push_module_delete_is_noop_without_mapping() {
        let client =
            PlaneClient::new("http://127.0.0.1:1".into(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();

        // No mapping means nothing to delete remotely, so the dead endpoint is
        // never contacted.
        push_module_delete(&client, &storage, 5).await.unwrap();
    }

    #[tokio::test]
    async fn push_cycle_delete_removes_plane_cycle_and_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc-5/"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 5, "plane-cyc-5");

        push_cycle_delete(&client, &storage, 5).await.unwrap();

        assert!(storage.get_sync_mapping("cycle", 5).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn push_cycle_delete_failure_keeps_the_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc-5/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 5, "plane-cyc-5");

        let err = push_cycle_delete(&client, &storage, 5).await.unwrap_err();
        assert!(
            err.to_string().contains("deleting Plane cycle plane-cyc-5"),
            "unexpected error: {err}"
        );
        assert!(
            storage.get_sync_mapping("cycle", 5).await.unwrap().is_some(),
            "the mapping must survive a failed remote delete so the delete can be retried"
        );
    }

    // -- mapping-write failures --

    #[tokio::test]
    async fn push_module_create_surfaces_mapping_write_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-mod-1",
                "name": "Auth"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().failing_upsert_mapping();

        let err = push_module(&client, &storage, &module(7, "Auth"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("storing module sync mapping"),
            "a Plane module created without a local mapping cannot be updated later, \
             so the failure must surface: {err}"
        );
    }

    #[tokio::test]
    async fn push_module_update_surfaces_mapping_write_failure() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/plane-mod-9/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-mod-9",
                "name": "Auth v2"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("module", 7, "plane-mod-9")
            .failing_upsert_mapping();

        let err = push_module(&client, &storage, &module(7, "Auth v2"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("updating sync mapping timestamp"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn push_cycle_create_surfaces_mapping_write_failure() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-cyc-1",
                "name": "Sprint 1"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().failing_upsert_mapping();

        let err = push_cycle(&client, &storage, &cycle(3, "Sprint 1"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("storing cycle sync mapping"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn push_work_package_update_failure_is_contextual() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wp-existing/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "k".into(), "ws".into(), "proj".into());
        let sync = OutboundSync::new(client, PlaneStateMapper::new());

        let err = sync
            .push_work_package("WP02", "Updated", None, &[], "parent-id", Some("wp-existing"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("updating Plane sub-issue wp-existing"),
            "unexpected error: {err}"
        );
    }
}
