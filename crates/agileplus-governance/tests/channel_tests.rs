//! Integration tests for ReleaseChannel, ChannelMetadata, PromotionRequest, PromotionResult.
//! Complements the inline unit tests in src/channel.rs.

use agileplus_governance::*;
use agileplus_governance::channel::PromotionResult;
use chrono::Utc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ── ReleaseChannel serde ─────────────────────────────────────────────

#[test]
fn channel_serde_roundtrip_all_variants() {
    for ch in ReleaseChannel::all() {
        let json = serde_json::to_string(ch).unwrap();
        let back: ReleaseChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(*ch, back);
    }
}

#[test]
fn channel_serde_uses_lowercase() {
    let json = serde_json::to_string(&ReleaseChannel::Alpha).unwrap();
    assert_eq!(json, "\"alpha\"");
    let json = serde_json::to_string(&ReleaseChannel::Prod).unwrap();
    assert_eq!(json, "\"prod\"");
}

#[test]
fn channel_serde_deserializes_various_casings() {
    for (input, expected) in [
        ("\"alpha\"", ReleaseChannel::Alpha),
        ("\"canary\"", ReleaseChannel::Canary),
        ("\"beta\"", ReleaseChannel::Beta),
        ("\"rc\"", ReleaseChannel::Rc),
        ("\"prod\"", ReleaseChannel::Prod),
    ] {
        let ch: ReleaseChannel = serde_json::from_str(input).unwrap();
        assert_eq!(ch, expected);
    }
}

#[test]
fn channel_is_hashable() {
    let mut hasher = DefaultHasher::new();
    ReleaseChannel::Alpha.hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher2 = DefaultHasher::new();
    ReleaseChannel::Alpha.hash(&mut hasher2);
    let h2 = hasher2.finish();

    assert_eq!(h1, h2);
}

#[test]
fn channel_is_partialeq_reflexive() {
    for ch in ReleaseChannel::all() {
        assert_eq!(*ch, *ch);
    }
}

#[test]
fn channel_ord_is_strictly_monotonic() {
    let channels = ReleaseChannel::all();
    for w in channels.windows(2) {
        assert!(w[0] < w[1], "{:?} should be < {:?}", w[0], w[1]);
        assert!(w[1] > w[0], "{:?} should be > {:?}", w[1], w[0]);
    }
}

#[test]
fn channel_copy_semantics() {
    let ch = ReleaseChannel::Beta;
    let ch2 = ch; // Copy
    assert_eq!(ch, ch2);
}

// ── ChannelMetadata serde ────────────────────────────────────────────

#[test]
fn channel_metadata_serde_roundtrip() {
    let meta = ChannelMetadata {
        channel: ReleaseChannel::Beta,
        version: "2.0.0".to_string(),
        set_at: Utc::now(),
        set_by: "ci-bot".to_string(),
        iteration: 5,
        metadata: Some(serde_json::json!({"build_id": "abc"})),
    };
    let json = serde_json::to_string(&meta).unwrap();
    let back: ChannelMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(back.channel, ReleaseChannel::Beta);
    assert_eq!(back.version, "2.0.0");
    assert_eq!(back.iteration, 5);
    assert_eq!(back.set_by, "ci-bot");
}

#[test]
fn channel_metadata_full_version_all_channels() {
    let pairs = [
        (ReleaseChannel::Alpha, "1.0.0-alpha.1"),
        (ReleaseChannel::Canary, "1.0.0-canary.2"),
        (ReleaseChannel::Beta, "1.0.0-beta.3"),
        (ReleaseChannel::Rc, "1.0.0-rc.4"),
        (ReleaseChannel::Prod, "1.0.0"),
    ];
    for (ch, expected) in pairs {
        let meta = ChannelMetadata::new(ch, "1.0.0".into(), "dev".into(), ch.order() as u32);
        assert_eq!(meta.full_version(), expected, "for {:?}", ch);
    }
}

// ── PromotionRequest ─────────────────────────────────────────────────

#[test]
fn promotion_request_serde_roundtrip() {
    let req = PromotionRequest {
        package: "my-crate".into(),
        from: ReleaseChannel::Canary,
        to: ReleaseChannel::Rc,
        requested_by: "dev".into(),
        version: "3.0.0".into(),
        metadata: Some(serde_json::json!({"notes": "test"})),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: PromotionRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.from, ReleaseChannel::Canary);
    assert_eq!(back.to, ReleaseChannel::Rc);
    assert_eq!(back.package, "my-crate");
}

#[test]
fn promotion_request_skips_channels_consecutive() {
    let req = PromotionRequest::new(
        "pkg".into(),
        ReleaseChannel::Beta,
        ReleaseChannel::Rc,
        "u".into(),
        "1.0.0".into(),
    );
    assert!(req.is_valid_transition());
    assert!(req.skips_channels().is_empty());
}

#[test]
fn promotion_request_skips_channels_backward_invalid() {
    let req = PromotionRequest::new(
        "pkg".into(),
        ReleaseChannel::Prod,
        ReleaseChannel::Alpha,
        "u".into(),
        "1.0.0".into(),
    );
    assert!(!req.is_valid_transition());
}

#[test]
fn promotion_request_from_alpha_to_prod_skips_three() {
    let req = PromotionRequest::new(
        "pkg".into(),
        ReleaseChannel::Alpha,
        ReleaseChannel::Prod,
        "u".into(),
        "1.0.0".into(),
    );
    let skips = req.skips_channels();
    assert_eq!(skips.len(), 3);
    assert_eq!(skips[0], ReleaseChannel::Canary);
    assert_eq!(skips[1], ReleaseChannel::Beta);
    assert_eq!(skips[2], ReleaseChannel::Rc);
}

// ── PromotionResult ──────────────────────────────────────────────────

#[test]
fn promotion_result_serde_roundtrip() {
    let meta = ChannelMetadata::new(
        ReleaseChannel::Beta,
        "1.0.0".into(),
        "dev".into(),
        1,
    );
    let result = PromotionResult::allowed(meta);
    let json = serde_json::to_string(&result).unwrap();
    let back: PromotionResult = serde_json::from_str(&json).unwrap();
    assert!(back.allowed);
    assert!(back.channel_metadata.is_some());
}

#[test]
fn promotion_result_denied_has_no_metadata() {
    let result = PromotionResult::denied("blocked".into(), vec!["gate-fail".into()]);
    assert!(!result.allowed);
    assert!(result.channel_metadata.is_none());
    assert_eq!(result.policy_failures, vec!["gate-fail"]);
}

#[test]
fn promotion_result_add_check_passing() {
    let meta = ChannelMetadata::new(
        ReleaseChannel::Alpha,
        "1.0.0".into(),
        "dev".into(),
        1,
    );
    let mut result = PromotionResult::allowed(meta);
    result.add_check("test_gate".into(), true);
    result.add_check("security_gate".into(), false);
    assert_eq!(result.policy_checks, vec!["test_gate"]);
    assert_eq!(result.policy_failures, vec!["security_gate"]);
}

#[test]
fn promotion_result_add_warning() {
    let meta = ChannelMetadata::new(
        ReleaseChannel::Alpha,
        "1.0.0".into(),
        "dev".into(),
        1,
    );
    let mut result = PromotionResult::allowed(meta);
    result.add_warning("test coverage below 80%");
    result.add_warning("no CHANGELOG entry");
    assert_eq!(result.warnings.len(), 2);
}
