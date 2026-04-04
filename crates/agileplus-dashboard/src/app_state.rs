//! Shared application state threaded through Axum handlers.

use std::collections::HashMap;
use std::sync::Arc;

use agileplus_domain::domain::{
    cycle::Cycle, feature::Feature, module::Module, project::Project, state_machine::FeatureState,
    work_package::WorkPackage,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Phenotype Health Integration
use phenotype_health::{HealthStatus, HealthRegistry};
use phenotype_health_cli::{UnifiedHealthReport, UnifiedHealthScanner};

/// A lightweight health snapshot for one service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
}

impl ServiceHealth {
    /// Convert from phenotype-health HealthStatus
    pub fn from_phenotype_status(name: String, status: &HealthStatus, latency_ms: Option<u64>) -> Self {
        let (healthy, degraded) = match status {
            HealthStatus::Healthy => (true, false),
            HealthStatus::Degraded => (true, true),
            HealthStatus::Unhealthy => (false, false),
        };
        
        Self {
            name,
            healthy,
            degraded,
            latency_ms,
            last_check: Utc::now(),
        }
    }
}

/// Unified health snapshot combining phenotype-health with dashboard health
#[derive(Debug, Clone, Default)]
pub struct UnifiedHealthSnapshot {
    /// Legacy service health entries
    pub services: Vec<ServiceHealth>,
    /// Full unified health report from phenotype-health
    pub unified_report: Option<UnifiedHealthReport>,
    /// When the health was last updated
    pub last_updated: DateTime<Utc>,
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
    /// Unified health snapshot with phenotype-health integration
    pub unified_health: UnifiedHealthSnapshot,
    pub projects: Vec<Project>,
    pub active_project_id: Option<i64>,
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
        self.cycle_feature_ids(cycle_id)
            .into_iter()
            .all(|feature_id| {
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

    /// Update unified health from phenotype-health scanner
    /// 
    /// This method runs the unified health scanner and updates both
    /// the legacy health vector and the new unified health snapshot.
    pub async fn update_unified_health(&mut self, scan_path: &std::path::Path) -> anyhow::Result<()> {
        use std::path::PathBuf;
        
        let mut scanner = UnifiedHealthScanner::new();
        let report = scanner.scan_workspace(scan_path).await;
        
        // Convert unified report to legacy ServiceHealth entries
        let mut services: Vec<ServiceHealth> = report
            .projects
            .iter()
            .map(|p| ServiceHealth::from_phenotype_status(
                p.name.clone(),
                &p.status,
                None,
            ))
            .collect();
        
        // Add default infrastructure services if not already present
        let existing_names: std::collections::HashSet<_> = services
            .iter()
            .map(|s| s.name.clone())
            .collect();
        
        for default_service in default_health() {
            if !existing_names.contains(&default_service.name) {
                services.push(default_service);
            }
        }
        
        // Update legacy health field (for backward compatibility)
        self.health = services.clone();
        
        // Update unified health snapshot
        self.unified_health = UnifiedHealthSnapshot {
            services,
            unified_report: Some(report),
            last_updated: Utc::now(),
        };
        
        Ok(())
    }
    
    /// Get unified health metrics for dashboard display
    pub fn unified_health_metrics(&self) -> Option<UnifiedHealthMetrics> {
        self.unified_health.unified_report.as_ref().map(|report| {
            UnifiedHealthMetrics {
                total_projects: report.metrics.total_projects,
                healthy_projects: report.metrics.healthy_projects,
                degraded_projects: report.metrics.degraded_projects,
                unhealthy_projects: report.metrics.unhealthy_projects,
                critical_findings: report.metrics.critical_findings,
                high_findings: report.metrics.high_findings,
                compliance_score: if report.projects.is_empty() {
                    100.0
                } else {
                    report.projects.iter().map(|p| p.compliance_score).sum::<f32>() 
                        / report.projects.len() as f32
                },
                security_risk_score: if report.projects.is_empty() {
                    0.0
                } else {
                    report.projects.iter().map(|p| p.security_risk_score).sum::<f32>() 
                        / report.projects.len() as f32
                },
            }
        })
    }
}

/// Unified health metrics for dashboard display
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedHealthMetrics {
    pub total_projects: usize,
    pub healthy_projects: usize,
    pub degraded_projects: usize,
    pub unhealthy_projects: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub compliance_score: f32,
    pub security_risk_score: f32,
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
