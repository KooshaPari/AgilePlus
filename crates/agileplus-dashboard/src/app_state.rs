//! Shared application state threaded through Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_domain::domain::{
    cycle::Cycle, feature::Feature, module::Module, project::Project, state_machine::FeatureState,
    work_package::WorkPackage,
};
use agileplus_governance::client::GovernanceClient;
use agileplus_plane::client::PlaneClient;
use agileplus_plane::daemon::{PlaneDaemonConfig, PlaneSyncDaemon};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// A lightweight health snapshot for one service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
}

/// In-memory store used by dashboard handlers.
/// In production this would delegate to repositories.
#[derive(Default)]
pub struct DashboardStore {
    pub features: Vec<Feature>,
    pub work_packages: HashMap<i64, Vec<WorkPackage>>,
    pub modules: Vec<Module>,
    pub cycles: Vec<Cycle>,
    pub cycle_features: HashMap<i64, Vec<i64>>,
    pub health: Vec<ServiceHealth>,
    pub projects: Vec<Project>,
    pub active_project_id: Option<i64>,
    /// Optional live governance service client (for /api/dashboard/governance/*).
    pub governance_client: Option<Arc<GovernanceClient>>,
    /// Optional live plane.so sync client (for /api/dashboard/plane/*).
    pub plane_client: Option<Arc<PlaneClient>>,
    /// Optional plane.so sync daemon (background loop pulling from plane.so).
    pub plane_daemon: Option<Arc<PlaneSyncDaemon>>,
}

pub type SharedState = Arc<RwLock<DashboardStore>>;

impl DashboardStore {
    /// Create a new DashboardStore seeded with all AgilePlus dogfood features.
    ///
    /// Populates the store with:
    /// - All 4 AgilePlus kitty-specs as features (001-004)
    /// - Work packages for each feature (2-4 per feature)
    /// - Modules and cycles for native dashboard views
    /// - Default health status for all services
    /// - Seeded projects for workspace filtering
    pub fn seeded() -> Self {
        crate::seed_bridge::build_dashboard_store()
    }

    pub fn features_by_state(&self) -> HashMap<FeatureState, Vec<&Feature>> {
        let mut map: HashMap<FeatureState, Vec<&Feature>> = HashMap::new();
        for f in &self.features {
            map.entry(f.state).or_default().push(f);
        }
        map
    }

    pub fn active_project(&self) -> Option<&Project> {
        self.active_project_id
            .and_then(|id| self.projects.iter().find(|p| p.id == id))
    }

    pub fn features_for_active_project(&self) -> Vec<&Feature> {
        match self.active_project_id {
            Some(pid) => self
                .features
                .iter()
                .filter(|f| f.project_id == Some(pid))
                .collect(),
            None => self.features.iter().collect(),
        }
    }

    pub fn project_for_feature(&self, feature: &Feature) -> Option<&Project> {
        feature
            .project_id
            .and_then(|pid| self.projects.iter().find(|p| p.id == pid))
    }

    pub fn feature_counts_for_project(&self, project_id: i64) -> (usize, usize, usize) {
        let features: Vec<&Feature> = self
            .features
            .iter()
            .filter(|f| f.project_id == Some(project_id))
            .collect();
        let total = features.len();
        let active = features
            .iter()
            .filter(|f| !matches!(f.state, FeatureState::Shipped | FeatureState::Retrospected))
            .count();
        let shipped = features
            .iter()
            .filter(|f| matches!(f.state, FeatureState::Shipped | FeatureState::Retrospected))
            .count();
        (total, active, shipped)
    }

    pub fn feature_counts_for_module(&self, module_id: i64) -> (usize, usize, usize) {
        let features: Vec<&Feature> = self
            .features
            .iter()
            .filter(|feature| feature.module_id == Some(module_id))
            .collect();
        let total = features.len();
        let active = features
            .iter()
            .filter(|feature| {
                !matches!(
                    feature.state,
                    FeatureState::Shipped | FeatureState::Retrospected
                )
            })
            .count();
        let shipped = features
            .iter()
            .filter(|feature| {
                matches!(
                    feature.state,
                    FeatureState::Shipped | FeatureState::Retrospected
                )
            })
            .count();
        (total, active, shipped)
    }

    pub fn work_package_count_for_module(&self, module_id: i64) -> usize {
        self.features
            .iter()
            .filter(|feature| feature.module_id == Some(module_id))
            .map(|feature| self.work_packages.get(&feature.id).map_or(0, Vec::len))
            .sum()
    }

    pub fn cycle_feature_ids(&self, cycle_id: i64) -> Vec<i64> {
        self.cycle_features
            .get(&cycle_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn cycle_work_package_count(&self, cycle_id: i64) -> usize {
        self.cycle_feature_ids(cycle_id)
            .into_iter()
            .map(|feature_id| self.work_packages.get(&feature_id).map_or(0, Vec::len))
            .sum()
    }

    pub fn cycle_is_shippable(&self, cycle_id: i64) -> bool {
        let feature_ids = self.cycle_feature_ids(cycle_id);
        !feature_ids.is_empty()
            && feature_ids.into_iter().all(|feature_id| {
                self.features
                    .iter()
                    .find(|feature| feature.id == feature_id)
                    .map(|feature| {
                        matches!(
                            feature.state,
                            FeatureState::Validated | FeatureState::Shipped
                        )
                    })
                    .unwrap_or(false)
            })
    }
}

impl DashboardStore {
    /// Install the live governance client (after with_defaults())
    pub fn with_governance(mut self, client: agileplus_governance::GovernanceClient) -> Self {
        self.governance_client = Some(std::sync::Arc::new(client));
        self
    }

    /// Install the live plane client (after PlaneClient::new)
    pub fn with_plane(mut self, client: agileplus_plane::PlaneClient) -> Self {
        self.plane_client = Some(std::sync::Arc::new(client));
        self
    }

    /// Install the plane.so sync daemon handle (after daemon.spawn).
    pub fn with_plane_daemon(mut self, daemon: PlaneSyncDaemon) -> Self {
        self.plane_daemon = Some(std::sync::Arc::new(daemon));
        self
    }

    /// Build a default daemon config from env vars.
    pub fn default_plane_daemon_config() -> PlaneDaemonConfig {
        PlaneDaemonConfig::from_env()
    }
}

pub fn default_health() -> Vec<ServiceHealth> {
    let now = Utc::now();
    vec![
        ServiceHealth {
            name: "NATS".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(2),
            last_check: now,
        },
        ServiceHealth {
            name: "Dragonfly".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(1),
            last_check: now,
        },
        ServiceHealth {
            name: "Neo4j".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(8),
            last_check: now,
        },
        ServiceHealth {
            name: "MinIO".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(5),
            last_check: now,
        },
        ServiceHealth {
            name: "SQLite".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(0),
            last_check: now,
        },
        ServiceHealth {
            name: "API".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(3),
            last_check: now,
        },
        ServiceHealth {
            name: "Plane API".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(12),
            last_check: now,
        },
        ServiceHealth {
            name: "Plane Web".into(),
            healthy: true,
            degraded: false,
            latency_ms: Some(8),
            last_check: now,
        },
    ]
}

#[cfg(test)]
mod tests {
    use agileplus_domain::domain::state_machine::FeatureState;

    use super::DashboardStore;

    #[test]
    fn cycle_without_feature_scope_is_not_shippable() {
        let store = DashboardStore::seeded();

        assert!(!store.cycle_is_shippable(999));
    }

    #[test]
    fn seeded_store_groups_feature_states_without_losing_features() {
        let store = DashboardStore::seeded();
        let by_state = store.features_by_state();

        assert_eq!(by_state[&FeatureState::Shipped].len(), 36);
        assert_eq!(by_state[&FeatureState::Implementing].len(), 1);
        assert_eq!(
            by_state.values().map(Vec::len).sum::<usize>(),
            store.features.len()
        );
    }

    #[test]
    fn active_project_scopes_seeded_feature_counts() {
        let store = DashboardStore::seeded();

        assert_eq!(
            store.active_project().map(|project| project.slug.as_str()),
            Some("agileplus-internal")
        );
        assert_eq!(store.features_for_active_project().len(), 37);
        assert_eq!(store.feature_counts_for_project(1), (37, 1, 36));
        assert_eq!(store.feature_counts_for_project(999), (0, 0, 0));
    }

    #[test]
    fn cycle_aggregation_counts_seeded_features_and_work_packages() {
        let store = DashboardStore::seeded();

        assert_eq!(store.cycle_feature_ids(1).len(), 37);
        assert_eq!(store.cycle_work_package_count(1), 80);
        assert!(!store.cycle_is_shippable(1));
    }

    // ── Custom-store unit tests ──────────────────────────────────────────────

    use agileplus_domain::domain::project::Project;
    use agileplus_domain::domain::work_package::{WorkPackage, WpState};

    fn make_feature(
        id: i64,
        state: FeatureState,
        project_id: Option<i64>,
        module_id: Option<i64>,
    ) -> agileplus_domain::domain::feature::Feature {
        let mut f = agileplus_domain::domain::feature::Feature::new(
            &format!("feat-{id}"),
            &format!("Feature {id}"),
            [0; 32],
            None,
        );
        f.id = id;
        f.state = state;
        f.project_id = project_id;
        f.module_id = module_id;
        f
    }

    fn make_project(id: i64, name: &str, slug: &str) -> Project {
        let mut p = Project::new(name, slug).expect("valid project");
        p.id = id;
        p
    }

    fn make_wp(id: i64, feature_id: i64, state: WpState) -> WorkPackage {
        let mut wp = WorkPackage::new(feature_id, &format!("WP-{id}"), 1, "done");
        wp.id = id;
        wp.state = state;
        wp
    }

    // ── features_by_state ─────────────────────────────────────────────────────

    #[test]
    fn custom_features_by_state_empty_store() {
        let store = DashboardStore::default();
        let map = store.features_by_state();
        assert!(map.is_empty());
    }

    #[test]
    fn custom_features_by_state_groups_correctly() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, None, None),
            make_feature(2, FeatureState::Created, None, None),
            make_feature(3, FeatureState::Shipped, None, None),
            make_feature(4, FeatureState::Implementing, None, None),
        ];
        let map = store.features_by_state();
        assert_eq!(
            map.get(&FeatureState::Created).map_or(0, |v| v.len()),
            2
        );
        assert_eq!(
            map.get(&FeatureState::Shipped).map_or(0, |v| v.len()),
            1
        );
        assert_eq!(
            map.get(&FeatureState::Implementing).map_or(0, |v| v.len()),
            1
        );
        assert_eq!(
            map.get(&FeatureState::Planned).map_or(0, |v| v.len()),
            0
        );
    }

    // ── active_project ────────────────────────────────────────────────────────

    #[test]
    fn custom_active_project_none_when_no_active_id() {
        let store = DashboardStore::default();
        assert!(store.active_project().is_none());
    }

    #[test]
    fn custom_active_project_none_when_id_not_found() {
        let mut store = DashboardStore::default();
        store.projects = vec![make_project(1, "P1", "p1")];
        store.active_project_id = Some(999);
        assert!(store.active_project().is_none());
    }

    #[test]
    fn custom_active_project_returns_matching_project() {
        let mut store = DashboardStore::default();
        store.projects = vec![make_project(1, "P1", "p1"), make_project(2, "P2", "p2")];
        store.active_project_id = Some(2);
        let proj = store.active_project().expect("should find project");
        assert_eq!(proj.id, 2);
        assert_eq!(proj.name, "P2");
    }

    // ── features_for_active_project ───────────────────────────────────────────

    #[test]
    fn custom_features_for_active_project_returns_all_when_no_active() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, Some(1), None),
            make_feature(2, FeatureState::Created, Some(2), None),
        ];
        store.active_project_id = None;
        assert_eq!(store.features_for_active_project().len(), 2);
    }

    #[test]
    fn custom_features_for_active_project_filters_by_project() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, Some(1), None),
            make_feature(2, FeatureState::Created, Some(2), None),
            make_feature(3, FeatureState::Created, Some(1), None),
        ];
        store.active_project_id = Some(1);
        let features = store.features_for_active_project();
        assert_eq!(features.len(), 2);
        assert!(features.iter().all(|f| f.project_id == Some(1)));
    }

    // ── project_for_feature ───────────────────────────────────────────────────

    #[test]
    fn custom_project_for_feature_none_when_no_project_id() {
        let mut store = DashboardStore::default();
        store.projects = vec![make_project(1, "P1", "p1")];
        let f = make_feature(1, FeatureState::Created, None, None);
        assert!(store.project_for_feature(&f).is_none());
    }

    #[test]
    fn custom_project_for_feature_none_when_project_missing() {
        let mut store = DashboardStore::default();
        store.projects = vec![make_project(1, "P1", "p1")];
        let f = make_feature(1, FeatureState::Created, Some(999), None);
        assert!(store.project_for_feature(&f).is_none());
    }

    #[test]
    fn custom_project_for_feature_finds_match() {
        let mut store = DashboardStore::default();
        store.projects = vec![make_project(1, "P1", "p1"), make_project(2, "P2", "p2")];
        let f = make_feature(1, FeatureState::Created, Some(2), None);
        let proj = store.project_for_feature(&f).expect("should find project");
        assert_eq!(proj.id, 2);
    }

    // ── feature_counts_for_project ────────────────────────────────────────────

    #[test]
    fn custom_feature_counts_for_project_zero_when_no_features() {
        let store = DashboardStore::default();
        let (total, active, shipped) = store.feature_counts_for_project(1);
        assert_eq!(total, 0);
        assert_eq!(active, 0);
        assert_eq!(shipped, 0);
    }

    #[test]
    fn custom_feature_counts_for_project_correct_counts() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, Some(1), None),
            make_feature(2, FeatureState::Implementing, Some(1), None),
            make_feature(3, FeatureState::Shipped, Some(1), None),
            make_feature(4, FeatureState::Retrospected, Some(1), None),
            make_feature(5, FeatureState::Created, Some(2), None),
        ];
        let (total, active, shipped) = store.feature_counts_for_project(1);
        assert_eq!(total, 4);
        assert_eq!(active, 2);
        assert_eq!(shipped, 2);
    }

    #[test]
    fn custom_feature_counts_for_project_ignores_other_projects() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, Some(2), None),
            make_feature(2, FeatureState::Shipped, Some(2), None),
        ];
        let (total, active, shipped) = store.feature_counts_for_project(1);
        assert_eq!(total, 0);
        assert_eq!(active, 0);
        assert_eq!(shipped, 0);
    }

    // ── feature_counts_for_module ─────────────────────────────────────────────

    #[test]
    fn custom_feature_counts_for_module_zero_when_no_features() {
        let store = DashboardStore::default();
        let (total, active, shipped) = store.feature_counts_for_module(1);
        assert_eq!(total, 0);
        assert_eq!(active, 0);
        assert_eq!(shipped, 0);
    }

    #[test]
    fn custom_feature_counts_for_module_correct_counts() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, None, Some(1)),
            make_feature(2, FeatureState::Implementing, None, Some(1)),
            make_feature(3, FeatureState::Shipped, None, Some(1)),
            make_feature(4, FeatureState::Created, None, Some(2)),
        ];
        let (total, active, shipped) = store.feature_counts_for_module(1);
        assert_eq!(total, 3);
        assert_eq!(active, 2);
        assert_eq!(shipped, 1);
    }

    // ── work_package_count_for_module ─────────────────────────────────────────

    #[test]
    fn custom_work_package_count_for_module_zero_when_no_features() {
        let store = DashboardStore::default();
        assert_eq!(store.work_package_count_for_module(1), 0);
    }

    #[test]
    fn custom_work_package_count_for_module_sums_work_packages() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Created, None, Some(1)),
            make_feature(2, FeatureState::Created, None, Some(1)),
            make_feature(3, FeatureState::Created, None, Some(2)),
        ];
        store.work_packages.insert(
            1,
            vec![make_wp(10, 1, WpState::Planned), make_wp(11, 1, WpState::Doing)],
        );
        store
            .work_packages
            .insert(2, vec![make_wp(20, 2, WpState::Done)]);
        store
            .work_packages
            .insert(3, vec![make_wp(30, 3, WpState::Planned)]);
        assert_eq!(store.work_package_count_for_module(1), 3);
        assert_eq!(store.work_package_count_for_module(2), 1);
    }

    // ── cycle_feature_ids ─────────────────────────────────────────────────────

    #[test]
    fn custom_cycle_feature_ids_empty_when_not_found() {
        let store = DashboardStore::default();
        assert!(store.cycle_feature_ids(999).is_empty());
    }

    #[test]
    fn custom_cycle_feature_ids_returns_stored_ids() {
        let mut store = DashboardStore::default();
        store.cycle_features.insert(1, vec![10, 20, 30]);
        assert_eq!(store.cycle_feature_ids(1), vec![10, 20, 30]);
    }

    // ── cycle_work_package_count ──────────────────────────────────────────────

    #[test]
    fn custom_cycle_work_package_count_zero_when_no_features() {
        let store = DashboardStore::default();
        assert_eq!(store.cycle_work_package_count(1), 0);
    }

    #[test]
    fn custom_cycle_work_package_count_sums_wp_for_cycle_features() {
        let mut store = DashboardStore::default();
        store.cycle_features.insert(1, vec![10, 20]);
        store
            .work_packages
            .insert(10, vec![make_wp(100, 10, WpState::Planned)]);
        store.work_packages.insert(
            20,
            vec![
                make_wp(200, 20, WpState::Doing),
                make_wp(201, 20, WpState::Done),
            ],
        );
        assert_eq!(store.cycle_work_package_count(1), 3);
    }

    // ── cycle_is_shippable ───────────────────────────────────────────────────

    #[test]
    fn custom_cycle_is_shippable_false_when_empty() {
        let store = DashboardStore::default();
        assert!(!store.cycle_is_shippable(1));
    }

    #[test]
    fn custom_cycle_is_shippable_true_when_all_validated_or_shipped() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Validated, None, None),
            make_feature(2, FeatureState::Shipped, None, None),
        ];
        store.cycle_features.insert(1, vec![1, 2]);
        assert!(store.cycle_is_shippable(1));
    }

    #[test]
    fn custom_cycle_is_shippable_false_when_any_feature_not_ready() {
        let mut store = DashboardStore::default();
        store.features = vec![
            make_feature(1, FeatureState::Validated, None, None),
            make_feature(2, FeatureState::Implementing, None, None),
        ];
        store.cycle_features.insert(1, vec![1, 2]);
        assert!(!store.cycle_is_shippable(1));
    }

    #[test]
    fn custom_cycle_is_shippable_false_when_feature_not_found() {
        let mut store = DashboardStore::default();
        store.cycle_features.insert(1, vec![999]);
        assert!(!store.cycle_is_shippable(1));
    }

    // ── default_health ────────────────────────────────────────────────────────

    #[test]
    fn custom_default_health_returns_all_services() {
        let health = super::default_health();
        assert!(health.len() >= 7);
        let names: Vec<&str> = health.iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"NATS"));
        assert!(names.contains(&"Dragonfly"));
        assert!(names.contains(&"Neo4j"));
        assert!(names.contains(&"SQLite"));
        assert!(names.contains(&"API"));
    }

    #[test]
    fn custom_default_health_all_healthy() {
        let health = super::default_health();
        assert!(health.iter().all(|h| h.healthy));
        assert!(health.iter().all(|h| !h.degraded));
    }

    // ── default_plane_daemon_config ───────────────────────────────────────────

    #[test]
    fn default_plane_daemon_config_reads_daemon_env_vars() {
        use std::time::Duration;

        // The `PLANE_DAEMON_*` variables are read only by this helper inside the
        // dashboard crate, so no other test in this binary can observe them.
        for key in [
            "PLANE_DAEMON_INTERVAL_SECS",
            "PLANE_DAEMON_BATCH_SIZE",
            "PLANE_DAEMON_DRY_RUN",
        ] {
            // SAFETY: single-threaded use of variables no concurrent test reads.
            unsafe { std::env::remove_var(key) };
        }
        let defaults = DashboardStore::default_plane_daemon_config();
        assert_eq!(defaults.interval, Duration::from_secs(5 * 60));
        assert_eq!(defaults.batch_size, 25);
        assert!(!defaults.dry_run);

        unsafe {
            std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "7");
            std::env::set_var("PLANE_DAEMON_BATCH_SIZE", "3");
            std::env::set_var("PLANE_DAEMON_DRY_RUN", "true");
        }
        let configured = DashboardStore::default_plane_daemon_config();
        assert_eq!(configured.interval, Duration::from_secs(7));
        assert_eq!(configured.batch_size, 3);
        assert!(configured.dry_run);

        // An unparseable interval falls back to the default rather than panicking.
        unsafe { std::env::set_var("PLANE_DAEMON_INTERVAL_SECS", "not-a-number") };
        assert_eq!(
            DashboardStore::default_plane_daemon_config().interval,
            Duration::from_secs(5 * 60)
        );

        unsafe {
            std::env::remove_var("PLANE_DAEMON_INTERVAL_SECS");
            std::env::remove_var("PLANE_DAEMON_BATCH_SIZE");
            std::env::remove_var("PLANE_DAEMON_DRY_RUN");
        }
    }
}
