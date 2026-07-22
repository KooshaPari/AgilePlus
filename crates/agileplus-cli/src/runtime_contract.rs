//! Resolved local listener endpoints used by AgilePlus runtime diagnostics.

/// The process-local listener contract shared by CLI diagnostics and health probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRuntime {
    http_port: u16,
    grpc_port: u16,
}

impl ResolvedRuntime {
    /// Build a runtime contract after rejecting invalid or ambiguous listeners.
    pub fn new(http_port: u16, grpc_port: u16) -> Result<Self, String> {
        if http_port == 0 || grpc_port == 0 {
            return Err("HTTP and gRPC ports must be non-zero".to_owned());
        }
        if http_port == grpc_port {
            return Err("HTTP and gRPC ports must be distinct".to_owned());
        }
        Ok(Self {
            http_port,
            grpc_port,
        })
    }

    pub fn http_port(self) -> u16 {
        self.http_port
    }

    pub fn grpc_port(self) -> u16 {
        self.grpc_port
    }

    pub fn health_url(self) -> String {
        format!("http://127.0.0.1:{}/health", self.http_port)
    }
}

#[cfg(test)]
mod tests {
    use super::ResolvedRuntime;

    #[test]
    fn resolved_runtime_rejects_zero_or_duplicate_listener_ports() {
        assert!(ResolvedRuntime::new(0, 50_051).is_err());
        assert!(ResolvedRuntime::new(3_014, 3_014).is_err());
    }

    #[test]
    fn resolved_runtime_exposes_one_http_and_grpc_endpoint_pair() {
        let runtime = ResolvedRuntime::new(3_014, 50_051).expect("valid endpoints");

        assert_eq!(runtime.http_port(), 3_014);
        assert_eq!(runtime.grpc_port(), 50_051);
        assert_eq!(runtime.health_url(), "http://127.0.0.1:3014/health");
    }
}
