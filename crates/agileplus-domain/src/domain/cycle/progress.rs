//! Cycle progress summary types.

use serde::{Deserialize, Serialize};

/// Aggregate count of work packages per state across all assigned features.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WpProgressSummary {
    pub total: u32,
    pub planned: u32,
    pub in_progress: u32,
    pub done: u32,
    pub blocked: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_zero() {
        let s = WpProgressSummary::default();
        assert_eq!(s.total, 0);
        assert_eq!(s.planned, 0);
        assert_eq!(s.in_progress, 0);
        assert_eq!(s.done, 0);
        assert_eq!(s.blocked, 0);
    }

    #[test]
    fn serde_roundtrip() {
        let s = WpProgressSummary {
            total: 10,
            planned: 3,
            in_progress: 4,
            done: 2,
            blocked: 1,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: WpProgressSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total, 10);
        assert_eq!(back.planned, 3);
        assert_eq!(back.in_progress, 4);
        assert_eq!(back.done, 2);
        assert_eq!(back.blocked, 1);
    }
}
