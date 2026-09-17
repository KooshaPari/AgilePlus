//! Endpoint path builders for Plane.so API URLs.

/// Internal helpers for formatting Plane.so endpoint URLs.
pub(super) struct ClientEndpoints;

impl ClientEndpoints {
    pub(super) fn work_items_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/work-items/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn modules_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn module_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/",
            base_url, workspace_slug, project_id, module_id
        )
    }

    pub(super) fn module_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/module-issues/",
            base_url, workspace_slug, project_id, module_id
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
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/module-issues/{}/",
            base_url, workspace_slug, project_id, module_id, work_item_id
        )
    }

    pub(super) fn cycles_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn cycle_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/",
            base_url, workspace_slug, project_id, cycle_id
        )
    }

    pub(super) fn cycle_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/cycle-issues/",
            base_url, workspace_slug, project_id, cycle_id
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
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/cycle-issues/{}/",
            base_url, workspace_slug, project_id, cycle_id, work_item_id
        )
    }

    pub(super) fn work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/work-items/{}/",
            base_url, workspace_slug, project_id, work_item_id
        )
    }

    pub(super) fn labels_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/labels/",
            base_url, workspace_slug, project_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://api.plane.so";
    const WS: &str = "acme";
    const PROJ: &str = "proj-42";

    #[test]
    fn work_items_url_shape() {
        assert_eq!(
            ClientEndpoints::work_items_url(BASE, WS, PROJ),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/work-items/"
        );
    }

    #[test]
    fn modules_url_shape() {
        assert_eq!(
            ClientEndpoints::modules_url(BASE, WS, PROJ),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/modules/"
        );
    }

    #[test]
    fn module_url_appends_module_id() {
        assert_eq!(
            ClientEndpoints::module_url(BASE, WS, PROJ, "m1"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/modules/m1/"
        );
    }

    #[test]
    fn module_work_items_url_shape() {
        assert_eq!(
            ClientEndpoints::module_work_items_url(BASE, WS, PROJ, "m1"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/modules/m1/module-issues/"
        );
    }

    #[test]
    fn module_work_item_url_shape() {
        assert_eq!(
            ClientEndpoints::module_work_item_url(BASE, WS, PROJ, "m1", "wi9"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/modules/m1/module-issues/wi9/"
        );
    }

    #[test]
    fn cycles_url_shape() {
        assert_eq!(
            ClientEndpoints::cycles_url(BASE, WS, PROJ),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/cycles/"
        );
    }

    #[test]
    fn cycle_url_shape() {
        assert_eq!(
            ClientEndpoints::cycle_url(BASE, WS, PROJ, "c1"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/cycles/c1/"
        );
    }

    #[test]
    fn cycle_work_items_url_shape() {
        assert_eq!(
            ClientEndpoints::cycle_work_items_url(BASE, WS, PROJ, "c1"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/cycles/c1/cycle-issues/"
        );
    }

    #[test]
    fn cycle_work_item_url_shape() {
        assert_eq!(
            ClientEndpoints::cycle_work_item_url(BASE, WS, PROJ, "c1", "wi9"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/cycles/c1/cycle-issues/wi9/"
        );
    }

    #[test]
    fn work_item_url_shape() {
        assert_eq!(
            ClientEndpoints::work_item_url(BASE, WS, PROJ, "wi7"),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/work-items/wi7/"
        );
    }

    #[test]
    fn labels_url_shape() {
        assert_eq!(
            ClientEndpoints::labels_url(BASE, WS, PROJ),
            "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/labels/"
        );
    }

    #[test]
    fn urls_are_distinct_per_resource() {
        let a = ClientEndpoints::work_items_url(BASE, WS, PROJ);
        let b = ClientEndpoints::modules_url(BASE, WS, PROJ);
        let c = ClientEndpoints::cycles_url(BASE, WS, PROJ);
        let d = ClientEndpoints::labels_url(BASE, WS, PROJ);
        let all = [a, b, c, d];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j]);
            }
        }
    }

    #[test]
    fn empty_components_are_formatted_verbatim() {
        assert_eq!(
            ClientEndpoints::work_items_url("", "", ""),
            "/api/v1/workspaces//projects//work-items/"
        );
    }

    #[test]
    fn url_builders_are_prefix_stable() {
        let prefix = "https://api.plane.so/api/v1/workspaces/acme/projects/proj-42/";
        for url in [
            ClientEndpoints::work_items_url(BASE, WS, PROJ),
            ClientEndpoints::modules_url(BASE, WS, PROJ),
            ClientEndpoints::cycles_url(BASE, WS, PROJ),
            ClientEndpoints::labels_url(BASE, WS, PROJ),
        ] {
            assert!(url.starts_with(prefix), "{url}");
        }
    }
}
