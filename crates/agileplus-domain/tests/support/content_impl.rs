use super::*;

#[async_trait]
impl ContentStoragePort for RecordingContentStorage {
    async fn create_feature(&self, feature: &Feature) -> Result<i64, DomainError> {
        let _ = feature;
        self.record("create_feature");
        Ok(self.next_id)
    }
    async fn get_feature_by_slug(&self, slug: &str) -> Result<Option<Feature>, DomainError> {
        let _ = slug;
        self.record("get_feature_by_slug");
        Ok(None)
    }
    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>, DomainError> {
        let _ = id;
        self.record("get_feature_by_id");
        Ok(None)
    }
    async fn update_feature_state(&self, id: i64, state: FeatureState) -> Result<(), DomainError> {
        let _ = id;
        let _ = state;
        self.record("update_feature_state");
        Ok(())
    }
    async fn update_feature(&self, feature: &Feature) -> Result<(), DomainError> {
        let _ = feature;
        self.record("update_feature");
        Ok(())
    }
    async fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> Result<Vec<Feature>, DomainError> {
        let _ = state;
        self.record("list_features_by_state");
        Ok(Vec::new())
    }
    async fn list_all_features(&self) -> Result<Vec<Feature>, DomainError> {
        self.record("list_all_features");
        Ok(Vec::new())
    }
    async fn create_work_package(&self, wp: &WorkPackage) -> Result<i64, DomainError> {
        let _ = wp;
        self.record("create_work_package");
        Ok(self.next_id)
    }
    async fn get_work_package(&self, id: i64) -> Result<Option<WorkPackage>, DomainError> {
        let _ = id;
        self.record("get_work_package");
        Ok(None)
    }
    async fn update_wp_state(&self, id: i64, state: WpState) -> Result<(), DomainError> {
        let _ = id;
        let _ = state;
        self.record("update_wp_state");
        Ok(())
    }
    async fn update_work_package(&self, wp: &WorkPackage) -> Result<(), DomainError> {
        let _ = wp;
        self.record("update_work_package");
        Ok(())
    }
    async fn list_wps_by_feature(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let _ = feature_id;
        self.record("list_wps_by_feature");
        Ok(Vec::new())
    }
    async fn add_wp_dependency(&self, dep: &WpDependency) -> Result<(), DomainError> {
        let _ = dep;
        self.record("add_wp_dependency");
        Ok(())
    }
    async fn get_wp_dependencies(&self, wp_id: i64) -> Result<Vec<WpDependency>, DomainError> {
        let _ = wp_id;
        self.record("get_wp_dependencies");
        Ok(Vec::new())
    }
    async fn get_ready_wps(&self, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
        let _ = feature_id;
        self.record("get_ready_wps");
        Ok(Vec::new())
    }
    async fn create_backlog_item(&self, item: &BacklogItem) -> Result<i64, DomainError> {
        let _ = item;
        self.record("create_backlog_item");
        Ok(self.next_id)
    }
    async fn get_backlog_item(&self, id: i64) -> Result<Option<BacklogItem>, DomainError> {
        let _ = id;
        self.record("get_backlog_item");
        Ok(None)
    }
    async fn list_backlog_items(
        &self,
        filters: &BacklogFilters,
    ) -> Result<Vec<BacklogItem>, DomainError> {
        let _ = filters;
        self.record("list_backlog_items");
        Ok(Vec::new())
    }
    async fn update_backlog_status(
        &self,
        id: i64,
        status: BacklogStatus,
    ) -> Result<(), DomainError> {
        let _ = id;
        let _ = status;
        self.record("update_backlog_status");
        Ok(())
    }
    async fn update_backlog_priority(
        &self,
        id: i64,
        priority: BacklogPriority,
    ) -> Result<(), DomainError> {
        let _ = id;
        let _ = priority;
        self.record("update_backlog_priority");
        Ok(())
    }
    async fn pop_next_backlog_item(&self) -> Result<Option<BacklogItem>, DomainError> {
        self.record("pop_next_backlog_item");
        Ok(None)
    }
}
