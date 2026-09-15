//! Agent event bus — tokio broadcast channel for real-time events.
//!
//! Traceability: WP14-T083

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Events published during agent execution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentEvent {
    AgentStarted {
        feature_slug: String,
        wp_sequence: i32,
        agent_id: String,
    },
    PrCreated {
        feature_slug: String,
        wp_sequence: i32,
        pr_url: String,
    },
    ReviewReceived {
        feature_slug: String,
        wp_sequence: i32,
        review_status: String,
        comments: usize,
    },
    AgentFixing {
        feature_slug: String,
        wp_sequence: i32,
        cycle: u32,
    },
    AgentCompleted {
        feature_slug: String,
        wp_sequence: i32,
        success: bool,
    },
    WpStateChanged {
        feature_slug: String,
        wp_sequence: i32,
        old_state: String,
        new_state: String,
    },
}

impl AgentEvent {
    /// Returns the feature slug this event belongs to.
    pub fn feature_slug(&self) -> &str {
        match self {
            AgentEvent::AgentStarted { feature_slug, .. } => feature_slug,
            AgentEvent::PrCreated { feature_slug, .. } => feature_slug,
            AgentEvent::ReviewReceived { feature_slug, .. } => feature_slug,
            AgentEvent::AgentFixing { feature_slug, .. } => feature_slug,
            AgentEvent::AgentCompleted { feature_slug, .. } => feature_slug,
            AgentEvent::WpStateChanged { feature_slug, .. } => feature_slug,
        }
    }

    /// Returns true if this event matches a given feature slug filter.
    /// Empty filter matches all events.
    pub fn matches_feature(&self, filter: &str) -> bool {
        filter.is_empty() || self.feature_slug() == filter
    }

    /// Returns the event_type string for the proto message.
    pub fn event_type(&self) -> &'static str {
        match self {
            AgentEvent::AgentStarted { .. } => "agent_started",
            AgentEvent::PrCreated { .. } => "pr_created",
            AgentEvent::ReviewReceived { .. } => "review_received",
            AgentEvent::AgentFixing { .. } => "agent_fixing",
            AgentEvent::AgentCompleted { .. } => "agent_completed",
            AgentEvent::WpStateChanged { .. } => "wp_state_changed",
        }
    }

    pub fn wp_sequence(&self) -> i32 {
        match self {
            AgentEvent::AgentStarted { wp_sequence, .. } => *wp_sequence,
            AgentEvent::PrCreated { wp_sequence, .. } => *wp_sequence,
            AgentEvent::ReviewReceived { wp_sequence, .. } => *wp_sequence,
            AgentEvent::AgentFixing { wp_sequence, .. } => *wp_sequence,
            AgentEvent::AgentCompleted { wp_sequence, .. } => *wp_sequence,
            AgentEvent::WpStateChanged { wp_sequence, .. } => *wp_sequence,
        }
    }

    pub fn agent_id(&self) -> &str {
        match self {
            AgentEvent::AgentStarted { agent_id, .. } => agent_id,
            _ => "",
        }
    }

    pub fn payload(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Shared event bus backed by a tokio broadcast channel.
///
/// Capacity defaults to 1024. When the channel is full the oldest events
/// are dropped to avoid blocking publishers.
#[derive(Clone, Debug)]
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event. If no receivers are connected the send is silently
    /// dropped.
    pub fn publish(&self, event: AgentEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to the event stream.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_subscribe_round_trip() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "feat-a".into(),
            wp_sequence: 1,
            agent_id: "agent-1".into(),
        });
        let event = rx.recv().await.unwrap();
        assert_eq!(event.feature_slug(), "feat-a");
        assert_eq!(event.event_type(), "agent_started");
    }

    #[test]
    fn matches_feature_filter() {
        let e = AgentEvent::AgentCompleted {
            feature_slug: "feat-x".into(),
            wp_sequence: 2,
            success: true,
        };
        assert!(e.matches_feature("feat-x"));
        assert!(e.matches_feature(""));
        assert!(!e.matches_feature("feat-y"));
    }

    #[test]
    fn agent_event_wp_sequence_accessor() {
        let events = [
            AgentEvent::AgentStarted { feature_slug: "f".into(), wp_sequence: 1, agent_id: "a".into() },
            AgentEvent::PrCreated { feature_slug: "f".into(), wp_sequence: 2, pr_url: "u".into() },
            AgentEvent::ReviewReceived { feature_slug: "f".into(), wp_sequence: 3, review_status: "ok".into(), comments: 0 },
            AgentEvent::AgentFixing { feature_slug: "f".into(), wp_sequence: 4, cycle: 1 },
            AgentEvent::AgentCompleted { feature_slug: "f".into(), wp_sequence: 5, success: true },
            AgentEvent::WpStateChanged { feature_slug: "f".into(), wp_sequence: 6, old_state: "a".into(), new_state: "b".into() },
        ];
        for (i, e) in events.iter().enumerate() {
            assert_eq!(e.wp_sequence(), (i + 1) as i32);
        }
    }

    #[test]
    fn agent_event_feature_slug_accessor() {
        let e = AgentEvent::PrCreated {
            feature_slug: "my-feat".into(),
            wp_sequence: 1,
            pr_url: "url".into(),
        };
        assert_eq!(e.feature_slug(), "my-feat");
    }

    #[test]
    fn agent_event_agent_id_accessor() {
        let e = AgentEvent::AgentStarted {
            feature_slug: "f".into(),
            wp_sequence: 1,
            agent_id: "agent-42".into(),
        };
        assert_eq!(e.agent_id(), "agent-42");

        let e2 = AgentEvent::PrCreated {
            feature_slug: "f".into(),
            wp_sequence: 1,
            pr_url: "u".into(),
        };
        assert_eq!(e2.agent_id(), "");
    }

    #[test]
    fn agent_event_event_type_strings() {
        let cases: Vec<(AgentEvent, &str)> = vec![
            (AgentEvent::AgentStarted { feature_slug: "f".into(), wp_sequence: 1, agent_id: "a".into() }, "agent_started"),
            (AgentEvent::PrCreated { feature_slug: "f".into(), wp_sequence: 1, pr_url: "u".into() }, "pr_created"),
            (AgentEvent::ReviewReceived { feature_slug: "f".into(), wp_sequence: 1, review_status: "s".into(), comments: 0 }, "review_received"),
            (AgentEvent::AgentFixing { feature_slug: "f".into(), wp_sequence: 1, cycle: 1 }, "agent_fixing"),
            (AgentEvent::AgentCompleted { feature_slug: "f".into(), wp_sequence: 1, success: true }, "agent_completed"),
            (AgentEvent::WpStateChanged { feature_slug: "f".into(), wp_sequence: 1, old_state: "a".into(), new_state: "b".into() }, "wp_state_changed"),
        ];
        for (event, expected_type) in &cases {
            assert_eq!(event.event_type(), *expected_type);
        }
    }

    #[test]
    fn agent_event_payload_is_valid_json() {
        let e = AgentEvent::AgentCompleted {
            feature_slug: "feat".into(),
            wp_sequence: 1,
            success: true,
        };
        let json = e.payload();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn event_bus_default_capacity() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe();
        bus.publish(AgentEvent::AgentStarted {
            feature_slug: "f".into(),
            wp_sequence: 1,
            agent_id: "a".into(),
        });
        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type(), "agent_started");
    }

    #[tokio::test]
    async fn event_bus_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        bus.publish(AgentEvent::PrCreated {
            feature_slug: "f".into(),
            wp_sequence: 1,
            pr_url: "u".into(),
        });
        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1.event_type(), "pr_created");
        assert_eq!(e2.event_type(), "pr_created");
    }
}
