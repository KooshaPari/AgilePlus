//! Core test fixtures for feature and work package data.
//!
//! Traceability: WP19-T107

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use chrono::Utc;

/// A collection of pre-built test fixtures for use across integration tests.
#[derive(Clone)]
pub struct TestFixtures {
    /// A feature ready to be inserted as "feature-1".
    pub feature1: Feature,
    /// A second feature ready to be inserted as "feature-2".
    pub feature2: Feature,
}

impl TestFixtures {
    /// Build fixtures using deterministic values (no DB interaction).
    pub fn new() -> Self {
        let feature1 = Feature {
            id: 1,
            slug: "implement-caching-layer".to_string(),
            friendly_name: "Implement caching layer".to_string(),
            state: FeatureState::Created,
            spec_hash: [0x01u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_at_commit: None,
            last_modified_commit: None,
        };

        let feature2 = Feature {
            id: 2,
            slug: "add-api-rate-limiting".to_string(),
            friendly_name: "Add API rate limiting".to_string(),
            state: FeatureState::Created,
            spec_hash: [0x02u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_at_commit: None,
            last_modified_commit: None,
        };

        Self { feature1, feature2 }
    }
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}

/// Create and return in-memory test fixtures.
///
/// For full integration tests this would also seed a database; here we return
/// pure data so that unit tests can use fixtures without any I/O.
pub async fn seed_test_data() -> TestFixtures {
    TestFixtures::new()
}

// ---------------------------------------------------------------------------
// Unit tests — always run (no external services)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_build_without_panic() {
        let f = TestFixtures::new();
        assert_eq!(f.feature1.slug, "implement-caching-layer");
        assert_eq!(f.feature2.slug, "add-api-rate-limiting");
        assert_eq!(f.feature1.state, FeatureState::Created);
        assert_eq!(f.feature2.state, FeatureState::Created);
    }

    #[tokio::test]
    async fn seed_test_data_returns_fixtures() {
        let f = seed_test_data().await;
        assert_eq!(f.feature1.id, 1);
        assert_eq!(f.feature2.id, 2);
    }

    #[test]
    fn default_matches_new() {
        let a = TestFixtures::default();
        let b = TestFixtures::new();
        assert_eq!(a.feature1.id, b.feature1.id);
        assert_eq!(a.feature1.slug, b.feature1.slug);
        assert_eq!(a.feature2.id, b.feature2.id);
        assert_eq!(a.feature2.slug, b.feature2.slug);
    }

    #[test]
    fn clone_is_independent() {
        let fixtures = TestFixtures::new();
        let mut cloned = fixtures.clone();
        cloned.feature1.slug = "mutated".to_string();
        assert_eq!(fixtures.feature1.slug, "implement-caching-layer");
    }

    #[test]
    fn features_have_expected_identity() {
        let f = TestFixtures::new();
        assert_eq!(f.feature1.id, 1);
        assert_eq!(f.feature2.id, 2);
        assert_eq!(f.feature1.friendly_name, "Implement caching layer");
        assert_eq!(f.feature2.friendly_name, "Add API rate limiting");
    }

    #[test]
    fn features_have_distinct_spec_hashes() {
        let f = TestFixtures::new();
        assert_eq!(f.feature1.spec_hash, [0x01u8; 32]);
        assert_eq!(f.feature2.spec_hash, [0x02u8; 32]);
        assert_ne!(f.feature1.spec_hash, f.feature2.spec_hash);
    }

    #[test]
    fn features_share_default_branch_and_empty_labels() {
        let f = TestFixtures::new();
        assert_eq!(f.feature1.target_branch, "main");
        assert_eq!(f.feature2.target_branch, "main");
        assert!(f.feature1.labels.is_empty());
        assert!(f.feature2.labels.is_empty());
    }

    #[test]
    fn fixture_features_have_no_plane_or_module_links() {
        let f = TestFixtures::new();
        assert!(f.feature1.plane_issue_id.is_none());
        assert!(f.feature1.plane_state_id.is_none());
        assert!(f.feature1.module_id.is_none());
        assert!(f.feature1.project_id.is_none());
        assert!(f.feature2.plane_issue_id.is_none());
        assert!(f.feature2.project_id.is_none());
    }

    #[test]
    fn fixture_timestamps_are_set() {
        let f = TestFixtures::new();
        assert!(f.feature1.created_at <= f.feature1.updated_at);
        assert!(f.feature2.created_at <= f.feature2.updated_at);
    }

    #[tokio::test]
    async fn seed_test_data_matches_synchronous_construction() {
        let seeded = seed_test_data().await;
        let direct = TestFixtures::new();
        assert_eq!(seeded.feature1.slug, direct.feature1.slug);
        assert_eq!(seeded.feature2.slug, direct.feature2.slug);
    }
}
