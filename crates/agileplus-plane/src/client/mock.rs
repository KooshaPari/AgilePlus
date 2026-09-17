use std::collections::HashMap;
use std::sync::Mutex;

pub use super::models::{PlaneIssue, PlaneWorkItem, PlaneWorkItemResponse};

#[derive(Debug, Default)]
pub struct InMemoryPlaneClient {
    issues: Mutex<HashMap<String, PlaneWorkItemResponse>>,
    created: Mutex<Vec<PlaneWorkItem>>,
    #[allow(dead_code)] // reserved - tracks updates for test assertions
    updated: Mutex<Vec<(String, PlaneWorkItem)>>,
}

impl InMemoryPlaneClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_issue(mut self, id: &str, name: &str) -> Self {
        self.issues
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                id.to_string(),
                PlaneWorkItemResponse {
                    id: id.to_string(),
                    name: name.to_string(),
                    description_html: None,
                    state: None,
                    updated_at: None,
                },
            );
        self
    }

    pub async fn create_work_item(
        &self,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let mut created = self.created.lock().unwrap_or_else(|e| e.into_inner());
        let id = format!("issue-{}", created.len() + 1);
        created.push(work_item.clone());
        drop(created);

        let response = PlaneWorkItemResponse {
            id: id.clone(),
            name: work_item.name.clone(),
            description_html: work_item.description_html.clone(),
            state: work_item.state.clone(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        self.issues
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, response.clone());
        Ok(response)
    }

    pub async fn update_work_item(
        &self,
        id: &str,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let issues = self.issues.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = issues.get(id) {
            return Ok(PlaneWorkItemResponse {
                id: existing.id.clone(),
                name: work_item.name.clone(),
                description_html: work_item.description_html.clone(),
                state: work_item.state.clone(),
                updated_at: Some(chrono::Utc::now().to_rfc3339()),
            });
        }
        anyhow::bail!("issue {} not found", id)
    }

    pub async fn get_work_item(&self, id: &str) -> anyhow::Result<PlaneWorkItemResponse> {
        self.issues
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("issue {} not found", id))
    }

    pub async fn list_work_items(&self) -> anyhow::Result<Vec<PlaneWorkItemResponse>> {
        Ok(self
            .issues
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect())
    }

    pub async fn create_sub_issue(
        &self,
        parent_id: &str,
        title: &str,
        description_html: Option<String>,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let work_item = PlaneWorkItem {
            id: None,
            name: title.to_string(),
            description_html,
            state: None,
            priority: Some(3),
            parent: Some(parent_id.to_string()),
            labels: vec![],
        };
        self.create_work_item(&work_item).await
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_issue_returns_response() {
        let client = InMemoryPlaneClient::new();
        let issue = PlaneWorkItem {
            id: None,
            name: "Test Issue".to_string(),
            description_html: None,
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec![],
        };
        let result = client.create_issue(&issue).await.unwrap();
        assert!(result.id.starts_with("issue-"));
        assert_eq!(result.name, "Test Issue");
    }

    #[tokio::test]
    async fn list_issues_returns_all() {
        let client = InMemoryPlaneClient::new()
            .with_issue("1", "Issue 1")
            .with_issue("2", "Issue 2");
        let issues = client.list_issues().await.unwrap();
        assert_eq!(issues.len(), 2);
    }

    #[tokio::test]
    async fn create_sub_issue_sets_parent() {
        let client = InMemoryPlaneClient::new().with_issue("parent-1", "Parent Issue");
        let result = client
            .create_sub_issue("parent-1", "Child Issue", None)
            .await
            .unwrap();
        assert!(result.id.starts_with("issue-"));
    }

    #[tokio::test]
    async fn get_issue_returns_issue() {
        let client = InMemoryPlaneClient::new().with_issue("issue-1", "My Issue");
        let result = client.get_issue("issue-1").await.unwrap();
        assert_eq!(result.name, "My Issue");
    }

    #[tokio::test]
    async fn get_issue_not_found() {
        let client = InMemoryPlaneClient::new();
        let result = client.get_issue("nonexistent").await;
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;

    fn item(name: &str) -> PlaneWorkItem {
        PlaneWorkItem {
            id: None,
            name: name.to_string(),
            description_html: Some("<p>x</p>".to_string()),
            state: Some("started".to_string()),
            priority: Some(1),
            parent: None,
            labels: vec!["l".to_string()],
        }
    }

    #[tokio::test]
    async fn new_client_is_empty() {
        let client = InMemoryPlaneClient::new();
        assert!(client.list_issues().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn with_issue_is_retrievable() {
        let client = InMemoryPlaneClient::new().with_issue("a", "Alpha");
        let got = client.get_issue("a").await.unwrap();
        assert_eq!(got.name, "Alpha");
    }

    #[tokio::test]
    async fn create_assigns_sequential_ids() {
        let client = InMemoryPlaneClient::new();
        let first = client.create_issue(&item("one")).await.unwrap();
        let second = client.create_issue(&item("two")).await.unwrap();
        assert_eq!(first.id, "issue-1");
        assert_eq!(second.id, "issue-2");
    }

    #[tokio::test]
    async fn update_missing_issue_errors() {
        let client = InMemoryPlaneClient::new();
        assert!(client.update_issue("nope", &item("x")).await.is_err());
    }

    #[tokio::test]
    async fn update_existing_issue_reflects_new_name() {
        let client = InMemoryPlaneClient::new().with_issue("a", "Alpha");
        let updated = client.update_issue("a", &item("Alpha v2")).await.unwrap();
        assert_eq!(updated.name, "Alpha v2");
    }

    #[tokio::test]
    async fn create_sub_issue_records_parent() {
        let client = InMemoryPlaneClient::new();
        let child = client
            .create_sub_issue("parent-9", "Child", Some("<p>c</p>".to_string()))
            .await
            .unwrap();
        assert_eq!(child.id, "issue-1");
        let stored = client.get_issue("issue-1").await.unwrap();
        assert_eq!(stored.name, "Child");
    }

    #[tokio::test]
    async fn list_work_items_reflects_with_issue_calls() {
        let client = InMemoryPlaneClient::new()
            .with_issue("a", "A")
            .with_issue("b", "B")
            .with_issue("c", "C");
        assert_eq!(client.list_work_items().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn created_item_preserves_state_and_labels() {
        let client = InMemoryPlaneClient::new();
        let resp = client.create_work_item(&item("Feature")).await.unwrap();
        assert_eq!(resp.state.as_deref(), Some("started"));
        assert!(resp.updated_at.is_some());
    }
}
