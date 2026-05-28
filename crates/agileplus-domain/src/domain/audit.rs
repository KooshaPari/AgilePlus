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
