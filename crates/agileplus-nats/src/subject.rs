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

    #[test]
    fn empty_middle_segment_is_significant() {
        // "a..b" splits to ["a", "", "b"]: distinct from "a.b".
        let with_gap = Subject::new("a..b");
        let without_gap = Subject::new("a.b");
        assert_ne!(with_gap, without_gap);
        assert!(with_gap.matches(&Subject::new("a..b")));
        assert!(!without_gap.matches(&with_gap));
        assert!(!with_gap.matches(&without_gap));
    }

    #[test]
    fn star_matches_empty_token() {
        // "*" matches any single token including the empty token produced by "..".
        let pat = Subject::new("a.*.b");
        assert!(pat.matches(&Subject::new("a..b")));
        assert!(pat.matches(&Subject::new("a.x.b")));
        assert!(!pat.matches(&Subject::new("a.b")));
    }

    #[test]
    fn leading_dot_is_distinct_token() {
        let leading = Subject::new(".a");
        let plain = Subject::new("a");
        assert_ne!(leading, plain);
        assert!(leading.matches(&Subject::new(".a")));
        assert!(!plain.matches(&leading));
        // "*.*" requires two tokens; ".a" has exactly two (["", "a"]).
        assert!(Subject::new("*.*").matches(&leading));
    }

    #[test]
    fn trailing_dot_is_distinct_token() {
        let trailing = Subject::new("a.");
        let plain = Subject::new("a");
        assert_ne!(trailing, plain);
        assert!(trailing.matches(&Subject::new("a.")));
        assert!(!plain.matches(&trailing));
        // "a.>" needs a (possibly empty) token after "a": "a." provides one.
        assert!(Subject::new("a.>").matches(&trailing));
        assert!(!Subject::new("a.>").matches(&plain));
    }

    #[test]
    fn chevron_matches_empty_token_suffix() {
        // "a." splits to ["a", ""], so "a.>" matches it (one token, empty).
        assert!(Subject::new("a.>").matches(&Subject::new("a.")));
    }

    #[test]
    fn chevron_only_pattern_matches_every_subject() {
        // "".split('.') yields [""] (one empty token), never zero tokens,
        // so ">" matches every possible subject string.
        let pat = Subject::new(">");
        assert!(pat.matches(&Subject::new("")));
        assert!(pat.matches(&Subject::new("a")));
        assert!(pat.matches(&Subject::new("a.b.c")));
        assert!(pat.matches(&Subject::new("a.")));
    }

    #[test]
    fn chevron_after_chevron_is_greedy_terminal() {
        // First ">" wins and consumes everything remaining.
        let pat = Subject::new("a.>.>");
        assert!(pat.matches(&Subject::new("a.b")));
        assert!(pat.matches(&Subject::new("a.b.c.d")));
        assert!(!pat.matches(&Subject::new("a")));
    }

    #[test]
    fn whitespace_token_matched_literally() {
        let pat = Subject::new("a.b c.d");
        assert!(pat.matches(&Subject::new("a.b c.d")));
        assert!(!pat.matches(&Subject::new("a.bc.d")));
        // Documents that no validation rejects spaces today.
    }

    #[test]
    fn unicode_tokens_match_literally() {
        let pat = Subject::new("a.héllo");
        assert!(pat.matches(&Subject::new("a.héllo")));
        assert!(!pat.matches(&Subject::new("a.hello")));
    }

    #[test]
    fn very_long_subject_exact_match() {
        let tokens: Vec<String> = (0..500).map(|i| i.to_string()).collect();
        let raw = tokens.join(".");
        let s = Subject::new(raw.clone());
        assert_eq!(s.as_str(), raw);
        assert!(s.matches(&Subject::new(raw.clone())));
        assert!(!s.matches(&Subject::new("0.1")));
    }

    #[test]
    fn very_long_subject_chevron_match() {
        let tokens: Vec<String> = (0..500).map(|i| i.to_string()).collect();
        let raw = format!("root.{}", tokens.join("."));
        let pat = Subject::new("root.>");
        assert!(pat.matches(&Subject::new(raw)));
        assert!(!pat.matches(&Subject::new("root")));
    }

    #[test]
    fn long_star_chain_matches_equal_token_count() {
        let stars: Vec<&str> = vec!["*"; 100];
        let pat = Subject::new(stars.join("."));
        let toks: Vec<String> = (0..100).map(|i| format!("t{i}")).collect();
        assert!(pat.matches(&Subject::new(toks.join("."))));
        let short: Vec<String> = (0..99).map(|i| format!("t{i}")).collect();
        assert!(!pat.matches(&Subject::new(short.join("."))));
    }

    #[test]
    fn single_token_with_dots_is_not_split_by_star() {
        // "*" is one token; it must not match a token containing a dot.
        let pat = Subject::new("a.*");
        assert!(!pat.matches(&Subject::new("a.b.c")));
        assert!(pat.matches(&Subject::new("a.b")));
    }

    #[test]
    fn roundtrip_new_from_as_str_is_identity() {
        for raw in [
            "plain",
            "a.b.c",
            "a..b",
            ".leading",
            "trailing.",
            "agileplus.feature.42.state_transitioned",
            "",
        ] {
            let s = Subject::new(raw);
            assert_eq!(s.as_str(), raw);
            let rebuilt = Subject::new(s.as_str());
            assert_eq!(rebuilt, s);
            assert_eq!(format!("{s}"), raw, "Display must round-trip {raw:?}");
        }
    }

    #[test]
    fn for_event_roundtrips_through_all_of_type_pattern() {
        // Every for_event subject must match its all_of_type and
        // all_for_entity patterns — the crate's core routing contract.
        for id in [0, 1, 7, 999_999, i64::MAX] {
            for event in ["created", "updated", "state_transitioned"] {
                let s = Subject::for_event("agileplus", "feature", id, event);
                assert!(Subject::all_of_type("agileplus", "feature", event).matches(&s));
                assert!(Subject::all_for_entity("agileplus", "feature").matches(&s));
                assert!(!Subject::all_for_entity("agileplus", "wp").matches(&s));
            }
        }
    }
}
