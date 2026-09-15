//! # AgilePlus Governance System
//!
//! A comprehensive governance system for the AgilePlus platform providing:
//! - **Release Channel Governance**: 5-tier release model (alpha → canary → beta → rc → prod)
//! - **Policy Enforcement**: Rule-based policy checks for promotions and operations
//! - **Audit Logging**: Complete audit trail of all governance actions
//! - **Rate Limiting**: Protection against abuse
//!
//! ## Release Channels
//!
//! | Channel | Version Suffix | Description |
//! |---------|----------------|-------------|
//! | alpha   | `-alpha.N`     | Experimental features |
//! | canary  | `-canary.N`    | Early access, unstable |
//! | beta    | `-beta.N`      | Public testing |
//! | rc      | `-rc.N`        | Release candidate |
//! | prod    | (none)         | Production stable |
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use agileplus_governance::{GovernanceClient, PolicyCheck, PolicyContext, ReleaseChannel};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let governance = GovernanceClient::with_defaults().await?;
//!
//! // Check if a promotion is allowed
//! let check = PolicyCheck {
//!     resource: "my-crate".to_string(),
//!     action: "promote".to_string(),
//!     context: PolicyContext::new()
//!         .with_channel(ReleaseChannel::Canary),
//! };
//!
//! let result = governance.check_policy(check).await?;
//! println!("Promotion allowed: {}", result.allowed);
//! # Ok(())
//! # }
//! ```

pub mod audit;
pub mod channel;
pub mod client;
pub mod code_scanner;
pub mod config;
pub mod error;
pub mod policy;
pub mod rate_limiter;
pub mod rubric;
pub mod scoring_engine;
pub mod types;

pub use audit::AuditLogger;
pub use channel::{ChannelMetadata, PromotionRequest, ReleaseChannel};
pub use client::GovernanceClient;
pub use code_scanner::{scan_repo, EvidenceItem, RepoScan};
pub use config::GovernanceConfig;
pub use error::{GovernanceError, Result};
pub use policy::{PolicyCheck, PolicyContext, PolicyEngine, PolicyResult};
pub use rate_limiter::RateLimiter;
pub use rubric::{Pillar, RubricCatalog, ScoringSpec, SubPillar};
pub use scoring_engine::{evaluate, render_markdown, ClusterScore, PillarScore, ScoreReport};
pub use types::*;

// Re-export commonly used types
pub use channel::ReleaseChannel as Channel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exported_types_are_accessible() {
        // Verify that all re-exports are usable from the crate root
        let _channel = ReleaseChannel::Alpha;
        let _config = GovernanceConfig::default();
        let _rate_limiter = RateLimiter::default_limiter();
        let _policy_result = PolicyResult::allowed("test");
        let _policy_context = PolicyContext::new();
        let _policy_check = PolicyCheck {
            resource: "test".into(),
            action: "test".into(),
            context: PolicyContext::new(),
        };
        let _error = GovernanceError::Config("test".into());
    }

    #[test]
    fn channel_alias_works() {
        // The Channel type alias should work identically to ReleaseChannel
        let ch: Channel = ReleaseChannel::Beta;
        assert_eq!(ch.order(), 3);
    }

    #[test]
    fn release_channel_all_has_five() {
        assert_eq!(ReleaseChannel::all().len(), 5);
    }

    #[test]
    fn policy_engine_default_construction() {
        let engine = PolicyEngine::default();
        assert!(!engine.policies().is_empty());
    }

    #[test]
    fn rubric_catalog_roundtrip() {
        let json = r#"{
            "version": "1.0",
            "schema": "test",
            "clusters": 1,
            "sub_pillars_enumerated": 0,
            "note": "test",
            "pillars": [{
                "cluster": "C00",
                "pillar_range": "L0-L9",
                "category": "Test",
                "source": "test/",
                "defs_ref": "test.md",
                "scoring": {"scale": "0-3", "glyphs": {"0": "x"}, "grade": {"A": 90}},
                "sub_pillars": []
            }]
        }"#;
        let catalog = RubricCatalog::from_json(json).unwrap();
        assert_eq!(catalog.clusters, 1);
    }

    #[test]
    fn evidence_item_present_helper() {
        let item = EvidenceItem {
            artifact_id: "file:README.md".into(),
            kind: "file_presence".into(),
            path: "README.md".into(),
            metadata: {
                let mut m = std::collections::BTreeMap::new();
                m.insert("present".into(), "true".into());
                m
            },
        };
        assert!(item.present());
    }

    #[test]
    fn evidence_item_count_helper() {
        let item = EvidenceItem {
            artifact_id: "count:test_files".into(),
            kind: "count".into(),
            path: String::new(),
            metadata: {
                let mut m = std::collections::BTreeMap::new();
                m.insert("count".into(), "42".into());
                m
            },
        };
        assert_eq!(item.count_value(), 42);
    }

    #[test]
    fn evidence_item_present_defaults_false() {
        let item = EvidenceItem {
            artifact_id: "x".into(),
            kind: "x".into(),
            path: String::new(),
            metadata: std::collections::BTreeMap::new(),
        };
        assert!(!item.present());
        assert_eq!(item.count_value(), 0);
    }
}
