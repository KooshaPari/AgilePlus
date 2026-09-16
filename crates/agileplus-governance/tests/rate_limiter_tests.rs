//! Integration tests for RateLimiter token bucket logic.
//! Complements the inline unit tests in src/rate_limiter.rs.

use agileplus_governance::*;
use agileplus_governance::rate_limiter::{RateLimitConfig, RateLimitKey, RateLimitResult};
use std::time::Duration;

#[test]
fn rate_limit_config_default() {
    let cfg = RateLimitConfig::default();
    assert_eq!(cfg.max_requests, 100);
    assert_eq!(cfg.window, Duration::from_secs(3600));
}

#[test]
fn rate_limit_key_anonymous() {
    let key = RateLimitKey::anonymous(Some("10.0.0.1".into()), "promote");
    assert!(key.user_id.is_none());
    assert_eq!(key.client_ip.as_deref(), Some("10.0.0.1"));
    assert_eq!(key.action, "promote");
}

#[test]
fn rate_limit_key_new_with_all_fields() {
    let key = RateLimitKey::new(
        Some("user-1".into()),
        Some("127.0.0.1".into()),
        "deploy",
    );
    assert_eq!(key.user_id.as_deref(), Some("user-1"));
    assert_eq!(key.client_ip.as_deref(), Some("127.0.0.1"));
    assert_eq!(key.action, "deploy");
}

#[test]
fn rate_limit_key_equality() {
    let k1 = RateLimitKey::new(Some("u".into()), Some("ip".into()), "act");
    let k2 = RateLimitKey::new(Some("u".into()), Some("ip".into()), "act");
    assert_eq!(k1, k2);
}

#[test]
fn rate_limit_key_different_action_not_equal() {
    let k1 = RateLimitKey::anonymous(None, "action1");
    let k2 = RateLimitKey::anonymous(None, "action2");
    assert_ne!(k1, k2);
}

#[test]
fn rate_limit_result_allowed_factory() {
    let result = RateLimitResult::allowed(99, std::time::Instant::now() + Duration::from_secs(60));
    assert!(result.allowed);
    assert_eq!(result.remaining, 99);
    assert!(result.retry_after.is_none());
}

#[test]
fn rate_limit_result_denied_factory() {
    let reset = std::time::Instant::now() + Duration::from_secs(60);
    let result = RateLimitResult::denied(0, reset, Duration::from_secs(5));
    assert!(!result.allowed);
    assert_eq!(result.remaining, 0);
    assert!(result.retry_after.is_some());
    assert_eq!(result.retry_after.unwrap(), Duration::from_secs(5));
}

#[tokio::test]
async fn rate_limiter_allows_up_to_max() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 5,
        window: Duration::from_secs(60),
    });
    let key = RateLimitKey::anonymous(Some("127.0.0.1".into()), "test");
    for _ in 0..5 {
        let result = limiter.check(&key).await;
        assert!(result.allowed);
    }
    // 6th should be denied
    let result = limiter.check(&key).await;
    assert!(!result.allowed);
}

#[tokio::test]
async fn rate_limiter_different_keys_independent() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 1,
        window: Duration::from_secs(60),
    });
    let k1 = RateLimitKey::anonymous(Some("10.0.0.1".into()), "promote");
    let k2 = RateLimitKey::anonymous(Some("10.0.0.2".into()), "promote");

    let r1 = limiter.check(&k1).await;
    assert!(r1.allowed);
    let r2 = limiter.check(&k2).await;
    assert!(r2.allowed);
    // Second request to k1 should be denied
    let r1b = limiter.check(&k1).await;
    assert!(!r1b.allowed);
}

#[tokio::test]
async fn rate_limiter_different_actions_independent() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 1,
        window: Duration::from_secs(60),
    });
    let k1 = RateLimitKey::anonymous(None, "promote");
    let k2 = RateLimitKey::anonymous(None, "deploy");

    let r1 = limiter.check(&k1).await;
    assert!(r1.allowed);
    let r2 = limiter.check(&k2).await;
    assert!(r2.allowed);
    let r1b = limiter.check(&k1).await;
    assert!(!r1b.allowed);
}

#[tokio::test]
async fn rate_limiter_peek_does_not_consume() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 2,
        window: Duration::from_secs(60),
    });
    let key = RateLimitKey::anonymous(None, "peek_test");

    // Peek should show 2 remaining
    let peek = limiter.peek(&key).await;
    assert!(peek.allowed);
    assert_eq!(peek.remaining, 2);

    // Check twice should exhaust
    let _ = limiter.check(&key).await;
    let _ = limiter.check(&key).await;

    // Peek should show 0 remaining
    let peek = limiter.peek(&key).await;
    assert_eq!(peek.remaining, 0);
}

#[tokio::test]
async fn rate_limiter_peek_unknown_key_shows_max() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 10,
        window: Duration::from_secs(60),
    });
    let key = RateLimitKey::anonymous(None, "unknown");
    let peek = limiter.peek(&key).await;
    assert!(peek.allowed);
    assert_eq!(peek.remaining, 10);
}

#[tokio::test]
async fn rate_limiter_reset_removes_entry() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 1,
        window: Duration::from_secs(60),
    });
    let key = RateLimitKey::anonymous(None, "reset_test");

    let _ = limiter.check(&key).await;
    assert!(!limiter.check(&key).await.allowed);

    limiter.reset(&key).await;

    // After reset, should be allowed again
    let result = limiter.check(&key).await;
    assert!(result.allowed);
}

#[tokio::test]
async fn rate_limiter_reset_all_clears_everything() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 1,
        window: Duration::from_secs(60),
    });
    let k1 = RateLimitKey::anonymous(None, "a");
    let k2 = RateLimitKey::anonymous(None, "b");

    let _ = limiter.check(&k1).await;
    let _ = limiter.check(&k2).await;
    assert_eq!(limiter.len().await, 2);

    limiter.reset_all().await;
    assert_eq!(limiter.len().await, 0);
    assert!(limiter.is_empty().await);
}

#[tokio::test]
async fn rate_limiter_len_tracks_entries() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 100,
        window: Duration::from_secs(60),
    });
    assert_eq!(limiter.len().await, 0);

    let k1 = RateLimitKey::anonymous(None, "x");
    let _ = limiter.check(&k1).await;
    assert_eq!(limiter.len().await, 1);

    let k2 = RateLimitKey::anonymous(None, "y");
    let _ = limiter.check(&k2).await;
    assert_eq!(limiter.len().await, 2);
}

#[tokio::test]
async fn rate_limiter_is_empty_initially() {
    let limiter = RateLimiter::default_limiter();
    assert!(limiter.is_empty().await);
}

#[tokio::test]
async fn rate_limiter_default_limiter_uses_defaults() {
    let limiter = RateLimiter::default_limiter();
    let key = RateLimitKey::anonymous(None, "test");
    let result = limiter.check(&key).await;
    assert!(result.allowed);
    // Default is 100 requests per hour, so we should have 99 remaining
    assert_eq!(result.remaining, 99);
}

#[tokio::test]
async fn rate_limiter_concurrent_access() {
    use std::sync::Arc;
    let limiter = Arc::new(RateLimiter::new(RateLimitConfig {
        max_requests: 10,
        window: Duration::from_secs(60),
    }));
    let key = RateLimitKey::anonymous(None, "concurrent");

    // Launch 10 concurrent checks
    let mut handles = Vec::new();
    for _ in 0..10 {
        let limiter = Arc::clone(&limiter);
        let key = key.clone();
        handles.push(tokio::spawn(async move { limiter.check(&key).await }));
    }

    let mut allowed_count = 0;
    for h in handles {
        let result = h.await.unwrap();
        if result.allowed {
            allowed_count += 1;
        }
    }
    assert_eq!(allowed_count, 10);

    // 11th should be denied
    let result = limiter.check(&key).await;
    assert!(!result.allowed);
}
