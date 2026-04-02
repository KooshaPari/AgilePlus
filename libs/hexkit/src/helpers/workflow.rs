//! Workflow helper functions

use std::future::Future;
use std::time::Duration;

use crate::error::{HexkitError, HexkitResult};

pub async fn with_timeout<F, T>(future: F, duration: Duration) -> HexkitResult<T>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| HexkitError::Timeout(duration.as_secs()))
}

pub async fn retry_on_failure<F, Fut, T>(
    mut operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> HexkitResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(_) if attempts >= max_retries => return Err(HexkitError::RetryExhausted(attempts)),
            Err(_) => {
                tokio::time::sleep(base_delay * 2u32.pow(attempts - 1)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn retry_success() {
        let mut attempts = 0;
        let result = retry_on_failure(
            || {
                attempts += 1;
                async move {
                    if attempts < 2 {
                        Err("fail".into())
                    } else {
                        Ok(42)
                    }
                }
            },
            3,
            Duration::from_millis(1),
        )
        .await
        .unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn timeout_failure() {
        let res = with_timeout(
            tokio::time::sleep(Duration::from_millis(50)),
            Duration::from_millis(10),
        )
        .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn retry_failure() {
        let res = retry_on_failure(
            || async { Err::<i32, Box<dyn std::error::Error>>("always fail".into()) },
            2,
            Duration::from_millis(1),
        )
        .await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn with_timeout_success() {
        let res = with_timeout(async { 42 }, Duration::from_millis(100)).await;
        assert_eq!(res.unwrap(), 42);
    }
}
