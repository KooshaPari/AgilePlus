// SPDX-License-Identifier: MIT OR Apache-2.0
//! Wave-10 integration tests for `agileplus-triage::engine`.
//!
//! Cross-cuts the rule-based `classify` function with the data types
//! `SyncedItem`, `TriageRule`, `TriageRules`, `TriageOutcome`.  Locks in:
//! - default rules produce the documented intent/priority matrix
//! - rule precedence is stable across rule-set shapes
//! - case-insensitive keyword matching across all three corpus fields
//! - serde round-trips for every public type
//! - empty / whitespace-only titles do not panic and fall to default
//! - rule `priority == None` resolves to `intent.default_priority()`

use agileplus_domain::domain::backlog::{BacklogPriority as BP, Intent};
use agileplus_triage::engine::{
    classify, SyncedItem, TriageOutcome, TriageRule, TriageRules,
};

fn item(title: &str, body: Option<&str>, labels: &[&str]) -> SyncedItem {
    SyncedItem {
        title: title.into(),
        body: body.map(String::from),
        labels: labels.iter().map(|s| s.to_string()).collect(),
    }
}

// ─── Default rules invariants ───────────────────────────────────────────────

#[test]
fn default_rules_have_three_categories_in_canonical_order() {
    // Bug -> Docs -> Feature.  Order matters because the engine is
    // first-match-wins and several keyword sets overlap.
    let rules = TriageRules::default_rules();
    assert_eq!(rules.rules.len(), 3);
    assert_eq!(rules.rules[0].name, "bug-keywords");
    assert_eq!(rules.rules[1].name, "docs-keywords");
    assert_eq!(rules.rules[2].name, "feature-keywords");
    assert_eq!(rules.default.intent, Intent::Task);
    assert_eq!(rules.default.priority, BP::Medium);
    assert_eq!(rules.default.matched_rule, "default");
}

#[test]
fn default_rules_priority_matrix_is_documented() {
    // Bug -> High, Docs -> Low, Feature -> None (resolves to Medium).
    let rules = TriageRules::default_rules();
    let by_name = |n: &str| rules.rules.iter().find(|r| r.name == n).unwrap();
    assert_eq!(by_name("bug-keywords").priority, Some(BP::High));
    assert_eq!(by_name("docs-keywords").priority, Some(BP::Low));
    assert!(by_name("feature-keywords").priority.is_none());

    // Cross-check via classify: feature priority resolves to Medium.
    let out = classify(&item("add new feature", None, &[]), &rules);
    assert_eq!(out.intent, Intent::Feature);
    assert_eq!(out.priority, BP::Medium);
}

#[test]
fn default_rules_intent_default_priority_table() {
    // Public contract: each intent has a documented default priority.
    use agileplus_domain::domain::backlog::Intent as I;
    let table: &[(Intent, BP)] = &[
        (I::Bug, BP::High),
        (I::Feature, BP::Medium),
        (I::Task, BP::Medium),
        (I::Idea, BP::Low),
        (I::Docs, BP::Low),
    ];
    for &(intent, expected) in table {
        assert_eq!(intent.default_priority(), expected, "{intent:?}");
    }
}

// ─── classify behavior ──────────────────────────────────────────────────────

#[test]
fn classify_first_match_wins_even_if_later_rule_higher_priority() {
    // Two rules share the keyword "alpha"; the first one wins even
    // though the second has Critical priority.  The engine is purely
    // first-match-wins.
    let rules = TriageRules {
        rules: vec![
            TriageRule {
                name: "first".into(),
                keywords: vec!["alpha".into()],
                intent: Intent::Idea,
                priority: Some(BP::Low),
            },
            TriageRule {
                name: "second".into(),
                keywords: vec!["alpha".into(), "beta".into()],
                intent: Intent::Bug,
                priority: Some(BP::Critical),
            },
        ],
        default: TriageOutcome {
            priority: BP::Medium,
            intent: Intent::Task,
            matched_rule: "default".into(),
        },
    };
    let out = classify(&item("alpha", None, &[]), &rules);
    assert_eq!(out.matched_rule, "first");
    assert_eq!(out.intent, Intent::Idea);
    assert_eq!(out.priority, BP::Low);
}

#[test]
fn classify_is_case_insensitive_across_corpus_fields() {
    // The corpus is lowercased before keyword search, so the source's
    // original case is irrelevant.
    let rules = TriageRules {
        rules: vec![TriageRule {
            name: "k".into(),
            keywords: vec!["alpha".into()],
            intent: Intent::Feature,
            priority: None,
        }],
        default: TriageOutcome {
            priority: BP::Low,
            intent: Intent::Task,
            matched_rule: "default".into(),
        },
    };
    // Each of these three items must classify as Feature because the
    // keyword `alpha` appears (case-insensitively) in one of the three
    // corpus fields.
    for it in [
        item("ALPHA-beta-GAMMA", None, &[]),
        item("x", Some("ALPHA in body"), &[]),
        item("x", None, &["Alpha"]),
    ] {
        assert_eq!(classify(&it, &rules).intent, Intent::Feature, "{it:?}");
    }
}

#[test]
fn classify_keyword_match_is_substring_not_word_boundary() {
    // `contains`, not word-bounded.  `fix` embedded in `prefix` still
    // matches the rule.
    let rules = TriageRules {
        rules: vec![TriageRule {
            name: "fix".into(),
            keywords: vec!["fix".into()],
            intent: Intent::Bug,
            priority: Some(BP::High),
        }],
        default: TriageOutcome {
            priority: BP::Medium,
            intent: Intent::Task,
            matched_rule: "default".into(),
        },
    };
    let out = classify(&item("prefix problem", None, &[]), &rules);
    assert_eq!(out.matched_rule, "fix");
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn classify_does_not_fuzzy_match_across_corpus_fields() {
    // `oauth` split across title+body is NOT contiguous in the corpus,
    // so `contains()` won't find it.
    let rules = TriageRules {
        rules: vec![TriageRule {
            name: "oauth".into(),
            keywords: vec!["oauth".into()],
            intent: Intent::Feature,
            priority: None,
        }],
        default: TriageOutcome {
            priority: BP::Low,
            intent: Intent::Task,
            matched_rule: "default".into(),
        },
    };
    let split = item("support for oa protocol", Some("uth is standard"), &[]);
    assert_eq!(classify(&split, &rules).matched_rule, "default");
    let contiguous = item("support for oauth protocol", None, &[]);
    assert_eq!(classify(&contiguous, &rules).matched_rule, "oauth");
}

#[test]
fn classify_empty_keywords_rule_is_skipped() {
    // A rule with no keywords can never match; the engine falls
    // through to the next rule.
    let rules = TriageRules {
        rules: vec![
            TriageRule {
                name: "empty".into(),
                keywords: vec![],
                intent: Intent::Feature,
                priority: Some(BP::High),
            },
            TriageRule {
                name: "second".into(),
                keywords: vec!["hello".into()],
                intent: Intent::Bug,
                priority: Some(BP::High),
            },
        ],
        default: TriageOutcome {
            priority: BP::Low,
            intent: Intent::Idea,
            matched_rule: "default".into(),
        },
    };
    let out = classify(&item("hello world", None, &[]), &rules);
    assert_eq!(out.matched_rule, "second");
}

#[test]
fn classify_corpus_field_search() {
    // Default rules; verify the keyword is found in each corpus field.
    let rules = TriageRules::default_rules();
    assert_eq!(
        classify(&item("Crash here", None, &[]), &rules).intent,
        Intent::Bug
    );
    assert_eq!(
        classify(&item("x", Some("Crash here"), &[]), &rules).intent,
        Intent::Bug
    );
    assert_eq!(
        classify(&item("x", None, &["crash"]), &rules).intent,
        Intent::Bug
    );
}

#[test]
fn classify_empty_or_whitespace_title_falls_through() {
    let rules = TriageRules::default_rules();
    let out = classify(&item("   \n\t  ", None, &[]), &rules);
    assert_eq!(out.matched_rule, "default");
    assert_eq!(out.intent, Intent::Task);
    assert_eq!(out.priority, BP::Medium);
}

#[test]
fn classify_only_labels_match() {
    let rules = TriageRules::default_rules();
    let out = classify(&item("", None, &["bug"]), &rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BP::High);
}

#[test]
fn classify_only_body_match() {
    let rules = TriageRules::default_rules();
    let out = classify(&item("", Some("Documentation updates"), &[]), &rules);
    assert_eq!(out.intent, Intent::Docs);
}

#[test]
fn classify_handles_huge_corpus_without_panic() {
    let rules = TriageRules::default_rules();
    let mut title = String::new();
    for i in 0..10_000 {
        if i > 0 {
            title.push(' ');
        }
        title.push_str(&format!("word{i}"));
    }
    title.push_str(" crash");
    let out = classify(&item(&title, None, &[]), &rules);
    assert_eq!(out.intent, Intent::Bug);
}

// ─── Serde round-trips ──────────────────────────────────────────────────────

#[test]
fn synced_item_default_is_empty_and_roundtrips() {
    let item = SyncedItem::default();
    assert_eq!(item.title, "");
    assert!(item.body.is_none());
    assert!(item.labels.is_empty());
    let json = serde_json::to_string(&item).unwrap();
    let back: SyncedItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "");
    assert!(back.body.is_none());
    assert!(back.labels.is_empty());
}

#[test]
fn synced_item_serde_preserves_field_values() {
    let it = item("title", Some("body"), &["a", "b", "c"]);
    let json = serde_json::to_string(&it).unwrap();
    let back: SyncedItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, "title");
    assert_eq!(back.body.as_deref(), Some("body"));
    assert_eq!(back.labels, vec!["a", "b", "c"]);
}

#[test]
fn triage_rule_serde_roundtrips_with_and_without_priority() {
    let with_prio = TriageRule {
        name: "my-rule".into(),
        keywords: vec!["alpha".into(), "beta".into()],
        intent: Intent::Idea,
        priority: Some(BP::Critical),
    };
    let back: TriageRule = serde_json::from_str(&serde_json::to_string(&with_prio).unwrap()).unwrap();
    assert_eq!(back.priority, Some(BP::Critical));

    let no_prio = TriageRule {
        name: "no-prio".into(),
        keywords: vec!["x".into()],
        intent: Intent::Feature,
        priority: None,
    };
    let json = serde_json::to_string(&no_prio).unwrap();
    assert!(json.contains("\"priority\":null"));
    let back: TriageRule = serde_json::from_str(&json).unwrap();
    assert!(back.priority.is_none());
}

#[test]
fn triage_outcome_eq_is_value_based_and_roundtrips() {
    let a = TriageOutcome {
        priority: BP::High,
        intent: Intent::Bug,
        matched_rule: "r".into(),
    };
    let b = a.clone();
    assert_eq!(a, b);
    let json = serde_json::to_string(&a).unwrap();
    let back: TriageOutcome = serde_json::from_str(&json).unwrap();
    assert_eq!(back, a);
}

#[test]
fn triage_rules_serde_preserves_rule_order() {
    let rules = TriageRules {
        rules: vec![
            TriageRule {
                name: "first".into(),
                keywords: vec!["alpha".into()],
                intent: Intent::Bug,
                priority: Some(BP::High),
            },
            TriageRule {
                name: "second".into(),
                keywords: vec!["beta".into()],
                intent: Intent::Docs,
                priority: Some(BP::Low),
            },
        ],
        default: TriageOutcome {
            priority: BP::Medium,
            intent: Intent::Task,
            matched_rule: "default".into(),
        },
    };
    let back: TriageRules = serde_json::from_str(&serde_json::to_string(&rules).unwrap()).unwrap();
    assert_eq!(back.rules.len(), 2);
    assert_eq!(back.rules[0].name, "first");
    assert_eq!(back.rules[1].name, "second");
    assert_eq!(back.rules[1].intent, Intent::Docs);
    assert_eq!(back.default, rules.default);
}

#[test]
fn classify_uses_intent_default_priority_when_rule_priority_is_none() {
    // A custom rule with priority=None falls back to the intent's
    // documented default priority.  Verify across all five intents.
    fn check(intent: Intent, expected: BP) {
        let rules = TriageRules {
            rules: vec![TriageRule {
                name: "k".into(),
                keywords: vec!["sentinel".into()],
                intent,
                priority: None,
            }],
            default: TriageOutcome {
                priority: BP::Low,
                intent: Intent::Task,
                matched_rule: "default".into(),
            },
        };
        let out = classify(&item("sentinel", None, &[]), &rules);
        assert_eq!(out.priority, expected, "intent={intent:?}");
    }
    check(Intent::Bug, BP::High);
    check(Intent::Feature, BP::Medium);
    check(Intent::Task, BP::Medium);
    check(Intent::Idea, BP::Low);
    check(Intent::Docs, BP::Low);
}
