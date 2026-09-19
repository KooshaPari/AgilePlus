//! `CachePool` and `CacheHealthChecker` behaviour.
//!
//! Exercised against the in-process RESP double (`tests/common/redis.rs`), so
//! no external Dragonfly/Redis instance is required.

mod common;

use agileplus_cache::config::CacheConfig;
use agileplus_cache::health::{CacheHealth, CacheHealthChecker};
use agileplus_cache::pool::{CachePool, PoolError};
use common::redis::{MockRedisServer, PingBehaviour};

/// bb8 validates connections on checkout and retries for up to
/// `connection_timeout` before giving up, so a short budget keeps the
/// tests that deliberately make the double unhealthy quick. The budget does
/// not change any observed outcome.
fn config_for(server: &MockRedisServer) -> CacheConfig {
    let mut config = CacheConfig::new(server.host(), server.port()).with_pool_size(2);
    config.connection_timeout_secs = 1;
    config
}

// ===========================================================================
// Construction
// ===========================================================================

#[tokio::test]
async fn pool_build_is_lazy_and_needs_no_server() {
    // Port 0 never accepts connections, yet construction must still succeed:
    // bb8 only connects on the first checkout.
    let config = CacheConfig::new("127.0.0.1".to_string(), 0);
    let pool = CachePool::new(&config).await.expect("lazy build");

    assert_eq!(pool.raw_pool().state().connections, 0);
    assert_eq!(pool.raw_pool().state().idle_connections, 0);
}

#[tokio::test]
async fn pool_applies_configured_pool_size_and_timeout() {
    let config = CacheConfig::new("127.0.0.1".to_string(), 0).with_pool_size(7);
    let pool = CachePool::new(&config).await.expect("pool");

    let applied = pool.raw_pool().config();
    assert_eq!(applied.max_size, 7);
    assert_eq!(applied.connection_timeout.as_secs(), 5);
}

#[tokio::test]
async fn pool_applies_configured_connection_timeout() {
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 3;
    let pool = CachePool::new(&config).await.expect("pool");

    assert_eq!(pool.raw_pool().config().connection_timeout.as_secs(), 3);
}

#[tokio::test]
async fn pool_rejects_unparseable_host() {
    let config = CacheConfig::new("bad host name".to_string(), 6379);
    let error = CachePool::new(&config)
        .await
        .err()
        .expect("whitespace in host must not parse");

    match error {
        PoolError::ConnectionError(message) => {
            assert!(!message.is_empty(), "reason must be preserved");
        }
        other => panic!("expected ConnectionError, got {other:?}"),
    }
}

#[tokio::test]
async fn pool_rejects_empty_host() {
    let config = CacheConfig::new(String::new(), 6379);
    assert!(matches!(
        CachePool::new(&config).await,
        Err(PoolError::ConnectionError(_))
    ));
}

// ===========================================================================
// get_connection
// ===========================================================================

#[tokio::test]
async fn first_checkout_opens_exactly_one_connection() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");
    assert_eq!(server.connections_accepted(), 0, "build must not connect");

    let mut connection = pool.get_connection().await.expect("pooled connection");
    let pong: String = redis::cmd("PING")
        .query_async(&mut *connection)
        .await
        .expect("PING");
    assert_eq!(pong, "PONG");

    assert_eq!(server.connections_accepted(), 1);
    assert_eq!(pool.raw_pool().state().connections, 1);
}

#[tokio::test]
async fn sequential_checkouts_reuse_the_pooled_connection() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");

    for _ in 0..4 {
        let mut connection = pool.get_connection().await.expect("connection");
        let pong: String = redis::cmd("PING")
            .query_async(&mut *connection)
            .await
            .expect("PING");
        assert_eq!(pong, "PONG");
    }

    assert_eq!(
        server.connections_accepted(),
        1,
        "pool_size 2 with sequential use must not open more connections"
    );
}

#[tokio::test]
async fn returned_connection_becomes_idle() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");

    let connection = pool.get_connection().await.expect("connection");
    assert_eq!(pool.raw_pool().state().idle_connections, 0);
    drop(connection);

    assert_eq!(pool.raw_pool().state().idle_connections, 1);
}

#[tokio::test]
async fn checkout_failure_is_reported_as_timeout() {
    // Port 0 can never be connected to, and a 1s budget keeps the test short
    // without asserting on wall-clock behaviour.
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 1;
    let pool = CachePool::new(&config).await.expect("lazy build");

    match pool.get_connection().await {
        Err(PoolError::Timeout(message)) => {
            assert!(!message.is_empty(), "bb8 reason must be preserved");
        }
        Err(other) => panic!("expected Timeout, got {other:?}"),
        Ok(_) => panic!("port 0 must not yield a connection"),
    }
}

// ===========================================================================
// Health checks
// ===========================================================================

#[tokio::test]
async fn health_check_pings_and_reports_healthy() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");

    assert_eq!(
        CacheHealthChecker::new(pool).check().await,
        CacheHealth::Healthy
    );
    assert!(server.count_commands("PING") >= 1, "health check must PING");
}

#[tokio::test]
async fn health_check_reports_unavailable_for_unexpected_payload() {
    let server = MockRedisServer::start().await;
    server.set_ping_behaviour(PingBehaviour::UnexpectedPayload);
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");

    assert_eq!(
        CacheHealthChecker::new(pool).check().await,
        CacheHealth::Unavailable
    );
}

#[tokio::test]
async fn health_check_reports_unavailable_when_ping_errors() {
    let server = MockRedisServer::start().await;
    server.fail_command("PING", "LOADING Redis is loading the dataset in memory");
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");

    assert_eq!(
        CacheHealthChecker::new(pool).check().await,
        CacheHealth::Unavailable
    );
}

#[tokio::test]
async fn health_check_recovers_once_the_backend_does() {
    let server = MockRedisServer::start().await;
    let pool = CachePool::new(&config_for(&server)).await.expect("pool");
    let checker = CacheHealthChecker::new(pool);

    server.fail_command("PING", "transient");
    assert_eq!(checker.check().await, CacheHealth::Unavailable);
    server.clear_failure();
    assert_eq!(checker.check().await, CacheHealth::Healthy);
}

#[tokio::test]
async fn health_check_reports_unavailable_for_unreachable_backend() {
    let mut config = CacheConfig::new("127.0.0.1".to_string(), 0);
    config.connection_timeout_secs = 1;
    let pool = CachePool::new(&config).await.expect("lazy build");

    assert_eq!(
        CacheHealthChecker::new(pool).check().await,
        CacheHealth::Unavailable
    );
}
