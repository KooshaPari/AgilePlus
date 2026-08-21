//! Seed bridge for the dashboard.
//!
//! Breaks the prior `DashboardStore::seeded() ↔ build_dashboard_store()` recursion
//! by constructing the store inline (no call back to `seeded()`).

use crate::app_state::DashboardStore;

pub fn build_dashboard_store() -> DashboardStore {
    DashboardStore::default()
}
