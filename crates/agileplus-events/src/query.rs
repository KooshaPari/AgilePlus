//! Event query builder with fluent API.

use agileplus_domain::domain::event::Event;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Query error: {0}")]
    Error(String),
}

/// Fluent event query builder for in-memory filtering.
#[derive(Default)]
pub struct EventQuery {
    entity_type: Option<String>,
    entity_id: Option<i64>,
    event_type: Option<String>,
    actor: Option<String>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    from_sequence: Option<i64>,
    to_sequence: Option<i64>,
    limit: Option<usize>,
}

impl EventQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entity_type(mut self, et: impl Into<String>) -> Self {
        self.entity_type = Some(et.into());
        self
    }

    pub fn entity_id(mut self, id: i64) -> Self {
        self.entity_id = Some(id);
        self
    }

    pub fn event_type(mut self, et: impl Into<String>) -> Self {
        self.event_type = Some(et.into());
        self
    }

    pub fn actor(mut self, a: impl Into<String>) -> Self {
        self.actor = Some(a.into());
        self
    }

    pub fn start_time(mut self, t: DateTime<Utc>) -> Self {
        self.from_time = Some(t);
        self
    }

    pub fn end_time(mut self, t: DateTime<Utc>) -> Self {
        self.to_time = Some(t);
        self
    }

    pub fn after_sequence(mut self, s: i64) -> Self {
        self.from_sequence = Some(s);
        self
    }

    pub fn end_sequence(mut self, s: i64) -> Self {
        self.to_sequence = Some(s);
        self
    }

    pub fn limit(mut self, l: usize) -> Self {
        self.limit = Some(l);
        self
    }

    /// Filter an in-memory event list using this query's criteria.
    pub fn filter(&self, events: &[Event]) -> Vec<Event> {
        events
            .iter()
            .filter(|e| {
                if let Some(ref et) = self.entity_type
                    && &e.entity_type != et
                {
                    return false;
                }
                if let Some(id) = self.entity_id
                    && e.entity_id != id
                {
                    return false;
                }
                if let Some(ref et) = self.event_type
                    && &e.event_type != et
                {
                    return false;
                }
                if let Some(ref a) = self.actor
                    && &e.actor != a
                {
                    return false;
                }
                if let Some(from) = self.from_time
                    && e.timestamp < from
                {
                    return false;
                }
                if let Some(to) = self.to_time
                    && e.timestamp > to
                {
                    return false;
                }
                if let Some(from) = self.from_sequence
                    && e.sequence < from
                {
                    return false;
                }
                if let Some(to) = self.to_sequence
                    && e.sequence > to
                {
                    return false;
                }
                true
            })
            .take(self.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(seq: i64, entity_type: &str, event_type: &str, actor: &str) -> Event {
        Event {
            id: seq,
            entity_type: entity_type.into(),
            entity_id: 1,
            event_type: event_type.into(),
            payload: serde_json::json!({}),
            actor: actor.into(),
            timestamp: chrono::Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: seq,
        }
    }

    #[test]
    fn filter_by_entity_type() {
        let events = vec![
            make_event(1, "Feature", "created", "a"),
            make_event(2, "WP", "created", "a"),
        ];
        let result = EventQuery::new().entity_type("Feature").filter(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity_type, "Feature");
    }

    #[test]
    fn filter_by_actor() {
        let events = vec![
            make_event(1, "F", "c", "alice"),
            make_event(2, "F", "c", "bob"),
        ];
        let result = EventQuery::new().actor("bob").filter(&events);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn limit_works() {
        let events = vec![
            make_event(1, "F", "c", "a"),
            make_event(2, "F", "c", "a"),
            make_event(3, "F", "c", "a"),
        ];
        let result = EventQuery::new().limit(2).filter(&events);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn sequence_range() {
        let events = vec![
            make_event(1, "F", "c", "a"),
            make_event(2, "F", "c", "a"),
            make_event(3, "F", "c", "a"),
        ];
        let result = EventQuery::new()
            .after_sequence(2)
            .end_sequence(2)
            .filter(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sequence, 2);
    }
}

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use chrono::Duration;

    fn mk(
        seq: i64,
        entity_type: &str,
        entity_id: i64,
        event_type: &str,
        actor: &str,
        age_secs: i64,
    ) -> Event {
        Event {
            id: seq,
            entity_type: entity_type.into(),
            entity_id,
            event_type: event_type.into(),
            payload: serde_json::json!({}),
            actor: actor.into(),
            timestamp: Utc::now() - Duration::seconds(age_secs),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: seq,
        }
    }

    fn sample() -> Vec<Event> {
        vec![
            mk(1, "Feature", 1, "created", "alice", 300),
            mk(2, "Feature", 1, "updated", "bob", 200),
            mk(3, "Feature", 2, "created", "alice", 100),
            mk(4, "WorkPackage", 3, "created", "carol", 50),
        ]
    }

    #[test]
    fn empty_query_returns_all() {
        assert_eq!(EventQuery::new().filter(&sample()).len(), 4);
    }

    #[test]
    fn default_matches_new() {
        assert_eq!(EventQuery::default().filter(&sample()).len(), 4);
    }

    #[test]
    fn filter_by_entity_type() {
        let got = EventQuery::new().entity_type("Feature").filter(&sample());
        assert_eq!(got.len(), 3);
        assert!(got.iter().all(|e| e.entity_type == "Feature"));
    }

    #[test]
    fn filter_by_entity_id() {
        let got = EventQuery::new().entity_id(1).filter(&sample());
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|e| e.entity_id == 1));
    }

    #[test]
    fn filter_by_event_type() {
        let got = EventQuery::new().event_type("created").filter(&sample());
        assert_eq!(got.len(), 3);
    }

    #[test]
    fn filter_by_actor() {
        let got = EventQuery::new().actor("alice").filter(&sample());
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn filter_by_start_time() {
        let cutoff = Utc::now() - Duration::seconds(150);
        let got = EventQuery::new().start_time(cutoff).filter(&sample());
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn filter_by_end_time() {
        let cutoff = Utc::now() - Duration::seconds(150);
        let got = EventQuery::new().end_time(cutoff).filter(&sample());
        assert_eq!(got.len(), 2);
    }

    #[test]
    fn filter_by_after_sequence() {
        let got = EventQuery::new().after_sequence(2).filter(&sample());
        assert_eq!(got.len(), 3);
        assert!(got.iter().all(|e| e.sequence >= 2));
    }

    #[test]
    fn filter_by_end_sequence() {
        let got = EventQuery::new().end_sequence(2).filter(&sample());
        assert_eq!(got.len(), 2);
        assert!(got.iter().all(|e| e.sequence <= 2));
    }

    #[test]
    fn limit_zero_returns_empty() {
        assert!(EventQuery::new().limit(0).filter(&sample()).is_empty());
    }

    #[test]
    fn limit_truncates_results() {
        assert_eq!(EventQuery::new().limit(2).filter(&sample()).len(), 2);
    }

    #[test]
    fn combined_filters_narrow_results() {
        let got = EventQuery::new()
            .entity_type("Feature")
            .entity_id(1)
            .event_type("created")
            .filter(&sample());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].sequence, 1);
    }

    #[test]
    fn combined_entity_type_and_id() {
        let got = EventQuery::new().entity_type("Feature").entity_id(2).filter(&sample());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].actor, "alice");
    }

    #[test]
    fn no_match_returns_empty() {
        assert!(EventQuery::new().actor("nobody").filter(&sample()).is_empty());
    }

    #[test]
    fn empty_input_returns_empty() {
        assert!(EventQuery::new().filter(&[]).is_empty());
    }

    #[test]
    fn query_error_display() {
        assert_eq!(QueryError::Error("bad".into()).to_string(), "Query error: bad");
    }

    #[test]
    fn sequence_range_inclusive_bounds() {
        let got = EventQuery::new()
            .after_sequence(2)
            .end_sequence(3)
            .filter(&sample());
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].sequence, 2);
        assert_eq!(got[1].sequence, 3);
    }

    #[test]
    fn time_range_both_bounds() {
        let from = Utc::now() - Duration::seconds(250);
        let to = Utc::now() - Duration::seconds(75);
        let got = EventQuery::new().start_time(from).end_time(to).filter(&sample());
        assert_eq!(got.len(), 2);
    }
}
