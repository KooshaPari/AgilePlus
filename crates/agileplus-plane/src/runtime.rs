use std::env;

use agileplus_domain::ports::StoragePort;
use anyhow::Context;

use crate::client::PlaneClient;
use crate::outbound::{
    push_cycle, push_cycle_delete, push_feature_cycle_assignment, push_feature_cycle_unassignment,
    push_feature_module_assignment, push_feature_module_unassignment, push_module,
    push_module_delete,
};

const DEFAULT_PLANE_API_URL: &str = "https://api.plane.so";

fn plane_client_from_env() -> Option<PlaneClient> {
    let api_key = env::var("PLANE_API_KEY").ok()?;
    let workspace_slug = env::var("PLANE_WORKSPACE").ok()?;
    let project_id = env::var("PLANE_PROJECT").ok()?;
    let base_url = env::var("PLANE_API_URL").unwrap_or_else(|_| DEFAULT_PLANE_API_URL.to_string());

    Some(PlaneClient::new(
        base_url,
        api_key,
        workspace_slug,
        project_id,
    ))
}

pub async fn maybe_sync_module_from_env<S: StoragePort>(
    storage: &S,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    let module = storage
        .get_module(module_id)
        .await
        .context("loading module for Plane sync")?
        .ok_or_else(|| anyhow::anyhow!("module {module_id} not found for Plane sync"))?;
    push_module(&client, storage, &module).await
}

pub async fn maybe_delete_module_from_env<S: StoragePort>(
    storage: &S,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_module_delete(&client, storage, module_id).await
}

pub async fn maybe_sync_cycle_from_env<S: StoragePort>(
    storage: &S,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    let cycle = storage
        .get_cycle(cycle_id)
        .await
        .context("loading cycle for Plane sync")?
        .ok_or_else(|| anyhow::anyhow!("cycle {cycle_id} not found for Plane sync"))?;
    push_cycle(&client, storage, &cycle).await
}

pub async fn maybe_delete_cycle_from_env<S: StoragePort>(
    storage: &S,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_cycle_delete(&client, storage, cycle_id).await
}

pub async fn maybe_sync_feature_module_assignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_module_assignment(&client, storage, feature_id, module_id).await
}

pub async fn maybe_sync_feature_module_unassignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    module_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_module_unassignment(&client, storage, feature_id, module_id).await
}

pub async fn maybe_sync_feature_cycle_assignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_cycle_assignment(&client, storage, feature_id, cycle_id).await
}

pub async fn maybe_sync_feature_cycle_unassignment_from_env<S: StoragePort>(
    storage: &S,
    feature_id: i64,
    cycle_id: i64,
) -> anyhow::Result<()> {
    let Some(client) = plane_client_from_env() else {
        return Ok(());
    };
    push_feature_cycle_unassignment(&client, storage, feature_id, cycle_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::lock_env;
    use std::env;

    fn clear_plane_env() {
        unsafe {
            env::remove_var("PLANE_API_KEY");
            env::remove_var("PLANE_WORKSPACE");
            env::remove_var("PLANE_PROJECT");
            env::remove_var("PLANE_API_URL");
        }
    }

    #[test]
    fn plane_client_from_env_returns_none_without_api_key() {
        let _env = lock_env();
        clear_plane_env();
        assert!(plane_client_from_env().is_none());
    }

    #[test]
    fn plane_client_from_env_returns_none_without_workspace() {
        let _env = lock_env();
        unsafe {
            env::set_var("PLANE_API_KEY", "test-key");
            env::remove_var("PLANE_WORKSPACE");
            env::remove_var("PLANE_PROJECT");
        }
        assert!(plane_client_from_env().is_none());
        clear_plane_env();
    }

    #[test]
    fn plane_client_from_env_returns_none_without_project() {
        let _env = lock_env();
        unsafe {
            env::set_var("PLANE_API_KEY", "test-key");
            env::set_var("PLANE_WORKSPACE", "test-ws");
            env::remove_var("PLANE_PROJECT");
        }
        assert!(plane_client_from_env().is_none());
        clear_plane_env();
    }

    #[test]
    fn plane_client_from_env_succeeds_with_all_vars() {
        let _env = lock_env();
        unsafe {
            env::set_var("PLANE_API_KEY", "test-key");
            env::set_var("PLANE_WORKSPACE", "test-ws");
            env::set_var("PLANE_PROJECT", "test-proj");
            env::remove_var("PLANE_API_URL");
        }
        assert!(plane_client_from_env().is_some());
        clear_plane_env();
    }

    #[test]
    fn plane_client_from_env_uses_custom_api_url() {
        let _env = lock_env();
        unsafe {
            env::set_var("PLANE_API_KEY", "key");
            env::set_var("PLANE_WORKSPACE", "ws");
            env::set_var("PLANE_PROJECT", "proj");
            env::set_var("PLANE_API_URL", "https://custom.plane.so");
        }
        assert!(plane_client_from_env().is_some());
        clear_plane_env();
    }
}

/// Behaviour of the `maybe_*_from_env` orchestration helpers.
///
/// These are the functions the sync daemon calls on every tick, so their three
/// observable outcomes matter: skip (no Plane configuration), "entity missing"
/// error, and a propagated outbound failure.
#[cfg(test)]
mod maybe_sync_tests {
    // The environment lock is deliberately held across `await` points: these are
    // current-thread `#[tokio::test]`s, so there is no scheduler on which the
    // guard could deadlock, and dropping it early would let another test rewrite
    // PLANE_* halfway through a sync call.
    #![allow(clippy::await_holding_lock)]

    use super::*;
    use crate::mock_storage::MockStoragePort;
    use crate::test_env::lock_env;
    use agileplus_domain::domain::cycle::{Cycle, CycleState};
    use agileplus_domain::domain::module::Module;
    use chrono::NaiveDate;
    use std::env;

    /// A port that refuses connections immediately, so outbound calls fail
    /// without touching the real Plane.so API.
    const DEAD_ENDPOINT: &str = "http://127.0.0.1:1";

    fn clear_plane_env() {
        unsafe {
            env::remove_var("PLANE_API_KEY");
            env::remove_var("PLANE_WORKSPACE");
            env::remove_var("PLANE_PROJECT");
            env::remove_var("PLANE_API_URL");
        }
    }

    fn point_plane_at_dead_endpoint() {
        unsafe {
            env::set_var("PLANE_API_KEY", "test-key");
            env::set_var("PLANE_WORKSPACE", "test-ws");
            env::set_var("PLANE_PROJECT", "test-proj");
            env::set_var("PLANE_API_URL", DEAD_ENDPOINT);
        }
    }

    fn sample_cycle(name: &str) -> Cycle {
        Cycle {
            id: 0,
            name: name.to_string(),
            description: None,
            state: CycleState::Active,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
            module_scope_id: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn storage_with(mappings: bool) -> MockStoragePort {
        if mappings {
            MockStoragePort::new()
                .with_sync_mapping("feature", 1, "plane-feat")
                .with_sync_mapping("module", 2, "plane-mod")
                .with_sync_mapping("cycle", 3, "plane-cyc")
        } else {
            MockStoragePort::new()
        }
    }

    // -- no Plane configuration: every helper is a no-op --

    #[tokio::test]
    async fn module_helpers_return_ok_without_plane_env() {
        let _env = lock_env();
        clear_plane_env();
        let store = storage_with(false);

        assert!(maybe_sync_module_from_env(&store, 123).await.is_ok());
        assert!(maybe_delete_module_from_env(&store, 123).await.is_ok());
    }

    #[tokio::test]
    async fn cycle_helpers_return_ok_without_plane_env() {
        let _env = lock_env();
        clear_plane_env();
        let store = storage_with(false);

        assert!(maybe_sync_cycle_from_env(&store, 7).await.is_ok());
        assert!(maybe_delete_cycle_from_env(&store, 7).await.is_ok());
    }

    #[tokio::test]
    async fn assignment_helpers_return_ok_without_plane_env() {
        let _env = lock_env();
        clear_plane_env();
        let store = storage_with(false);

        assert!(maybe_sync_feature_module_assignment_from_env(&store, 1, 2)
            .await
            .is_ok());
        assert!(maybe_sync_feature_module_unassignment_from_env(&store, 1, 2)
            .await
            .is_ok());
        assert!(maybe_sync_feature_cycle_assignment_from_env(&store, 1, 3)
            .await
            .is_ok());
        assert!(maybe_sync_feature_cycle_unassignment_from_env(&store, 1, 3)
            .await
            .is_ok());
    }

    // -- configured, but the local entity is missing --

    #[tokio::test]
    async fn module_sync_reports_missing_module() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);

        let err = maybe_sync_module_from_env(&store, 123).await.unwrap_err();
        assert!(
            err.to_string().contains("module 123 not found for Plane sync"),
            "unexpected error: {err}"
        );
        clear_plane_env();
    }

    #[tokio::test]
    async fn cycle_sync_reports_missing_cycle() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);

        let err = maybe_sync_cycle_from_env(&store, 7).await.unwrap_err();
        assert!(
            err.to_string().contains("cycle 7 not found for Plane sync"),
            "unexpected error: {err}"
        );
        clear_plane_env();
    }

    // -- configured, entity present: outbound failures propagate --

    #[tokio::test]
    async fn module_sync_propagates_outbound_failure() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);
        let module_id = store.create_module(&Module::new("Auth", None)).await.unwrap();

        let err = maybe_sync_module_from_env(&store, module_id).await.unwrap_err();
        assert!(
            err.to_string().contains("creating Plane module"),
            "unexpected error: {err}"
        );
        // A failed push must not leave a sync mapping behind.
        assert!(store
            .get_sync_mapping("module", module_id)
            .await
            .unwrap()
            .is_none());
        clear_plane_env();
    }

    #[tokio::test]
    async fn cycle_sync_propagates_outbound_failure() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);
        let cycle_id = store.create_cycle(&sample_cycle("Sprint")).await.unwrap();

        let err = maybe_sync_cycle_from_env(&store, cycle_id).await.unwrap_err();
        assert!(
            err.to_string().contains("creating Plane cycle"),
            "unexpected error: {err}"
        );
        clear_plane_env();
    }

    #[tokio::test]
    async fn module_and_cycle_delete_are_noops_without_mapping() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);

        // No mapping means nothing to delete remotely, so no HTTP call is made
        // and the dead endpoint never has to answer.
        assert!(maybe_delete_module_from_env(&store, 99).await.is_ok());
        assert!(maybe_delete_cycle_from_env(&store, 99).await.is_ok());
        clear_plane_env();
    }

    #[tokio::test]
    async fn assignment_helpers_skip_when_mappings_are_absent() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(false);

        assert!(maybe_sync_feature_module_assignment_from_env(&store, 1, 2)
            .await
            .is_ok());
        assert!(maybe_sync_feature_cycle_assignment_from_env(&store, 1, 3)
            .await
            .is_ok());
        clear_plane_env();
    }

    #[tokio::test]
    async fn assignment_helpers_propagate_outbound_failures() {
        let _env = lock_env();
        point_plane_at_dead_endpoint();
        let store = storage_with(true);

        let err = maybe_sync_feature_module_assignment_from_env(&store, 1, 2)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("adding Plane issue"),
            "unexpected error: {err}"
        );

        let err = maybe_sync_feature_module_unassignment_from_env(&store, 1, 2)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("removing Plane work item"),
            "unexpected error: {err}"
        );

        let err = maybe_sync_feature_cycle_assignment_from_env(&store, 1, 3)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("adding Plane issue"),
            "unexpected error: {err}"
        );

        let err = maybe_sync_feature_cycle_unassignment_from_env(&store, 1, 3)
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("removing Plane work item"),
            "unexpected error: {err}"
        );
        clear_plane_env();
    }
}
