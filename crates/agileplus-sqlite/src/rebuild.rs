// SPDX-License-Identifier: MIT OR Apache-2.0
//! rebuild_from_git â€” reconstruct SQLite state from git artifacts (FR-017).

use agileplus_domain::{
    domain::{
        audit::{AuditEntry, EvidenceRef, hash_entry},
        feature::Feature,
        state_machine::FeatureState,
    },
    error::DomainError,
    ports::VcsPort,
};

use crate::SqliteStorageAdapter;

/// Summary of what was restored during a rebuild.
#[derive(Debug, Default)]
pub struct RebuildReport {
    pub features_restored: usize,
    pub wps_restored: usize,
    pub audit_entries_restored: usize,
    pub evidence_restored: usize,
}

/// Git meta.json schema (minimal â€” matches what spec-kitty would write).
#[derive(Debug, serde::Deserialize)]
struct MetaJson {
    slug: String,
    friendly_name: String,
    state: String,
    spec_hash: String,
    #[serde(default = "default_branch")]
    target_branch: String,
    created_at: String,
    updated_at: String,
}

fn default_branch() -> String {
    "main".into()
}

/// A line from audit/chain.jsonl.
#[derive(Debug, serde::Deserialize)]
struct AuditLine {
    #[allow(dead_code)]
    // serde-deserialized from audit/chain.jsonl - reserved for rebuild filtering
    feature_id: i64,
    wp_id: Option<i64>,
    timestamp: String,
    actor: String,
    transition: String,
    #[serde(default)]
    evidence_refs: Vec<EvidenceRef>,
    prev_hash: String,
    hash: String,
}

fn hex_to_32(s: &str) -> Result<[u8; 32], DomainError> {
    if s.len() != 64 {
        return Err(DomainError::Storage(format!(
            "expected 64-char hex, got {}",
            s.len()
        )));
    }
    let mut out = [0u8; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)
            .map_err(|e| DomainError::Storage(format!("hex parse error: {e}")))?;
    }
    Ok(out)
}

impl SqliteStorageAdapter {
    /// Rebuild SQLite from git artifacts via the VCS port.
    ///
    /// This clears all existing data and reconstructs from scratch within a
    /// single transaction. If any error occurs, the database is left unchanged.
    #[allow(clippy::await_holding_lock)] // Guard is explicitly dropped before any .await
    pub async fn rebuild_from_git<V: VcsPort>(
        &self,
        vcs: &V,
        feature_slugs: &[&str],
    ) -> Result<RebuildReport, DomainError> {
        let mut report = RebuildReport::default();

        // Lock the connection and do everything in a single transaction.
        // The guard is explicitly dropped below before any `.await`.
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        conn.execute_batch("BEGIN;")
            .map_err(|e| DomainError::Storage(e.to_string()))?;

        // Clear existing data
        conn.execute_batch(
            "DELETE FROM wp_dependencies;
             DELETE FROM evidence;
             DELETE FROM audit_log;
             DELETE FROM governance_contracts;
             DELETE FROM policy_rules;
             DELETE FROM metrics;
             DELETE FROM work_packages;
             DELETE FROM features;",
        )
        .map_err(|e| {
            let _ = conn.execute_batch("ROLLBACK;");
            DomainError::Storage(format!("clear failed: {e}"))
        })?;

        drop(conn); // Release lock before async VCS calls

        // Process each feature
        for slug in feature_slugs {
            match self.restore_feature::<V>(vcs, slug, &mut report).await {
                Ok(()) => {}
                Err(e) => {
                    // Rollback the transaction
                    let conn = self
                        .conn
                        .lock()
                        .map_err(|e| DomainError::Storage(e.to_string()))?;
                    let _ = conn.execute_batch("ROLLBACK;");
                    return Err(e);
                }
            }
        }

        // Commit
        let conn = self
            .conn
            .lock()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        conn.execute_batch("COMMIT;")
            .map_err(|e| DomainError::Storage(format!("commit failed: {e}")))?;

        Ok(report)
    }

    async fn restore_feature<V: VcsPort>(
        &self,
        vcs: &V,
        slug: &str,
        report: &mut RebuildReport,
    ) -> Result<(), DomainError> {
        // Read meta.json
        let meta_content = match vcs.read_artifact(slug, "meta.json").await {
            Ok(c) => c,
            Err(_) => return Ok(()), // Feature has no meta.json, skip
        };

        let meta: MetaJson = serde_json::from_str(&meta_content)
            .map_err(|e| DomainError::Storage(format!("meta.json parse error for {slug}: {e}")))?;

        let spec_hash = hex_to_32(&meta.spec_hash)?;
        let created_at = meta
            .created_at
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let updated_at = meta
            .updated_at
            .parse::<chrono::DateTime<chrono::Utc>>()
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let state = meta
            .state
            .parse::<FeatureState>()
            .map_err(|e| DomainError::Storage(format!("invalid state: {e}")))?;

        let feature = Feature {
            id: 0,
            slug: meta.slug.clone(),
            friendly_name: meta.friendly_name,
            state,
            spec_hash,
            target_branch: meta.target_branch,
            plane_issue_id: None,
            plane_state_id: None,
            labels: Vec::new(),
            module_id: None,
            project_id: None,
            created_at,
            updated_at,
            created_at_commit: None,
            last_modified_commit: None,
        };

        let feature_id = {
            let conn = self
                .conn
                .lock()
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            crate::repository::features::create_feature(&conn, &feature)?
        };
        report.features_restored += 1;

        // Read audit chain
        if let Ok(chain_content) = vcs.read_artifact(slug, "audit/chain.jsonl").await {
            let mut prev_hash = [0u8; 32];
            for (i, line) in chain_content.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let al: AuditLine = serde_json::from_str(line).map_err(|e| {
                    DomainError::Storage(format!("chain.jsonl line {i} parse error: {e}"))
                })?;

                let entry_prev_hash = hex_to_32(&al.prev_hash)?;
                let entry_hash = hex_to_32(&al.hash)?;

                // Verify chain integrity
                if i > 0 && entry_prev_hash != prev_hash {
                    return Err(DomainError::Storage(format!(
                        "audit chain broken at line {i} for feature {slug}"
                    )));
                }

                let timestamp = al
                    .timestamp
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| DomainError::Storage(e.to_string()))?;

                let entry = AuditEntry {
                    id: 0,
                    feature_id,
                    wp_id: al.wp_id,
                    timestamp,
                    actor: al.actor,
                    transition: al.transition,
                    evidence_refs: al.evidence_refs,
                    prev_hash: entry_prev_hash,
                    hash: entry_hash,
                    event_id: None,
                    archived_to: None,
                };

                // Verify self-hash
                let computed = hash_entry(&entry);
                if computed != entry_hash {
                    return Err(DomainError::Storage(format!(
                        "audit entry {i} hash mismatch for feature {slug}"
                    )));
                }

                {
                    let conn = self
                        .conn
                        .lock()
                        .map_err(|e| DomainError::Storage(e.to_string()))?;
                    // Insert directly (bypass chain check since we already verified above)
                    let evidence_refs_json = serde_json::to_string(&entry.evidence_refs)
                        .map_err(|e| DomainError::Storage(e.to_string()))?;
                    conn.execute(
                        "INSERT INTO audit_log
                         (feature_id, wp_id, timestamp, actor, transition, evidence_refs, prev_hash, hash)
                         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                        rusqlite::params![
                            entry.feature_id,
                            entry.wp_id,
                            entry.timestamp.to_rfc3339(),
                            entry.actor,
                            entry.transition,
                            evidence_refs_json,
                            entry.prev_hash.as_slice(),
                            entry.hash.as_slice(),
                        ],
                    )
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                }

                prev_hash = entry_hash;
                report.audit_entries_restored += 1;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use agileplus_domain::{
        domain::audit::hash_entry,
        error::DomainError,
        ports::{
            StoragePort, VcsPort,
            vcs::{BranchInfo, ConflictInfo, FeatureArtifacts, MergeResult, WorktreeInfo},
        },
    };

    use super::*;
    use crate::SqliteStorageAdapter;

    /// A simple in-memory VcsPort mock.
    struct MockVcs {
        artifacts: HashMap<String, String>,
    }

    impl MockVcs {
        fn new() -> Self {
            Self {
                artifacts: HashMap::new(),
            }
        }

        fn add(&mut self, slug: &str, path: &str, content: &str) {
            self.artifacts
                .insert(format!("{slug}/{path}"), content.to_string());
        }
    }

    #[async_trait::async_trait]
    impl VcsPort for MockVcs {
        async fn create_worktree(
            &self,
            _feature_slug: &str,
            _wp_id: &str,
        ) -> Result<PathBuf, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn cleanup_worktree(&self, _worktree_path: &Path) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn create_branch(&self, _branch_name: &str, _base: &str) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn list_branches(
            &self,
            _pattern: Option<&str>,
            _remote: bool,
        ) -> Result<Vec<BranchInfo>, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn delete_branch(
            &self,
            _branch_name: &str,
            _force: bool,
            _remote: Option<&str>,
        ) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn checkout_branch(&self, _branch_name: &str) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn merge_to_target(
            &self,
            _source: &str,
            _target: &str,
        ) -> Result<MergeResult, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn detect_conflicts(
            &self,
            _source: &str,
            _target: &str,
        ) -> Result<Vec<ConflictInfo>, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn read_artifact(
            &self,
            feature_slug: &str,
            relative_path: &str,
        ) -> Result<String, DomainError> {
            let key = format!("{feature_slug}/{relative_path}");
            self.artifacts
                .get(&key)
                .cloned()
                .ok_or(DomainError::NotFound(key))
        }

        async fn write_artifact(
            &self,
            _feature_slug: &str,
            _relative_path: &str,
            _content: &str,
        ) -> Result<(), DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn artifact_exists(
            &self,
            _feature_slug: &str,
            _relative_path: &str,
        ) -> Result<bool, DomainError> {
            Err(DomainError::NotImplemented)
        }

        async fn scan_feature_artifacts(
            &self,
            _feature_slug: &str,
        ) -> Result<FeatureArtifacts, DomainError> {
            Err(DomainError::NotImplemented)
        }
    }

    fn make_chain_jsonl(feature_id: i64) -> String {
        let now = chrono::Utc::now();

        // Entry 1: genesis
        let mut e1 = AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: now,
            actor: "system".into(),
            transition: "created".into(),
            evidence_refs: vec![],
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        e1.hash = hash_entry(&e1);

        // Entry 2
        let mut e2 = AuditEntry {
            id: 0,
            feature_id,
            wp_id: None,
            timestamp: now,
            actor: "agent".into(),
            transition: "created->specified".into(),
            evidence_refs: vec![],
            prev_hash: e1.hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        e2.hash = hash_entry(&e2);

        let hex = |b: [u8; 32]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();

        // Note: feature_id in JSON must match the one passed (will be 1 since fresh db)
        // Use feature_id=1 since rebuild sets it from the db insert
        let line1 = serde_json::json!({
            "feature_id": 1i64,
            "wp_id": null,
            "timestamp": now.to_rfc3339(),
            "actor": "system",
            "transition": "created",
            "evidence_refs": [],
            "prev_hash": hex([0u8; 32]),
            "hash": hex(e1.hash),
        });
        let line2 = serde_json::json!({
            "feature_id": 1i64,
            "wp_id": null,
            "timestamp": now.to_rfc3339(),
            "actor": "agent",
            "transition": "created->specified",
            "evidence_refs": [],
            "prev_hash": hex(e1.hash),
            "hash": hex(e2.hash),
        });

        format!("{line1}\n{line2}\n")
    }

    #[tokio::test]
    async fn rebuild_from_fixtures() {
        let now = chrono::Utc::now();
        let slug = "test-feature";

        let meta = serde_json::json!({
            "slug": slug,
            "friendly_name": "Test Feature",
            "state": "specified",
            "spec_hash": "a".repeat(64),
            "target_branch": "main",
            "created_at": now.to_rfc3339(),
            "updated_at": now.to_rfc3339(),
        });

        let mut mock = MockVcs::new();
        mock.add(slug, "meta.json", &meta.to_string());
        mock.add(slug, "audit/chain.jsonl", &make_chain_jsonl(1));

        let db = SqliteStorageAdapter::in_memory().unwrap();
        let report = db.rebuild_from_git(&mock, &[slug]).await.unwrap();

        assert_eq!(report.features_restored, 1);
        assert_eq!(report.audit_entries_restored, 2);

        let feat = StoragePort::get_feature_by_slug(&db, slug)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feat.slug, slug);
        assert_eq!(
            feat.state,
            agileplus_domain::domain::state_machine::FeatureState::Specified
        );
    }

    #[tokio::test]
    async fn rebuild_skips_missing_meta() {
        let mock = MockVcs::new(); // No artifacts
        let db = SqliteStorageAdapter::in_memory().unwrap();
        let report = db
            .rebuild_from_git(&mock, &["no-such-feature"])
            .await
            .unwrap();
        assert_eq!(report.features_restored, 0);
    }

    fn hex(b: [u8; 32]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    /// Build a `meta.json` payload; `spec_hash` and `state` are injectable so
    /// malformed values can be exercised.
    fn meta_json(slug: &str, state: &str, spec_hash: &str, target_branch: Option<&str>) -> String {
        let now = chrono::Utc::now().to_rfc3339();
        let mut value = serde_json::json!({
            "slug": slug,
            "friendly_name": "Test Feature",
            "state": state,
            "spec_hash": spec_hash,
            "created_at": now,
            "updated_at": now,
        });
        if let Some(branch) = target_branch {
            value["target_branch"] = serde_json::Value::String(branch.to_string());
        }
        value.to_string()
    }

    /// Build one `audit/chain.jsonl` line plus the hash it claims.
    fn chain_line(
        actor: &str,
        transition: &str,
        prev_hash: [u8; 32],
        evidence_refs: Vec<EvidenceRef>,
    ) -> (String, [u8; 32]) {
        let entry = AuditEntry {
            id: 0,
            feature_id: 1,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: actor.to_string(),
            transition: transition.to_string(),
            evidence_refs: evidence_refs.clone(),
            prev_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let hash = hash_entry(&entry);
        let line = serde_json::json!({
            "feature_id": 1i64,
            "wp_id": null,
            "timestamp": entry.timestamp.to_rfc3339(),
            "actor": actor,
            "transition": transition,
            "evidence_refs": evidence_refs,
            "prev_hash": hex(prev_hash),
            "hash": hex(hash),
        });
        (line.to_string(), hash)
    }

    async fn expect_storage_error(mock: &MockVcs, slug: &str) -> String {
        let db = SqliteStorageAdapter::in_memory().unwrap();
        let err = db.rebuild_from_git(mock, &[slug]).await.unwrap_err();
        let DomainError::Storage(message) = err else {
            panic!("expected DomainError::Storage, got {err:?}");
        };
        // A failed rebuild must leave no partially restored features behind.
        assert!(
            StoragePort::list_all_features(&db)
                .await
                .unwrap()
                .is_empty()
        );
        message
    }

    #[tokio::test]
    async fn rebuild_rejects_spec_hash_with_wrong_length() {
        let slug = "short-hash";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", "abc", Some("main")),
        );

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("64-char hex"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rejects_non_hex_spec_hash() {
        let slug = "not-hex";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", &"z".repeat(64), Some("main")),
        );

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("hex parse error"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rejects_malformed_meta_json() {
        let slug = "broken-meta";
        let mut mock = MockVcs::new();
        mock.add(slug, "meta.json", "{not valid json");

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("meta.json parse error"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rejects_unknown_feature_state() {
        let slug = "bad-state";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "not-a-state", &"a".repeat(64), Some("main")),
        );

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("invalid state"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rejects_unparseable_meta_timestamp() {
        let slug = "bad-timestamp";
        let mut mock = MockVcs::new();
        let mut value: serde_json::Value =
            serde_json::from_str(&meta_json(slug, "specified", &"a".repeat(64), Some("main")))
                .unwrap();
        value["created_at"] = serde_json::Value::String("not-a-timestamp".into());
        mock.add(slug, "meta.json", &value.to_string());

        let message = expect_storage_error(&mock, slug).await;
        assert!(!message.is_empty(), "error must describe the failure");
    }

    #[tokio::test]
    async fn rebuild_defaults_target_branch_to_main_and_skips_missing_chain() {
        let slug = "no-branch";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", &"a".repeat(64), None),
        );

        let db = SqliteStorageAdapter::in_memory().unwrap();
        let report = db.rebuild_from_git(&mock, &[slug]).await.unwrap();

        assert_eq!(report.features_restored, 1);
        assert_eq!(report.audit_entries_restored, 0);
        let feat = StoragePort::get_feature_by_slug(&db, slug)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(feat.target_branch, "main");
    }

    #[tokio::test]
    async fn rebuild_restores_audit_evidence_refs() {
        let slug = "with-evidence";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", &"a".repeat(64), Some("main")),
        );
        let refs = vec![EvidenceRef {
            evidence_id: 42,
            fr_id: "FR-AGP-001".to_string(),
        }];
        let (line, _) = chain_line("agent", "created", [0u8; 32], refs);
        mock.add(slug, "audit/chain.jsonl", &format!("{line}\n"));

        let db = SqliteStorageAdapter::in_memory().unwrap();
        let report = db.rebuild_from_git(&mock, &[slug]).await.unwrap();
        assert_eq!(report.audit_entries_restored, 1);

        let feature = StoragePort::get_feature_by_slug(&db, slug)
            .await
            .unwrap()
            .unwrap();
        let trail = StoragePort::get_audit_trail(&db, feature.id).await.unwrap();
        assert_eq!(trail.len(), 1);
        assert_eq!(trail[0].evidence_refs.len(), 1);
        assert_eq!(trail[0].evidence_refs[0].evidence_id, 42);
        assert_eq!(trail[0].evidence_refs[0].fr_id, "FR-AGP-001");
    }

    #[tokio::test]
    async fn rebuild_rejects_broken_audit_chain() {
        let slug = "broken-chain";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", &"a".repeat(64), Some("main")),
        );
        let (first, first_hash) = chain_line("agent", "created", [0u8; 32], vec![]);
        // Second entry claims a prev_hash that does not match the first entry.
        let second_entry = AuditEntry {
            id: 0,
            feature_id: 1,
            wp_id: None,
            timestamp: chrono::Utc::now(),
            actor: "agent".to_string(),
            transition: "created->specified".to_string(),
            evidence_refs: vec![],
            prev_hash: [9u8; 32],
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        let second = serde_json::json!({
            "feature_id": 1i64,
            "wp_id": null,
            "timestamp": second_entry.timestamp.to_rfc3339(),
            "actor": "agent",
            "transition": "created->specified",
            "evidence_refs": [],
            "prev_hash": hex([9u8; 32]),
            "hash": hex(hash_entry(&second_entry)),
        });
        assert_ne!(first_hash, [9u8; 32]);
        mock.add(slug, "audit/chain.jsonl", &format!("{first}\n{second}\n"));

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("audit chain broken"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rejects_tampered_audit_entry_hash() {
        let slug = "tampered";
        let mut mock = MockVcs::new();
        mock.add(
            slug,
            "meta.json",
            &meta_json(slug, "specified", &"a".repeat(64), Some("main")),
        );
        // Claim a hash that does not match the entry contents.
        let line = serde_json::json!({
            "feature_id": 1i64,
            "wp_id": null,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "actor": "agent",
            "transition": "created",
            "evidence_refs": [],
            "prev_hash": hex([0u8; 32]),
            "hash": hex([7u8; 32]),
        });
        mock.add(slug, "audit/chain.jsonl", &format!("{line}\n"));

        let message = expect_storage_error(&mock, slug).await;
        assert!(message.contains("hash mismatch"), "{message}");
    }

    #[tokio::test]
    async fn rebuild_rolls_back_all_features_when_a_later_feature_fails() {
        let good = "good-feature";
        let bad = "bad-feature";
        let mut mock = MockVcs::new();
        mock.add(
            good,
            "meta.json",
            &meta_json(good, "specified", &"a".repeat(64), Some("main")),
        );
        mock.add(
            bad,
            "meta.json",
            &meta_json(bad, "specified", "too-short", Some("main")),
        );

        let db = SqliteStorageAdapter::in_memory().unwrap();
        let err = db.rebuild_from_git(&mock, &[good, bad]).await.unwrap_err();
        assert!(matches!(err, DomainError::Storage(_)), "got {err:?}");
        assert!(
            StoragePort::list_all_features(&db)
                .await
                .unwrap()
                .is_empty(),
            "the first feature must be rolled back with the transaction"
        );
    }
}
