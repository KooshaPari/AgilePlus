use std::collections::HashMap;
use std::sync::Arc;

use crate::client::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};

#[derive(Debug, Clone)]
pub struct InMemoryPlaneClient {
    pub work_items: Arc<std::sync::Mutex<HashMap<String, PlaneWorkItemResponse>>>,
    pub modules: Arc<std::sync::Mutex<HashMap<String, PlaneModuleResponse>>>,
    pub cycles: Arc<std::sync::Mutex<HashMap<String, PlaneCycleResponse>>>,
    pub labels: Arc<std::sync::Mutex<HashMap<String, String>>>,
    pub next_id: Arc<std::sync::Mutex<u32>>,
}

impl InMemoryPlaneClient {
    pub fn new() -> Self {
        Self {
            work_items: Arc::new(std::sync::Mutex::new(HashMap::new())),
            modules: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cycles: Arc::new(std::sync::Mutex::new(HashMap::new())),
            labels: Arc::new(std::sync::Mutex::new(HashMap::new())),
            next_id: Arc::new(std::sync::Mutex::new(1)),
        }
    }

    fn next_id(&self) -> u32 {
        let mut guard = self.next_id.lock().unwrap();
        let id = *guard;
        *guard = id + 1;
        id
    }
}

impl Default for InMemoryPlaneClient {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPlaneClient {
    pub async fn create_work_item(
        &self,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let id = format!("wi-{}", self.next_id());
        let response = PlaneWorkItemResponse {
            id: id.clone(),
            name: work_item.name.clone(),
            description_html: work_item.description_html.clone(),
            state: work_item.state.clone(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.work_items.lock().unwrap().insert(id, response.clone());
        Ok(response)
    }

    pub async fn update_work_item(
        &self,
        work_item_id: &str,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let mut items = self.work_items.lock().unwrap();
        if let Some(existing) = items.get_mut(work_item_id) {
            existing.name = work_item.name.clone();
            existing.description_html = work_item.description_html.clone();
            existing.state = work_item.state.clone();
            existing.updated_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(existing.clone())
        } else {
            anyhow::bail!("work item {} not found", work_item_id)
        }
    }

    pub async fn get_work_item(
        &self,
        work_item_id: &str,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let items = self.work_items.lock().unwrap();
        items
            .get(work_item_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("work item {} not found", work_item_id))
    }

    pub async fn list_work_items(&self) -> anyhow::Result<Vec<PlaneWorkItemResponse>> {
        let items = self.work_items.lock().unwrap();
        Ok(items.values().cloned().collect())
    }

    pub async fn create_issue(&self, issue: &PlaneIssue) -> anyhow::Result<PlaneWorkItemResponse> {
        self.create_work_item(issue).await
    }

    pub async fn update_issue(
        &self,
        issue_id: &str,
        issue: &PlaneIssue,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        self.update_work_item(issue_id, issue).await
    }

    pub async fn get_issue(&self, issue_id: &str) -> anyhow::Result<PlaneWorkItemResponse> {
        self.get_work_item(issue_id).await
    }

    pub async fn list_issues(&self) -> anyhow::Result<Vec<PlaneWorkItemResponse>> {
        self.list_work_items().await
    }

    pub async fn create_sub_issue(
        &self,
        parent_id: &str,
        issue: &PlaneIssue,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let mut sub_issue = issue.clone();
        sub_issue.parent = Some(parent_id.to_string());
        self.create_issue(&sub_issue).await
    }

    pub async fn create_module(
        &self,
        req: &PlaneCreateModuleRequest,
    ) -> anyhow::Result<PlaneModuleResponse> {
        let id = format!("mod-{}", self.next_id());
        let response = PlaneModuleResponse {
            id: id.clone(),
            name: req.name.clone(),
            description: req.description.clone(),
        };
        self.modules.lock().unwrap().insert(id, response.clone());
        Ok(response)
    }

    pub async fn update_module(
        &self,
        plane_module_id: &str,
        req: &PlaneCreateModuleRequest,
    ) -> anyhow::Result<()> {
        let mut modules = self.modules.lock().unwrap();
        if let Some(existing) = modules.get_mut(plane_module_id) {
            existing.name = req.name.clone();
            existing.description = req.description.clone();
            Ok(())
        } else {
            anyhow::bail!("module {} not found", plane_module_id)
        }
    }

    pub async fn delete_module(&self, plane_module_id: &str) -> anyhow::Result<()> {
        if self.modules.lock().unwrap().remove(plane_module_id).is_some() {
            Ok(())
        } else {
            anyhow::bail!("module {} not found", plane_module_id)
        }
    }

    pub async fn add_work_item_to_module(
        &self,
        _plane_module_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn delete_work_item_from_module(
        &self,
        _plane_module_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn add_issue_to_module(
        &self,
        plane_module_id: &str,
        plane_issue_id: &str,
    ) -> anyhow::Result<()> {
        self.add_work_item_to_module(plane_module_id, plane_issue_id).await
    }

    pub async fn create_cycle(
        &self,
        req: &PlaneCreateCycleRequest,
    ) -> anyhow::Result<PlaneCycleResponse> {
        let id = format!("cyc-{}", self.next_id());
        let response = PlaneCycleResponse {
            id: id.clone(),
            name: req.name.clone(),
            start_date: Some(req.start_date.clone()),
            end_date: Some(req.end_date.clone()),
        };
        self.cycles.lock().unwrap().insert(id, response.clone());
        Ok(response)
    }

    pub async fn update_cycle(
        &self,
        plane_cycle_id: &str,
        req: &PlaneCreateCycleRequest,
    ) -> anyhow::Result<()> {
        let mut cycles = self.cycles.lock().unwrap();
        if let Some(existing) = cycles.get_mut(plane_cycle_id) {
            existing.name = req.name.clone();
            existing.start_date = Some(req.start_date.clone());
            existing.end_date = Some(req.end_date.clone());
            Ok(())
        } else {
            anyhow::bail!("cycle {} not found", plane_cycle_id)
        }
    }

    pub async fn delete_cycle(&self, plane_cycle_id: &str) -> anyhow::Result<()> {
        if self.cycles.lock().unwrap().remove(plane_cycle_id).is_some() {
            Ok(())
        } else {
            anyhow::bail!("cycle {} not found", plane_cycle_id)
        }
    }

    pub async fn add_work_item_to_cycle(
        &self,
        _plane_cycle_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn delete_work_item_from_cycle(
        &self,
        _plane_cycle_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn add_issue_to_cycle(
        &self,
        plane_cycle_id: &str,
        plane_issue_id: &str,
    ) -> anyhow::Result<()> {
        self.add_work_item_to_cycle(plane_cycle_id, plane_issue_id).await
    }

    pub async fn sync_labels_to_remote(
        &self,
        local_labels: &[String],
    ) -> anyhow::Result<std::collections::HashMap<String, String>> {
        let mut labels = self.labels.lock().unwrap();
        let mut result = std::collections::HashMap::new();
        for label in local_labels {
            let id = format!("label-{}", self.next_id());
            labels.insert(label.clone(), id.clone());
            result.insert(label.clone(), id);
        }
        Ok(result)
    }

    pub async fn sync_labels_from_remote(&self) -> anyhow::Result<Vec<String>> {
        let labels = self.labels.lock().unwrap();
        Ok(labels.keys().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_and_get_work_item() {
        let client = InMemoryPlaneClient::new();
        let work_item = PlaneWorkItem {
            id: None,
            name: "Test Issue".to_string(),
            description_html: Some("<p>Description</p>".to_string()),
            state: Some("backlog".to_string()),
            priority: Some(2),
            parent: None,
            labels: vec!["bug".to_string()],
        };

        let created = client.create_work_item(&work_item).await.unwrap();
        assert!(!created.id.is_empty());

        let retrieved = client.get_work_item(&created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Issue");
    }

    #[tokio::test]
    async fn list_issues_empty() {
        let client = InMemoryPlaneClient::new();
        let issues = client.list_issues().await.unwrap();
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn list_issues_after_create() {
        let client = InMemoryPlaneClient::new();
        let work_item = PlaneWorkItem {
            id: None,
            name: "Issue 1".to_string(),
            description_html: None,
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec![],
        };
        client.create_work_item(&work_item).await.unwrap();
        let issues = client.list_issues().await.unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[tokio::test]
    async fn create_sub_issue() {
        let client = InMemoryPlaneClient::new();
        let parent = PlaneWorkItem {
            id: None,
            name: "Parent".to_string(),
            description_html: None,
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec![],
        };
        let parent_resp = client.create_issue(&parent).await.unwrap();

        let child = PlaneWorkItem {
            id: None,
            name: "Child".to_string(),
            description_html: None,
            state: None,
            priority: Some(3),
            parent: None,
            labels: vec![],
        };
        let child_resp = client.create_sub_issue(&parent_resp.id, &child).await.unwrap();

        assert_eq!(child_resp.name, "Child");
    }

    #[tokio::test]
    async fn create_and_get_module() {
        let client = InMemoryPlaneClient::new();
        let req = PlaneCreateModuleRequest {
            name: "Test Module".to_string(),
            description: Some("Module desc".to_string()),
        };
        let created = client.create_module(&req).await.unwrap();
        assert!(!created.id.is_empty());
        assert_eq!(created.name, "Test Module");
    }

    #[tokio::test]
    async fn create_and_get_cycle() {
        let client = InMemoryPlaneClient::new();
        let req = PlaneCreateCycleRequest {
            name: "Sprint 1".to_string(),
            description: Some("Sprint desc".to_string()),
            start_date: "2026-01-01".to_string(),
            end_date: "2026-01-14".to_string(),
        };
        let created = client.create_cycle(&req).await.unwrap();
        assert!(!created.id.is_empty());
        assert_eq!(created.name, "Sprint 1");
    }

    #[tokio::test]
    async fn delete_module() {
        let client = InMemoryPlaneClient::new();
        let req = PlaneCreateModuleRequest {
            name: "To Delete".to_string(),
            description: None,
        };
        let created = client.create_module(&req).await.unwrap();
        client.delete_module(&created.id).await.unwrap();
        let result = client.update_module(&created.id, &req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_cycle() {
        let client = InMemoryPlaneClient::new();
        let req = PlaneCreateCycleRequest {
            name: "To Delete".to_string(),
            description: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-01-14".to_string(),
        };
        let created = client.create_cycle(&req).await.unwrap();
        client.delete_cycle(&created.id).await.unwrap();
        let result = client.update_cycle(&created.id, &req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn sync_labels() {
        let client = InMemoryPlaneClient::new();
        let labels = vec!["bug".to_string(), "feature".to_string()];
        let mapping = client.sync_labels_to_remote(&labels).await.unwrap();
        assert_eq!(mapping.len(), 2);
        assert!(mapping.contains_key("bug"));
        assert!(mapping.contains_key("feature"));
    }
}
