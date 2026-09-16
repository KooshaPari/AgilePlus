//! Integration tests for policy engine, conditions, and context.
//! Complements the inline unit tests in src/policy.rs.

use agileplus_governance::*;
use agileplus_governance::policy::{Policy, PolicyCondition, PolicyDetail, PolicyEffect, default_policies};

// ── Policy builder chain ─────────────────────────────────────────────

#[test]
fn policy_new_generates_unique_ids() {
    let p1 = Policy::new("r", "a", PolicyEffect::Allow);
    let p2 = Policy::new("r", "a", PolicyEffect::Allow);
    assert_ne!(p1.id, p2.id);
    assert!(p1.id.starts_with("pol_"));
}

#[test]
fn policy_builder_full_chain() {
    let p = Policy::new("res", "act", PolicyEffect::Deny)
        .with_name("Full Policy")
        .with_description("A comprehensive policy")
        .with_priority(500)
        .with_condition(PolicyCondition::Exists {
            key: "user_id".into(),
        })
        .with_condition(PolicyCondition::Equals {
            key: "role".into(),
            value: serde_json::json!("admin"),
        });

    assert_eq!(p.resource, "res");
    assert_eq!(p.action, "act");
    assert_eq!(p.effect, PolicyEffect::Deny);
    assert_eq!(p.name, "Full Policy");
    assert_eq!(p.description.as_deref(), Some("A comprehensive policy"));
    assert_eq!(p.priority, 500);
    assert_eq!(p.conditions.len(), 2);
    assert!(p.enabled);
}

#[test]
fn policy_serde_roundtrip() {
    let p = Policy::new("resource", "action", PolicyEffect::Allow)
        .with_name("Test Policy")
        .with_description("desc")
        .with_priority(42)
        .with_condition(PolicyCondition::Exists {
            key: "x".into(),
        });

    let json = serde_json::to_string(&p).unwrap();
    let back: Policy = serde_json::from_str(&json).unwrap();
    assert_eq!(back.resource, "resource");
    assert_eq!(back.action, "action");
    assert_eq!(back.effect, PolicyEffect::Allow);
    assert_eq!(back.priority, 42);
    assert_eq!(back.conditions.len(), 1);
}

// ── PolicyContext ────────────────────────────────────────────────────

#[test]
fn policy_context_builder_chain() {
    let ctx = PolicyContext::new()
        .with_user("user1")
        .with_resource("my_resource", Some("res-123".into()))
        .with_action("deploy")
        .with_channel(ReleaseChannel::Beta);

    assert_eq!(ctx.user_id.as_deref(), Some("user1"));
    assert_eq!(ctx.resource.as_deref(), Some("my_resource"));
    assert_eq!(ctx.resource_id.as_deref(), Some("res-123"));
    assert_eq!(ctx.action.as_deref(), Some("deploy"));
    assert_eq!(ctx.channel, Some(ReleaseChannel::Beta));
}

#[test]
fn policy_context_get_returns_all_keys() {
    let mut ctx = PolicyContext::new()
        .with_user("u1")
        .with_action("read")
        .with_channel(ReleaseChannel::Rc)
        .with_resource("res", Some("r1".into()));
    ctx.client_ip = Some("127.0.0.1".into());
    ctx.package = Some("my-pkg".into());
    ctx.version = Some("2.0.0".into());

    assert_eq!(ctx.get("user_id").unwrap(), serde_json::json!("u1"));
    assert_eq!(ctx.get("client_ip").unwrap(), serde_json::json!("127.0.0.1"));
    assert_eq!(ctx.get("resource").unwrap(), serde_json::json!("res"));
    assert_eq!(ctx.get("resource_id").unwrap(), serde_json::json!("r1"));
    assert_eq!(ctx.get("action").unwrap(), serde_json::json!("read"));
    assert_eq!(ctx.get("channel").unwrap(), serde_json::json!("rc"));
    assert_eq!(ctx.get("package").unwrap(), serde_json::json!("my-pkg"));
    assert_eq!(ctx.get("version").unwrap(), serde_json::json!("2.0.0"));
}

#[test]
fn policy_context_get_unknown_key_returns_metadata() {
    let mut ctx = PolicyContext::new();
    ctx.metadata
        .insert("custom_key".into(), serde_json::json!("custom_value"));
    assert_eq!(ctx.get("custom_key").unwrap(), serde_json::json!("custom_value"));
}

#[test]
fn policy_context_get_missing_key_returns_none() {
    let ctx = PolicyContext::new();
    assert!(ctx.get("nonexistent").is_none());
}

#[test]
fn policy_context_default_is_empty() {
    let ctx = PolicyContext::default();
    assert!(ctx.user_id.is_none());
    assert!(ctx.channel.is_none());
    assert!(ctx.env.is_empty());
    assert!(ctx.metadata.is_empty());
}

#[test]
fn policy_context_serde_roundtrip() {
    let mut ctx = PolicyContext::new()
        .with_user("user1")
        .with_channel(ReleaseChannel::Alpha);
    ctx.metadata
        .insert("k".into(), serde_json::json!("v"));
    let json = serde_json::to_string(&ctx).unwrap();
    let back: PolicyContext = serde_json::from_str(&json).unwrap();
    assert_eq!(back.user_id.as_deref(), Some("user1"));
    assert_eq!(back.channel, Some(ReleaseChannel::Alpha));
    assert_eq!(back.metadata.get("k").unwrap(), &serde_json::json!("v"));
}

// ── PolicyEngine ─────────────────────────────────────────────────────

#[test]
fn engine_with_policies_sorted_by_priority_desc() {
    let engine = PolicyEngine::with_policies(vec![
        Policy::new("r", "a", PolicyEffect::Allow).with_priority(10),
        Policy::new("r", "a", PolicyEffect::Deny).with_priority(100),
    ]);
    // Higher priority should be evaluated first
    let result = engine.check("r", "a", &PolicyContext::new());
    assert!(!result.allowed, "Deny with priority 100 should be evaluated first");
}

#[test]
fn engine_disabled_policy_is_skipped() {
    let mut p = Policy::new("r", "a", PolicyEffect::Deny);
    p.enabled = false;
    let engine = PolicyEngine::with_policies(vec![p]);
    let result = engine.check("r", "a", &PolicyContext::new());
    // Default action is Allow
    assert!(result.allowed);
}

#[test]
fn engine_default_action_allow_when_no_match() {
    let engine = PolicyEngine::with_policies(vec![]);
    let result = engine.check("resource", "action", &PolicyContext::new());
    assert!(result.allowed);
    assert!(result.reason.contains("default allow"));
}

#[test]
fn engine_default_action_deny_when_no_match() {
    let engine = PolicyEngine::with_policies(vec![]).with_default_action(PolicyEffect::Deny);
    let result = engine.check("resource", "action", &PolicyContext::new());
    assert!(!result.allowed);
    assert!(result.reason.contains("default deny"));
}

// ── PolicyEffect serde ───────────────────────────────────────────────

#[test]
fn policy_effect_serde_roundtrip() {
    assert_eq!(
        serde_json::to_string(&PolicyEffect::Allow).unwrap(),
        "\"allow\""
    );
    assert_eq!(
        serde_json::to_string(&PolicyEffect::Deny).unwrap(),
        "\"deny\""
    );
    let a: PolicyEffect = serde_json::from_str("\"allow\"").unwrap();
    assert_eq!(a, PolicyEffect::Allow);
    let d: PolicyEffect = serde_json::from_str("\"deny\"").unwrap();
    assert_eq!(d, PolicyEffect::Deny);
}

// ── PolicyResult ─────────────────────────────────────────────────────

#[test]
fn policy_result_allowed_factory() {
    let r = PolicyResult::allowed("all good");
    assert!(r.allowed);
    assert_eq!(r.reason, "all good");
    assert!(r.policy.is_none());
    assert!(r.details.is_empty());
}

#[test]
fn policy_result_denied_factory() {
    let r = PolicyResult::denied("denied", "my-policy");
    assert!(!r.allowed);
    assert_eq!(r.reason, "denied");
    assert_eq!(r.policy.as_deref(), Some("my-policy"));
}

#[test]
fn policy_result_add_detail() {
    let mut r = PolicyResult::allowed("ok");
    r.add_detail(PolicyDetail {
        policy: "p1".into(),
        matched: true,
        reason: "condition met".into(),
    });
    r.add_detail(PolicyDetail {
        policy: "p2".into(),
        matched: false,
        reason: "condition not met".into(),
    });
    assert_eq!(r.details.len(), 2);
    assert!(r.details[0].matched);
    assert!(!r.details[1].matched);
}

#[test]
fn policy_result_serde_roundtrip() {
    let mut r = PolicyResult::allowed("ok");
    r.policy = Some("test-policy".into());
    let json = serde_json::to_string(&r).unwrap();
    let back: PolicyResult = serde_json::from_str(&json).unwrap();
    assert!(back.allowed);
    assert_eq!(back.policy.as_deref(), Some("test-policy"));
}

// ── Default policies ─────────────────────────────────────────────────

#[test]
fn default_policies_include_rate_limit() {
    let policies = default_policies();
    assert!(policies.iter().any(|p| p.name == "Rate Limit Policy"));
}

#[test]
fn default_policies_include_admin_bypass() {
    let policies = default_policies();
    assert!(policies.iter().any(|p| p.name == "Admin Bypass"));
}

#[test]
fn default_policies_include_security_audit_requirement() {
    let policies = default_policies();
    assert!(policies.iter().any(|p| p.name == "Require Security Audit for RC"));
}

#[test]
fn default_policies_include_rollback_plan_requirement() {
    let policies = default_policies();
    assert!(policies.iter().any(|p| p.name == "Require Rollback Plan for Production"));
}

// ── PolicyCheck ──────────────────────────────────────────────────────

#[test]
fn policy_check_serde_roundtrip() {
    let check = PolicyCheck {
        resource: "release".into(),
        action: "promote".into(),
        context: PolicyContext::new().with_channel(ReleaseChannel::Alpha),
    };
    let json = serde_json::to_string(&check).unwrap();
    let back: PolicyCheck = serde_json::from_str(&json).unwrap();
    assert_eq!(back.resource, "release");
    assert_eq!(back.action, "promote");
    assert_eq!(back.context.channel, Some(ReleaseChannel::Alpha));
}
