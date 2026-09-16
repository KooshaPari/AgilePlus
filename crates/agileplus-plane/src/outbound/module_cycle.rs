use agileplus_domain::domain::cycle::Cycle;
use agileplus_domain::domain::module::Module;
use agileplus_domain::domain::sync_mapping::SyncMapping;
use agileplus_domain::ports::StoragePort;
use anyhow::{Context, Result};
use chrono::Utc;
use crate::client::{PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest};
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
/// When a Feature is assigned to a Module, sync the Plane work-item-to-module link.
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
                .add_work_item_to_module(&mm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane work item {} to module {}",
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
/// When a Feature is unassigned from a Module, sync the Plane work-item-to-module unlink.
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
/// When a Feature is assigned to a Cycle, sync the Plane work-item-to-cycle link.
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
                .add_work_item_to_cycle(&cm.plane_issue_id, &fm.plane_issue_id)
                .await
                .with_context(|| {
                    format!(
                        "adding Plane work item {} to cycle {}",
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
/// When a Feature is unassigned from a Cycle, sync the Plane work-item-to-cycle unlink.
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
    use crate::mock_storage::MockStoragePort;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_module(id: i64, name: &str) -> Module {
        Module {
            id,
            slug: name.to_lowercase().replace(' ', "-"),
            friendly_name: name.to_string(),
            description: Some(format!("{name} description")),
            parent_module_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn test_cycle(id: i64, name: &str) -> Cycle {
        use chrono::NaiveDate;
        Cycle {
            id,
            name: name.to_string(),
            description: None,
            state: agileplus_domain::domain::cycle::CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── push_module ──────────────────────────────────────────

    #[tokio::test]
    async fn push_module_creates_new_when_no_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-mod-1",
                "name": "Auth Module"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        let module = test_module(1, "Auth Module");

        push_module(&client, &storage, &module).await.unwrap();

        // Verify sync mapping was stored
        let mapping = storage.get_sync_mapping("module", 1).await.unwrap();
        assert!(mapping.is_some());
        assert_eq!(mapping.unwrap().plane_issue_id, "plane-mod-1");
    }

    #[tokio::test]
    async fn push_module_updates_existing_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/existing-mod/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "existing-mod",
                "name": "Updated"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 1, "existing-mod");
        let module = test_module(1, "Updated");

        push_module(&client, &storage, &module).await.unwrap();
    }

    #[tokio::test]
    async fn push_module_create_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        let module = test_module(1, "Fail");

        let result = push_module(&client, &storage, &module).await;
        assert!(result.is_err());
    }

    // ── push_cycle ──────────────────────────────────────────

    #[tokio::test]
    async fn push_cycle_creates_new_when_no_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "plane-cyc-1",
                "name": "Sprint 1"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        let cycle = test_cycle(1, "Sprint 1");

        push_cycle(&client, &storage, &cycle).await.unwrap();

        let mapping = storage.get_sync_mapping("cycle", 1).await.unwrap();
        assert!(mapping.is_some());
        assert_eq!(mapping.unwrap().plane_issue_id, "plane-cyc-1");
    }

    #[tokio::test]
    async fn push_cycle_updates_existing_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/existing-cyc/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "existing-cyc",
                "name": "Updated Sprint"
            })))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 1, "existing-cyc");
        let cycle = test_cycle(1, "Updated Sprint");

        push_cycle(&client, &storage, &cycle).await.unwrap();
    }

    #[tokio::test]
    async fn push_cycle_create_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new();
        let cycle = test_cycle(1, "Fail Sprint");

        let result = push_cycle(&client, &storage, &cycle).await;
        assert!(result.is_err());
    }

    // ── push_module_delete ──────────────────────────────────

    #[tokio::test]
    async fn push_module_delete_removes_plane_and_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/plane-mod-del/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 42, "plane-mod-del");

        push_module_delete(&client, &storage, 42).await.unwrap();

        // Verify sync mapping was deleted
        let mapping = storage.get_sync_mapping("module", 42).await.unwrap();
        assert!(mapping.is_none());
    }

    #[tokio::test]
    async fn push_module_delete_noop_when_no_mapping() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();

        // Should not error when no mapping exists
        push_module_delete(&client, &storage, 999).await.unwrap();
    }

    #[tokio::test]
    async fn push_module_delete_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/mod-del/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("module", 1, "mod-del");

        let result = push_module_delete(&client, &storage, 1).await;
        assert!(result.is_err());
    }

    // ── push_cycle_delete ──────────────────────────────────

    #[tokio::test]
    async fn push_cycle_delete_removes_plane_and_mapping() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc-del/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 55, "plane-cyc-del");

        push_cycle_delete(&client, &storage, 55).await.unwrap();

        let mapping = storage.get_sync_mapping("cycle", 55).await.unwrap();
        assert!(mapping.is_none());
    }

    #[tokio::test]
    async fn push_cycle_delete_noop_when_no_mapping() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();

        push_cycle_delete(&client, &storage, 999).await.unwrap();
    }

    #[tokio::test]
    async fn push_cycle_delete_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/cyc-del/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new().with_sync_mapping("cycle", 1, "cyc-del");

        let result = push_cycle_delete(&client, &storage, 1).await;
        assert!(result.is_err());
    }

    // ── push_feature_module_assignment ──────────────────────

    #[tokio::test]
    async fn push_feature_module_assignment_links_issue_to_module() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/modules/plane-mod/module-issues/",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 10, "plane-feat")
            .with_sync_mapping("module", 20, "plane-mod");

        push_feature_module_assignment(&client, &storage, 10, 20)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn push_feature_module_assignment_skips_when_missing_mapping() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();
        // Should succeed but skip the actual API call
        push_feature_module_assignment(&client, &storage, 1, 1)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn push_feature_module_assignment_error_propagates() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/modules/mod1/module-issues/",
            ))
            .respond_with(ResponseTemplate::new(500).set_body_string("error"))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 1, "feat1")
            .with_sync_mapping("module", 1, "mod1");

        let result = push_feature_module_assignment(&client, &storage, 1, 1).await;
        assert!(result.is_err());
    }

    // ── push_feature_module_unassignment ────────────────────

    #[tokio::test]
    async fn push_feature_module_unassignment_removes_link() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/modules/plane-mod/module-issues/plane-feat/",
            ))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 10, "plane-feat")
            .with_sync_mapping("module", 20, "plane-mod");

        push_feature_module_unassignment(&client, &storage, 10, 20)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn push_feature_module_unassignment_skips_when_missing() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();

        push_feature_module_unassignment(&client, &storage, 1, 1)
            .await
            .unwrap();
    }

    // ── push_feature_cycle_assignment ──────────────────────

    #[tokio::test]
    async fn push_feature_cycle_assignment_links_issue_to_cycle() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc/cycle-issues/",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 10, "plane-feat")
            .with_sync_mapping("cycle", 20, "plane-cyc");

        push_feature_cycle_assignment(&client, &storage, 10, 20)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn push_feature_cycle_assignment_skips_when_missing() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();

        push_feature_cycle_assignment(&client, &storage, 1, 1)
            .await
            .unwrap();
    }

    // ── push_feature_cycle_unassignment ────────────────────

    #[tokio::test]
    async fn push_feature_cycle_unassignment_removes_link() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/cycles/plane-cyc/cycle-issues/plane-feat/",
            ))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = PlaneClient::new(server.uri(), "key".into(), "ws".into(), "proj".into());
        let storage = MockStoragePort::new()
            .with_sync_mapping("feature", 10, "plane-feat")
            .with_sync_mapping("cycle", 20, "plane-cyc");

        push_feature_cycle_unassignment(&client, &storage, 10, 20)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn push_feature_cycle_unassignment_skips_when_missing() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "ws".into(),
            "proj".into(),
        );
        let storage = MockStoragePort::new();

        push_feature_cycle_unassignment(&client, &storage, 1, 1)
            .await
            .unwrap();
    }
}
