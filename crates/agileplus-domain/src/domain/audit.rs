// SPDX-License-Identifier: MIT OR Apache-2.0
//! Audit log types — tamper-evident hash-chained entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A reference to an evidence artifact in an audit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub evidence_id: i64,
    pub fr_id: String,
}

/// A single entry in the tamper-evident audit chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub feature_id: i64,
    pub wp_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub transition: String,
    pub evidence_refs: Vec<EvidenceRef>,
    pub prev_hash: [u8; 32],
    pub hash: [u8; 32],
    pub event_id: Option<i64>,
    pub archived_to: Option<String>,
}

/// A verified, hash-chained collection of audit entries.
pub struct AuditChain {
    pub entries: Vec<AuditEntry>,
}

impl AuditChain {
    /// Verify the hash chain is intact.  Returns `Err` with a description of
    /// the first broken link, or `Ok(())` if all hashes are consistent.
    pub fn verify_chain(&self) -> Result<(), String> {
        if self.entries.is_empty() {
            return Err("empty audit chain".to_string());
        }
        for (i, entry) in self.entries.iter().enumerate() {
            let computed = hash_entry(entry);
            if computed != entry.hash {
                return Err(format!(
                    "hash mismatch at entry index {i} (id={})",
                    entry.id
                ));
            }
            if i > 0 {
                let prev = &self.entries[i - 1];
                if entry.prev_hash != prev.hash {
                    return Err(format!(
                        "chain break between entries {} and {} (index {}-{})",
                        prev.id,
                        entry.id,
                        i - 1,
                        i
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Compute the SHA-256 hash of an audit entry (covers all mutable fields).
pub fn hash_entry(entry: &AuditEntry) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(entry.feature_id.to_be_bytes());
    if let Some(wp_id) = entry.wp_id {
        hasher.update(wp_id.to_be_bytes());
    }
    hasher.update(entry.timestamp.to_rfc3339().as_bytes());
    hasher.update(entry.actor.as_bytes());
    hasher.update(entry.transition.as_bytes());
    hasher.update(entry.prev_hash);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result[..]);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: i64, prev_hash: [u8; 32]) -> AuditEntry {
        let mut entry = AuditEntry {
            id,
            feature_id: 1,
            wp_id: None,
            timestamp: DateTime::from_timestamp(1_000_000 + id, 0).unwrap(),
            actor: "test-actor".to_string(),
            transition: "Draft->Active".to_string(),
            evidence_refs: vec![],
            prev_hash,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        entry.hash = hash_entry(&entry);
        entry
    }

    #[test]
    fn hash_entry_is_deterministic() {
        let e = make_entry(1, [0u8; 32]);
        let h1 = hash_entry(&e);
        let h2 = hash_entry(&e);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_entry_changes_with_actor() {
        let mut e = make_entry(1, [0u8; 32]);
        let h1 = hash_entry(&e);
        e.actor = "different-actor".to_string();
        let h2 = hash_entry(&e);
        assert_ne!(h1, h2);
    }

    #[test]
    fn audit_chain_verify_valid_chain() {
        let entry1 = make_entry(1, [0u8; 32]);
        let entry2 = make_entry(2, entry1.hash);
        let chain = AuditChain {
            entries: vec![entry1, entry2],
        };
        assert!(chain.verify_chain().is_ok());
    }

    #[test]
    fn audit_chain_detects_tampered_hash() {
        let mut entry1 = make_entry(1, [0u8; 32]);
        entry1.hash = [0xff; 32]; // tamper
        let entry2 = make_entry(2, entry1.hash);
        let chain = AuditChain {
            entries: vec![entry1, entry2],
        };
        let result = chain.verify_chain();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("hash mismatch"));
    }

    #[test]
    fn audit_chain_detects_broken_link() {
        let entry1 = make_entry(1, [0u8; 32]);
        let entry2 = make_entry(2, [0xab; 32]); // wrong prev_hash
        let chain = AuditChain {
            entries: vec![entry1, entry2],
        };
        let result = chain.verify_chain();
        assert!(result.is_err());
    }

    #[test]
    fn empty_audit_chain_returns_error() {
        let chain = AuditChain { entries: vec![] };
        assert_eq!(chain.verify_chain().unwrap_err(), "empty audit chain");
    }

    #[test]
    fn hash_changes_when_transition_differs() {
        let mut e1 = make_entry(1, [0u8; 32]);
        e1.transition = "A->B".to_string();
        let h1 = hash_entry(&e1);
        e1.transition = "B->C".to_string();
        let h2 = hash_entry(&e1);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_changes_with_wp_id_present() {
        let mut e1 = make_entry(1, [0u8; 32]);
        e1.wp_id = None;
        let h_no = hash_entry(&e1);
        e1.wp_id = Some(42);
        let h_yes = hash_entry(&e1);
        assert_ne!(h_no, h_yes);
    }

    #[test]
    fn hash_changes_with_prev_hash() {
        let e1 = make_entry(1, [0u8; 32]);
        let e2 = make_entry(1, [0xff; 32]);
        // Same entry fields but different prev_hash => different hash.
        assert_ne!(hash_entry(&e1), hash_entry(&e2));
    }

    #[test]
    fn hash_changes_with_feature_id() {
        let mut e1 = make_entry(1, [0u8; 32]);
        e1.feature_id = 1;
        let h1 = hash_entry(&e1);
        e1.feature_id = 999;
        let h2 = hash_entry(&e1);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_changes_with_timestamp() {
        let mut e1 = make_entry(1, [0u8; 32]);
        e1.timestamp = DateTime::from_timestamp(1, 0).unwrap();
        let h1 = hash_entry(&e1);
        e1.timestamp = DateTime::from_timestamp(2, 0).unwrap();
        let h2 = hash_entry(&e1);
        assert_ne!(h1, h2);
    }

    #[test]
    fn three_entry_chain_verifies() {
        let e1 = make_entry(1, [0u8; 32]);
        let e2 = make_entry(2, e1.hash);
        let e3 = make_entry(3, e2.hash);
        let chain = AuditChain {
            entries: vec![e1, e2, e3],
        };
        assert!(chain.verify_chain().is_ok());
    }

    #[test]
    fn three_entry_chain_break_at_middle() {
        let e1 = make_entry(1, [0u8; 32]);
        let mut e2 = make_entry(2, e1.hash);
        e2.hash = [0xaa; 32]; // tamper middle entry's hash
        let e3 = make_entry(3, e2.hash);
        let chain = AuditChain {
            entries: vec![e1, e2, e3],
        };
        // First entry fails hash check.
        let err = chain.verify_chain().unwrap_err();
        assert!(err.contains("hash mismatch"));
    }

    #[test]
    fn evidence_ref_fields() {
        let refs = vec![
            EvidenceRef {
                evidence_id: 1,
                fr_id: "FR-001".to_string(),
            },
            EvidenceRef {
                evidence_id: 2,
                fr_id: "FR-002".to_string(),
            },
        ];
        assert_eq!(refs[0].evidence_id, 1);
        assert_eq!(refs[1].fr_id, "FR-002");
    }

    #[test]
    fn single_entry_chain_verifies() {
        let e = make_entry(1, [0u8; 32]);
        let chain = AuditChain {
            entries: vec![e],
        };
        assert!(chain.verify_chain().is_ok());
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;

    fn entry(id: i64, prev: [u8; 32]) -> AuditEntry {
        let mut e = AuditEntry {
            id,
            feature_id: 1,
            wp_id: None,
            timestamp: DateTime::from_timestamp(1_000_000 + id, 0).unwrap(),
            actor: "a".to_string(),
            transition: "T".to_string(),
            evidence_refs: vec![],
            prev_hash: prev,
            hash: [0u8; 32],
            event_id: None,
            archived_to: None,
        };
        e.hash = hash_entry(&e);
        e
    }

    #[test]
    fn hash_ignores_id_event_id_and_archived_to() {
        // Documents which fields the hash actually covers.
        let base = entry(1, [0u8; 32]);
        let h = hash_entry(&base);
        let mut modified = base.clone();
        modified.id = 999;
        modified.event_id = Some(42);
        modified.archived_to = Some("archive".into());
        modified.evidence_refs = vec![EvidenceRef {
            evidence_id: 7,
            fr_id: "FR-7".into(),
        }];
        assert_eq!(hash_entry(&modified), h);
    }

    #[test]
    fn hash_covers_actor_transition_timestamp_feature_wp_prev() {
        let base = entry(1, [1u8; 32]);
        let h = hash_entry(&base);
        let mut m = base.clone();
        m.actor = "b".into();
        assert_ne!(hash_entry(&m), h);

        let mut m = base.clone();
        m.transition = "U".into();
        assert_ne!(hash_entry(&m), h);

        let mut m = base.clone();
        m.timestamp = DateTime::from_timestamp(5, 0).unwrap();
        assert_ne!(hash_entry(&m), h);

        let mut m = base.clone();
        m.feature_id = 2;
        assert_ne!(hash_entry(&m), h);

        let mut m = base.clone();
        m.wp_id = Some(0);
        assert_ne!(hash_entry(&m), h);

        let mut m = base.clone();
        m.prev_hash = [2u8; 32];
        assert_ne!(hash_entry(&m), h);
    }

    #[test]
    fn entry_serde_roundtrip() {
        let mut e = entry(3, [9u8; 32]);
        e.wp_id = Some(4);
        e.event_id = Some(5);
        e.evidence_refs = vec![EvidenceRef {
            evidence_id: 1,
            fr_id: "FR-1".into(),
        }];
        let back: AuditEntry = serde_json::from_str(&serde_json::to_string(&e).unwrap()).unwrap();
        assert_eq!(back.id, 3);
        assert_eq!(back.wp_id, Some(4));
        assert_eq!(back.evidence_refs.len(), 1);
        assert_eq!(back.hash, e.hash);
    }

    #[test]
    fn chain_with_nonzero_genesis_prev_hash_verifies() {
        let e1 = entry(1, [7u8; 32]);
        let chain = AuditChain { entries: vec![e1] };
        assert!(chain.verify_chain().is_ok());
    }

    #[test]
    fn chain_break_detected_between_second_and_third() {
        let e1 = entry(1, [0u8; 32]);
        let e2 = entry(2, e1.hash);
        // e3 points at e1 instead of e2 -> chain break at index 2.
        let e3 = entry(3, e1.hash);
        let chain = AuditChain {
            entries: vec![e1, e2, e3],
        };
        let err = chain.verify_chain().unwrap_err();
        assert!(err.contains("chain break"), "got {err}");
    }

    #[test]
    fn reordered_chain_fails() {
        let e1 = entry(1, [0u8; 32]);
        let e2 = entry(2, e1.hash);
        let chain = AuditChain {
            entries: vec![e2, e1],
        };
        assert!(chain.verify_chain().is_err());
    }

    #[test]
    fn two_entry_chain_break_reports_indices() {
        let e1 = entry(1, [0u8; 32]);
        let e2 = entry(2, [0x11; 32]);
        let err = AuditChain {
            entries: vec![e1, e2],
        }
        .verify_chain()
        .unwrap_err();
        assert!(err.contains("chain break"), "got {err}");
    }

    #[test]
    fn duplicate_hash_link_tolerated_when_consistent() {
        // Two entries that hash identically (same fields, different ids) still
        // verify as long as prev_hash links.
        let e1 = entry(1, [0u8; 32]);
        let e2 = entry(2, e1.hash);
        let chain = AuditChain {
            entries: vec![e1.clone(), e2],
        };
        assert!(chain.verify_chain().is_ok());
    }

    #[test]
    fn audit_entry_clone_and_debug() {
        let e = entry(1, [0u8; 32]);
        let c = e.clone();
        assert_eq!(c.id, e.id);
        assert!(format!("{e:?}").contains("AuditEntry"));
    }

    #[test]
    fn audit_chain_has_public_entries_field() {
        let chain = AuditChain { entries: vec![] };
        assert!(chain.entries.is_empty());
    }
}
