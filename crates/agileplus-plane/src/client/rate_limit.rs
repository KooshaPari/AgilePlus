use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    pub fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

#[cfg(test)]
mod extra_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn fresh_bucket_allows_max_tokens() {
        let mut bucket = TokenBucket::new(4.0, 0.0);
        for _ in 0..4 {
            assert!(bucket.try_acquire());
        }
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn time_until_available_is_zero_when_token_present() {
        let bucket = TokenBucket::new(1.0, 1.0);
        assert_eq!(bucket.time_until_available(), Duration::ZERO);
    }

    #[test]
    fn time_until_available_grows_as_tokens_drain() {
        let mut bucket = TokenBucket::new(2.0, 1.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        let wait = bucket.time_until_available();
        assert!(wait >= Duration::from_millis(900), "wait={wait:?}");
    }

    #[test]
    fn refill_never_exceeds_capacity() {
        // 200 tokens/s refills well past the capacity of 2 during the sleep,
        // so the bucket must saturate. The rate is deliberately slow enough
        // (1 token per 5ms) that the three acquires below cannot span long
        // enough to earn a token back, which keeps the exhaustion check
        // deterministic instead of racing the refill.
        let mut bucket = TokenBucket::new(2.0, 200.0);
        assert!(bucket.try_acquire());
        std::thread::sleep(Duration::from_millis(50));
        // A refill must still cap at max_tokens (2).
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn zero_refill_rate_stays_exhausted() {
        let mut bucket = TokenBucket::new(1.0, 0.0);
        assert!(bucket.try_acquire());
        std::thread::sleep(Duration::from_millis(5));
        assert!(!bucket.try_acquire());
    }

    #[test]
    fn debug_does_not_panic() {
        let bucket = TokenBucket::new(1.0, 1.0);
        assert!(!format!("{bucket:?}").is_empty());
    }
}
