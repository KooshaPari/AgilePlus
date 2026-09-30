//! Shared governance evaluation result semantics.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernanceResult {
    Satisfied,
    Unsatisfied,
    Inconclusive,
    Unknown,
    NotConfigured,
    Stale,
}

impl GovernanceResult {
    pub fn as_str(self)->&'static str {
        match self {
            Self::Satisfied=>"satisfied",
            Self::Unsatisfied=>"unsatisfied",
            Self::Inconclusive=>"inconclusive",
            Self::Unknown=>"unknown",
            Self::NotConfigured=>"not_configured",
            Self::Stale=>"stale",
        }
    }
    pub fn compliant(self)->bool { self == Self::Satisfied }
}

/// Reduce a simple configured-rule summary without vacuous success.
/// Rich evidence/policy evaluation should feed this semantic result rather than
/// inventing transport-specific booleans.
pub fn summarize_rules(total:usize,satisfied:usize)->GovernanceResult{
    if total==0 { GovernanceResult::NotConfigured }
    else if satisfied==total { GovernanceResult::Satisfied }
    else { GovernanceResult::Unsatisfied }
}

#[cfg(test)]
mod tests{
 use super::*;
 #[test] fn empty_is_not_configured(){assert_eq!(summarize_rules(0,0),GovernanceResult::NotConfigured)}
 #[test] fn all_configured_rules_pass(){assert_eq!(summarize_rules(2,2),GovernanceResult::Satisfied)}
 #[test] fn partial_is_unsatisfied(){assert_eq!(summarize_rules(2,1),GovernanceResult::Unsatisfied)}
}
