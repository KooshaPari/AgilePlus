// SPDX-License-Identifier: MIT OR Apache-2.0
//! Unit tests for `GovernanceEvaluation::configured` and `passed` —
//! kill cargo-mutants true/false/&&/|| mutations.

use agileplus_domain::domain::governance::{
    GovernanceContract, GovernanceRule,
};
use agileplus_domain::domain::governance_evaluator::{EvidenceCheck, GovernanceEvaluation, PolicyEvalResult};
use chrono::Utc;

fn make_contract(rules: Vec<GovernanceRule>) -> GovernanceContract {
    GovernanceContract {
        id: 1,
        feature_id: 1,
        version: 1,
        rules,
        bound_at: Utc::now(),
    }
}

fn rule(ev: Vec<String>, pol: Vec<i64>) -> GovernanceRule {
    GovernanceRule {
        transition: "t".to_string(),
        required_evidence: ev,
        policy_refs: pol,
    }
}

fn empty_eval() -> GovernanceEvaluation {
    GovernanceEvaluation {
        evidence_results: vec![],
        policy_results: vec![],
        missing_evidence: vec![],
        rule_results: vec![],
    }
}

#[test]
fn configured_empty_rules_false() {
    let c = make_contract(vec![]);
    assert!(!empty_eval().configured(&c));
}

#[test]
fn configured_vacuous_rule_false() {
    let c = make_contract(vec![rule(vec![], vec![])]);
    assert!(!empty_eval().configured(&c));
}

#[test]
fn configured_rule_with_evidence_true() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![])]);
    assert!(empty_eval().configured(&c));
}

#[test]
fn configured_rule_with_policy_refs_true() {
    let c = make_contract(vec![rule(vec![], vec![1])]);
    assert!(empty_eval().configured(&c));
}

#[test]
fn configured_mixed_rules_true() {
    let c = make_contract(vec![rule(vec![], vec![]), rule(vec![], vec![1])]);
    assert!(empty_eval().configured(&c));
}

#[test]
fn passed_requires_configured() {
    // configured true, everything else clean -> passed true
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![],
        policy_results: vec![],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(ev.passed(&c));
}

#[test]
fn passed_empty_contract_false() {
    let c = make_contract(vec![]);
    assert!(!empty_eval().passed(&c));
}

#[test]
fn passed_false_missing_evidence() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![],
        policy_results: vec![],
        missing_evidence: vec![("FR-001".to_string(), "CiOutput".to_string())],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}

#[test]
fn passed_false_evidence_not_found() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![agileplus_domain::domain::governance_evaluator::EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: false,
            threshold_met: true,
            message: "m".to_string(),
        }],
        policy_results: vec![],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}

#[test]
fn passed_false_threshold_not_met() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![agileplus_domain::domain::governance_evaluator::EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: true,
            threshold_met: false,
            message: "m".to_string(),
        }],
        policy_results: vec![],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}

#[test]
fn passed_false_policy_fails() {
    let c = make_contract(vec![rule(vec![], vec![1])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![],
        policy_results: vec![PolicyEvalResult {
            policy_id: 1,
            domain: "quality".to_string(),
            passed: false,
            message: "fail".to_string(),
        }],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}

#[test]
fn passed_true_all_good() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![1])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![agileplus_domain::domain::governance_evaluator::EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }],
        policy_results: vec![PolicyEvalResult {
            policy_id: 1,
            domain: "quality".to_string(),
            passed: true,
            message: "ok".to_string(),
        }],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(ev.passed(&c));
}

#[test]
fn passed_false_when_evidence_found_but_threshold_false_even_if_policy_ok() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![1])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![agileplus_domain::domain::governance_evaluator::EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: true,
            threshold_met: false,
            message: "m".to_string(),
        }],
        policy_results: vec![PolicyEvalResult {
            policy_id: 1,
            domain: "quality".to_string(),
            passed: true,
            message: "ok".to_string(),
        }],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}

#[test]
fn passed_false_when_policy_fails_even_if_evidence_ok() {
    let c = make_contract(vec![rule(vec!["FR-001:ci_output".to_string()], vec![1])]);
    let ev = GovernanceEvaluation {
        evidence_results: vec![agileplus_domain::domain::governance_evaluator::EvidenceCheck {
            fr_id: "FR-001".to_string(),
            evidence_type: "CiOutput".to_string(),
            found: true,
            threshold_met: true,
            message: "OK".to_string(),
        }],
        policy_results: vec![PolicyEvalResult {
            policy_id: 1,
            domain: "quality".to_string(),
            passed: false,
            message: "fail".to_string(),
        }],
        missing_evidence: vec![],
        rule_results: vec![],
    };
    assert!(!ev.passed(&c));
}
