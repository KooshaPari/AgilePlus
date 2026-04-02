use super::rate_limit::TokenBucket;
use super::*;

#[test]
fn token_bucket_basic() {
    let mut bucket = TokenBucket::new(5.0, 1.0);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
}

#[test]
fn token_bucket_exhaustion() {
    let mut bucket = TokenBucket::new(2.0, 0.1);
    assert!(bucket.try_acquire());
    assert!(bucket.try_acquire());
    assert!(!bucket.try_acquire()); // exhausted
}

#[tokio::test]
async fn create_module_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "mod-uuid-1",
            "name": "Auth",
            "description": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateModuleRequest {
        name: "Auth".to_string(),
        description: None,
    };
    let resp = client.create_module(&req).await.unwrap();
    assert_eq!(resp.id, "mod-uuid-1");
    assert_eq!(resp.name, "Auth");
}

#[tokio::test]
async fn create_module_http_error_propagates() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateModuleRequest {
        name: "Fail".to_string(),
        description: None,
    };
    let result = client.create_module(&req).await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("500"), "expected 500 in error: {err_msg}");
}

#[tokio::test]
async fn create_cycle_sends_correct_dates() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "cyc-uuid-1",
            "name": "Sprint 1",
            "start_date": "2026-01-01",
            "end_date": "2026-01-14"
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let req = PlaneCreateCycleRequest {
        name: "Sprint 1".to_string(),
        description: None,
        start_date: "2026-01-01".to_string(),
        end_date: "2026-01-14".to_string(),
    };
    let resp = client.create_cycle(&req).await.unwrap();
    assert_eq!(resp.id, "cyc-uuid-1");
}

#[tokio::test]
async fn add_work_item_to_cycle_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.add_work_item_to_cycle("cyc-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn create_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let work_item = PlaneWorkItem {
        id: None,
        name: "Feature".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.create_work_item(&work_item).await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn update_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let work_item = PlaneWorkItem {
        id: None,
        name: "Feature".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.update_work_item("wi-1", &work_item).await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn get_work_item_uses_work_items_root() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/workspaces/ws/projects/proj/work-items/wi-1/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "wi-1",
            "name": "Feature",
            "description_html": null,
            "state": null,
            "updated_at": null
        })))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let resp = client.get_work_item("wi-1").await.unwrap();
    assert_eq!(resp.id, "wi-1");
}

#[tokio::test]
async fn delete_work_item_from_cycle_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/wi-1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_work_item_from_cycle("cyc-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn add_work_item_to_module_sends_post() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.add_work_item_to_module("mod-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_work_item_from_module_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path(
            "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/wi-1/",
        ))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_work_item_from_module("mod-1", "wi-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_module_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/modules/mod-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_module("mod-1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn delete_cycle_sends_delete() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
    let result = client.delete_cycle("cyc-1").await;
    assert!(result.is_ok());
}

#[test]
fn plane_work_item_serialize() {
    let work_item = PlaneWorkItem {
        id: None,
        name: "Test work item".to_string(),
        description_html: Some("<p>desc</p>".to_string()),
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec!["agileplus".to_string()],
    };
    let json = serde_json::to_string(&work_item).unwrap();
    assert!(json.contains("Test work item"));
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct InMemoryWorkItem {
    response: PlaneWorkItemResponse,
    parent: Option<String>,
    labels: Vec<String>,
}

#[derive(Debug, Clone)]
struct InMemoryLabel {
    id: String,
    name: String,
    color: Option<String>,
}

#[derive(Debug)]
struct InMemoryPlaneStore {
    work_items: HashMap<String, InMemoryWorkItem>,
    labels: HashMap<String, InMemoryLabel>,
    next_id: std::sync::atomic::AtomicU64,
    modules: HashMap<String, PlaneModuleResponse>,
    cycles: HashMap<String, PlaneCycleResponse>,
}

impl InMemoryPlaneStore {
    fn new() -> Self {
        Self {
            work_items: HashMap::new(),
            labels: HashMap::new(),
            next_id: std::sync::atomic::AtomicU64::new(1),
            modules: HashMap::new(),
            cycles: HashMap::new(),
        }
    }

    fn next_id(&self) -> String {
        format!("mem-{}", self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryPlaneClient {
    store: Arc<Mutex<InMemoryPlaneStore>>,
}

impl InMemoryPlaneClient {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(InMemoryPlaneStore::new())),
        }
    }

    pub async fn create_work_item(
        &self,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let id = {
            let store = self.store.lock().unwrap();
            store.next_id()
        };
        let response = PlaneWorkItemResponse {
            id: id.clone(),
            name: work_item.name.clone(),
            description_html: work_item.description_html.clone(),
            state: work_item.state.clone(),
            updated_at: Some(chrono::Utc::now().to_rfc3339()),
        };
        let parent = work_item.parent.clone();
        let labels = work_item.labels.clone();
        {
            let mut store = self.store.lock().unwrap();
            store.work_items.insert(
                id.clone(),
                InMemoryWorkItem {
                    response: response.clone(),
                    parent,
                    labels,
                },
            );
        }
        Ok(response)
    }

    pub async fn update_work_item(
        &self,
        work_item_id: &str,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let mut store = self.store.lock().unwrap();
        let existing = store
            .work_items
            .get_mut(work_item_id)
            .ok_or_else(|| anyhow::anyhow!("work item {} not found", work_item_id))?;
        existing.response.name = work_item.name.clone();
        existing.response.description_html = work_item.description_html.clone();
        existing.response.state = work_item.state.clone();
        existing.response.updated_at = Some(chrono::Utc::now().to_rfc3339());
        existing.parent = work_item.parent.clone();
        existing.labels = work_item.labels.clone();
        Ok(existing.response.clone())
    }

    pub async fn get_work_item(&self, work_item_id: &str) -> anyhow::Result<PlaneWorkItemResponse> {
        let store = self.store.lock().unwrap();
        store
            .work_items
            .get(work_item_id)
            .map(|wi| wi.response.clone())
            .ok_or_else(|| anyhow::anyhow!("work item {} not found", work_item_id))
    }

    pub async fn list_work_items(&self) -> anyhow::Result<Vec<PlaneWorkItemResponse>> {
        let store = self.store.lock().unwrap();
        Ok(store
            .work_items
            .values()
            .map(|wi| wi.response.clone())
            .collect())
    }

    pub async fn create_sub_issue(
        &self,
        title: &str,
        description_html: Option<&str>,
        parent_issue_id: &str,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        let work_item = PlaneWorkItem {
            id: None,
            name: title.to_string(),
            description_html: description_html.map(String::from),
            state: None,
            priority: Some(3),
            parent: Some(parent_issue_id.to_string()),
            labels: vec!["agileplus".to_string(), "work-package".to_string()],
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

    pub async fn sync_labels(
        &self,
        local_labels: &[String],
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut store = self.store.lock().unwrap();
        let mut name_to_id = HashMap::new();
        for label in store.labels.values() {
            name_to_id.insert(label.name.clone(), label.id.clone());
        }
        for label_name in local_labels {
            if !name_to_id.contains_key(label_name) {
                let id = store.next_id();
                let label = InMemoryLabel {
                    id: id.clone(),
                    name: label_name.clone(),
                    color: None,
                };
                name_to_id.insert(label_name.clone(), id.clone());
                store.labels.insert(id, label);
            }
        }
        Ok(name_to_id)
    }

    pub async fn create_module(
        &self,
        req: &PlaneCreateModuleRequest,
    ) -> anyhow::Result<PlaneModuleResponse> {
        let id = {
            let store = self.store.lock().unwrap();
            store.next_id()
        };
        let response = PlaneModuleResponse {
            id: id.clone(),
            name: req.name.clone(),
            description: req.description.clone(),
        };
        let mut store = self.store.lock().unwrap();
        store.modules.insert(id, response.clone());
        Ok(response)
    }

    pub async fn add_work_item_to_module(
        &self,
        _plane_module_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn add_work_item_to_cycle(
        &self,
        _plane_cycle_id: &str,
        _plane_work_item_id: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn create_cycle(
        &self,
        req: &PlaneCreateCycleRequest,
    ) -> anyhow::Result<PlaneCycleResponse> {
        let id = {
            let store = self.store.lock().unwrap();
            store.next_id()
        };
        let response = PlaneCycleResponse {
            id: id.clone(),
            name: req.name.clone(),
            start_date: Some(req.start_date.clone()),
            end_date: Some(req.end_date.clone()),
        };
        let mut store = self.store.lock().unwrap();
        store.cycles.insert(id, response.clone());
        Ok(response)
    }
}

impl Default for InMemoryPlaneClient {
    fn default() -> Self {
        Self::new()
    }
}

#[tokio::test]
async fn in_memory_plane_client_create_issue() {
    let client = InMemoryPlaneClient::new();
    let issue = PlaneWorkItem {
        id: None,
        name: "Test Issue".to_string(),
        description_html: Some("<p>Test</p>".to_string()),
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.create_issue(&issue).await.unwrap();
    assert_eq!(resp.name, "Test Issue");
    assert!(resp.id.starts_with("mem-"));
}

#[tokio::test]
async fn in_memory_plane_client_get_issue() {
    let client = InMemoryPlaneClient::new();
    let issue = PlaneWorkItem {
        id: None,
        name: "Get Test".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let created = client.create_issue(&issue).await.unwrap();
    let retrieved = client.get_issue(&created.id).await.unwrap();
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, "Get Test");
}

#[tokio::test]
async fn in_memory_plane_client_update_issue() {
    let client = InMemoryPlaneClient::new();
    let issue = PlaneWorkItem {
        id: None,
        name: "Original".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let created = client.create_issue(&issue).await.unwrap();
    let updated_issue = PlaneWorkItem {
        id: None,
        name: "Updated".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let resp = client.update_issue(&created.id, &updated_issue).await.unwrap();
    assert_eq!(resp.name, "Updated");
}

#[tokio::test]
async fn in_memory_plane_client_list_issues() {
    let client = InMemoryPlaneClient::new();
    let issue1 = PlaneWorkItem {
        id: None,
        name: "Issue 1".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    let issue2 = PlaneWorkItem {
        id: None,
        name: "Issue 2".to_string(),
        description_html: None,
        state: None,
        priority: Some(2),
        parent: None,
        labels: vec![],
    };
    client.create_issue(&issue1).await.unwrap();
    client.create_issue(&issue2).await.unwrap();
    let list = client.list_issues().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn in_memory_plane_client_create_sub_issue() {
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
    let sub_resp = client
        .create_sub_issue("Child", Some("<p>child desc</p>"), &parent_resp.id)
        .await
        .unwrap();
    assert_eq!(sub_resp.name, "Child");
}

#[tokio::test]
async fn in_memory_plane_client_sync_labels() {
    let client = InMemoryPlaneClient::new();
    let labels = vec!["bug".to_string(), "feature".to_string()];
    let name_to_id = client.sync_labels(&labels).await.unwrap();
    assert_eq!(name_to_id.len(), 2);
    assert!(name_to_id.contains_key("bug"));
    assert!(name_to_id.contains_key("feature"));
    let second_sync = client.sync_labels(&["bug".to_string()]).await.unwrap();
    assert_eq!(second_sync.len(), 1);
}

#[tokio::test]
async fn in_memory_plane_client_sync_labels_no_duplicates() {
    let client = InMemoryPlaneClient::new();
    let labels = vec!["bug".to_string()];
    client.sync_labels(&labels).await.unwrap();
    let second_result = client.sync_labels(&["bug".to_string()]).await.unwrap();
    assert_eq!(second_result.len(), 1);
}
