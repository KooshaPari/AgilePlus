//! Analytics tracking infrastructure

use crate::{error::Result, event::Event};

/// Analytics tracker trait
pub trait Tracker: Send + Sync {
    /// Track an event
    fn track(&self, event: Event) -> Result<()>;
}

/// No-op tracker that discards all events
#[derive(Debug, Default, Clone)]
pub struct NoOpTracker;

impl Tracker for NoOpTracker {
    fn track(&self, _event: Event) -> Result<()> {
        Ok(())
    }
}
