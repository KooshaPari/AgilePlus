//! Release channel governance for the 5-tier release model
//!
//! | Channel | Order | Version Suffix | Risk Profile |
//! |---------|-------|----------------|--------------|
//! | alpha   | 1     | `-alpha.N`     | Experimental |
//! | canary  | 2     | `-canary.N`    | Early Access |
//! | beta    | 3     | `-beta.N`      | Testing     |
//! | rc      | 4     | `-rc.N`        | Pre-release |
//! | prod    | 5     | (none)         | Stable      |

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 5-tier release channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    /// Experimental features - may change or break
    Alpha,
    /// Early access - unstable but visible
    Canary,
    /// Public testing - feature complete
    Beta,
    /// Release candidate - ready for production
    Rc,
    /// Production stable
    Prod,
}

impl ReleaseChannel {
    /// Get the channel order (1-based)
    pub fn order(&self) -> u8 {
        match self {
            ReleaseChannel::Alpha => 1,
            ReleaseChannel::Canary => 2,
            ReleaseChannel::Beta => 3,
            ReleaseChannel::Rc => 4,
            ReleaseChannel::Prod => 5,
        }
    }

    /// Get the version suffix for this channel
    pub fn version_suffix(&self, iteration: u32) -> String {
        match self {
            ReleaseChannel::Alpha => format!("-alpha.{}", iteration),
            ReleaseChannel::Canary => format!("-canary.{}", iteration),
            ReleaseChannel::Beta => format!("-beta.{}", iteration),
            ReleaseChannel::Rc => format!("-rc.{}", iteration),
            ReleaseChannel::Prod => String::new(), // No suffix for prod
        }
    }

    /// Get the PEP 440 compliant suffix for PyPI
    pub fn pep440_suffix(&self, iteration: u32) -> String {
        match self {
            ReleaseChannel::Alpha => format!("a{}", iteration),
            ReleaseChannel::Canary => format!("rc{}", iteration), // PEP 440 doesn't have canary
            ReleaseChannel::Beta => format!("b{}", iteration),
            ReleaseChannel::Rc => format!("rc{}", iteration),
            ReleaseChannel::Prod => String::new(),
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ReleaseChannel::Alpha => "Experimental features - may change or break",
            ReleaseChannel::Canary => "Early access - unstable but visible",
            ReleaseChannel::Beta => "Public testing - feature complete",
            ReleaseChannel::Rc => "Release candidate - ready for production",
            ReleaseChannel::Prod => "Production stable - guaranteed API stability",
        }
    }

    /// Check if promotion to next channel is allowed for high-risk packages
    pub fn next_channel(&self) -> Option<ReleaseChannel> {
        match self {
            ReleaseChannel::Alpha => Some(ReleaseChannel::Canary),
            ReleaseChannel::Canary => Some(ReleaseChannel::Beta),
            ReleaseChannel::Beta => Some(ReleaseChannel::Rc),
            ReleaseChannel::Rc => Some(ReleaseChannel::Prod),
            ReleaseChannel::Prod => None,
        }
    }

    /// Get all possible channel values
    pub fn all() -> &'static [ReleaseChannel; 5] {
        &[
            ReleaseChannel::Alpha,
            ReleaseChannel::Canary,
            ReleaseChannel::Beta,
            ReleaseChannel::Rc,
            ReleaseChannel::Prod,
        ]
    }

    /// Check if this channel requires tests before promotion
    pub fn requires_tests(&self) -> bool {
        matches!(
            self,
            ReleaseChannel::Beta | ReleaseChannel::Rc | ReleaseChannel::Prod
        )
    }

    /// Check if this channel requires security audit
    pub fn requires_security_audit(&self) -> bool {
        matches!(self, ReleaseChannel::Rc | ReleaseChannel::Prod)
    }

    /// Check if this channel requires documentation
    pub fn requires_docs(&self) -> bool {
        matches!(
            self,
            ReleaseChannel::Beta | ReleaseChannel::Rc | ReleaseChannel::Prod
        )
    }

    /// Check if this channel requires rollback plan
    pub fn requires_rollback_plan(&self) -> bool {
        matches!(self, ReleaseChannel::Rc | ReleaseChannel::Prod)
    }
}

impl fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseChannel::Alpha => write!(f, "alpha"),
            ReleaseChannel::Canary => write!(f, "canary"),
            ReleaseChannel::Beta => write!(f, "beta"),
            ReleaseChannel::Rc => write!(f, "rc"),
            ReleaseChannel::Prod => write!(f, "prod"),
        }
    }
}

impl FromStr for ReleaseChannel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alpha" | "a" => Ok(ReleaseChannel::Alpha),
            "canary" | "c" => Ok(ReleaseChannel::Canary),
            "beta" | "b" => Ok(ReleaseChannel::Beta),
            "rc" | "release-candidate" => Ok(ReleaseChannel::Rc),
            "prod" | "production" | "stable" | "p" => Ok(ReleaseChannel::Prod),
            _ => Err(format!("Unknown channel: {}", s)),
        }
    }
}

/// Metadata about a channel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMetadata {
    /// Channel name
    pub channel: ReleaseChannel,
    /// Version at this channel
    pub version: String,
    /// When this channel was set
    pub set_at: DateTime<Utc>,
    /// Who set this channel
    pub set_by: String,
    /// Iteration number (e.g., alpha.1, alpha.2)
    pub iteration: u32,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl ChannelMetadata {
    /// Create new channel metadata
    pub fn new(channel: ReleaseChannel, version: String, set_by: String, iteration: u32) -> Self {
        Self {
            channel,
            version,
            set_at: Utc::now(),
            set_by,
            iteration,
            metadata: None,
        }
    }

    /// Get the full version with channel suffix
    pub fn full_version(&self) -> String {
        if self.channel == ReleaseChannel::Prod {
            self.version.clone()
        } else {
            format!(
                "{}{}",
                self.version,
                self.channel.version_suffix(self.iteration)
            )
        }
    }
}

/// Promotion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRequest {
    /// Package name
    pub package: String,
    /// Current channel
    pub from: ReleaseChannel,
    /// Target channel
    pub to: ReleaseChannel,
    /// Requested by
    pub requested_by: String,
    /// Version to promote
    pub version: String,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl PromotionRequest {
    /// Create a new promotion request
    pub fn new(
        package: String,
        from: ReleaseChannel,
        to: ReleaseChannel,
        requested_by: String,
        version: String,
    ) -> Self {
        Self {
            package,
            from,
            to,
            requested_by,
            version,
            metadata: None,
        }
    }

    /// Check if this is a valid channel transition
    pub fn is_valid_transition(&self) -> bool {
        self.from < self.to
    }

    /// Check if this skips required channels (for high-risk)
    pub fn skips_channels(&self) -> Vec<ReleaseChannel> {
        let mut skipped = Vec::new();
        let mut current = self.from.next_channel();

        while let Some(channel) = current {
            if channel == self.to {
                break;
            }
            skipped.push(channel);
            current = channel.next_channel();
        }

        skipped
    }
}

/// Promotion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionResult {
    /// Whether promotion is allowed
    pub allowed: bool,
    /// Channel metadata if allowed
    pub channel_metadata: Option<ChannelMetadata>,
    /// Reason for decision
    pub reason: String,
    /// Policy checks passed
    pub policy_checks: Vec<String>,
    /// Policy checks failed
    pub policy_failures: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl PromotionResult {
    /// Create an allowed result
    pub fn allowed(channel_metadata: ChannelMetadata) -> Self {
        Self {
            allowed: true,
            channel_metadata: Some(channel_metadata),
            reason: "All checks passed".to_string(),
            policy_checks: Vec::new(),
            policy_failures: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a denied result
    pub fn denied(reason: String, failures: Vec<String>) -> Self {
        Self {
            allowed: false,
            channel_metadata: None,
            reason,
            policy_checks: Vec::new(),
            policy_failures: failures,
            warnings: Vec::new(),
        }
    }

    /// Add a policy check result
    pub fn add_check(&mut self, name: String, passed: bool) {
        if passed {
            self.policy_checks.push(name);
        } else {
            self.policy_failures.push(name);
        }
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_order() {
        assert!(ReleaseChannel::Alpha < ReleaseChannel::Canary);
        assert!(ReleaseChannel::Canary < ReleaseChannel::Beta);
        assert!(ReleaseChannel::Beta < ReleaseChannel::Rc);
        assert!(ReleaseChannel::Rc < ReleaseChannel::Prod);
    }

    #[test]
    fn test_version_suffix() {
        assert_eq!(ReleaseChannel::Alpha.version_suffix(1), "-alpha.1");
        assert_eq!(ReleaseChannel::Beta.version_suffix(3), "-beta.3");
        assert_eq!(ReleaseChannel::Prod.version_suffix(1), "");
    }

    #[test]
    fn test_channel_from_str() {
        assert_eq!(
            "alpha".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Alpha
        );
        assert_eq!("b".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Beta);
        assert_eq!(
            "production".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Prod
        );
    }

    #[test]
    fn test_promotion_request() {
        let req = PromotionRequest::new(
            "my-crate".to_string(),
            ReleaseChannel::Alpha,
            ReleaseChannel::Beta,
            "dev".to_string(),
            "0.1.0".to_string(),
        );

        assert!(req.is_valid_transition());
        assert_eq!(req.skips_channels(), vec![ReleaseChannel::Canary]);
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;

    #[test]
    fn version_suffix_all_channels() {
        assert_eq!(ReleaseChannel::Canary.version_suffix(1), "-canary.1");
        assert_eq!(ReleaseChannel::Rc.version_suffix(5), "-rc.5");
    }

    #[test]
    fn pep440_suffix() {
        assert_eq!(ReleaseChannel::Alpha.pep440_suffix(1), "a1");
        assert_eq!(ReleaseChannel::Beta.pep440_suffix(2), "b2");
        assert_eq!(ReleaseChannel::Rc.pep440_suffix(3), "rc3");
        assert_eq!(ReleaseChannel::Prod.pep440_suffix(1), "");
    }

    #[test]
    fn descriptions() {
        assert!(!ReleaseChannel::Alpha.description().is_empty());
        assert!(!ReleaseChannel::Prod.description().is_empty());
    }

    #[test]
    fn next_channel_chain() {
        assert_eq!(ReleaseChannel::Alpha.next_channel(), Some(ReleaseChannel::Canary));
        assert_eq!(ReleaseChannel::Canary.next_channel(), Some(ReleaseChannel::Beta));
        assert_eq!(ReleaseChannel::Beta.next_channel(), Some(ReleaseChannel::Rc));
        assert_eq!(ReleaseChannel::Rc.next_channel(), Some(ReleaseChannel::Prod));
        assert_eq!(ReleaseChannel::Prod.next_channel(), None);
    }

    #[test]
    fn requires_tests() {
        assert!(!ReleaseChannel::Alpha.requires_tests());
        assert!(!ReleaseChannel::Canary.requires_tests());
        assert!(ReleaseChannel::Beta.requires_tests());
        assert!(ReleaseChannel::Rc.requires_tests());
        assert!(ReleaseChannel::Prod.requires_tests());
    }

    #[test]
    fn requires_security_audit() {
        assert!(!ReleaseChannel::Alpha.requires_security_audit());
        assert!(!ReleaseChannel::Canary.requires_security_audit());
        assert!(!ReleaseChannel::Beta.requires_security_audit());
        assert!(ReleaseChannel::Rc.requires_security_audit());
        assert!(ReleaseChannel::Prod.requires_security_audit());
    }

    #[test]
    fn requires_docs() {
        assert!(!ReleaseChannel::Alpha.requires_docs());
        assert!(!ReleaseChannel::Canary.requires_docs());
        assert!(ReleaseChannel::Beta.requires_docs());
        assert!(ReleaseChannel::Rc.requires_docs());
        assert!(ReleaseChannel::Prod.requires_docs());
    }

    #[test]
    fn requires_rollback_plan() {
        assert!(!ReleaseChannel::Alpha.requires_rollback_plan());
        assert!(!ReleaseChannel::Canary.requires_rollback_plan());
        assert!(!ReleaseChannel::Beta.requires_rollback_plan());
        assert!(ReleaseChannel::Rc.requires_rollback_plan());
        assert!(ReleaseChannel::Prod.requires_rollback_plan());
    }

    #[test]
    fn channel_display() {
        assert_eq!(ReleaseChannel::Alpha.to_string(), "alpha");
        assert_eq!(ReleaseChannel::Canary.to_string(), "canary");
        assert_eq!(ReleaseChannel::Beta.to_string(), "beta");
        assert_eq!(ReleaseChannel::Rc.to_string(), "rc");
        assert_eq!(ReleaseChannel::Prod.to_string(), "prod");
    }

    #[test]
    fn channel_from_str_aliases() {
        assert_eq!("a".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Alpha);
        assert_eq!("c".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Canary);
        assert_eq!("b".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Beta);
        assert_eq!("release-candidate".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Rc);
        assert_eq!("stable".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
        assert_eq!("p".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
    }

    #[test]
    fn channel_from_str_invalid() {
        assert!("unknown".parse::<ReleaseChannel>().is_err());
    }

    #[test]
    fn channel_order_values() {
        assert_eq!(ReleaseChannel::Alpha.order(), 1);
        assert_eq!(ReleaseChannel::Canary.order(), 2);
        assert_eq!(ReleaseChannel::Beta.order(), 3);
        assert_eq!(ReleaseChannel::Rc.order(), 4);
        assert_eq!(ReleaseChannel::Prod.order(), 5);
    }

    #[test]
    fn channel_metadata_full_version_prod() {
        let meta = ChannelMetadata::new(ReleaseChannel::Prod, "1.0.0".into(), "dev".into(), 1);
        assert_eq!(meta.full_version(), "1.0.0");
    }

    #[test]
    fn channel_metadata_full_version_alpha() {
        let meta = ChannelMetadata::new(ReleaseChannel::Alpha, "1.0.0".into(), "dev".into(), 3);
        assert_eq!(meta.full_version(), "1.0.0-alpha.3");
    }

    #[test]
    fn promotion_request_invalid_reverse() {
        let req = PromotionRequest::new(
            "pkg".into(), ReleaseChannel::Prod, ReleaseChannel::Alpha,
            "dev".into(), "1.0.0".into(),
        );
        assert!(!req.is_valid_transition());
    }

    #[test]
    fn promotion_request_invalid_same_channel() {
        let req = PromotionRequest::new(
            "pkg".into(), ReleaseChannel::Beta, ReleaseChannel::Beta,
            "dev".into(), "1.0.0".into(),
        );
        assert!(!req.is_valid_transition());
    }

    #[test]
    fn promotion_request_skips_multiple() {
        let req = PromotionRequest::new(
            "pkg".into(), ReleaseChannel::Alpha, ReleaseChannel::Prod,
            "dev".into(), "1.0.0".into(),
        );
        assert_eq!(req.skips_channels(), vec![
            ReleaseChannel::Canary, ReleaseChannel::Beta, ReleaseChannel::Rc
        ]);
    }

    #[test]
    fn promotion_result_denied() {
        let result = PromotionResult::denied("bad".into(), vec!["check1".into()]);
        assert!(!result.allowed);
        assert!(result.channel_metadata.is_none());
        assert_eq!(result.policy_failures, vec!["check1"]);
    }

    #[test]
    fn promotion_result_allowed_with_metadata() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "1.0.0".into(), "dev".into(), 1);
        let mut result = PromotionResult::allowed(meta);
        result.add_check("test_check".into(), true);
        result.add_check("fail_check".into(), false);
        result.add_warning("deprecation ahead");
        assert!(result.allowed);
        assert!(result.policy_checks.contains(&"test_check".to_string()));
        assert!(result.policy_failures.contains(&"fail_check".to_string()));
        assert!(result.warnings.contains(&"deprecation ahead".to_string()));
    }

    #[test]
    fn channel_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        for ch in ReleaseChannel::all() {
            set.insert(*ch);
        }
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn promotion_request_metadata_serde() {
        let req = PromotionRequest::new(
            "pkg".into(), ReleaseChannel::Alpha, ReleaseChannel::Beta,
            "dev".into(), "1.0.0".into(),
        );
        let json = serde_json::to_string(&req).unwrap();
        let deser: PromotionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.package, deser.package);
        assert_eq!(req.from, deser.from);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    #[test]
    fn version_suffix_iteration_zero() {
        assert_eq!(ReleaseChannel::Alpha.version_suffix(0), "-alpha.0");
    }

    #[test]
    fn version_suffix_all_channels_present() {
        assert_eq!(ReleaseChannel::Canary.version_suffix(2), "-canary.2");
        assert_eq!(ReleaseChannel::Beta.version_suffix(7), "-beta.7");
        assert_eq!(ReleaseChannel::Rc.version_suffix(9), "-rc.9");
        assert_eq!(ReleaseChannel::Prod.version_suffix(9), "");
    }

    #[test]
    fn pep440_canary_maps_to_rc() {
        assert_eq!(ReleaseChannel::Canary.pep440_suffix(4), "rc4");
    }

    #[test]
    fn pep440_all_channels() {
        assert_eq!(ReleaseChannel::Alpha.pep440_suffix(1), "a1");
        assert_eq!(ReleaseChannel::Beta.pep440_suffix(2), "b2");
        assert_eq!(ReleaseChannel::Rc.pep440_suffix(3), "rc3");
        assert_eq!(ReleaseChannel::Prod.pep440_suffix(3), "");
    }

    #[test]
    fn descriptions_are_distinct_and_non_empty() {
        let all = ReleaseChannel::all();
        let mut seen = std::collections::HashSet::new();
        for ch in all {
            let d = ch.description();
            assert!(!d.is_empty());
            assert!(seen.insert(d), "duplicate description: {d}");
        }
    }

    #[test]
    fn all_returns_five_in_ascending_order() {
        let all = ReleaseChannel::all();
        assert_eq!(all.len(), 5);
        for w in all.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn next_channel_chain_terminates_at_prod() {
        assert_eq!(ReleaseChannel::Alpha.next_channel(), Some(ReleaseChannel::Canary));
        assert_eq!(ReleaseChannel::Rc.next_channel(), Some(ReleaseChannel::Prod));
        assert_eq!(ReleaseChannel::Prod.next_channel(), None);
    }

    #[test]
    fn from_str_alpha_full_and_short() {
        assert_eq!("alpha".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Alpha);
        assert_eq!("A".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Alpha);
    }

    #[test]
    fn from_str_canary_full_and_short() {
        assert_eq!("canary".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Canary);
        assert_eq!("C".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Canary);
    }

    #[test]
    fn from_str_beta_full_and_short() {
        assert_eq!("beta".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Beta);
        assert_eq!("B".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Beta);
    }

    #[test]
    fn from_str_rc_variants() {
        assert_eq!("rc".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Rc);
        assert_eq!("release-candidate".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Rc);
    }

    #[test]
    fn from_str_prod_variants() {
        assert_eq!("prod".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
        assert_eq!("production".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
        assert_eq!("stable".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
        assert_eq!("p".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Prod);
    }

    #[test]
    fn from_str_invalid_is_error() {
        assert!("nightly".parse::<ReleaseChannel>().is_err());
    }

    #[test]
    fn hash_set_deduplicates_channels() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ReleaseChannel::Alpha);
        set.insert(ReleaseChannel::Alpha);
        set.insert(ReleaseChannel::Prod);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn channel_metadata_new_sets_fields() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "2.0.0".into(), "alice".into(), 4);
        assert_eq!(meta.channel, ReleaseChannel::Beta);
        assert_eq!(meta.version, "2.0.0");
        assert_eq!(meta.set_by, "alice");
        assert_eq!(meta.iteration, 4);
        assert!(meta.metadata.is_none());
    }

    #[test]
    fn channel_metadata_full_version_canary() {
        let meta = ChannelMetadata::new(ReleaseChannel::Canary, "1.2.3".into(), "bob".into(), 2);
        assert_eq!(meta.full_version(), "1.2.3-canary.2");
    }

    #[test]
    fn channel_metadata_full_version_beta() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "1.0.0".into(), "bob".into(), 1);
        assert_eq!(meta.full_version(), "1.0.0-beta.1");
    }

    #[test]
    fn channel_metadata_full_version_rc() {
        let meta = ChannelMetadata::new(ReleaseChannel::Rc, "3.0.0".into(), "bob".into(), 5);
        assert_eq!(meta.full_version(), "3.0.0-rc.5");
    }

    #[test]
    fn channel_metadata_serde_roundtrip() {
        let meta = ChannelMetadata::new(ReleaseChannel::Rc, "3.0.0".into(), "bob".into(), 5);
        let json = serde_json::to_string(&meta).unwrap();
        let back: ChannelMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.channel, ReleaseChannel::Rc);
        assert_eq!(back.iteration, 5);
    }

    #[test]
    fn promotion_request_new_sets_fields() {
        let req = PromotionRequest::new(
            "crate-a".into(),
            ReleaseChannel::Alpha,
            ReleaseChannel::Canary,
            "carol".into(),
            "0.1.0".into(),
        );
        assert_eq!(req.package, "crate-a");
        assert_eq!(req.from, ReleaseChannel::Alpha);
        assert_eq!(req.to, ReleaseChannel::Canary);
        assert_eq!(req.requested_by, "carol");
        assert_eq!(req.version, "0.1.0");
        assert!(req.metadata.is_none());
    }

    #[test]
    fn promotion_request_adjacent_skips_nothing() {
        let req = PromotionRequest::new(
            "c".into(), ReleaseChannel::Beta, ReleaseChannel::Rc, "x".into(), "1".into(),
        );
        assert!(req.skips_channels().is_empty());
    }

    #[test]
    fn promotion_request_skips_from_canary_to_prod() {
        let req = PromotionRequest::new(
            "c".into(), ReleaseChannel::Canary, ReleaseChannel::Prod, "x".into(), "1".into(),
        );
        assert_eq!(req.skips_channels(), vec![ReleaseChannel::Beta, ReleaseChannel::Rc]);
    }

    #[test]
    fn promotion_request_forward_transition_is_valid() {
        let req = PromotionRequest::new(
            "c".into(), ReleaseChannel::Beta, ReleaseChannel::Prod, "x".into(), "1".into(),
        );
        assert!(req.is_valid_transition());
    }

    #[test]
    fn promotion_result_allowed_defaults() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "1.0.0".into(), "x".into(), 1);
        let result = PromotionResult::allowed(meta);
        assert!(result.allowed);
        assert_eq!(result.reason, "All checks passed");
        assert!(result.channel_metadata.is_some());
        assert!(result.policy_checks.is_empty());
        assert!(result.policy_failures.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn promotion_result_denied_records_failures() {
        let result = PromotionResult::denied("no".into(), vec!["a".into(), "b".into()]);
        assert!(!result.allowed);
        assert_eq!(result.policy_failures, vec!["a".to_string(), "b".to_string()]);
        assert!(result.channel_metadata.is_none());
    }

    #[test]
    fn promotion_result_add_check_routes_by_pass() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "1.0.0".into(), "x".into(), 1);
        let mut result = PromotionResult::allowed(meta);
        result.add_check("ok".into(), true);
        result.add_check("bad".into(), false);
        assert!(result.policy_checks.contains(&"ok".to_string()));
        assert!(result.policy_failures.contains(&"bad".to_string()));
    }

    #[test]
    fn promotion_result_add_warning_accepts_owned_string() {
        let meta = ChannelMetadata::new(ReleaseChannel::Beta, "1.0.0".into(), "x".into(), 1);
        let mut result = PromotionResult::allowed(meta);
        result.add_warning(String::from("careful"));
        assert_eq!(result.warnings, vec!["careful".to_string()]);
    }

    #[test]
    fn promotion_request_serde_roundtrip_all_fields() {
        let mut req = PromotionRequest::new(
            "pkg".into(), ReleaseChannel::Alpha, ReleaseChannel::Beta, "dev".into(), "1.0.0".into(),
        );
        req.metadata = Some(serde_json::json!({"ticket": 42}));
        let json = serde_json::to_string(&req).unwrap();
        let back: PromotionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.package, "pkg");
        assert_eq!(back.to, ReleaseChannel::Beta);
        assert!(back.metadata.is_some());
    }

    #[test]
    fn promotion_result_serde_roundtrip() {
        let meta = ChannelMetadata::new(ReleaseChannel::Prod, "1.0.0".into(), "x".into(), 1);
        let result = PromotionResult::allowed(meta);
        let json = serde_json::to_string(&result).unwrap();
        let back: PromotionResult = serde_json::from_str(&json).unwrap();
        assert!(back.allowed);
        assert_eq!(back.reason, result.reason);
    }
}
