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
        clear_plane_env();
        assert!(plane_client_from_env().is_none());
    }

    #[test]
    fn plane_client_from_env_returns_none_without_workspace() {
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
