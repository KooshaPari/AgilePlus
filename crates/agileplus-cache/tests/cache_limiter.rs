//! `RateLimiter` sliding-window behaviour, driven by the in-process RESP double
//! (`tests/common/redis.rs`) so the real adapter is exercised without a live
//! Dragonfly/Redis server.

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::limiter::{LimiterError, RateLimiter};
use agileplus_cache::pool::CachePool;
use common::redis::MockRedisServer;
use std::sync::Arc;

async fn limiter_with(server: &MockRedisServer) -> RateLimiter {
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(8))
        .await
        .expect("pool");
    RateLimiter::new(pool)
}

// ===========================================================================
// Sliding window
// ===========================================================================

#[tokio::test]
async fn first_request_is_allowed_and_opens_the_window() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    assert!(limiter.is_allowed("user:1", 5, 60).await.expect("allowed"));
    assert_eq!(server.count_commands("INCRBY"), 1);
    assert_eq!(server.count_commands("EXPIRE"), 1);
    assert_eq!(
        server.first_command("EXPIRE").expect("EXPIRE").args,
        vec!["ratelimit:user:1".to_string(), "60".to_string()]
    );
}

#[tokio::test]
async fn requests_are_allowed_up_to_the_limit_then_denied() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    for expected_call in 1..=3 {
        assert!(
            limiter.is_allowed("user:1", 3, 60).await.expect("allowed"),
            "call {expected_call} of 3 must be allowed"
        );
    }
    assert!(
        !limiter.is_allowed("user:1", 3, 60).await.expect("denied"),
        "call 4 must exceed the limit"
    );
}

#[tokio::test]
async fn limit_of_one_allows_exactly_one_request() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    assert!(limiter.is_allowed("k", 1, 60).await.expect("first"));
    assert!(!limiter.is_allowed("k", 1, 60).await.expect("second"));
}

#[tokio::test]
async fn window_is_set_only_on_the_first_request() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    for _ in 0..5 {
        let _ = limiter.is_allowed("user:1", 10, 30).await;
    }

    assert_eq!(server.count_commands("INCRBY"), 5);
    assert_eq!(
        server.count_commands("EXPIRE"),
        1,
        "later requests must not extend the window"
    );
}

#[tokio::test]
async fn window_seconds_are_forwarded() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    limiter.is_allowed("k", 5, 45).await.expect("allowed");

    assert_eq!(
        server.first_command("EXPIRE").expect("EXPIRE").args[1],
        "45"
    );
}

#[tokio::test]
async fn keys_are_namespaced_and_independent() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    assert_eq!(
        server.count_commands("INCRBY"),
        0,
        "no counter exists before use"
    );
    assert!(limiter.is_allowed("alice", 1, 60).await.expect("alice"));
    assert!(
        limiter.is_allowed("bob", 1, 60).await.expect("bob"),
        "bob has his own window"
    );
    assert!(
        !limiter
            .is_allowed("alice", 1, 60)
            .await
            .expect("alice again"),
        "alice is already at her limit"
    );

    let incremented: Vec<String> = server
        .received()
        .into_iter()
        .filter(|command| command.name == "INCRBY")
        .map(|command| command.args[0].clone())
        .collect();
    assert_eq!(
        incremented,
        vec![
            "ratelimit:alice".to_string(),
            "ratelimit:bob".to_string(),
            "ratelimit:alice".to_string()
        ]
    );
}

#[tokio::test]
async fn separate_limiter_instances_share_the_same_counter() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(4))
        .await
        .expect("pool");
    let first = RateLimiter::new(pool);
    let second = RateLimiter::new(
        CachePool::new(&CacheConfig::new(server.host(), server.port()).with_pool_size(4))
            .await
            .expect("pool"),
    );

    assert!(first.is_allowed("shared", 1, 60).await.expect("first"));
    assert!(
        !second.is_allowed("shared", 1, 60).await.expect("second"),
        "the window lives in the cache, not in the limiter struct"
    );
}

// ===========================================================================
// Remaining budget
// ===========================================================================

#[tokio::test]
async fn remaining_is_full_budget_before_any_request() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    assert_eq!(
        limiter.get_remaining("fresh", 5).await.expect("remaining"),
        5
    );
}

#[tokio::test]
async fn remaining_decreases_with_each_allowed_request() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    for _ in 0..2 {
        limiter.is_allowed("k", 5, 60).await.expect("allowed");
    }

    assert_eq!(limiter.get_remaining("k", 5).await.expect("remaining"), 3);
}

#[tokio::test]
async fn remaining_saturates_at_zero_when_over_budget() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    for _ in 0..5 {
        limiter.is_allowed("k", 2, 60).await.expect("counted");
    }

    assert_eq!(limiter.get_remaining("k", 2).await.expect("remaining"), 0);
}

#[tokio::test]
async fn reset_clears_the_window() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;
    limiter.is_allowed("k", 1, 60).await.expect("first");
    assert!(!limiter.is_allowed("k", 1, 60).await.expect("denied"));

    limiter.reset("k").await.expect("reset");

    assert_eq!(server.count_commands("DEL"), 1);
    assert_eq!(
        server.first_command("DEL").expect("DEL").args,
        vec!["ratelimit:k".to_string()]
    );
    assert!(limiter.is_allowed("k", 1, 60).await.expect("allowed again"));
}

#[tokio::test]
async fn reset_of_untouched_key_succeeds() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;

    limiter
        .reset("never-used")
        .await
        .expect("reset is idempotent");
}

// ===========================================================================
// Error propagation
// ===========================================================================

#[tokio::test]
async fn incrby_failure_surfaces_as_limiter_error() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;
    server.fail_command("INCRBY", "MISCONF Redis is configured to save RDB");

    let error = limiter.is_allowed("k", 1, 60).await.expect_err("must fail");
    assert!(matches!(error, LimiterError::Error(_)), "got {error:?}");
    assert!(error.to_string().contains("MISCONF"));
}

#[tokio::test]
async fn expire_failure_surfaces_as_limiter_error() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;
    server.fail_command("EXPIRE", "noexpire");

    let error = limiter.is_allowed("k", 1, 60).await.expect_err("must fail");
    assert!(error.to_string().contains("noexpire"), "got {error}");
}

#[tokio::test]
async fn get_remaining_surfaces_server_error() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;
    server.fail_command("GET", "READONLY");

    let error = limiter.get_remaining("k", 5).await.expect_err("must fail");
    assert!(error.to_string().contains("READONLY"), "got {error}");
}

#[tokio::test]
async fn reset_surfaces_server_error() {
    let server = MockRedisServer::start().await;
    let limiter = limiter_with(&server).await;
    server.fail_command("DEL", "NOPROTO");

    let error = limiter.reset("k").await.expect_err("must fail");
    assert!(error.to_string().contains("NOPROTO"), "got {error}");
}

// ===========================================================================
// Concurrency
// ===========================================================================

#[tokio::test]
async fn concurrent_requests_admit_exactly_the_budget() {
    let server = MockRedisServer::start().await;
    let limiter = Arc::new(limiter_with(&server).await);

    let mut handles = Vec::new();
    for _ in 0..40 {
        let limiter = Arc::clone(&limiter);
        handles.push(tokio::spawn(async move {
            limiter.is_allowed("burst", 10, 60).await.expect("counted")
        }));
    }

    let mut allowed = 0;
    for handle in handles {
        if handle.await.expect("join") {
            allowed += 1;
        }
    }

    // INCR is atomic server-side, so exactly `max_requests` callers may win.
    assert_eq!(allowed, 10, "exactly the budget must be admitted");
    assert_eq!(server.count_commands("INCRBY"), 40);
    assert_eq!(server.count_commands("EXPIRE"), 1);
}

#[tokio::test]
async fn concurrent_remaining_reads_never_exceed_budget() {
    let server = MockRedisServer::start().await;
    let limiter = Arc::new(limiter_with(&server).await);

    let mut handles = Vec::new();
    for _ in 0..20 {
        let limiter = Arc::clone(&limiter);
        handles.push(tokio::spawn(async move {
            limiter.get_remaining("quiet", 5).await.expect("remaining")
        }));
    }

    for handle in handles {
        assert_eq!(handle.await.expect("join"), 5);
    }
}

// ===========================================================================
// Connection failures
// ===========================================================================

/// A pool that parses but can never connect: port 0 is not a valid target, and
/// bb8 builds lazily, so the limiter is constructible and only fails on use.
async fn unreachable_limiter() -> RateLimiter {
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 1;
    let pool = CachePool::new(&config).await.expect("lazy build");
    RateLimiter::new(pool)
}

#[tokio::test]
async fn is_allowed_reports_error_when_the_backend_is_unreachable() {
    let limiter = unreachable_limiter().await;

    let error = limiter.is_allowed("k", 1, 60).await.expect_err("must fail");
    assert!(matches!(error, LimiterError::Error(_)), "got {error:?}");
    assert!(!error.to_string().is_empty());
}

#[tokio::test]
async fn get_remaining_reports_error_when_the_backend_is_unreachable() {
    let limiter = unreachable_limiter().await;

    assert!(matches!(
        limiter.get_remaining("k", 5).await,
        Err(LimiterError::Error(_))
    ));
}

#[tokio::test]
async fn reset_reports_error_when_the_backend_is_unreachable() {
    let limiter = unreachable_limiter().await;

    assert!(matches!(
        limiter.reset("k").await,
        Err(LimiterError::Error(_))
    ));
}
