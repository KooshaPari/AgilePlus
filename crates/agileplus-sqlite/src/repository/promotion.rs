//! Serialized promotion journal and atomic terminal persistence.
use super::{audit, events, features};
use crate::SqliteStorageAdapter;
use agileplus_domain::{
    domain::{
        acceptance::FeatureAcceptanceReceipt,
        audit::{AuditChain, AuditEntry, hash_entry},
        event::Event,
        promotion::{PromotionJournal, PromotionReceipt, PromotionStep},
        state_machine::FeatureState,
    },
    error::DomainError,
    ports::promotion::PromotionPort,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
fn storage(e: impl std::fmt::Display) -> DomainError {
    DomainError::Storage(e.to_string())
}
fn conflict(message: &str) -> DomainError {
    DomainError::Conflict(message.into())
}
fn load(c: &Connection, id: i64) -> Result<Option<PromotionJournal>, DomainError> {
    let json: Option<String> = c
        .query_row(
            "SELECT journal_json FROM promotion_journals WHERE feature_id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    json.map(|s| serde_json::from_str(&s).map_err(storage))
        .transpose()
}
fn save(c: &Connection, j: &PromotionJournal) -> Result<(), DomainError> {
    c.execute(
        "UPDATE promotion_journals SET journal_json=?1 WHERE feature_id=?2",
        params![
            serde_json::to_string(j).map_err(storage)?,
            j.plan.feature_id
        ],
    )
    .map_err(storage)?;
    Ok(())
}
fn acceptance(
    c: &Connection,
    j: &PromotionJournal,
) -> Result<FeatureAcceptanceReceipt, DomainError> {
    let json: Option<String> = c.query_row("SELECT receipt_json FROM feature_acceptance_receipts WHERE feature_id=?1 AND request_id=?2", params![j.plan.feature_id, j.acceptance_request_id], |r| r.get(0)).optional().map_err(storage)?;
    let receipt: FeatureAcceptanceReceipt = serde_json::from_str(
        &json.ok_or_else(|| conflict("durable acceptance receipt is required for promotion"))?,
    )
    .map_err(storage)?;
    if j.plan.entries.is_empty()
        || receipt.accepted_candidates.len() != j.plan.entries.len()
        || j.plan.entries.iter().any(|entry| {
            !receipt
                .accepted_candidates
                .iter()
                .any(|c| c.wp_id == entry.wp_id && c.candidate_ref == entry.accepted_candidate_ref)
        })
    {
        return Err(conflict(
            "promotion plan does not match the immutable acceptance receipt",
        ));
    }
    Ok(receipt)
}
#[async_trait::async_trait]
impl PromotionPort for SqliteStorageAdapter {
    async fn get_promotion(&self, id: i64) -> Result<Option<PromotionJournal>, DomainError> {
        let connection = self.lock()?;
        load(&connection, id)
    }
    async fn begin_promotion(
        &self,
        journal: &PromotionJournal,
    ) -> Result<PromotionJournal, DomainError> {
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        if let Some(prior) = load(&tx, journal.plan.feature_id)? {
            if prior.plan != journal.plan
                || prior.acceptance_request_id != journal.acceptance_request_id
            {
                return Err(conflict(
                    "another promotion plan is already recorded for this feature",
                ));
            }
            return Ok(prior);
        }
        if !journal.steps.is_empty()
            || journal.receipt.is_some()
            || journal.cleanup_complete
            || journal.initial_target.is_empty()
        {
            return Err(conflict(
                "promotion must begin with an empty journal and exact target",
            ));
        }
        let feature = features::get_feature_by_id(&tx, journal.plan.feature_id)?
            .ok_or_else(|| DomainError::NotFound("feature".into()))?;
        if feature.state != FeatureState::Validated {
            return Err(conflict("promotion requires Validated"));
        }
        acceptance(&tx, journal)?;
        tx.execute(
            "INSERT INTO promotion_journals(feature_id,journal_json) VALUES (?1,?2)",
            params![
                journal.plan.feature_id,
                serde_json::to_string(journal).map_err(storage)?
            ],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(journal.clone())
    }
    async fn prepare_promotion_step(
        &self,
        id: i64,
        index: usize,
        step: &PromotionStep,
    ) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut j = load(&tx, id)?.ok_or_else(|| conflict("promotion journal missing"))?;
        let expected = j
            .steps
            .last()
            .map(|s| &s.resulting_commit)
            .unwrap_or(&j.initial_target);
        if j.receipt.is_some()
            || index != j.steps.len()
            || index >= j.plan.entries.len()
            || j.steps.iter().any(|s| !s.confirmed)
            || step.confirmed
            || step.expected_target != *expected
            || step.resulting_commit.is_empty()
        {
            return Err(conflict("promotion step changed or is out of order"));
        }
        j.steps.push(step.clone());
        save(&tx, &j)?;
        tx.commit().map_err(storage)
    }
    async fn confirm_promotion_step(&self, id: i64, index: usize) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut j = load(&tx, id)?.ok_or_else(|| conflict("promotion journal missing"))?;
        j.steps
            .get_mut(index)
            .ok_or_else(|| conflict("prepared promotion step missing"))?
            .confirmed = true;
        save(&tx, &j)?;
        tx.commit().map_err(storage)
    }
    async fn finalize_promotion(&self, id: i64) -> Result<PromotionReceipt, DomainError> {
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut j = load(&tx, id)?.ok_or_else(|| conflict("promotion journal missing"))?;
        if let Some(receipt) = j.receipt {
            return Ok(receipt);
        }
        if j.steps.len() != j.plan.entries.len()
            || j.steps.is_empty()
            || j.steps.iter().any(|s| !s.confirmed)
        {
            return Err(conflict(
                "all exact merge publications must be confirmed before Shipped",
            ));
        }
        acceptance(&tx, &j)?;
        let history = audit::get_audit_trail(&tx, id)?;
        if !history.is_empty() {
            AuditChain { entries: history }
                .verify_chain()
                .map_err(DomainError::Validation)?;
        }
        let now = chrono::Utc::now();
        if tx.execute("UPDATE features SET state='shipped',updated_at=?1 WHERE id=?2 AND state='validated'",params![now.to_rfc3339(),id]).map_err(storage)? != 1 {
            return Err(conflict("feature changed before promotion finalization"));
        }
        let mut entry = AuditEntry {
            id: 0,
            feature_id: id,
            wp_id: None,
            timestamp: now,
            actor: "user".into(),
            transition: "Validated -> Shipped".into(),
            evidence_refs: vec![],
            prev_hash: audit::get_latest_audit_entry(&tx, id)?
                .map(|e| e.hash)
                .unwrap_or([0; 32]),
            hash: [0; 32],
            event_id: None,
            archived_to: None,
        };
        entry.hash = hash_entry(&entry);
        let audit_id = audit::append_audit_entry(&tx, &entry)?;
        let result = j.steps.last().unwrap().resulting_commit.clone();
        let candidates: Vec<String> = j
            .plan
            .entries
            .iter()
            .map(|e| e.accepted_candidate_ref.clone())
            .collect();
        let mut event = Event::new(
            "feature",
            id,
            "state_transitioned",
            serde_json::json!({"from":"Validated","to":"Shipped","acceptance_request_id":j.acceptance_request_id,"target_branch":j.plan.target_branch,"resulting_commit":result,"accepted_candidates":candidates}),
            "user",
        );
        event.timestamp = now;
        event.sequence = events::get_latest_sequence(&tx, "feature", id)? + 1;
        event.prev_hash = events::get_events(&tx, "feature", id)?
            .last()
            .map(|e| e.hash)
            .unwrap_or([0; 32]);
        event.hash = agileplus_events::compute_hash(
            event.entity_id,
            &event.entity_type,
            &event.event_type,
            &event.payload,
            event.timestamp,
            &event.actor,
            &event.prev_hash,
        )
        .map_err(storage)?;
        let event_id = events::append_event(&tx, &event)?;
        let receipt = PromotionReceipt {
            feature_id: id,
            acceptance_request_id: j.acceptance_request_id.clone(),
            target_branch: j.plan.target_branch.clone(),
            resulting_commit: result,
            accepted_candidates: candidates,
            audit_id,
            event_id,
            committed_at: now,
        };
        j.receipt = Some(receipt.clone());
        save(&tx, &j)?;
        tx.commit().map_err(storage)?;
        Ok(receipt)
    }
    async fn complete_promotion_cleanup(&self, id: i64) -> Result<(), DomainError> {
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let mut j = load(&tx, id)?.ok_or_else(|| conflict("promotion journal missing"))?;
        if j.receipt.is_none() {
            return Err(conflict("cleanup requires committed promotion"));
        }
        j.cleanup_complete = true;
        save(&tx, &j)?;
        tx.commit().map_err(storage)
    }
}
