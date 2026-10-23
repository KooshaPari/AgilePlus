//! Serialized SQLite transactions bind PM changes to append-only hash chains.
use crate::SqliteStorageAdapter;
use agileplus_domain::{domain::work_item::*, error::DomainError, ports::work_item::WorkItemPort};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
fn db(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}
fn encode<T: serde::Serialize>(v: &T) -> Result<String, DomainError> {
    serde_json::to_value(v)
        .map_err(|e| DomainError::Storage(e.to_string()))?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| DomainError::Storage("expected enum string".into()))
}
fn decode<T: serde::de::DeserializeOwned>(s: String) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(s)).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}
fn item(c: &Connection, id: i64) -> Result<WorkItem, DomainError> {
    c.query_row("SELECT id,kind,title,state,cycle_id,feature_id,created_at,updated_at FROM work_item WHERE id=?1", [id], |r| Ok(WorkItem {
        id:r.get(0)?, kind:decode(r.get(1)?)?, title:r.get(2)?, state:decode(r.get(3)?)?,
        cycle_id:r.get(4)?, feature_id:r.get(5)?, created_at:decode(r.get(6)?)?, updated_at:decode(r.get(7)?)?,
    })).optional().map_err(db)?.ok_or_else(|| DomainError::NotFound(format!("work item {id}")))
}
fn audit(c: &Connection, id: i64) -> Result<Vec<AuditEvent>, DomainError> {
    let mut q=c.prepare("SELECT id,work_item_id,actor,action,payload_json,prev_hash,hash,created_at FROM audit_event WHERE work_item_id=?1 ORDER BY id").map_err(db)?;
    let rows = q
        .query_map([id], |r| {
            Ok(AuditEvent {
                id: r.get(0)?,
                work_item_id: r.get(1)?,
                actor: r.get(2)?,
                action: r.get(3)?,
                payload_json: r.get(4)?,
                prev_hash: r.get(5)?,
                hash: r.get(6)?,
                created_at: decode(r.get(7)?)?,
            })
        })
        .map_err(db)?;
    let events = rows.collect::<rusqlite::Result<Vec<_>>>().map_err(db)?;
    let mut previous = String::new();
    for e in &events {
        if e.prev_hash != previous || e.hash != e.compute_hash() {
            return Err(DomainError::Conflict(
                "invalid work-item audit chain".into(),
            ));
        }
        previous = e.hash.clone();
    }
    Ok(events)
}
fn append(
    c: &Connection,
    w: &WorkItem,
    actor: &str,
    action: &str,
    payload: serde_json::Value,
    previous_hash: String,
) -> Result<(), DomainError> {
    let id = c
        .query_row("SELECT COALESCE(MAX(id),0)+1 FROM audit_event", [], |r| {
            r.get(0)
        })
        .map_err(db)?;
    let mut e = AuditEvent {
        id,
        work_item_id: w.id,
        actor: actor.into(),
        action: action.into(),
        payload_json: payload.to_string(),
        prev_hash: previous_hash,
        hash: String::new(),
        created_at: w.updated_at,
    };
    e.hash = e.compute_hash();
    c.execute("INSERT INTO audit_event(id,work_item_id,actor,action,payload_json,prev_hash,hash,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![e.id,e.work_item_id,e.actor,e.action,e.payload_json,e.prev_hash,e.hash,e.created_at.to_rfc3339()]).map_err(db)?;
    Ok(())
}
#[async_trait::async_trait]
impl WorkItemPort for SqliteStorageAdapter {
    async fn create_work_item(
        &self,
        input: NewWorkItem,
        actor: &str,
    ) -> Result<WorkItem, DomainError> {
        input.validate()?;
        validate_actor(actor)?;
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db)?;
        for (table, reference) in [("cycles", input.cycle_id), ("features", input.feature_id)] {
            if let Some(id) = reference {
                let exists: bool = tx
                    .query_row(
                        &format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=?1)"),
                        [id],
                        |row| row.get(0),
                    )
                    .map_err(db)?;
                if !exists {
                    return Err(DomainError::Validation(format!(
                        "unknown {table} reference {id}"
                    )));
                }
            }
        }
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute("INSERT INTO work_item(kind,title,state,cycle_id,feature_id,created_at,updated_at) VALUES (?1,?2,'intake',?3,?4,?5,?5)", params![encode(&input.kind)?, input.title, input.cycle_id,input.feature_id,now]).map_err(db)?;
        let w = item(&tx, tx.last_insert_rowid())?;
        append(
            &tx,
            &w,
            actor,
            "created",
            serde_json::to_value(&w).map_err(|e| DomainError::Storage(e.to_string()))?,
            String::new(),
        )?;
        tx.commit().map_err(db)?;
        Ok(w)
    }
    async fn transition_work_item(
        &self,
        id: i64,
        target: WorkItemState,
        actor: &str,
    ) -> Result<WorkItem, DomainError> {
        validate_actor(actor)?;
        let mut c = self.lock()?;
        let tx = c
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db)?;
        let before = item(&tx, id)?;
        before.state.transition(target)?;
        let events = audit(&tx, id)?;
        if events.is_empty() {
            return Err(DomainError::Conflict("missing creation audit event".into()));
        }
        tx.execute(
            "UPDATE work_item SET state=?1,updated_at=?2 WHERE id=?3",
            params![encode(&target)?, chrono::Utc::now().to_rfc3339(), id],
        )
        .map_err(db)?;
        let w = item(&tx, id)?;
        append(
            &tx,
            &w,
            actor,
            "state_transitioned",
            serde_json::json!({"from":before.state,"to":target}),
            events.last().expect("creation event checked").hash.clone(),
        )?;
        tx.commit().map_err(db)?;
        Ok(w)
    }
    async fn list_work_item_audit(&self, id: i64) -> Result<Vec<AuditEvent>, DomainError> {
        let c = self.lock()?;
        item(&c, id)?;
        let events = audit(&c, id)?;
        if events.is_empty() {
            return Err(DomainError::Conflict("missing creation audit event".into()));
        }
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> NewWorkItem {
        NewWorkItem {
            kind: WorkItemKind::Task,
            title: "JOURNEY-schema".into(),
            cycle_id: None,
            feature_id: None,
        }
    }
    #[tokio::test]
    async fn work_item_chain_and_forbidden_edges() {
        let s = SqliteStorageAdapter::in_memory().unwrap();
        let w = s.create_work_item(input(), "test").await.unwrap();
        assert!(
            s.transition_work_item(w.id, WorkItemState::InReview, "test")
                .await
                .is_err()
        );
        for state in [
            WorkItemState::Triaged,
            WorkItemState::Planned,
            WorkItemState::InProgress,
            WorkItemState::InReview,
        ] {
            s.transition_work_item(w.id, state, "test").await.unwrap();
        }
        let events = s.list_work_item_audit(w.id).await.unwrap();
        assert_eq!(events.len(), 5);
        assert!(events[0].prev_hash.is_empty());
        for pair in events.windows(2) {
            assert_eq!(pair[1].prev_hash, pair[0].hash);
        }
        let c = s.lock().unwrap();
        assert!(
            c.execute(
                "UPDATE audit_event SET actor='tamper' WHERE work_item_id=?1",
                [w.id]
            )
            .is_err()
        );
        assert!(
            c.execute("DELETE FROM audit_event WHERE work_item_id=?1", [w.id])
                .is_err()
        );
    }
    #[tokio::test]
    async fn work_item_append_failure_rolls_back_and_input_is_validated() {
        let s = SqliteStorageAdapter::in_memory().unwrap();
        let mut bad = input();
        bad.title = " ".into();
        assert!(s.create_work_item(bad, "test").await.is_err());
        assert!(s.create_work_item(input(), "").await.is_err());
        let mut bad = input();
        bad.cycle_id = Some(9999);
        assert!(s.create_work_item(bad, "test").await.is_err());
        let w = s.create_work_item(input(), "test").await.unwrap();
        s.lock().unwrap().execute_batch("CREATE TRIGGER fail_audit BEFORE INSERT ON audit_event BEGIN SELECT RAISE(ABORT,'forced failure'); END;").unwrap();
        assert!(
            s.transition_work_item(w.id, WorkItemState::Triaged, "test")
                .await
                .is_err()
        );
        assert_eq!(
            item(&s.lock().unwrap(), w.id).unwrap().state,
            WorkItemState::Intake
        );
        assert_eq!(s.list_work_item_audit(w.id).await.unwrap().len(), 1);
        assert!(s.create_work_item(input(), "test").await.is_err());
        assert_eq!(
            s.lock()
                .unwrap()
                .query_row("SELECT count(*) FROM work_item", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[tokio::test]
    async fn work_item_tamper_detection() {
        let s = SqliteStorageAdapter::in_memory().unwrap();
        let w = s.create_work_item(input(), "test").await.unwrap();
        s.lock()
            .unwrap()
            .execute_batch(
                "DROP TRIGGER audit_event_no_update; UPDATE audit_event SET payload_json='{}';",
            )
            .unwrap();
        assert!(s.list_work_item_audit(w.id).await.is_err());
        assert!(
            s.transition_work_item(w.id, WorkItemState::Triaged, "test")
                .await
                .is_err()
        );
        assert_eq!(
            item(&s.lock().unwrap(), w.id).unwrap().state,
            WorkItemState::Intake
        );
    }

    #[tokio::test]
    async fn work_item_missing_creation_is_corruption() {
        let s = SqliteStorageAdapter::in_memory().unwrap();
        s.lock().unwrap().execute(
            "INSERT INTO work_item(kind,title,state,created_at,updated_at) VALUES ('task','JOURNEY-missing-audit','intake',?1,?1)",
            [chrono::Utc::now().to_rfc3339()],
        ).unwrap();
        assert!(s.list_work_item_audit(1).await.is_err());
        assert!(
            s.transition_work_item(1, WorkItemState::Triaged, "test")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn work_item_migration_cannot_discard_append_only_history() {
        let s = SqliteStorageAdapter::in_memory().unwrap();
        let w = s.create_work_item(input(), "test").await.unwrap();
        let c = s.lock().unwrap();
        let runner = crate::migrations::MigrationRunner::new(&c);
        assert!(runner.rollback_last().is_err());
        runner.run_all().unwrap();
        assert_eq!(audit(&c, w.id).unwrap().len(), 1);
    }
}
