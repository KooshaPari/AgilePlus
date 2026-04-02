//! In-memory task repository adapter

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;

use crate::domain::entity::Entity;
use crate::domain::{EntityId, Task};
use crate::ports::output::{RepositoryError, RepositoryResult, TaskRepository};

#[derive(Debug, Default)]
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    by_work_package: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn save(&self, task: Task) -> RepositoryResult<Task> {
        let id = task.id().as_str().to_string();
        let wp_id = task.work_package_id().as_str().to_string();

        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), task.clone());

        let mut by_wp = self.by_work_package.write().await;
        by_wp.entry(wp_id).or_default().push(id);

        Ok(task)
    }

    async fn find_by_id(&self, id: &EntityId) -> RepositoryResult<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .get(id.as_str())
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(id.as_str().to_string()))
    }

    async fn find_by_work_package(
        &self,
        work_package_id: &EntityId,
    ) -> RepositoryResult<Vec<Task>> {
        let by_wp = self.by_work_package.read().await;
        let task_ids = by_wp
            .get(work_package_id.as_str())
            .cloned()
            .unwrap_or_default();

        let tasks = self.tasks.read().await;
        Ok(task_ids
            .iter()
            .filter_map(|id| tasks.get(id).cloned())
            .collect())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    async fn delete(&self, id: &EntityId) -> RepositoryResult<()> {
        let mut tasks = self.tasks.write().await;
        tasks
            .remove(id.as_str())
            .ok_or_else(|| RepositoryError::NotFound(id.as_str().to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl crate::ports::OutputPort for InMemoryTaskRepository {
    fn name(&self) -> &'static str {
        "InMemoryTaskRepository"
    }
}
