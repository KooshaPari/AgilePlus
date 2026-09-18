// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-memory doubles for the smaller repository ports, plus the event spy.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;
use tokio::sync::RwLock;

use agileplus_domain::domain::epic::{Epic, EpicStatus};
use agileplus_domain::domain::story::{Story, StoryStatus};
use agileplus_domain::error::DomainError;
use agileplus_domain::ports::epic::EpicRepository;
use agileplus_domain::ports::events::{DomainEvent, DomainEventPublisher};
use agileplus_domain::ports::story::StoryRepository;

use crate::dto::PickedItem;
use crate::use_cases::triage::WpRepository;

/// In-memory Story store.
///
/// # Fault injection
///
/// The `failing_*` builders flip a flag so one operation starts returning
/// [`DomainError::Storage`] (see [`super::storage::InMemoryFeatureRepo`] for
/// the convention).
#[derive(Default)]
pub struct InMemoryStoryRepo {
    store: RwLock<HashMap<i64, Story>>,
    next_id: RwLock<i64>,
    fail_create: AtomicBool,
    fail_read: AtomicBool,
    fail_write: AtomicBool,
    fail_list: AtomicBool,
}

impl InMemoryStoryRepo {
    /// The error every injected fault returns.
    fn offline() -> DomainError {
        DomainError::Storage("in-memory store offline".to_string())
    }

    /// Make `create` fail.
    pub fn failing_create(self) -> Self {
        self.fail_create.store(true, Ordering::SeqCst);
        self
    }

    /// Make `get_by_id` fail.
    pub fn failing_read(self) -> Self {
        self.fail_read.store(true, Ordering::SeqCst);
        self
    }

    /// Make `update_status` fail.
    pub fn failing_status_write(self) -> Self {
        self.fail_write.store(true, Ordering::SeqCst);
        self
    }

    /// Make `list_by_epic` fail (also fails the default
    /// `upsert_by_requirement_id`).
    pub fn failing_list(self) -> Self {
        self.fail_list.store(true, Ordering::SeqCst);
        self
    }
}

#[async_trait]
impl StoryRepository for InMemoryStoryRepo {
    async fn create(&self, story: &Story) -> Result<i64, DomainError> {
        if self.fail_create.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut s = story.clone();
        s.id = id;
        self.store.write().await.insert(id, s);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Story>, DomainError> {
        if self.fail_read.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_status(&self, id: i64, status: StoryStatus) -> Result<(), DomainError> {
        if self.fail_write.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        let mut store = self.store.write().await;
        if let Some(s) = store.get_mut(&id) {
            s.status = status;
            Ok(())
        } else {
            Err(DomainError::NotFound(id.to_string()))
        }
    }

    async fn list_by_epic(&self, epic_id: i64) -> Result<Vec<Story>, DomainError> {
        if self.fail_list.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|s| s.epic_id == epic_id)
            .cloned()
            .collect())
    }
}

/// In-memory Epic store.
///
/// Has the same fault-injection surface as [`InMemoryStoryRepo`].
#[derive(Default)]
pub struct InMemoryEpicRepo {
    store: RwLock<HashMap<i64, Epic>>,
    next_id: RwLock<i64>,
    fail_create: AtomicBool,
    fail_read: AtomicBool,
}

impl InMemoryEpicRepo {
    /// The error every injected fault returns.
    fn offline() -> DomainError {
        DomainError::Storage("in-memory store offline".to_string())
    }

    /// Make `create` fail.
    pub fn failing_create(self) -> Self {
        self.fail_create.store(true, Ordering::SeqCst);
        self
    }

    /// Make `get_by_id` fail.
    pub fn failing_read(self) -> Self {
        self.fail_read.store(true, Ordering::SeqCst);
        self
    }
}

#[async_trait]
impl EpicRepository for InMemoryEpicRepo {
    async fn create(&self, epic: &Epic) -> Result<i64, DomainError> {
        if self.fail_create.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        let mut next = self.next_id.write().await;
        *next += 1;
        let id = *next;
        let mut e = epic.clone();
        e.id = id;
        self.store.write().await.insert(id, e);
        Ok(id)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<Epic>, DomainError> {
        if self.fail_read.load(Ordering::SeqCst) {
            return Err(Self::offline());
        }
        Ok(self.store.read().await.get(&id).cloned())
    }

    async fn update_status(&self, id: i64, status: EpicStatus) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if let Some(e) = store.get_mut(&id) {
            e.status = status;
            Ok(())
        } else {
            Err(DomainError::NotFound(id.to_string()))
        }
    }

    async fn list_by_project(&self, project_id: i64) -> Result<Vec<Epic>, DomainError> {
        Ok(self
            .store
            .read()
            .await
            .values()
            .filter(|e| e.project_id == project_id)
            .cloned()
            .collect())
    }
}

/// Spy publisher — records emitted events.
#[derive(Default)]
pub struct SpyPublisher {
    events: Mutex<Vec<DomainEvent>>,
}

#[async_trait]
impl DomainEventPublisher for SpyPublisher {
    fn publish(&self, event: DomainEvent) -> Result<(), DomainError> {
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

impl SpyPublisher {
    pub fn emitted(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }
}

/// A `list_pickable` call recorded by [`InMemoryWpRepo`].
///
/// Lets tests assert that a use case forwards request fields to the port
/// unchanged (agent, lane, category, limit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WpQuery {
    pub agent: String,
    pub lane: Option<String>,
    pub category: Option<String>,
    pub limit: usize,
}

/// In-memory `WpRepository` double.
///
/// Items are returned in insertion order (capped at the requested `limit`).
/// Every query and every `mark_done` / `add_dependency` call is recorded so
/// use-case behavior can be asserted from the outside.
#[derive(Default)]
pub struct InMemoryWpRepo {
    items: Vec<PickedItem>,
    dependencies: Vec<(String, String)>,
    done: Vec<String>,
    queries: Mutex<Vec<WpQuery>>,
    fail_pick: bool,
    fail_export: bool,
    fail_mark_done: bool,
}

impl InMemoryWpRepo {
    /// Build a repo holding `items`.
    pub fn with_items(items: Vec<PickedItem>) -> Self {
        Self {
            items,
            ..Self::default()
        }
    }

    /// Make `list_pickable` fail.
    pub fn failing_pick(mut self) -> Self {
        self.fail_pick = true;
        self
    }

    /// Make `all_for_export` fail.
    pub fn failing_export(mut self) -> Self {
        self.fail_export = true;
        self
    }

    /// Make `mark_done` fail.
    pub fn failing_mark_done(mut self) -> Self {
        self.fail_mark_done = true;
        self
    }

    /// Every `list_pickable` call seen so far, in order.
    pub fn recorded_queries(&self) -> Vec<WpQuery> {
        self.queries.lock().unwrap().clone()
    }

    /// Work-package ids passed to `mark_done`, in order.
    pub fn done_ids(&self) -> Vec<String> {
        self.done.clone()
    }

    /// `add_dependency(from, to)` pairs, in order.
    pub fn recorded_dependencies(&self) -> Vec<(String, String)> {
        self.dependencies.clone()
    }
}

impl WpRepository for InMemoryWpRepo {
    fn list_pickable(
        &self,
        agent: &str,
        lane: Option<&str>,
        category: Option<&str>,
        limit: usize,
    ) -> anyhow::Result<Vec<PickedItem>> {
        if self.fail_pick {
            return Err(anyhow::anyhow!("wp store offline"));
        }
        self.queries.lock().unwrap().push(WpQuery {
            agent: agent.to_string(),
            lane: lane.map(str::to_string),
            category: category.map(str::to_string),
            limit,
        });
        Ok(self.items.iter().take(limit).cloned().collect())
    }

    fn all_for_export(&self, _with_side: bool) -> anyhow::Result<Vec<PickedItem>> {
        if self.fail_export {
            return Err(anyhow::anyhow!("wp store offline"));
        }
        Ok(self.items.clone())
    }

    fn add_dependency(&mut self, from: &str, to: &str) -> anyhow::Result<()> {
        self.dependencies.push((from.to_string(), to.to_string()));
        Ok(())
    }

    fn mark_done(&mut self, wp_id: &str) -> anyhow::Result<()> {
        if self.fail_mark_done {
            return Err(anyhow::anyhow!("wp store offline"));
        }
        self.done.push(wp_id.to_string());
        Ok(())
    }

    fn claim_count(&self) -> usize {
        self.done.len()
    }

    fn wp_count(&self) -> usize {
        self.items.len()
    }

    fn stage_count(&self) -> usize {
        3
    }
}
