use chrono::Utc;

use agileplus_domain::domain::audit::{AuditEntry, hash_entry};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;

pub(super) fn build_import_audit_entry(feature_id: i64, state: &FeatureState) -> AuditEntry {
    let mut entry = AuditEntry {
        id: 0,
        feature_id,
        wp_id: None,
        timestamp: Utc::now(),
        actor: "import".into(),
        transition: format!("Imported spec -> {state}"),
        evidence_refs: vec![],
        prev_hash: [0u8; 32],
        hash: [0u8; 32],
        event_id: None,
        archived_to: None,
    };
    entry.hash = hash_entry(&entry);
    entry
}

pub(super) fn feature_meta_json(feature: &Feature, state: FeatureState) -> String {
    #[derive(serde::Serialize)]
    struct Meta<'a> {
        slug: &'a str,
        friendly_name: &'a str,
        state: String,
        spec_hash: String,
        target_branch: &'a str,
        created_at: String,
        updated_at: String,
    }

    let meta = Meta {
        slug: &feature.slug,
        friendly_name: &feature.friendly_name,
        state: state.to_string(),
        spec_hash: feature
            .spec_hash
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        target_branch: &feature.target_branch,
        created_at: feature.created_at.to_rfc3339(),
        updated_at: feature.updated_at.to_rfc3339(),
    };
    serde_json::to_string_pretty(&meta).unwrap_or_else(|_| "{}".to_string())
}

pub(super) fn sha256_bytes(content: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use agileplus_domain::domain::audit::hash_entry;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn sample_feature() -> Feature {
        let mut feature = Feature::new(
            "auth-login",
            "Auth Login",
            sha256_bytes("# Login"),
            Some("feature/auth-login"),
        );
        feature.id = 7;
        feature
    }

    #[test]
    fn sha256_empty_string_known_vector() {
        let digest = sha256_bytes("");
        assert_eq!(
            hex(&digest),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_abc_known_vector() {
        let digest = sha256_bytes("abc");
        assert_eq!(
            hex(&digest),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_is_deterministic() {
        assert_eq!(sha256_bytes("repeatable"), sha256_bytes("repeatable"));
    }

    #[test]
    fn sha256_differs_for_different_inputs() {
        assert_ne!(sha256_bytes("alpha"), sha256_bytes("alpha "));
    }

    #[test]
    fn sha256_is_length_prefixed_by_content_not_length() {
        // Different content of equal length must not collide.
        assert_ne!(sha256_bytes("aaaa"), sha256_bytes("aaab"));
    }

    #[test]
    fn sha256_result_is_32_bytes() {
        assert_eq!(sha256_bytes("anything").len(), 32);
    }

    #[test]
    fn feature_meta_json_parses_and_contains_identity() {
        let feature = sample_feature();
        let meta = feature_meta_json(&feature, FeatureState::Specified);
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();

        assert_eq!(parsed["slug"], "auth-login");
        assert_eq!(parsed["friendly_name"], "Auth Login");
        assert_eq!(parsed["state"], "specified");
        assert_eq!(parsed["target_branch"], "feature/auth-login");
    }

    #[test]
    fn feature_meta_json_spec_hash_is_lowercase_hex() {
        let feature = sample_feature();
        let meta = feature_meta_json(&feature, FeatureState::Specified);
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();
        let hash = parsed["spec_hash"].as_str().unwrap();

        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hash, hash.to_lowercase());
        assert_eq!(hash, hex(&feature.spec_hash));
    }

    #[test]
    fn feature_meta_json_state_reflects_argument() {
        let feature = sample_feature();
        let specified = feature_meta_json(&feature, FeatureState::Specified);
        let shipped = feature_meta_json(&feature, FeatureState::Shipped);

        assert_ne!(specified, shipped);
        assert!(shipped.contains("\"shipped\""));
    }

    #[test]
    fn feature_meta_json_includes_timestamps() {
        let feature = sample_feature();
        let meta = feature_meta_json(&feature, FeatureState::Specified);
        let parsed: serde_json::Value = serde_json::from_str(&meta).unwrap();

        assert!(parsed["created_at"].as_str().unwrap().contains('T'));
        assert!(parsed["updated_at"].as_str().unwrap().contains('T'));
    }

    #[test]
    fn audit_entry_uses_import_actor_and_transition() {
        let entry = build_import_audit_entry(42, &FeatureState::Specified);

        assert_eq!(entry.actor, "import");
        assert_eq!(entry.feature_id, 42);
        assert_eq!(entry.transition, "Imported spec -> specified");
        assert!(entry.wp_id.is_none());
        assert!(entry.evidence_refs.is_empty());
        assert!(entry.event_id.is_none());
        assert!(entry.archived_to.is_none());
    }

    #[test]
    fn audit_entry_hash_matches_recomputation() {
        let entry = build_import_audit_entry(1, &FeatureState::Planned);

        assert_eq!(entry.hash, hash_entry(&entry));
    }

    #[test]
    fn audit_entry_hash_is_not_all_zeroes() {
        let entry = build_import_audit_entry(1, &FeatureState::Planned);

        assert_ne!(entry.hash, [0u8; 32]);
        assert_eq!(entry.prev_hash, [0u8; 32]);
    }

    #[test]
    fn audit_entry_transition_varies_by_state() {
        let created = build_import_audit_entry(1, &FeatureState::Created);
        let shipped = build_import_audit_entry(1, &FeatureState::Shipped);

        assert_ne!(created.transition, shipped.transition);
        assert!(shipped.transition.contains("shipped"));
    }

    #[test]
    fn audit_entry_hash_differs_across_features() {
        let a = build_import_audit_entry(1, &FeatureState::Planned);
        let b = build_import_audit_entry(2, &FeatureState::Planned);

        assert_ne!(a.hash, b.hash);
    }
}
