//! Integration tests for core types: enums, structs, serde, FromStr, Display.
//! Complements the inline unit tests in src/types.rs.

use agileplus_governance::*;

// ── ConnectionStatus ─────────────────────────────────────────────────

#[test]
fn connection_status_all_variants_serde_roundtrip() {
    for s in [
        ConnectionStatus::Connected,
        ConnectionStatus::Disconnected,
        ConnectionStatus::Error,
        ConnectionStatus::Disabled,
    ] {
        let j = serde_json::to_string(&s).unwrap();
        let back: ConnectionStatus = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }
}

#[test]
fn connection_status_json_is_lowercase() {
    assert_eq!(serde_json::to_string(&ConnectionStatus::Connected).unwrap(), "\"connected\"");
    assert_eq!(serde_json::to_string(&ConnectionStatus::Error).unwrap(), "\"error\"");
}

// ── ActionCategory ───────────────────────────────────────────────────

#[test]
fn action_category_all_variants_from_str() {
    let cases = [
        ("release", ActionCategory::Release),
        ("repository", ActionCategory::Repository),
        ("policy", ActionCategory::Policy),
        ("audit", ActionCategory::Audit),
        ("config", ActionCategory::Config),
        ("general", ActionCategory::General),
    ];
    for (s, expected) in cases {
        assert_eq!(s.parse::<ActionCategory>().unwrap(), expected);
    }
}

#[test]
fn action_category_case_insensitive() {
    assert_eq!("RELEASE".parse::<ActionCategory>().unwrap(), ActionCategory::Release);
    assert_eq!("Repository".parse::<ActionCategory>().unwrap(), ActionCategory::Repository);
}

#[test]
fn action_category_serde_roundtrip_all() {
    for c in [
        ActionCategory::Release,
        ActionCategory::Repository,
        ActionCategory::Policy,
        ActionCategory::Audit,
        ActionCategory::Config,
        ActionCategory::General,
    ] {
        let j = serde_json::to_string(&c).unwrap();
        let back: ActionCategory = serde_json::from_str(&j).unwrap();
        assert_eq!(c, back);
    }
}

// ── OperationResult ──────────────────────────────────────────────────

#[test]
fn operation_result_from_str_variants() {
    assert_eq!("success".parse::<OperationResult>().unwrap(), OperationResult::Success);
    assert_eq!("failure".parse::<OperationResult>().unwrap(), OperationResult::Failure);
    assert_eq!("partial_success".parse::<OperationResult>().unwrap(), OperationResult::PartialSuccess);
    assert_eq!("partialsuccess".parse::<OperationResult>().unwrap(), OperationResult::PartialSuccess);
}

#[test]
fn operation_result_display_all() {
    assert_eq!(OperationResult::Success.to_string(), "success");
    assert_eq!(OperationResult::Failure.to_string(), "failure");
    assert_eq!(OperationResult::PartialSuccess.to_string(), "partial_success");
}

#[test]
fn operation_result_serde_roundtrip() {
    for r in [
        OperationResult::Success,
        OperationResult::Failure,
        OperationResult::PartialSuccess,
    ] {
        let j = serde_json::to_string(&r).unwrap();
        let back: OperationResult = serde_json::from_str(&j).unwrap();
        assert_eq!(r, back);
    }
}

// ── LogLevel ─────────────────────────────────────────────────────────

#[test]
fn log_level_from_str_all() {
    assert_eq!("debug".parse::<LogLevel>().unwrap(), LogLevel::Debug);
    assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
    assert_eq!("warn".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    assert_eq!("warning".parse::<LogLevel>().unwrap(), LogLevel::Warn);
    assert_eq!("error".parse::<LogLevel>().unwrap(), LogLevel::Error);
    assert_eq!("err".parse::<LogLevel>().unwrap(), LogLevel::Error);
}

#[test]
fn log_level_display_all() {
    assert_eq!(LogLevel::Debug.to_string(), "debug");
    assert_eq!(LogLevel::Info.to_string(), "info");
    assert_eq!(LogLevel::Warn.to_string(), "warn");
    assert_eq!(LogLevel::Error.to_string(), "error");
}

#[test]
fn log_level_serde_roundtrip() {
    for l in [LogLevel::Debug, LogLevel::Info, LogLevel::Warn, LogLevel::Error] {
        let j = serde_json::to_string(&l).unwrap();
        let back: LogLevel = serde_json::from_str(&j).unwrap();
        assert_eq!(l, back);
    }
}

#[test]
fn log_level_default_is_info() {
    assert_eq!(LogLevel::default(), LogLevel::Info);
}

// ── AuthMethod ───────────────────────────────────────────────────────

#[test]
fn auth_method_serde_roundtrip() {
    for m in [AuthMethod::ApiKey, AuthMethod::BearerToken, AuthMethod::None] {
        let j = serde_json::to_string(&m).unwrap();
        let back: AuthMethod = serde_json::from_str(&j).unwrap();
        assert_eq!(m, back);
    }
}

#[test]
fn auth_method_json_uses_kebab_case() {
    assert_eq!(serde_json::to_string(&AuthMethod::ApiKey).unwrap(), "\"api-key\"");
    assert_eq!(serde_json::to_string(&AuthMethod::BearerToken).unwrap(), "\"bearer-token\"");
    assert_eq!(serde_json::to_string(&AuthMethod::None).unwrap(), "\"none\"");
}

#[test]
fn auth_method_default_is_api_key() {
    assert_eq!(AuthMethod::default(), AuthMethod::ApiKey);
}

// ── AuditEventId / PolicyCheckId ─────────────────────────────────────

#[test]
fn audit_event_id_unique() {
    let id1 = AuditEventId::new();
    let id2 = AuditEventId::new();
    assert_ne!(id1.0, id2.0);
}

#[test]
fn audit_event_id_default_is_new() {
    let id1 = AuditEventId::new();
    let id2 = AuditEventId::default();
    // Both start with "evt_" but should have different UUIDs
    assert!(id1.0.starts_with("evt_"));
    assert!(id2.0.starts_with("evt_"));
}

#[test]
fn audit_event_id_serde_roundtrip() {
    let id = AuditEventId("evt_custom-123".into());
    let j = serde_json::to_string(&id).unwrap();
    let back: AuditEventId = serde_json::from_str(&j).unwrap();
    assert_eq!(id, back);
}

#[test]
fn policy_check_id_unique() {
    let id1 = PolicyCheckId::new();
    let id2 = PolicyCheckId::new();
    assert_ne!(id1.0, id2.0);
}

#[test]
fn policy_check_id_serde_roundtrip() {
    let id = PolicyCheckId("chk_custom-456".into());
    let j = serde_json::to_string(&id).unwrap();
    let back: PolicyCheckId = serde_json::from_str(&j).unwrap();
    assert_eq!(id, back);
}

// ── GovernanceStats / GovernanceStatus / GovernanceStatusConfig ───────

#[test]
fn governance_stats_serde_roundtrip() {
    let stats = GovernanceStats {
        total: 42,
        today: 7,
        errors: 3,
        by_level: {
            let mut m = std::collections::HashMap::new();
            m.insert("info".into(), 35);
            m.insert("error".into(), 3);
            m
        },
        top_actions: vec![
            TopAction { action: "promote".into(), count: 20 },
            TopAction { action: "check".into(), count: 15 },
        ],
    };
    let j = serde_json::to_string(&stats).unwrap();
    let back: GovernanceStats = serde_json::from_str(&j).unwrap();
    assert_eq!(back.total, 42);
    assert_eq!(back.today, 7);
    assert_eq!(back.errors, 3);
    assert_eq!(back.top_actions.len(), 2);
}

#[test]
fn governance_status_serde_roundtrip() {
    let status = GovernanceStatus {
        initialized: true,
        connection_status: ConnectionStatus::Connected,
        remote_enabled: true,
        local_enabled: true,
        sync_enabled: false,
        last_sync: Some(chrono::Utc::now()),
        pending_operations: 5,
        config: GovernanceStatusConfig {
            governance_url: "http://gov:8080".into(),
            auth_method: AuthMethod::BearerToken,
            policy_enabled: true,
            rate_limit_enabled: true,
        },
        stats: GovernanceStats::default(),
    };
    let j = serde_json::to_string(&status).unwrap();
    let back: GovernanceStatus = serde_json::from_str(&j).unwrap();
    assert!(back.initialized);
    assert_eq!(back.connection_status, ConnectionStatus::Connected);
    assert!(back.remote_enabled);
    assert_eq!(back.pending_operations, 5);
    assert_eq!(back.config.auth_method, AuthMethod::BearerToken);
}

#[test]
fn governance_status_config_defaults() {
    let c = GovernanceStatusConfig::default();
    assert!(c.governance_url.is_empty());
    assert_eq!(c.auth_method, AuthMethod::ApiKey);
    assert!(c.policy_enabled);
    assert!(!c.rate_limit_enabled);
}

#[test]
fn top_action_serde_roundtrip() {
    let a = TopAction { action: "deploy".into(), count: 99 };
    let j = serde_json::to_string(&a).unwrap();
    let back: TopAction = serde_json::from_str(&j).unwrap();
    assert_eq!(back.action, "deploy");
    assert_eq!(back.count, 99);
}
