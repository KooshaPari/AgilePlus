//! AgilePlus Dashboard — Askama HTML templates + htmx route handlers.
//! Traceability: WP12 (T071–T077)

pub mod app_state;
pub mod health;
pub mod process_detector;
pub mod routes;
pub mod seed;
pub mod seed_bridge;
pub mod templates;

/// Resolves the dashboard listener host, allowing wildcard binding only by exact opt-in.
pub fn resolve_dashboard_host(host: Option<&str>) -> [u8; 4] {
    match host {
        Some("0.0.0.0") => [0, 0, 0, 0],
        _ => [127, 0, 0, 1],
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_dashboard_host;

    #[test]
    fn dashboard_host_defaults_to_loopback_when_unset() {
        assert_eq!(resolve_dashboard_host(None), [127, 0, 0, 1]);
    }

    #[test]
    fn dashboard_host_allows_only_exact_wildcard_opt_in() {
        assert_eq!(resolve_dashboard_host(Some("0.0.0.0")), [0, 0, 0, 0]);
    }

    #[test]
    fn dashboard_host_rejects_non_wildcard_values_to_loopback() {
        for host in ["127.0.0.1", "192.168.1.42", "::1", "not-an-address"] {
            assert_eq!(resolve_dashboard_host(Some(host)), [127, 0, 0, 1]);
        }
    }
}
