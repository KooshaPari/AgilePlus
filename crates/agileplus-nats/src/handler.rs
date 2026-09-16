//! Subscription handler trait for processing incoming messages.

use async_trait::async_trait;

use crate::bus::EventBusError;
use crate::envelope::Envelope;

/// A handler that processes incoming envelopes on a subscribed subject.
#[async_trait]
pub trait Handler: Send + Sync {
    /// Process one envelope. Return `Ok(())` to acknowledge, or an error to
    /// signal processing failure (the bus may redeliver depending on config).
    async fn handle(&self, envelope: &Envelope) -> Result<(), EventBusError>;
}

/// A simple handler that delegates to a closure. Useful for tests.
pub struct FnHandler<F>(pub F)
where
    F: Fn(&Envelope) -> Result<(), EventBusError> + Send + Sync;

#[async_trait]
impl<F> Handler for FnHandler<F>
where
    F: Fn(&Envelope) -> Result<(), EventBusError> + Send + Sync,
{
    async fn handle(&self, envelope: &Envelope) -> Result<(), EventBusError> {
        (self.0)(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::subject::Subject;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn fn_handler_success() {
        let handler = FnHandler(|_env: &Envelope| Ok(()));
        let env = Envelope::new(
            &Subject::new("agileplus.test.1.created"),
            serde_json::json!({"key": "value"}),
        );
        assert!(handler.handle(&env).await.is_ok());
    }

    #[tokio::test]
    async fn fn_handler_error() {
        let handler = FnHandler(|_env: &Envelope| {
            Err(EventBusError::HandlerError("boom".into()))
        });
        let env = Envelope::new(
            &Subject::new("agileplus.test.1.created"),
            serde_json::json!({}),
        );
        let result = handler.handle(&env).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            EventBusError::HandlerError(msg) => assert_eq!(msg, "boom"),
            other => panic!("expected HandlerError, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn fn_handler_receives_correct_envelope() {
        let received_subject: Arc<Mutex<Option<String>>> =
            Arc::new(Mutex::new(None));
        let clone = received_subject.clone();
        let handler = FnHandler(move |env: &Envelope| {
            *clone.lock().unwrap() = Some(env.subject.clone());
            Ok(())
        });

        let env = Envelope::new(
            &Subject::new("agileplus.feature.42.state_transitioned"),
            serde_json::json!({}),
        );
        handler.handle(&env).await.unwrap();
        assert_eq!(
            received_subject.lock().unwrap().as_deref(),
            Some("agileplus.feature.42.state_transitioned")
        );
    }

    #[tokio::test]
    async fn fn_handler_preserves_payload() {
        let payload_val: Arc<Mutex<Option<serde_json::Value>>> =
            Arc::new(Mutex::new(None));
        let clone = payload_val.clone();
        let handler = FnHandler(move |env: &Envelope| {
            *clone.lock().unwrap() = Some(env.payload.clone());
            Ok(())
        });

        let env = Envelope::new(
            &Subject::new("test"),
            serde_json::json!({"nested": {"a": 1}}),
        );
        handler.handle(&env).await.unwrap();
        let val = payload_val.lock().unwrap();
        assert_eq!(val.as_ref().unwrap()["nested"]["a"], 1);
    }
}
