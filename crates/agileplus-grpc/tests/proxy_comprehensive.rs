//! Comprehensive tests for the proxy router and ProxyResult API.
//!
//! Complements the inline unit tests in `proxy.rs` and the existing
//! `grpc_integration.rs` tests.
//!
//! Traceability: WP14-T080b

use std::collections::HashMap;

use agileplus_grpc::proxy::{DownstreamHealth, ProxyResult, ProxyRouter};

// ---------------------------------------------------------------------------
// DownstreamHealth
// ---------------------------------------------------------------------------

#[test]
fn downstream_health_default_all_unreachable() {
    let h = DownstreamHealth::default();
    assert!(!h.agents_reachable);
    assert!(!h.integrations_reachable);
}

#[test]
fn downstream_health_clone() {
    let h = DownstreamHealth {
        agents_reachable: true,
        integrations_reachable: false,
    };
    let h2 = h.clone();
    assert!(h2.agents_reachable);
    assert!(!h2.integrations_reachable);
}

#[test]
fn downstream_health_debug() {
    let h = DownstreamHealth::default();
    let debug = format!("{:?}", h);
    assert!(debug.contains("DownstreamHealth"));
}

// ---------------------------------------------------------------------------
// ProxyResult
// ---------------------------------------------------------------------------

#[test]
fn forwarded_result_is_success_when_true() {
    let mut outputs = HashMap::new();
    outputs.insert("key".into(), "value".into());
    let r = ProxyResult::Forwarded {
        success: true,
        message: "ok".into(),
        outputs,
    };
    assert!(r.is_success());
}

#[test]
fn forwarded_result_not_success_when_false() {
    let r = ProxyResult::Forwarded {
        success: false,
        message: "failed".into(),
        outputs: Default::default(),
    };
    assert!(!r.is_success());
}

#[test]
fn stub_result_is_never_success() {
    let r = ProxyResult::Stub {
        message: "unavailable".into(),
    };
    assert!(!r.is_success());
}

#[test]
fn forwarded_result_message() {
    let r = ProxyResult::Forwarded {
        success: true,
        message: "forwarded to agents".into(),
        outputs: Default::default(),
    };
    assert_eq!(r.message(), "forwarded to agents");
}

#[test]
fn stub_result_message() {
    let r = ProxyResult::Stub {
        message: "service unavailable".into(),
    };
    assert_eq!(r.message(), "service unavailable");
}

#[test]
fn forwarded_result_outputs() {
    let mut outputs = HashMap::new();
    outputs.insert("branch".into(), "feature/test".into());
    outputs.insert("commit".into(), "abc123".into());
    let r = ProxyResult::Forwarded {
        success: true,
        message: "ok".into(),
        outputs,
    };
    let out = r.outputs();
    assert_eq!(out.len(), 2);
    assert_eq!(out.get("branch").unwrap(), "feature/test");
    assert_eq!(out.get("commit").unwrap(), "abc123");
}

#[test]
fn stub_result_outputs_is_empty() {
    let r = ProxyResult::Stub {
        message: "stub".into(),
    };
    let out = r.outputs();
    assert!(out.is_empty());
}

#[test]
fn proxy_result_debug() {
    let r = ProxyResult::Stub {
        message: "test".into(),
    };
    let debug = format!("{:?}", r);
    assert!(debug.contains("Stub"));
}

// ---------------------------------------------------------------------------
// ProxyRouter construction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn proxy_router_none_addresses_all_unreachable() {
    let router = ProxyRouter::new(None, None).await;
    let health = router.health();
    assert!(!health.agents_reachable);
    assert!(!health.integrations_reachable);
}

#[tokio::test]
async fn proxy_router_unreachable_agents_address() {
    // Port 59999 is unlikely to be in use
    let router = ProxyRouter::new(Some("localhost:59999".into()), None).await;
    assert!(!router.health().agents_reachable);
    assert!(!router.health().integrations_reachable);
}

#[tokio::test]
async fn proxy_router_unreachable_integrations_address() {
    let router = ProxyRouter::new(None, Some("localhost:59999".into())).await;
    assert!(!router.health().agents_reachable);
    assert!(!router.health().integrations_reachable);
}

// ---------------------------------------------------------------------------
// ProxyRouter dispatch_agent_command
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_agent_command_stub_mode_message_contains_slug() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_agent_command("implement", "my-feature", &Default::default())
        .await;
    let msg = result.message().to_string();
    assert!(msg.contains("my-feature"));
}

#[tokio::test]
async fn dispatch_agent_command_stub_mode_message_contains_command() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_agent_command("implement", "feat", &Default::default())
        .await;
    assert!(result.message().contains("implement"));
}

#[tokio::test]
async fn dispatch_agent_command_stub_outputs_are_empty() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_agent_command("implement", "feat", &Default::default())
        .await;
    assert!(result.outputs().is_empty());
}

#[tokio::test]
async fn dispatch_agent_command_stub_is_not_success() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_agent_command("implement", "feat", &Default::default())
        .await;
    assert!(!result.is_success());
}

// ---------------------------------------------------------------------------
// ProxyRouter dispatch_integration_command
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dispatch_integration_command_stub_mode() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_integration_command("connect", "my-feature")
        .await;
    assert!(!result.is_success());
    assert!(result.message().contains("my-feature"));
    assert!(result.message().contains("connect"));
}

#[tokio::test]
async fn dispatch_integration_command_stub_message_contains_slug() {
    let router = ProxyRouter::new(None, None).await;
    let result = router
        .dispatch_integration_command("sync", "feat-alpha")
        .await;
    assert!(result.message().contains("feat-alpha"));
}

// ---------------------------------------------------------------------------
// ProxyRouter health accessor
// ---------------------------------------------------------------------------

#[tokio::test]
async fn health_accessor_returns_reference() {
    let router = ProxyRouter::new(None, None).await;
    let health: &DownstreamHealth = router.health();
    assert!(!health.agents_reachable);
}

// ---------------------------------------------------------------------------
// Reachable downstreams (the forwarding branch)
//
// The router only probes TCP reachability, so a bound listener stands in for a
// live downstream service.
// ---------------------------------------------------------------------------

/// Bind a listener that accepts connections so `ProxyRouter::new` probes true.
fn reachable_address() -> (std::net::TcpListener, String) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let port = listener
        .local_addr()
        .expect("listener address should be readable")
        .port();
    (listener, format!("127.0.0.1:{port}"))
}

#[tokio::test]
async fn reachable_downstreams_are_reported_in_health() {
    let (_agents, agents_addr) = reachable_address();
    let (_integrations, integrations_addr) = reachable_address();

    let router = ProxyRouter::new(Some(agents_addr), Some(integrations_addr)).await;

    assert!(router.health().agents_reachable);
    assert!(router.health().integrations_reachable);
}

#[tokio::test]
async fn reachable_agents_address_produces_a_forwarded_result() {
    let (_agents, agents_addr) = reachable_address();
    let router = ProxyRouter::new(Some(agents_addr), None).await;

    let args = HashMap::from([("wp".to_string(), "2".to_string())]);
    let result = router
        .dispatch_agent_command("implement", "feat-a", &args)
        .await;

    assert!(result.is_success());
    assert!(result.message().contains("forwarded"));
    assert!(result.message().contains("feat-a"));
    assert_eq!(result.outputs(), args);
}

#[tokio::test]
async fn reachable_integrations_address_produces_a_forwarded_result() {
    let (_integrations, integrations_addr) = reachable_address();
    let router = ProxyRouter::new(None, Some(integrations_addr)).await;

    let result = router.dispatch_integration_command("sync", "feat-b").await;

    assert!(result.is_success());
    assert!(result.message().contains("forwarded"));
    assert!(result.message().contains("feat-b"));
    assert!(result.outputs().is_empty());
}

#[tokio::test]
async fn addresses_may_carry_a_scheme_prefix() {
    let (_agents, agents_addr) = reachable_address();
    let (_integrations, integrations_addr) = reachable_address();

    // `probe` strips `http://` and `grpc://` before connecting.
    let router = ProxyRouter::new(
        Some(format!("grpc://{agents_addr}")),
        Some(format!("http://{integrations_addr}")),
    )
    .await;

    assert!(router.health().agents_reachable);
    assert!(router.health().integrations_reachable);
}

#[tokio::test]
async fn a_closed_downstream_is_reported_unreachable() {
    let (listener, addr) = reachable_address();
    drop(listener);

    let router = ProxyRouter::new(Some(addr), None).await;

    assert!(!router.health().agents_reachable);
}
