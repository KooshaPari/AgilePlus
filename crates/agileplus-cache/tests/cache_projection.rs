//! `ProjectionCache` behaviour: key layout, payload shape, TTL inheritance,
//! invalidation, and error propagation, driven by the in-process RESP double
//! (`tests/common/redis.rs`).

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::pool::CachePool;
use agileplus_cache::projection::{
    FeatureProjection, ProjectionCache, ProjectionError, WorkPackageProjection,
};
use agileplus_cache::store::RedisCacheStore;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::work_package::WorkPackage;
use common::redis::MockRedisServer;
use serde_json::Value;
use std::sync::Arc;

const DEFAULT_TTL_SECS: u64 = 3600;

async fn cache_with(server: &MockRedisServer) -> ProjectionCache {
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(4))
        .await
        .expect("pool");
    ProjectionCache::new(Arc::new(RedisCacheStore::new(pool, DEFAULT_TTL_SECS)))
}

fn feature(id: i64) -> Feature {
    let mut feature = Feature::new("my-feat", "My Feature", [0xAB_u8; 32], Some("main"));
    feature.id = id;
    feature.labels = vec!["alpha".to_string(), "beta".to_string()];
    feature.plane_issue_id = Some("PLANE-1".to_string());
    feature
}

fn work_package(id: i64) -> WorkPackage {
    let mut wp = WorkPackage::new(1, "Implement caching", 2, "reads under 10ms");
    wp.id = id;
    wp.file_scope = vec!["src/store.rs".to_string()];
    wp
}

// ===========================================================================
// Feature projections
// ===========================================================================

#[tokio::test]
async fn set_feature_stores_projection_under_feature_key() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.set_feature(&feature(42)).await.expect("set");

    assert_eq!(server.stored_keys(), vec!["feature:42".to_string()]);
    let payload: Value =
        serde_json::from_str(&server.stored("feature:42").expect("payload")).expect("json");
    assert_eq!(payload["feature"]["slug"], "my-feat");
    assert_eq!(payload["feature"]["id"], 42);
    assert!(
        payload.get("cached_at").is_some(),
        "projection must record when it was cached"
    );
}

#[tokio::test]
async fn get_feature_roundtrips_every_field() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    let original = feature(7);
    cache.set_feature(&original).await.expect("set");

    let loaded: FeatureProjection = cache.get_feature(7).await.expect("get").expect("present");

    assert_eq!(loaded.feature.id, 7);
    assert_eq!(loaded.feature.slug, "my-feat");
    assert_eq!(loaded.feature.friendly_name, "My Feature");
    assert_eq!(loaded.feature.spec_hash, [0xAB_u8; 32]);
    assert_eq!(loaded.feature.target_branch, "main");
    assert_eq!(loaded.feature.labels, original.labels);
    assert_eq!(loaded.feature.plane_issue_id.as_deref(), Some("PLANE-1"));
    assert_eq!(loaded.feature.created_at, original.created_at);
}

#[tokio::test]
async fn get_feature_returns_none_when_not_cached() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    assert!(cache.get_feature(1).await.expect("get").is_none());
}

#[tokio::test]
async fn feature_projections_use_the_store_default_ttl() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.set_feature(&feature(9)).await.expect("set");

    assert_eq!(
        server.setex_ttls(),
        vec![("feature:9".to_string(), DEFAULT_TTL_SECS as i64)]
    );
}

#[tokio::test]
async fn cached_at_timestamp_is_recent_and_parseable() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    let before = chrono::Utc::now();
    cache.set_feature(&feature(3)).await.expect("set");

    let payload: Value =
        serde_json::from_str(&server.stored("feature:3").expect("payload")).expect("json");
    let raw = payload["cached_at"]
        .as_str()
        .expect("cached_at is a string");
    assert!(
        raw.ends_with('Z') || raw.contains('+'),
        "expected an RFC3339 UTC timestamp, got {raw}"
    );
    let cached_at = chrono::DateTime::parse_from_rfc3339(raw).expect("parseable timestamp");
    let age = chrono::Utc::now() - cached_at.to_utc();
    assert!(
        age.num_seconds() < 60 && age.num_seconds() >= 0,
        "cached_at must be recent, got age {age}"
    );
    assert!(cached_at.to_utc() >= before - chrono::Duration::seconds(1));
}

#[tokio::test]
async fn invalidate_feature_deletes_the_key() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_feature(&feature(11)).await.expect("set");

    cache.invalidate_feature(11).await.expect("invalidate");

    assert_eq!(server.stored("feature:11"), None);
    assert!(cache.get_feature(11).await.expect("get").is_none());
    assert_eq!(
        server.first_command("DEL").expect("DEL").args,
        vec!["feature:11".to_string()]
    );
}

#[tokio::test]
async fn invalidate_feature_of_absent_key_succeeds() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.invalidate_feature(999).await.expect("idempotent");
}

// ===========================================================================
// Work package projections
// ===========================================================================

#[tokio::test]
async fn set_workpackage_stores_projection_under_wp_key() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.set_workpackage(&work_package(5)).await.expect("set");

    assert_eq!(server.stored_keys(), vec!["wp:5".to_string()]);
    let payload: Value =
        serde_json::from_str(&server.stored("wp:5").expect("payload")).expect("json");
    assert_eq!(payload["workpackage"]["id"], 5);
    assert_eq!(payload["workpackage"]["title"], "Implement caching");
}

#[tokio::test]
async fn get_workpackage_roundtrips_every_field() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    let original = work_package(13);
    cache.set_workpackage(&original).await.expect("set");

    let loaded: WorkPackageProjection = cache
        .get_workpackage(13)
        .await
        .expect("get")
        .expect("present");

    assert_eq!(loaded.workpackage.id, 13);
    assert_eq!(loaded.workpackage.feature_id, 1);
    assert_eq!(loaded.workpackage.title, "Implement caching");
    assert_eq!(loaded.workpackage.sequence, 2);
    assert_eq!(loaded.workpackage.acceptance_criteria, "reads under 10ms");
    assert_eq!(loaded.workpackage.file_scope, original.file_scope);
    assert_eq!(loaded.workpackage.created_at, original.created_at);
}

#[tokio::test]
async fn get_workpackage_returns_none_when_not_cached() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    assert!(cache.get_workpackage(1).await.expect("get").is_none());
}

#[tokio::test]
async fn workpackage_projections_use_the_store_default_ttl() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.set_workpackage(&work_package(4)).await.expect("set");

    assert_eq!(
        server.setex_ttls(),
        vec![("wp:4".to_string(), DEFAULT_TTL_SECS as i64)]
    );
}

#[tokio::test]
async fn invalidate_workpackage_deletes_the_key() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_workpackage(&work_package(6)).await.expect("set");

    cache.invalidate_workpackage(6).await.expect("invalidate");

    assert_eq!(server.stored("wp:6"), None);
    assert!(cache.get_workpackage(6).await.expect("get").is_none());
}

// ===========================================================================
// Key isolation
// ===========================================================================

#[tokio::test]
async fn feature_and_workpackage_keys_coexist_for_the_same_id() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    cache.set_feature(&feature(1)).await.expect("set feature");
    cache
        .set_workpackage(&work_package(1))
        .await
        .expect("set wp");

    assert_eq!(
        server.stored_keys(),
        vec!["feature:1".to_string(), "wp:1".to_string()]
    );
    assert!(cache.get_feature(1).await.expect("feature").is_some());
    assert!(cache.get_workpackage(1).await.expect("wp").is_some());
}

#[tokio::test]
async fn invalidating_a_feature_leaves_its_workpackage_cached() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_feature(&feature(1)).await.expect("set feature");
    cache
        .set_workpackage(&work_package(1))
        .await
        .expect("set wp");

    cache.invalidate_feature(1).await.expect("invalidate");

    assert!(cache.get_feature(1).await.expect("feature").is_none());
    assert!(cache.get_workpackage(1).await.expect("wp").is_some());
}

#[tokio::test]
async fn negative_and_large_ids_have_distinct_keys() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache.set_feature(&feature(-1)).await.expect("negative");
    cache.set_feature(&feature(i64::MAX)).await.expect("large");

    assert_eq!(
        server.stored_keys(),
        vec!["feature:-1".to_string(), format!("feature:{}", i64::MAX)]
    );
    assert!(cache.get_feature(-1).await.expect("negative").is_some());
    assert!(cache.get_feature(i64::MAX).await.expect("large").is_some());
}

// ===========================================================================
// Error propagation
// ===========================================================================

#[tokio::test]
async fn get_feature_reports_cache_error_for_non_json_payload() {
    let server = MockRedisServer::start().await;
    server.seed("feature:2", "not-json");
    let cache = cache_with(&server).await;

    let error = cache.get_feature(2).await.expect_err("must fail");
    assert!(
        matches!(error, ProjectionError::CacheError(_)),
        "got {error:?}"
    );
    assert!(error.to_string().contains("Cache error"));
}

#[tokio::test]
async fn get_feature_reports_cache_error_for_a_workpackage_payload() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    cache
        .set_workpackage(&work_package(2))
        .await
        .expect("set wp");
    // Move the work-package payload under the feature key.
    let payload = server.stored("wp:2").expect("payload");
    server.seed("feature:2", &payload);

    assert!(matches!(
        cache.get_feature(2).await,
        Err(ProjectionError::CacheError(_))
    ));
}

#[tokio::test]
async fn set_feature_reports_cache_error_when_backend_rejects_write() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    server.fail_command("SETEX", "OOM command not allowed");

    let error = cache.set_feature(&feature(8)).await.expect_err("must fail");
    assert!(matches!(error, ProjectionError::CacheError(_)));
    assert!(error.to_string().contains("OOM command not allowed"));
    assert!(server.stored_keys().is_empty());
}

#[tokio::test]
async fn get_feature_reports_cache_error_when_backend_fails() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    server.fail_command("GET", "READONLY");

    let error = cache.get_feature(8).await.expect_err("must fail");
    assert!(error.to_string().contains("READONLY"), "got {error}");
}

#[tokio::test]
async fn invalidate_reports_cache_error_when_backend_fails() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;
    server.fail_command("DEL", "NOPROTO");

    assert!(matches!(
        cache.invalidate_feature(8).await,
        Err(ProjectionError::CacheError(_))
    ));
    assert!(matches!(
        cache.invalidate_workpackage(8).await,
        Err(ProjectionError::CacheError(_))
    ));
}

// ===========================================================================
// Volume
// ===========================================================================

#[tokio::test]
async fn many_projections_are_addressable_independently() {
    let server = MockRedisServer::start().await;
    let cache = cache_with(&server).await;

    for id in 0..25 {
        cache.set_feature(&feature(id)).await.expect("set");
    }

    for id in 0..25 {
        let loaded = cache.get_feature(id).await.expect("get").expect("present");
        assert_eq!(loaded.feature.id, id, "key feature:{id} must be distinct");
    }
    assert_eq!(server.stored_keys().len(), 25);
}

// ===========================================================================
// Connection failures
// ===========================================================================

#[tokio::test]
async fn every_operation_reports_a_cache_error_when_the_backend_is_unreachable() {
    // Port 0 is never connectable and bb8 builds lazily, so the cache is
    // constructible; the concurrent calls keep this to a single timeout budget.
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 1;
    let pool = CachePool::new(&config).await.expect("lazy build");
    let cache = ProjectionCache::new(Arc::new(RedisCacheStore::new(pool, 3600)));

    let feature = feature(1);
    let work_package = work_package(1);
    // One join keeps the whole check inside a single connection-timeout budget.
    let (
        get_feature,
        set_feature,
        invalidate_feature,
        get_workpackage,
        set_workpackage,
        invalidate_workpackage,
    ) = tokio::join!(
        cache.get_feature(1),
        cache.set_feature(&feature),
        cache.invalidate_feature(1),
        cache.get_workpackage(1),
        cache.set_workpackage(&work_package),
        cache.invalidate_workpackage(1),
    );

    for error in [
        get_feature.expect_err("get_feature must fail"),
        set_feature.expect_err("set_feature must fail"),
        invalidate_feature.expect_err("invalidate_feature must fail"),
        get_workpackage.expect_err("get_workpackage must fail"),
        set_workpackage.expect_err("set_workpackage must fail"),
        invalidate_workpackage.expect_err("invalidate_workpackage must fail"),
    ] {
        assert!(
            matches!(error, ProjectionError::CacheError(_)),
            "got {error:?}"
        );
    }
}
