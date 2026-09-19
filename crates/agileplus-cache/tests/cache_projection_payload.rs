//! Cached projection payload format.
//!
//! Cached projections outlive the process that wrote them (they are stored with
//! a TTL), so the JSON shape is a compatibility surface: renaming a field would
//! make already-cached entries unreadable. Exercised through the in-process RESP
//! double (`tests/common/redis.rs`).

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::pool::CachePool;
use agileplus_cache::projection::ProjectionCache;
use agileplus_cache::store::RedisCacheStore;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::WorkPackage;
use common::redis::MockRedisServer;
use serde_json::Value;
use std::sync::Arc;

async fn cache_with(server: &MockRedisServer) -> ProjectionCache {
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(2))
        .await
        .expect("pool");
    ProjectionCache::new(Arc::new(RedisCacheStore::new(pool, 3600)))
}

fn feature(id: i64) -> Feature {
    let mut feature = Feature::new("my-feat", "My Feature", [0xAB_u8; 32], Some("main"));
    feature.id = id;
    feature
}

fn work_package(id: i64) -> WorkPackage {
    let mut wp = WorkPackage::new(1, "Implement caching", 2, "reads under 10ms");
    wp.id = id;
    wp
}

// ===========================================================================
// Cached payload shape
// ===========================================================================
//
// Cached projections outlive the process that wrote them (they are stored with
// a TTL), so the JSON shape is a compatibility surface: renaming a field would
// make already-cached entries unreadable.

fn json_keys(value: &Value) -> Vec<String> {
    let mut keys: Vec<String> = value
        .as_object()
        .expect("payload must be a JSON object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

#[tokio::test]
async fn feature_projection_payload_shape_is_stable() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_feature(&feature(1)).await.expect("set");

    let payload: Value =
        serde_json::from_str(&server.stored("feature:1").expect("payload")).expect("json");

    assert_eq!(json_keys(&payload), vec!["cached_at", "feature"]);
    assert_eq!(
        json_keys(&payload["feature"]),
        vec![
            "created_at",
            "created_at_commit",
            "friendly_name",
            "id",
            "labels",
            "last_modified_commit",
            "module_id",
            "plane_issue_id",
            "plane_state_id",
            "project_id",
            "slug",
            "spec_hash",
            "state",
            "target_branch",
            "updated_at",
        ]
    );
}

#[tokio::test]
async fn workpackage_projection_payload_shape_is_stable() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_workpackage(&work_package(1)).await.expect("set");

    let payload: Value =
        serde_json::from_str(&server.stored("wp:1").expect("payload")).expect("json");

    assert_eq!(json_keys(&payload), vec!["cached_at", "workpackage"]);
    assert_eq!(
        json_keys(&payload["workpackage"]),
        vec![
            "acceptance_criteria",
            "agent_id",
            "created_at",
            "feature_id",
            "file_scope",
            "id",
            "pr_state",
            "pr_url",
            "sequence",
            "state",
            "title",
            "updated_at",
            "worktree_path",
        ]
    );
    for omitted in ["base_commit", "head_commit", "plane_sub_issue_id"] {
        assert!(
            payload["workpackage"].get(omitted).is_none(),
            "{omitted} must stay omitted while it is unset"
        );
    }
}

#[tokio::test]
async fn cached_payload_with_unknown_fields_still_decodes() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_feature(&feature(4)).await.expect("set");

    let mut payload: Value =
        serde_json::from_str(&server.stored("feature:4").expect("payload")).expect("json");
    payload["future_top_level"] = Value::from(1);
    payload["feature"]["future_field"] = Value::from("written by a newer release");
    server.seed("feature:4", &serde_json::to_string(&payload).expect("json"));

    let loaded = cache
        .get_feature(4)
        .await
        .expect("unknown fields must not break reads")
        .expect("present");

    assert_eq!(loaded.feature.id, 4);
    assert_eq!(loaded.feature.slug, "my-feat");
}
