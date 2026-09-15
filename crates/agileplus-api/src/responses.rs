// SPDX-License-Identifier: MIT OR Apache-2.0
//! API response types — stable JSON shapes for all endpoints.
//!
//! These types are separate from domain entities so internal representations
//! can change without breaking the public API contract.
//!
//! Traceability: WP15-T086

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use agileplus_domain::domain::audit::AuditEntry;
use agileplus_domain::domain::epic::Epic;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::governance::GovernanceContract;
use agileplus_domain::domain::project::Project;
use agileplus_domain::domain::story::Story;
use agileplus_domain::domain::user::User;
use agileplus_domain::domain::work_package::WorkPackage;

// ----- Features -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct FeatureResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub state: String,
    pub target_branch: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Feature> for FeatureResponse {
    fn from(f: Feature) -> Self {
        Self {
            id: f.id,
            slug: f.slug,
            name: f.friendly_name,
            state: format!("{:?}", f.state).to_lowercase(),
            target_branch: f.target_branch,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

// ----- Work Packages -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkPackageResponse {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: String,
    pub sequence: i32,
    pub acceptance_criteria: String,
    pub pr_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<WorkPackage> for WorkPackageResponse {
    fn from(wp: WorkPackage) -> Self {
        Self {
            id: wp.id,
            feature_id: wp.feature_id,
            title: wp.title,
            state: format!("{:?}", wp.state).to_lowercase(),
            sequence: wp.sequence,
            acceptance_criteria: wp.acceptance_criteria,
            pr_url: wp.pr_url,
            created_at: wp.created_at.to_rfc3339(),
            updated_at: wp.updated_at.to_rfc3339(),
        }
    }
}

// ----- Governance -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GovernanceResponse {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules_count: usize,
    pub bound_at: String,
}

impl From<GovernanceContract> for GovernanceResponse {
    fn from(c: GovernanceContract) -> Self {
        Self {
            id: c.id,
            feature_id: c.feature_id,
            version: c.version,
            rules_count: c.rules.len(),
            bound_at: c.bound_at.to_rfc3339(),
        }
    }
}

// ----- Audit -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AuditEntryResponse {
    pub id: i64,
    pub feature_id: i64,
    pub wp_id: Option<i64>,
    pub timestamp: String,
    pub actor: String,
    pub transition: String,
    pub hash: String,
}

impl From<AuditEntry> for AuditEntryResponse {
    fn from(e: AuditEntry) -> Self {
        Self {
            id: e.id,
            feature_id: e.feature_id,
            wp_id: e.wp_id,
            timestamp: e.timestamp.to_rfc3339(),
            actor: e.actor,
            transition: e.transition,
            hash: e.hash.iter().map(|b| format!("{b:02x}")).collect(),
        }
    }
}

// ----- Projects -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            slug: p.slug,
            name: p.name,
            description: p.description,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

// ----- Epics -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EpicResponse {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub owner_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Epic> for EpicResponse {
    fn from(e: Epic) -> Self {
        Self {
            id: e.id,
            project_id: e.project_id,
            title: e.title,
            description: e.description,
            status: e.status.to_string(),
            owner_id: e.owner_id,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

// ----- Stories -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct StoryResponse {
    pub id: i64,
    pub epic_id: i64,
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub points: Option<u32>,
    pub assignee_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Story> for StoryResponse {
    fn from(s: Story) -> Self {
        Self {
            id: s.id,
            epic_id: s.epic_id,
            project_id: s.project_id,
            title: s.title,
            description: s.description,
            status: s.status.to_string(),
            points: s.points,
            assignee_id: s.assignee_id,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

// ----- Users -----

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub avatar_url: Option<String>,
    pub github_login: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            display_name: u.display_name,
            email: u.email,
            role: u.role.to_string(),
            status: u.status.to_string(),
            avatar_url: u.avatar_url,
            github_login: u.github_login,
            created_at: u.created_at.to_rfc3339(),
            updated_at: u.updated_at.to_rfc3339(),
        }
    }
}

// ----- Health -----

#[derive(Debug, Serialize)]
pub struct SimpleHealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

impl SimpleHealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok",
            service: "agileplus-api",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    pub fn healthy() -> Self {
        Self {
            status: "healthy",
            service: "agileplus-api",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

impl HealthResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

// ----- Detailed Health (T070) -----

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ServiceHealth {
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            status: "healthy".to_owned(),
            latency_ms: Some(latency_ms),
            error: None,
        }
    }

    pub fn degraded(reason: impl Into<String>) -> Self {
        Self {
            status: "degraded".to_owned(),
            latency_ms: None,
            error: Some(reason.into()),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            status: "unavailable".to_owned(),
            latency_ms: None,
            error: Some(reason.into()),
        }
    }

    pub fn not_configured() -> Self {
        Self {
            status: "not_configured".to_string(),
            latency_ms: None,
            error: Some("not configured in this deployment".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiHealth {
    pub status: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub struct DetailedHealthResponse {
    /// Overall status: "healthy" | "degraded" | "unavailable"
    pub status: String,
    pub timestamp: String,
    pub services: std::collections::HashMap<String, ServiceHealth>,
    pub api: ApiHealth,
}

impl DetailedHealthResponse {
    /// Build a basic healthy response (used when no external services are wired up).
    pub fn basic(uptime_seconds: u64) -> Self {
        use std::collections::HashMap;
        let services: HashMap<String, ServiceHealth> = [("sqlite", ServiceHealth::healthy(0))]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        Self {
            status: "healthy".to_owned(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            services,
            api: ApiHealth {
                status: "healthy".to_owned(),
                uptime_seconds,
            },
        }
    }

    /// Derive overall status from individual service statuses.
    pub fn compute_status(
        services: &std::collections::HashMap<String, ServiceHealth>,
    ) -> &'static str {
        // Only consider services that are actually configured.
        let configured: Vec<_> = services
            .values()
            .filter(|s| s.status != "not_configured")
            .collect();
        if configured.iter().any(|s| s.status == "unavailable") {
            return "unavailable";
        }
        if configured.iter().any(|s| s.status == "degraded") {
            return "degraded";
        }
        "healthy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_health_healthy() {
        let h = SimpleHealthResponse::healthy();
        assert_eq!(h.status, "healthy");
        assert_eq!(h.service, "agileplus-api");
        assert!(!h.version.is_empty());
    }

    #[test]
    fn simple_health_ok() {
        let h = SimpleHealthResponse::ok();
        assert_eq!(h.status, "ok");
    }

    #[test]
    fn health_response_ok() {
        let h = HealthResponse::ok();
        assert_eq!(h.status, "ok");
        assert!(!h.version.is_empty());
    }

    #[test]
    fn service_health_healthy() {
        let h = ServiceHealth::healthy(42);
        assert_eq!(h.status, "healthy");
        assert_eq!(h.latency_ms, Some(42));
        assert!(h.error.is_none());
    }

    #[test]
    fn service_health_degraded() {
        let h = ServiceHealth::degraded("slow");
        assert_eq!(h.status, "degraded");
        assert!(h.latency_ms.is_none());
        assert_eq!(h.error.as_deref(), Some("slow"));
    }

    #[test]
    fn service_health_unavailable() {
        let h = ServiceHealth::unavailable("down");
        assert_eq!(h.status, "unavailable");
        assert_eq!(h.error.as_deref(), Some("down"));
    }

    #[test]
    fn service_health_not_configured() {
        let h = ServiceHealth::not_configured();
        assert_eq!(h.status, "not_configured");
        assert!(h.error.is_some());
    }

    #[test]
    fn detailed_health_basic() {
        let h = DetailedHealthResponse::basic(120);
        assert_eq!(h.status, "healthy");
        assert_eq!(h.api.uptime_seconds, 120);
        assert!(h.services.contains_key("sqlite"));
    }

    #[test]
    fn compute_status_all_healthy() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::healthy(1));
        assert_eq!(DetailedHealthResponse::compute_status(&services), "healthy");
    }

    #[test]
    fn compute_status_has_unavailable() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::healthy(1));
        services.insert("b".into(), ServiceHealth::unavailable("x"));
        assert_eq!(DetailedHealthResponse::compute_status(&services), "unavailable");
    }

    #[test]
    fn compute_status_has_degraded() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::degraded("slow"));
        assert_eq!(DetailedHealthResponse::compute_status(&services), "degraded");
    }

    #[test]
    fn compute_status_ignores_not_configured() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::not_configured());
        assert_eq!(DetailedHealthResponse::compute_status(&services), "healthy");
    }

    #[test]
    fn feature_response_from_domain() {
        let f = agileplus_domain::domain::feature::Feature::new(
            "my-slug", "My Feature", [0u8; 32], Some("main"),
        );
        let resp = FeatureResponse::from(f);
        assert_eq!(resp.slug, "my-slug");
        assert_eq!(resp.name, "My Feature");
        assert_eq!(resp.state, "created");
        assert_eq!(resp.target_branch, "main");
        assert!(resp.created_at.contains("T"));
    }

    #[test]
    fn work_package_response_from_domain() {
        use agileplus_domain::domain::work_package::WorkPackage;
        let wp = WorkPackage::new(10, "Test WP", 3, "must do x");
        let resp = WorkPackageResponse::from(wp);
        assert_eq!(resp.feature_id, 10);
        assert_eq!(resp.title, "Test WP");
        assert_eq!(resp.sequence, 3);
        assert_eq!(resp.state, "planned");
        assert_eq!(resp.acceptance_criteria, "must do x");
        assert!(resp.pr_url.is_none());
    }

    #[test]
    fn audit_entry_response_hash_is_hex() {
        use chrono::DateTime;
        let entry = agileplus_domain::domain::audit::AuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: Some(5),
            timestamp: DateTime::from_timestamp(1_000_000, 0).unwrap(),
            actor: "user".into(),
            transition: "Created->Specified".into(),
            evidence_refs: vec![],
            prev_hash: [0; 32],
            hash: [0xAB; 32],
            event_id: None,
            archived_to: None,
        };
        let resp = AuditEntryResponse::from(entry);
        assert_eq!(resp.hash, "ab".repeat(32));
        assert_eq!(resp.wp_id, Some(5));
    }

    #[test]
    fn governance_response_from_domain() {
        let contract = agileplus_domain::domain::governance::GovernanceContract {
            id: 1,
            feature_id: 10,
            version: 3,
            rules: vec![
                agileplus_domain::domain::governance::GovernanceRule {
                    transition: "validate".into(),
                    required_evidence: vec![],
                    policy_refs: vec![],
                },
                agileplus_domain::domain::governance::GovernanceRule {
                    transition: "ship".into(),
                    required_evidence: vec![],
                    policy_refs: vec![],
                },
            ],
            bound_at: chrono::Utc::now(),
        };
        let resp = GovernanceResponse::from(contract);
        assert_eq!(resp.id, 1);
        assert_eq!(resp.feature_id, 10);
        assert_eq!(resp.version, 3);
        assert_eq!(resp.rules_count, 2);
    }

    #[test]
    fn project_response_from_domain() {
        use chrono::Utc;
        let p = agileplus_domain::domain::project::Project {
            id: 1,
            slug: "proj".into(),
            name: "My Project".into(),
            description: Some("desc".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = ProjectResponse::from(p);
        assert_eq!(resp.slug, "proj");
        assert_eq!(resp.name, "My Project");
        assert_eq!(resp.description, Some("desc".into()));
    }

    #[test]
    fn project_response_no_description() {
        use chrono::Utc;
        let p = agileplus_domain::domain::project::Project {
            id: 2,
            slug: "p2".into(),
            name: "P2".into(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = ProjectResponse::from(p);
        assert!(resp.description.is_none());
    }
}
