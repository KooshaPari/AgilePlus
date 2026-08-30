//! gRPC proxy/router — forwards agent and integration requests to downstream
//! services when available, and reports explicit unavailability otherwise.
//!
//! Traceability: WP14-T080b

use tracing::{info, warn};

/// Health status for downstream services.
#[derive(Debug, Clone, Default)]
pub struct DownstreamHealth {
    pub agents_reachable: bool,
    pub integrations_reachable: bool,
}

/// Proxy router that optionally connects to downstream gRPC services.
///
/// At startup the router attempts to connect to the `agileplus-agents` and
/// `agileplus-integrations` services. If they are unavailable it logs a
/// warning and uses in-process stubs for development / single-binary mode.
pub struct ProxyRouter {
    #[allow(dead_code)] // WIP: used for future downstream forwarding
    agents_address: Option<String>,
    #[allow(dead_code)] // WIP: used for future downstream forwarding
    integrations_address: Option<String>,
    health: DownstreamHealth,
}

impl ProxyRouter {
    /// Create a new proxy router.
    ///
    /// `agents_address` and `integrations_address` are optional; pass `None`
    /// to disable forwarding for that service (stub mode).
    pub async fn new(agents_address: Option<String>, integrations_address: Option<String>) -> Self {
        let agents_reachable = if let Some(ref addr) = agents_address {
            let reachable = Self::probe(addr).await;
            if reachable {
                info!(addr, "agileplus-agents downstream reachable");
            } else {
                warn!(addr, "agileplus-agents unreachable — using stub");
            }
            reachable
        } else {
            false
        };

        let integrations_reachable = if let Some(ref addr) = integrations_address {
            let reachable = Self::probe(addr).await;
            if reachable {
                info!(addr, "agileplus-integrations downstream reachable");
            } else {
                warn!(addr, "agileplus-integrations unreachable — using stub");
            }
            reachable
        } else {
            false
        };

        Self {
            agents_address,
            integrations_address,
            health: DownstreamHealth {
                agents_reachable,
                integrations_reachable,
            },
        }
    }

    /// Probe whether a gRPC endpoint is reachable by attempting a TCP connect.
    async fn probe(addr: &str) -> bool {
        // Strip grpc:// scheme if present
        let host_port = addr
            .trim_start_matches("http://")
            .trim_start_matches("grpc://");
        tokio::net::TcpStream::connect(host_port).await.is_ok()
    }

    /// Returns the current health status of downstream services.
    pub fn health(&self) -> &DownstreamHealth {
        &self.health
    }

    /// Dispatch an agent-related command.
    ///
    /// Reachability is reported by [`Self::health`], but dispatch remains
    /// unsupported until an actual downstream RPC client is wired.
    pub async fn dispatch_agent_command(
        &self,
        command: &str,
        feature_slug: &str,
        args: &std::collections::HashMap<String, String>,
    ) -> ProxyResult {
        let _ = args;
        ProxyResult::Stub {
            message: format!(
                "agents forwarding unsupported for '{command}' on '{feature_slug}'; no RPC client is configured"
            ),
        }
    }

    /// Dispatch an integration-related command.
    pub async fn dispatch_integration_command(
        &self,
        command: &str,
        feature_slug: &str,
    ) -> ProxyResult {
        ProxyResult::Stub {
            message: format!(
                "integrations forwarding unsupported for '{command}' on '{feature_slug}'; no RPC client is configured"
            ),
        }
    }
}

/// Result from a proxy dispatch.
#[derive(Debug)]
pub enum ProxyResult {
    Forwarded {
        success: bool,
        message: String,
        outputs: std::collections::HashMap<String, String>,
    },
    Stub {
        message: String,
    },
}

impl ProxyResult {
    pub fn is_success(&self) -> bool {
        match self {
            ProxyResult::Forwarded { success, .. } => *success,
            // A stub proves only that no downstream service handled the request.  It
            // must never acknowledge a mutating command (notably `implement`) as
            // successful.
            ProxyResult::Stub { .. } => false,
        }
    }

    pub fn message(&self) -> &str {
        match self {
            ProxyResult::Forwarded { message, .. } => message,
            ProxyResult::Stub { message } => message,
        }
    }

    pub fn outputs(&self) -> std::collections::HashMap<String, String> {
        match self {
            ProxyResult::Forwarded { outputs, .. } => outputs.clone(),
            ProxyResult::Stub { .. } => Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn proxy_stub_mode_rejects_mutating_command() {
        let router = ProxyRouter::new(None, None).await;
        let result = router
            .dispatch_agent_command("implement", "feat-a", &Default::default())
            .await;
        assert!(!result.is_success());
        assert!(result.message().contains("unsupported"));
    }

    #[tokio::test]
    async fn health_default_is_not_reachable() {
        let router = ProxyRouter::new(None, None).await;
        assert!(!router.health().agents_reachable);
        assert!(!router.health().integrations_reachable);
    }

    #[tokio::test]
    async fn reachable_ports_do_not_imply_forwarding_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let router = ProxyRouter::new(Some(address.clone()), Some(address)).await;

        assert!(router.health().agents_reachable);
        assert!(router.health().integrations_reachable);

        let agent = router
            .dispatch_agent_command("implement", "feat-a", &Default::default())
            .await;
        assert!(!agent.is_success());
        assert!(agent.message().contains("unsupported"));

        let integration = router.dispatch_integration_command("sync", "feat-a").await;
        assert!(!integration.is_success());
        assert!(integration.message().contains("unsupported"));
    }
}
