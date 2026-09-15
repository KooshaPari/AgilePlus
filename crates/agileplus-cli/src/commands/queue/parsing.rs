use std::str::FromStr;

use anyhow::Result;

use agileplus_domain::domain::backlog::{BacklogPriority, BacklogSort, BacklogStatus, Intent};

pub(crate) fn parse_intent(value: Option<String>) -> Result<Intent> {
    let value = value.unwrap_or_else(|| "task".to_string());
    Intent::from_str(&value).map_err(|e| anyhow::anyhow!(e))
}

pub(crate) fn parse_intent_opt(value: Option<String>) -> Result<Option<Intent>> {
    value
        .map(|v| Intent::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_priority(value: String) -> Result<BacklogPriority> {
    BacklogPriority::from_str(&value).map_err(|e| anyhow::anyhow!(e))
}

pub(crate) fn parse_priority_opt(value: Option<String>) -> Result<Option<BacklogPriority>> {
    value
        .map(|v| BacklogPriority::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_status_opt(value: Option<String>) -> Result<Option<BacklogStatus>> {
    value
        .map(|v| BacklogStatus::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_sort(value: &str) -> Result<BacklogSort> {
    BacklogSort::from_str(value).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_intent_valid() {
        assert_eq!(parse_intent(Some("bug".into())).unwrap(), Intent::Bug);
        assert_eq!(
            parse_intent(Some("FEATURE".into())).unwrap(),
            Intent::Feature
        );
    }

    #[test]
    fn parse_intent_invalid() {
        assert!(parse_intent(Some("xxx".into())).is_err());
    }

    // ── parse_intent (None default) ─────────────────────────────────────

    #[test]
    fn parse_intent_none_defaults_to_task() {
        assert_eq!(parse_intent(None).unwrap(), Intent::Task);
    }

    // ── parse_intent_opt ────────────────────────────────────────────────

    #[test]
    fn parse_intent_opt_none() {
        assert!(parse_intent_opt(None).unwrap().is_none());
    }

    #[test]
    fn parse_intent_opt_valid() {
        let result = parse_intent_opt(Some("bug".into())).unwrap();
        assert_eq!(result, Some(Intent::Bug));
    }

    #[test]
    fn parse_intent_opt_invalid() {
        assert!(parse_intent_opt(Some("bad".into())).is_err());
    }

    // ── parse_priority ──────────────────────────────────────────────────

    #[test]
    fn parse_priority_valid() {
        let result = parse_priority("high".to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn parse_priority_invalid() {
        assert!(parse_priority("nonsense".to_string()).is_err());
    }

    // ── parse_priority_opt ──────────────────────────────────────────────

    #[test]
    fn parse_priority_opt_none() {
        assert!(parse_priority_opt(None).unwrap().is_none());
    }

    #[test]
    fn parse_priority_opt_invalid() {
        assert!(parse_priority_opt(Some("bad".into())).is_err());
    }

    // ── parse_status_opt ────────────────────────────────────────────────

    #[test]
    fn parse_status_opt_none() {
        assert!(parse_status_opt(None).unwrap().is_none());
    }

    #[test]
    fn parse_status_opt_invalid() {
        assert!(parse_status_opt(Some("bogus".into())).is_err());
    }

    // ── parse_sort ──────────────────────────────────────────────────────

    #[test]
    fn parse_sort_valid() {
        let result = parse_sort("priority");
        assert!(result.is_ok());
    }

    #[test]
    fn parse_sort_invalid() {
        assert!(parse_sort("unknown_sort").is_err());
    }
}
