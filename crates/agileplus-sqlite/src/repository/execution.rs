//! SQLite CRUD for immutable execution records.

use agileplus_domain::{domain::execution::{Assignment, AssignmentStatus, Attempt, AttemptStatus, Evaluation, EvaluationResult, SpecRevision}, error::DomainError};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

fn err(e: rusqlite::Error)->DomainError{DomainError::Storage(e.to_string())}
fn dt(s:String)->Result<DateTime<Utc>,DomainError>{DateTime::parse_from_rfc3339(&s).map(|d|d.with_timezone(&Utc)).map_err(|e|DomainError::Storage(e.to_string()))}
fn asg(s:&str)->Result<AssignmentStatus,DomainError>{match s{"active"=>Ok(AssignmentStatus::Active),"superseded"=>Ok(AssignmentStatus::Superseded),"cancelled"=>Ok(AssignmentStatus::Cancelled),x=>Err(DomainError::Storage(format!("unknown assignment status {x}")))}}
fn att(s:&str)->Result<AttemptStatus,DomainError>{match s{"pending"=>Ok(AttemptStatus::Pending),"running"=>Ok(AttemptStatus::Running),"failed"=>Ok(AttemptStatus::Failed),"cancelled"=>Ok(AttemptStatus::Cancelled),"completed"=>Ok(AttemptStatus::Completed),"expired"=>Ok(AttemptStatus::Expired),x=>Err(DomainError::Storage(format!("unknown attempt status {x}")))}}
fn ev(s:&str)->Result<EvaluationResult,DomainError>{match s{"satisfied"=>Ok(EvaluationResult::Satisfied),"unsatisfied"=>Ok(EvaluationResult::Unsatisfied),"inconclusive"=>Ok(EvaluationResult::Inconclusive),"unknown"=>Ok(EvaluationResult::Unknown),"not_configured"=>Ok(EvaluationResult::NotConfigured),"stale"=>Ok(EvaluationResult::Stale),x=>Err(DomainError::Storage(format!("unknown evaluation result {x}")))}}

pub fn create_spec_revision(c:&Connection,r:&SpecRevision)->Result<(),DomainError>{
 c.execute("INSERT INTO spec_revisions(id,feature_id,content_hash,parent_revision_id,accepted_at,authority) VALUES (?1,?2,?3,?4,?5,?6)",params![r.id,r.feature_id,r.content_hash,r.parent_revision_id,r.accepted_at.to_rfc3339(),r.authority]).map_err(err)?;Ok(())
}
pub fn create_assignment(c:&Connection,a:&Assignment)->Result<(),DomainError>{
 let st=match a.status{AssignmentStatus::Active=>"active",AssignmentStatus::Superseded=>"superseded",AssignmentStatus::Cancelled=>"cancelled"};
 c.execute("INSERT INTO assignments(id,wp_id,spec_revision_id,created_at,supersedes_assignment_id,status) VALUES (?1,?2,?3,?4,?5,?6)",params![a.id,a.wp_id,a.spec_revision_id,a.created_at.to_rfc3339(),a.supersedes_assignment_id,st]).map_err(err)?;Ok(())
}
pub fn create_attempt(c:&Connection,a:&Attempt)->Result<(),DomainError>{
 let st=match a.status{AttemptStatus::Pending=>"pending",AttemptStatus::Running=>"running",AttemptStatus::Failed=>"failed",AttemptStatus::Cancelled=>"cancelled",AttemptStatus::Completed=>"completed",AttemptStatus::Expired=>"expired"};
 c.execute("INSERT INTO attempts(id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status,failure_class,started_at,ended_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![a.id,a.assignment_id,a.worker_id,a.backend,a.job_id,a.worktree_path,a.base_candidate_ref,a.result_candidate_ref,st,a.failure_class,a.started_at.to_rfc3339(),a.ended_at.map(|d|d.to_rfc3339())]).map_err(err)?;Ok(())
}
pub fn create_evaluation(c:&Connection,e:&Evaluation)->Result<(),DomainError>{
 let result=match e.result{EvaluationResult::Satisfied=>"satisfied",EvaluationResult::Unsatisfied=>"unsatisfied",EvaluationResult::Inconclusive=>"inconclusive",EvaluationResult::Unknown=>"unknown",EvaluationResult::NotConfigured=>"not_configured",EvaluationResult::Stale=>"stale"};
 let refs=serde_json::to_string(&e.evidence_refs).map_err(|x|DomainError::Storage(x.to_string()))?;
 c.execute("INSERT INTO evaluations(id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,evidence_refs,started_at,finished_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![e.id,e.assignment_id,e.attempt_id,e.candidate_ref,e.evaluator_id,e.evaluator_version,result,refs,e.started_at.to_rfc3339(),e.finished_at.to_rfc3339()]).map_err(err)?;Ok(())
}

pub fn list_attempts(c:&Connection,assignment_id:&str)->Result<Vec<Attempt>,DomainError>{
 let mut q=c.prepare("SELECT id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status,failure_class,started_at,ended_at FROM attempts WHERE assignment_id=?1 ORDER BY started_at,id").map_err(err)?;
 let rows=q.query_map([assignment_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,String>(8)?,r.get::<_,Option<String>>(9)?,r.get::<_,String>(10)?,r.get::<_,Option<String>>(11)?))).map_err(err)?;
 rows.map(|x|{let (id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status,failure_class,started_at,ended_at)=x.map_err(err)?;Ok(Attempt{id,assignment_id,worker_id,backend,job_id,worktree_path,base_candidate_ref,result_candidate_ref,status:att(&status)?,failure_class,started_at:dt(started_at)?,ended_at:ended_at.map(dt).transpose()?})}).collect()
}

pub fn list_evaluations(c:&Connection,assignment_id:&str)->Result<Vec<Evaluation>,DomainError>{
 let mut q=c.prepare("SELECT id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,evidence_refs,started_at,finished_at FROM evaluations WHERE assignment_id=?1 ORDER BY finished_at,id").map_err(err)?;
 let rows=q.query_map([assignment_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,String>(6)?,r.get::<_,String>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?))).map_err(err)?;
 rows.map(|x|{let(id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result,refs,started,finished)=x.map_err(err)?;Ok(Evaluation{id,assignment_id,attempt_id,candidate_ref,evaluator_id,evaluator_version,result:ev(&result)?,evidence_refs:serde_json::from_str(&refs).map_err(|e|DomainError::Storage(e.to_string()))?,started_at:dt(started)?,finished_at:dt(finished)?})}).collect()
}


pub fn update_attempt_runtime(c:&Connection,id:&str,status:AttemptStatus,job_id:Option<&str>,result_candidate_ref:Option<&str>,failure_class:Option<&str>,ended_at:Option<DateTime<Utc>>)->Result<(),DomainError>{
 let st=match status{AttemptStatus::Pending=>"pending",AttemptStatus::Running=>"running",AttemptStatus::Failed=>"failed",AttemptStatus::Cancelled=>"cancelled",AttemptStatus::Completed=>"completed",AttemptStatus::Expired=>"expired"};
 let changed=c.execute("UPDATE attempts SET status=?2, job_id=COALESCE(?3,job_id), result_candidate_ref=COALESCE(?4,result_candidate_ref), failure_class=COALESCE(?5,failure_class), ended_at=COALESCE(?6,ended_at) WHERE id=?1",
 params![id,st,job_id,result_candidate_ref,failure_class,ended_at.map(|d|d.to_rfc3339())]).map_err(err)?;
 if changed!=1{return Err(DomainError::Storage(format!("attempt {id} not found")))} Ok(())
}
