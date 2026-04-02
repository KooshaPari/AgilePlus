use anyhow::{Context, Result};
use reqwest::Method;

use super::endpoints::ClientEndpoints;
use super::transport;
use super::{
    PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};

mod cycles;
mod modules;
mod work_items;

impl PlaneClient {
    fn work_items_url(&self) -> String {
        ClientEndpoints::work_items_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn modules_url(&self) -> String {
        ClientEndpoints::modules_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn module_url(&self, module_id: &str) -> String {
        ClientEndpoints::module_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
        )
    }

    fn module_work_items_url(&self, module_id: &str) -> String {
        ClientEndpoints::module_work_items_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
        )
    }

    fn module_work_item_url(&self, module_id: &str, work_item_id: &str) -> String {
        ClientEndpoints::module_work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
            work_item_id,
        )
    }

    fn cycles_url(&self) -> String {
        ClientEndpoints::cycles_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn cycle_url(&self, cycle_id: &str) -> String {
        ClientEndpoints::cycle_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
        )
    }

    fn cycle_work_items_url(&self, cycle_id: &str) -> String {
        ClientEndpoints::cycle_work_items_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
        )
    }

    fn cycle_work_item_url(&self, cycle_id: &str, work_item_id: &str) -> String {
        ClientEndpoints::cycle_work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
            work_item_id,
        )
    }

    fn work_item_url(&self, work_item_id: &str) -> String {
        ClientEndpoints::work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            work_item_id,
        )
    }

    pub fn labels_url(&self) -> String {
        ClientEndpoints::labels_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    /// Make a raw GET request and return response body as String.
    pub async fn get_raw(&self, url: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(Method::GET, url)
            .await
            .context("Plane.so GET request failed")?;
        transport::read_text_response(resp, "reading Plane.so response body").await
    }

    /// Make a raw POST request with JSON body and return response body as String.
    pub async fn post_raw(&self, url: &str, json_body: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp =
            transport::request_raw_body(&self.client, Method::POST, url, &self.api_key, json_body)
                .await
                .context("Plane.so POST request failed")?;
        transport::read_text_response(resp, "reading Plane.so response body").await
    }

    /// Sync labels to Plane.so: create any local labels that don't exist remotely.
    ///
    /// Returns a map of label name → Plane label ID.
    pub async fn sync_labels(&self, local_labels: &[String]) -> Result<std::collections::HashMap<String, String>> {
        let url = self.labels_url();
        let resp = self
            .get_raw(&url)
            .await
            .context("fetching Plane.so labels")?;

        #[derive(serde::Deserialize)]
        struct LabelListResponse {
            results: Vec<PlaneLabel>,
        }

        #[derive(serde::Deserialize)]
        struct PlaneLabel {
            id: String,
            name: String,
        }

        let remote: Vec<PlaneLabel> = if let Ok(list) = serde_json::from_str::<Vec<PlaneLabel>>(&resp) {
            list
        } else {
            let wrapped: LabelListResponse =
                serde_json::from_str(&resp).context("parsing label list response")?;
            wrapped.results
        };

        let mut name_to_id: std::collections::HashMap<String, String> =
            remote.into_iter().map(|l| (l.name, l.id)).collect();

        for label in local_labels {
            if !name_to_id.contains_key(label.as_str()) {
                let body = serde_json::json!({ "name": label });
                let json_body = serde_json::to_string(&body)?;
                let create_resp = self.post_raw(&url, &json_body).await?;
                let created: PlaneLabel =
                    serde_json::from_str(&create_resp).context("parsing created label")?;
                tracing::info!(
                    label_name = label,
                    plane_label_id = created.id,
                    "created remote label"
                );
                name_to_id.insert(label.clone(), created.id);
            }
        }

        Ok(name_to_id)
    }
}
