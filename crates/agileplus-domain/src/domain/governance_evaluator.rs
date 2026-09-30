//! Shared governance evaluation service.
//!
//! This is migrated from the CLI validator so transports do not invent
//! independent compliance semantics.

use std::collections::BTreeSet;
use crate::{
    domain::governance::{BuiltinPolicy, Evidence, EvidenceType, GovernanceContract, PolicyCheck},
    error::DomainError,
    ports::StoragePort,
};

#[derive(Debug, Clone)]
pub struct EvidenceCheck {
    pub fr_id:String,
    pub evidence_type:String,
    pub found:bool,
    pub threshold_met:bool,
    pub message:String,
}
#[derive(Debug, Clone)]
pub struct PolicyEvalResult {
    pub policy_id:i64,
    pub domain:String,
    pub passed:bool,
    pub message:String,
}
#[derive(Debug, Clone)]
pub struct GovernanceEvaluation {
    pub evidence_results:Vec<EvidenceCheck>,
    pub policy_results:Vec<PolicyEvalResult>,
    pub missing_evidence:Vec<(String,String)>,
}
impl GovernanceEvaluation {
    pub fn configured(&self, contract:&GovernanceContract)->bool { !contract.rules.is_empty() }
    pub fn passed(&self, contract:&GovernanceContract)->bool {
        self.configured(contract)
        && self.missing_evidence.is_empty()
        && self.evidence_results.iter().all(|e|e.found && e.threshold_met)
        && self.policy_results.iter().all(|p|p.passed)
    }
}

fn parse_requirement(raw:&str)->(String,Option<EvidenceType>){
 if let Some((fr,ty))=raw.split_once(':'){
  let t=match ty{"test_result"=>Some(EvidenceType::TestResult),"ci_output"=>Some(EvidenceType::CiOutput),"review_approval"=>Some(EvidenceType::ReviewApproval),"security_scan"=>Some(EvidenceType::SecurityScan),"lint_result"=>Some(EvidenceType::LintResult),"manual_attestation"=>Some(EvidenceType::ManualAttestation),_=>None};
  (fr.into(),t)
 } else {(raw.into(),None)}
}

async fn feature_evidence<S:StoragePort>(storage:&S,feature_id:i64)->Result<Vec<Evidence>,DomainError>{
 let mut out=Vec::new();
 for wp in storage.list_wps_by_feature(feature_id).await? {
  out.extend(storage.get_evidence_by_wp(wp.id).await?);
 }
 Ok(out)
}

pub async fn evaluate_governance<S:StoragePort>(
 storage:&S,contract:&GovernanceContract,feature_id:i64
)->Result<GovernanceEvaluation,DomainError>{
 let evidence=feature_evidence(storage,feature_id).await?;
 let mut evidence_results=Vec::new(); let mut missing=Vec::new();
 for rule in &contract.rules { for raw in &rule.required_evidence {
  let (fr,expected)=parse_requirement(raw);
  let relevant:Vec<&Evidence>=evidence.iter().filter(|e|e.fr_id==fr && expected.map(|t|t==e.evidence_type).unwrap_or(true)).collect();
  let found=!relevant.is_empty();
  if !found { missing.push((fr.clone(),expected.map(|t|t.as_str().to_string()).unwrap_or_else(||"any".into()))); }
  evidence_results.push(EvidenceCheck{fr_id:fr,evidence_type:expected.map(|t|t.as_str().to_string()).unwrap_or_else(||"any".into()),found,threshold_met:found,message:if found{"OK".into()}else{"missing evidence".into()}});
 }}
 let active=storage.list_active_policies().await?;
 let referenced:BTreeSet<String>=contract.rules.iter().flat_map(|r|r.policy_refs.iter().map(ToString::to_string)).collect();
 let mut handled=BTreeSet::new(); let mut policy_results=Vec::new();
 for policy in &active {
  let matched:Vec<&String>=referenced.iter().filter(|r|policy.matches_reference(r)).collect();
  if matched.is_empty(){continue}
  let (passed,message)=match &policy.rule.check {
   PolicyCheck::EvidencePresent{evidence_type}|PolicyCheck::ManualApproval|PolicyCheck::Automated=>{
    let ty=match &policy.rule.check{PolicyCheck::EvidencePresent{evidence_type}=>*evidence_type,_=>EvidenceType::ManualAttestation};
    let ok=evidence.iter().any(|e|e.evidence_type==ty);(ok,format!("{} evidence {}",ty.as_str(),if ok{"present"}else{"missing"}))
   }
   PolicyCheck::ThresholdMet{metric,min}=>{
    let metrics=storage.get_metrics_by_feature(feature_id).await?;
    let value=metrics.iter().find_map(|m|match metric.as_str(){"duration_ms"=>Some(m.duration_ms as f64),"agent_runs"=>Some(m.agent_runs as f64),"review_cycles"=>Some(m.review_cycles as f64),x=>m.metadata.as_ref().and_then(|v|v.get(x)).and_then(|v|v.as_f64())});
    (value.map(|v|v>=*min).unwrap_or(false),format!("metric {metric}={value:?}, min={min}"))
   }
   PolicyCheck::Custom{script}=>(false,format!("custom policy requires external evaluator: {}",script.chars().take(60).collect::<String>())),
  };
  for r in matched {handled.insert(r.clone());}
  policy_results.push(PolicyEvalResult{policy_id:policy.id,domain:policy.domain.as_str().into(),passed,message});
 }
 for r in referenced.difference(&handled) {
  if let Some(b)=BuiltinPolicy::from_ref(r) {
   let ok=evidence.iter().any(|e|e.evidence_type==b.evidence_type);
   policy_results.push(PolicyEvalResult{policy_id:0,domain:b.domain.as_str().into(),passed:ok,message:format!("{}: {} evidence {}",b.label,b.evidence_type.as_str(),if ok{"present"}else{"missing"})});
  } else {
   policy_results.push(PolicyEvalResult{policy_id:0,domain:"custom".into(),passed:false,message:format!("policy ref {r} has no evaluator")});
  }
 }
 Ok(GovernanceEvaluation{evidence_results,policy_results,missing_evidence:missing})
}
