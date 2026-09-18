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
        assert_eq!(
            DetailedHealthResponse::compute_status(&services),
            "unavailable"
        );
    }

    #[test]
    fn compute_status_has_degraded() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::degraded("slow"));
        assert_eq!(
            DetailedHealthResponse::compute_status(&services),
            "degraded"
        );
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
            "my-slug",
            "My Feature",
            [0u8; 32],
            Some("main"),
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
    fn epic_response_from_domain() {
        let epic = Epic::new(1, "My Epic").unwrap();
        let resp = EpicResponse::from(epic);
        assert_eq!(resp.project_id, 1);
        assert_eq!(resp.title, "My Epic");
        assert_eq!(resp.status, "backlog");
        assert!(resp.description.is_none());
        assert!(resp.owner_id.is_none());
        assert!(resp.created_at.contains("T"));
    }

    #[test]
    fn epic_response_with_description() {
        let mut epic = Epic::new(2, "Epic Two").unwrap();
        epic.description = Some("A longer description".into());
        let resp = EpicResponse::from(epic);
        assert_eq!(resp.description.as_deref(), Some("A longer description"));
    }

    #[test]
    fn story_response_from_domain() {
        let story = Story::new(10, 20, "My Story", Some(5)).unwrap();
        let resp = StoryResponse::from(story);
        assert_eq!(resp.epic_id, 10);
        assert_eq!(resp.project_id, 20);
        assert_eq!(resp.title, "My Story");
        assert_eq!(resp.points, Some(5));
        assert_eq!(resp.status, "todo");
        assert!(resp.description.is_none());
        assert!(resp.assignee_id.is_none());
    }

    #[test]
    fn story_response_no_points() {
        let story = Story::new(1, 2, "No Points", None).unwrap();
        let resp = StoryResponse::from(story);
        assert!(resp.points.is_none());
    }

    #[test]
    fn story_response_with_description() {
        let mut story = Story::new(1, 2, "With Desc", Some(3)).unwrap();
        story.description = Some("desc text".into());
        let resp = StoryResponse::from(story);
        assert_eq!(resp.description.as_deref(), Some("desc text"));
    }

    #[test]
    fn user_response_from_domain() {
        use agileplus_domain::domain::user::{User, UserRole};
        let user = User::new("Alice", "alice@example.com", UserRole::Admin).unwrap();
        let resp = UserResponse::from(user);
        assert_eq!(resp.display_name, "Alice");
        assert_eq!(resp.email, "alice@example.com");
        assert_eq!(resp.role, "admin");
        assert_eq!(resp.status, "active");
        assert!(resp.avatar_url.is_none());
        assert!(resp.github_login.is_none());
        assert!(resp.created_at.contains("T"));
    }

    #[test]
    fn user_response_member_role() {
        use agileplus_domain::domain::user::{User, UserRole};
        let user = User::new("Bob", "bob@example.com", UserRole::Member).unwrap();
        let resp = UserResponse::from(user);
        assert_eq!(resp.role, "member");
    }

    #[test]
    fn user_response_viewer_role() {
        use agileplus_domain::domain::user::{User, UserRole};
        let user = User::new("Carol", "carol@example.com", UserRole::Viewer).unwrap();
        let resp = UserResponse::from(user);
        assert_eq!(resp.role, "viewer");
    }

    #[test]
    fn feature_response_serializes_to_json() {
        let f = Feature::new("my-slug", "My Feature", [0u8; 32], Some("main"));
        let resp = FeatureResponse::from(f);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["slug"], "my-slug");
        assert_eq!(json["name"], "My Feature");
        assert_eq!(json["target_branch"], "main");
    }

    #[test]
    fn work_package_response_serializes_to_json() {
        let wp = WorkPackage::new(10, "Test WP", 3, "must do x");
        let resp = WorkPackageResponse::from(wp);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["title"], "Test WP");
        assert_eq!(json["sequence"], 3);
        assert!(json["pr_url"].is_null());
    }

    #[test]
    fn epic_response_serializes_to_json() {
        let epic = Epic::new(1, "My Epic").unwrap();
        let resp = EpicResponse::from(epic);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["title"], "My Epic");
        assert_eq!(json["project_id"], 1);
    }

    #[test]
    fn story_response_serializes_to_json() {
        let story = Story::new(1, 2, "My Story", Some(3)).unwrap();
        let resp = StoryResponse::from(story);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["title"], "My Story");
        assert_eq!(json["points"], 3);
    }

    #[test]
    fn user_response_serializes_to_json() {
        use agileplus_domain::domain::user::{User, UserRole};
        let user = User::new("Alice", "a@b.com", UserRole::Admin).unwrap();
        let resp = UserResponse::from(user);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["display_name"], "Alice");
        assert_eq!(json["email"], "a@b.com");
        assert_eq!(json["role"], "admin");
    }

    #[test]
    fn project_response_serializes_to_json() {
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
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["slug"], "proj");
        assert_eq!(json["name"], "My Project");
        assert_eq!(json["description"], "desc");
    }

    #[test]
    fn governance_response_serializes_to_json() {
        let contract = agileplus_domain::domain::governance::GovernanceContract {
            id: 1,
            feature_id: 10,
            version: 2,
            rules: vec![],
            bound_at: chrono::Utc::now(),
        };
        let resp = GovernanceResponse::from(contract);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["version"], 2);
        assert_eq!(json["rules_count"], 0);
    }

    #[test]
    fn audit_entry_response_serializes_to_json() {
        use chrono::DateTime;
        let entry = agileplus_domain::domain::audit::AuditEntry {
            id: 1,
            feature_id: 10,
            wp_id: None,
            timestamp: DateTime::from_timestamp(1_000_000, 0).unwrap(),
            actor: "bot".into(),
            transition: "Created->Specified".into(),
            evidence_refs: vec![],
            prev_hash: [0; 32],
            hash: [0xCD; 32],
            event_id: None,
            archived_to: None,
        };
        let resp = AuditEntryResponse::from(entry);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["actor"], "bot");
        assert!(json["wp_id"].is_null());
    }

    #[test]
    fn compute_status_empty_map() {
        let services = std::collections::HashMap::new();
        assert_eq!(DetailedHealthResponse::compute_status(&services), "healthy");
    }

    #[test]
    fn compute_status_mixed_degraded_and_unavailable() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::degraded("slow"));
        services.insert("b".into(), ServiceHealth::unavailable("down"));
        // unavailable takes precedence over degraded
        assert_eq!(
            DetailedHealthResponse::compute_status(&services),
            "unavailable"
        );
    }

    #[test]
    fn compute_status_all_not_configured() {
        let mut services = std::collections::HashMap::new();
        services.insert("a".into(), ServiceHealth::not_configured());
        services.insert("b".into(), ServiceHealth::not_configured());
        assert_eq!(DetailedHealthResponse::compute_status(&services), "healthy");
    }

    #[test]
    fn service_health_healthy_latency_zero() {
        let h = ServiceHealth::healthy(0);
        assert_eq!(h.latency_ms, Some(0));
        assert!(h.error.is_none());
    }

    #[test]
    fn detailed_health_basic_uptime() {
        let h = DetailedHealthResponse::basic(0);
        assert_eq!(h.api.uptime_seconds, 0);
        assert!(!h.timestamp.is_empty());
    }

    #[test]
    fn project_response_no_description_serializes_null() {
        use chrono::Utc;
        let p = agileplus_domain::domain::project::Project {
            id: 3,
            slug: "p3".into(),
            name: "P3".into(),
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = ProjectResponse::from(p);
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json["description"].is_null());
    }

    // ── /detailed-health payload shape ───────────────────────────────────────
    //
    // `ServiceHealth` skips its optional fields when they are absent. That is
    // the wire contract monitoring consumes: a healthy service must not carry an
    // `error` key, and an unhealthy one must not carry a `latency_ms` key.

    #[test]
    fn service_health_healthy_omits_error_field() {
        let json = serde_json::to_value(ServiceHealth::healthy(12)).unwrap();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["latency_ms"], 12);
        assert!(
            json.get("error").is_none(),
            "healthy entries must not serialize an error key, got: {json}"
        );
    }

    #[test]
    fn service_health_unavailable_omits_latency_field() {
        let json = serde_json::to_value(ServiceHealth::unavailable("connection refused")).unwrap();
        assert_eq!(json["status"], "unavailable");
        assert_eq!(json["error"], "connection refused");
        assert!(
            json.get("latency_ms").is_none(),
            "failed probes have no latency to report, got: {json}"
        );
    }

    #[test]
    fn service_health_not_configured_explains_why() {
        let json = serde_json::to_value(ServiceHealth::not_configured()).unwrap();
        assert_eq!(json["status"], "not_configured");
        assert_eq!(json["error"], "not configured in this deployment");
        assert!(json.get("latency_ms").is_none());
    }

    #[test]
    fn detailed_health_serializes_nested_services_and_api_block() {
        let json = serde_json::to_value(DetailedHealthResponse::basic(42)).unwrap();
        assert_eq!(json["status"], "healthy");
        assert!(json["timestamp"].is_string());
        assert_eq!(json["services"]["sqlite"]["status"], "healthy");
        assert_eq!(json["services"]["sqlite"]["latency_ms"], 0);
        assert_eq!(json["api"]["status"], "healthy");
        assert_eq!(json["api"]["uptime_seconds"], 42);
    }

    #[test]
    fn simple_health_response_serializes_expected_keys() {
        let json = serde_json::to_value(SimpleHealthResponse::healthy()).unwrap();
        assert_eq!(
            json.as_object().expect("object").len(),
            3,
            "the simple health payload is a fixed three-key object, got: {json}"
        );
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "agileplus-api");
    }
}
