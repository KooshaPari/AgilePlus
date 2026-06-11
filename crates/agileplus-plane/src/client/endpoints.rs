//! Endpoint path builders for Plane.so API URLs.

/// Internal helpers for formatting Plane.so endpoint URLs.
pub(super) struct ClientEndpoints;

impl ClientEndpoints {
    pub(super) fn work_items_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!("{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/work-items/")
    }

    pub(super) fn modules_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!("{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/modules/")
    }

    pub(super) fn module_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/modules/{module_id}/"
        )
    }

    pub(super) fn module_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/modules/{module_id}/module-issues/"
        )
    }

    pub(super) fn module_work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/modules/{module_id}/module-issues/{work_item_id}/"
        )
    }

    pub(super) fn cycles_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!("{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/cycles/")
    }

    pub(super) fn cycle_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/cycles/{cycle_id}/"
        )
    }

    pub(super) fn cycle_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/"
        )
    }

    pub(super) fn cycle_work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{work_item_id}/"
        )
    }

    pub(super) fn work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/work-items/{work_item_id}/"
        )
    }

    pub(super) fn labels_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!("{base_url}/api/v1/workspaces/{workspace_slug}/projects/{project_id}/labels/")
    }
}
