//! Hierarchical subject (topic) addressing for the event bus.
//!
//! Subjects follow the NATS dot-separated convention:
//!   `agileplus.feature.42.state_transitioned`
//!   `agileplus.wp.7.created`
//!
//! Wildcards:
//! - `*` matches a single token  (`agileplus.feature.*.created`)
//! - `>` matches one or more tokens (`agileplus.feature.>`)

use std::fmt;

/// A validated subject string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Subject(String);

impl Subject {
    /// Build a subject from dot-separated tokens.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// Convenience: `{prefix}.{entity_type}.{entity_id}.{event_type}`
    pub fn for_event(prefix: &str, entity_type: &str, entity_id: i64, event_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.{entity_id}.{event_type}"))
    }

    /// Wildcard subject matching all events for an entity type:
    /// `{prefix}.{entity_type}.>`
    pub fn all_for_entity(prefix: &str, entity_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.>"))
    }

    /// Wildcard subject matching a specific event type across all entities:
    /// `{prefix}.{entity_type}.*.{event_type}`
    pub fn all_of_type(prefix: &str, entity_type: &str, event_type: &str) -> Self {
        Self(format!("{prefix}.{entity_type}.*.{event_type}"))
    }

    /// Return the raw subject string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check whether this subject matches a concrete (non-wildcard) subject.
    pub fn matches(&self, concrete: &Subject) -> bool {
        let pat_tokens: Vec<&str> = self.0.split('.').collect();
        let sub_tokens: Vec<&str> = concrete.0.split('.').collect();
        matches_tokens(&pat_tokens, &sub_tokens)
    }
}

fn matches_tokens(pattern: &[&str], subject: &[&str]) -> bool {
    if pattern.is_empty() && subject.is_empty() {
        return true;
    }
    if pattern.is_empty() {
        return false;
    }
    if pattern[0] == ">" {
        return !subject.is_empty();
    }
    if subject.is_empty() {
        return false;
    }
    if pattern[0] == "*" || pattern[0] == subject[0] {
        return matches_tokens(&pattern[1..], &subject[1..]);
    }
    false
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let s = Subject::new("agileplus.feature.1.created");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(!s.matches(&Subject::new("agileplus.feature.2.created")));
    }

    #[test]
    fn star_wildcard() {
        let s = Subject::all_of_type("agileplus", "feature", "created");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(s.matches(&Subject::new("agileplus.feature.99.created")));
        assert!(!s.matches(&Subject::new("agileplus.feature.1.deleted")));
    }

    #[test]
    fn chevron_wildcard() {
        let s = Subject::all_for_entity("agileplus", "feature");
        assert!(s.matches(&Subject::new("agileplus.feature.1.created")));
        assert!(s.matches(
            &Subject::new("agileplus.feature.1.state_transitioned")
        ));
        assert!(!s.matches(&Subject::new("agileplus.wp.1.created")));
    }

    #[test]
    fn for_event_builds_correct_subject() {
        let s = Subject::for_event("agileplus", "feature", 42, "state_transitioned");
        assert_eq!(s.as_str(), "agileplus.feature.42.state_transitioned");
    }

    #[test]
    fn display_trait() {
        let s = Subject::new("a.b.c");
        assert_eq!(format!("{s}"), "a.b.c");
    }

    #[test]
    fn as_str_returns_inner() {
        let s = Subject::new("x.y.z");
        assert_eq!(s.as_str(), "x.y.z");
    }

    #[test]
    fn partial_eq_reflexive() {
        let s = Subject::new("t");
        assert_eq!(s, s);
    }

    #[test]
    fn clone_is_equal() {
        let a = Subject::new("cloned.subject");
        let b = a.clone();
        assert_eq!(a, b);
        assert_eq!(a.as_str(), b.as_str());
    }

    #[test]
    fn hash_is_consistent() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let s1 = Subject::new("key1");
        let s2 = Subject::new("key2");
        map.insert(s1.clone(), "val1");
        map.insert(s2.clone(), "val2");
        assert_eq!(map.get(&s1), Some(&"val1"));
        assert_eq!(map.get(&s2), Some(&"val2"));
    }

    #[test]
    fn multiple_stars() {
        let s = Subject::new("a.*.*.d");
        assert!(s.matches(&Subject::new("a.b.c.d")));
        assert!(s.matches(&Subject::new("a.x.y.d")));
        assert!(!s.matches(&Subject::new("a.b.c.e")));
        assert!(!s.matches(&Subject::new("a.b.c.d.e")));
    }

    #[test]
    fn star_does_not_match_multiple_tokens() {
        let s = Subject::new("a.*.c");
        assert!(s.matches(&Subject::new("a.b.c")));
        assert!(!s.matches(&Subject::new("a.b.c.d")));
        assert!(!s.matches(&Subject::new("a.b")));
    }

    #[test]
    fn chevron_matches_remaining_tokens() {
        let s = Subject::new("a.b.>");
        assert!(s.matches(&Subject::new("a.b.c")));
        assert!(s.matches(&Subject::new("a.b.c.d.e")));
        assert!(!s.matches(&Subject::new("a.b")));
    }

    #[test]
    fn chevron_at_end_matches_one_token() {
        let s = Subject::new("a.>");
        assert!(s.matches(&Subject::new("a.b")));
        assert!(s.matches(&Subject::new("a.b.c")));
    }

    #[test]
    fn empty_pattern_empty_subject() {
        let s = Subject::new("");
        assert!(s.matches(&Subject::new("")));
    }

    #[test]
    fn empty_pattern_does_not_match_nonempty() {
        let s = Subject::new("");
        assert!(!s.matches(&Subject::new("a")));
    }

    #[test]
    fn nonempty_pattern_does_not_match_empty() {
        let s = Subject::new("a");
        assert!(!s.matches(&Subject::new("")));
    }

    #[test]
    fn chevron_in_middle_matches_from_that_point() {
        // `>` matches one or more tokens from its position onward.
        // In NATS, `>` is always terminal: a.>.d is equivalent to a.>
        let s = Subject::new("a.>.d");
        assert!(s.matches(&Subject::new("a.b.d")));
        assert!(s.matches(&Subject::new("a.b.c.d")));
        // > consumes "b" and "c", so this also matches (> is terminal)
        assert!(s.matches(&Subject::new("a.b.c")));
    }

    #[test]
    fn for_event_with_zero_id() {
        let s = Subject::for_event("pre", "ent", 0, "evt");
        assert_eq!(s.as_str(), "pre.ent.0.evt");
    }

    #[test]
    fn for_event_with_large_id() {
        let s = Subject::for_event("pre", "ent", 999_999, "evt");
        assert_eq!(s.as_str(), "pre.ent.999999.evt");
    }

    #[test]
    fn all_for_entity_uses_chevron() {
        let s = Subject::all_for_entity("p", "e");
        assert!(s.as_str().ends_with(".>"));
        assert_eq!(s.as_str(), "p.e.>");
    }

    #[test]
    fn all_of_type_uses_star() {
        let s = Subject::all_of_type("p", "e", "ev");
        assert!(s.as_str().contains("*"));
        assert_eq!(s.as_str(), "p.e.*.ev");
    }

    #[test]
    fn new_from_string_owned() {
        let raw = String::from("owned.subject");
        let s = Subject::new(raw);
        assert_eq!(s.as_str(), "owned.subject");
    }

    #[test]
    fn exact_mismatch_different_prefix() {
        let s = Subject::new("other.feature.1.created");
        assert!(!s.matches(&Subject::new("agileplus.feature.1.created")));
    }

    #[test]
    fn exact_mismatch_different_event() {
        let s = Subject::new("agileplus.feature.1.deleted");
        assert!(!s.matches(&Subject::new("agileplus.feature.1.created")));
    }

    #[test]
    fn single_token_pattern_matches_single_token() {
        let s = Subject::new("hello");
        assert!(s.matches(&Subject::new("hello")));
        assert!(!s.matches(&Subject::new("hello.world")));
    }

    #[test]
    fn deeply_nested_pattern() {
        let s = Subject::new("*.*.*.*.e");
        assert!(s.matches(&Subject::new("a.b.c.d.e")));
        assert!(!s.matches(&Subject::new("a.b.c.d")));
        assert!(!s.matches(&Subject::new("a.b.c.d.e.f")));
    }

    #[test]
    fn partial_eq_symmetric() {
        let a = Subject::new("x");
        let b = Subject::new("x");
        assert!(a == b);
        assert!(b == a);
    }

    #[test]
    fn partial_eq_inequality() {
        let a = Subject::new("x");
        let b = Subject::new("y");
        assert!(a != b);
    }

    #[test]
    fn matches_self() {
        let s = Subject::new("agileplus.feature.1.created");
        assert!(s.matches(&s));
    }

    #[test]
    fn star_wildcard_does_not_cross_dots() {
        let pat = Subject::new("a.*.c");
        assert!(pat.matches(&Subject::new("a.b.c")));
        assert!(!pat.matches(&Subject::new("a.b.b.c")));
    }

    #[test]
    fn chevron_wildcard_matches_entire_suffix() {
        let pat = Subject::new("a.b.>");
        assert!(pat.matches(&Subject::new("a.b.c")));
        assert!(pat.matches(&Subject::new("a.b.c.d.e")));
    }

    #[test]
    fn chevron_wildcard_requires_at_least_one_token() {
        let pat = Subject::new("a.>");
        assert!(!pat.matches(&Subject::new("a")));
        assert!(pat.matches(&Subject::new("a.b")));
    }

    #[test]
    fn multiple_stars_in_pattern() {
        let pat = Subject::new("*.b.*");
        assert!(pat.matches(&Subject::new("a.b.c")));
        assert!(pat.matches(&Subject::new("x.b.y")));
        assert!(!pat.matches(&Subject::new("a.c.d"))); // b required
        assert!(!pat.matches(&Subject::new("a.b"))); // need trailing token
    }

    #[test]
    fn mixed_star_and_chevron() {
        let pat = Subject::new("*.b.>");
        assert!(pat.matches(&Subject::new("a.b.c")));
        assert!(pat.matches(&Subject::new("a.b.c.d.e")));
        assert!(!pat.matches(&Subject::new("a.b"))); // chevron needs token
        assert!(!pat.matches(&Subject::new("a.c.d"))); // second token not b
    }

    #[test]
    fn pattern_longer_than_subject_fails() {
        let pat = Subject::new("a.b.c");
        assert!(!pat.matches(&Subject::new("a.b")));
        assert!(!pat.matches(&Subject::new("a")));
    }

    #[test]
    fn subject_longer_than_pattern_no_chevron_fails() {
        let pat = Subject::new("a.b");
        assert!(!pat.matches(&Subject::new("a.b.c")));
    }

    #[test]
    fn empty_pattern_only_matches_empty() {
        let pat = Subject::new("");
        assert!(pat.matches(&Subject::new("")));
        assert!(!pat.matches(&Subject::new("a")));
    }

    #[test]
    fn tokens_with_special_chars_match_exactly() {
        let pat = Subject::new("a.b-c.d");
        assert!(pat.matches(&Subject::new("a.b-c.d")));
        assert!(!pat.matches(&Subject::new("a.b_c.d")));
    }

    #[test]
    fn for_event_zero_id_extended() {
        let s = Subject::for_event("pre", "ent", 0, "evt");
        assert_eq!(s.as_str(), "pre.ent.0.evt");
    }

    #[test]
    fn for_event_negative_id_extended() {
        let s = Subject::for_event("p", "e", -5, "e");
        assert_eq!(s.as_str(), "p.e.-5.e");
    }

    #[test]
    fn for_event_large_id_extended() {
        let s = Subject::for_event("p", "e", i64::MAX, "e");
        assert_eq!(s.as_str(), format!("p.e.{}.e", i64::MAX));
    }

    #[test]
    fn all_for_entity_uses_chevron_extended() {
        let s = Subject::all_for_entity("pre", "feat");
        assert_eq!(s.as_str(), "pre.feat.>");
    }

    #[test]
    fn all_of_type_uses_star_extended() {
        let s = Subject::all_of_type("pre", "feat", "created");
        assert_eq!(s.as_str(), "pre.feat.*.created");
    }

    #[test]
    fn hash_consistent_with_eq() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let a = Subject::new("a.b.c");
        let b = Subject::new("a.b.c");
        let mut ha = DefaultHasher::new();
        a.hash(&mut ha);
        let mut hb = DefaultHasher::new();
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn display_matches_as_str() {
        let s = Subject::new("disp.test");
        assert_eq!(format!("{s}"), s.as_str());
    }

    #[test]
    fn send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Subject>();
    }

    #[test]
    fn clone_is_independent() {
        let a = Subject::new("clone.test");
        let b = a.clone();
        assert_eq!(a, b);
        // No shared mutation possible (immutable struct)
    }
}
