//! Integration tests for `ids.rs` — newtype ID wrappers and CommitSha validation.

use std::str::FromStr;

use agileplus_domain::error::DomainError;
use agileplus_domain::ids::*;

#[test]
fn commit_sha_parse_valid_short() {
    let sha = CommitSha::parse_sha("abc1234").unwrap();
    assert_eq!(sha.as_str(), "abc1234");
}

#[test]
fn commit_sha_parse_valid_full() {
    let sha = CommitSha::parse_sha("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").unwrap();
    assert_eq!(sha.as_str(), "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2");
}

#[test]
fn commit_sha_rejects_too_short() {
    assert!(CommitSha::parse_sha("abc123").is_err());
}

#[test]
fn commit_sha_rejects_too_long() {
    let long = "a".repeat(41);
    assert!(CommitSha::parse_sha(&long).is_err());
}

#[test]
fn commit_sha_rejects_non_hex() {
    assert!(CommitSha::parse_sha("xyz1234").is_err());
}

#[test]
fn commit_sha_parse_display_roundtrip() {
    let sha = CommitSha::parse_sha("deadbeef0000000").unwrap();
    let display = sha.to_string();
    let sha2 = CommitSha::from_str(&display).unwrap();
    assert_eq!(sha, sha2);
}

#[test]
fn commit_sha_from_str_empty_is_err() {
    assert!(CommitSha::from_str("").is_err());
}

#[test]
fn newtype_id_as_ref_and_into_inner() {
    let id = FeatureId::new("feat-123");
    assert_eq!(id.as_ref(), "feat-123");
    assert_eq!(id.into_inner(), "feat-123");
}

#[test]
fn newtype_id_from_string() {
    let id = ProjectId::from("proj-abc".to_string());
    assert_eq!(id.as_str(), "proj-abc");
}

#[test]
fn newtype_id_from_str_ref() {
    let id = UserId::from("user-42");
    assert_eq!(id.as_str(), "user-42");
}

#[test]
fn newtype_id_from_str_empty_is_err() {
    assert!(EpicId::from_str("").is_err());
}

#[test]
fn newtype_id_from_str_nonempty_succeeds() {
    let id = EpicId::from_str("epic-7").unwrap();
    assert_eq!(id.as_str(), "epic-7");
}

#[test]
fn newtype_id_display() {
    let id = StoryId::new("story-99");
    assert_eq!(format!("{id}"), "story-99");
}

#[test]
fn newtype_id_clone() {
    let id = WorkPackageId::new("wp-1");
    let id2 = id.clone();
    assert_eq!(id, id2);
}

#[test]
fn newtype_id_debug() {
    let id = CycleId::new("cycle-3");
    let dbg = format!("{:?}", id);
    assert!(dbg.contains("cycle-3"));
}

#[test]
fn newtype_id_serde_roundtrip() {
    let id = BranchName::new("feature/auth");
    let json = serde_json::to_string(&id).unwrap();
    let back: BranchName = serde_json::from_str(&json).unwrap();
    assert_eq!(back.as_str(), "feature/auth");
}

#[test]
fn newtype_id_eq_and_hash() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let key = ModuleId::new("mod-1");
    map.insert(key.clone(), 42);
    assert_eq!(map.get(&key), Some(&42));
}

#[test]
fn commit_sha_eq() {
    let a = CommitSha::parse_sha("aabbccdd0000000").unwrap();
    let b = CommitSha::parse_sha("aabbccdd0000000").unwrap();
    assert_eq!(a, b);
}

#[test]
fn commit_sha_ne() {
    let a = CommitSha::parse_sha("aabbccdd0000000").unwrap();
    let b = CommitSha::parse_sha("112233445566666").unwrap();
    assert_ne!(a, b);
}
