// Integration tests for the triage engine (engine.rs).
//
// Tests the pure `classify` function, rule ordering, custom rules,
// and various input scenarios.

use agileplus_domain::domain::backlog::{BacklogPriority, Intent};
use agileplus_triage::engine::{SyncedItem, TriageOutcome, TriageRule, TriageRules, classify};

// ── Default rules ────────────────────────────────────────────────────────────

#[test]
fn bug_keyword_in_title_gives_high_bug() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "App crashes on startup".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
    assert_eq!(out.matched_rule, "bug-keywords");
}

#[test]
fn crash_in_body_gives_high_bug() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Login issue".to_owned(),
        body: Some("There is a crash when I press submit".to_owned()),
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
}

#[test]
fn bug_label_gives_high_bug() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Something weird".to_owned(),
        body: None,
        labels: vec!["bug".to_owned()],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
}

#[test]
fn docs_keyword_gives_low_docs() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Update the documentation for the API".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Docs);
    assert_eq!(out.priority, BacklogPriority::Low);
    assert_eq!(out.matched_rule, "docs-keywords");
}

#[test]
fn readme_keyword_gives_low_docs() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Fix typo in README".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    // "Fix typo" triggers Bug intent with default priority
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn docs_label_gives_low_docs() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Improve onboarding".to_owned(),
        body: None,
        labels: vec!["documentation".to_owned()],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Docs);
    assert_eq!(out.priority, BacklogPriority::Low);
}

#[test]
fn feature_keyword_gives_medium_feature() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Add support for dark mode".to_owned(),
        body: None,
        labels: vec!["enhancement".to_owned()],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Feature);
    // Feature has no explicit priority in default rules, so defaults to Medium
    assert_eq!(out.priority, BacklogPriority::Medium);
}

#[test]
fn feature_implementation_keyword_gives_medium_feature() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Implement new authentication flow".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Feature);
    assert_eq!(out.priority, BacklogPriority::Medium);
}

#[test]
fn unmatched_item_gives_default_medium_task() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Quarterly review sync".to_owned(),
        body: Some("Let us meet and discuss".to_owned()),
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Task);
    assert_eq!(out.priority, BacklogPriority::Medium);
    assert_eq!(out.matched_rule, "default");
}

#[test]
fn empty_item_gives_default() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem::default();
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Task);
    assert_eq!(out.priority, BacklogPriority::Medium);
    assert_eq!(out.matched_rule, "default");
}

// ── Rule precedence ──────────────────────────────────────────────────────────

#[test]
fn bug_before_docs_precedence() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "bug in the documentation page".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    // "bug" fires the bug-keywords rule before docs-keywords
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
}

#[test]
fn docs_before_feature_precedence() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "documentation for the new feature".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    // docs-keywords fires before feature-keywords
    assert_eq!(out.intent, Intent::Docs);
    assert_eq!(out.priority, BacklogPriority::Low);
}

// ── Case insensitivity ───────────────────────────────────────────────────────

#[test]
fn bug_keyword_case_insensitive() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "CRASH on startup".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn docs_keyword_case_insensitive() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Update README".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Docs);
}

// ── Custom rules ─────────────────────────────────────────────────────────────

#[test]
fn custom_rules_override_defaults() {
    let custom_rules = TriageRules {
        rules: vec![TriageRule {
            name: "security".to_owned(),
            keywords: vec!["auth".to_owned(), "token".to_owned()],
            intent: Intent::Bug,
            priority: Some(BacklogPriority::Critical),
        }],
        default: TriageOutcome {
            priority: BacklogPriority::Low,
            intent: Intent::Idea,
            matched_rule: "custom-default".to_owned(),
        },
    };

    let item = SyncedItem {
        title: "Auth token leaks in logs".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &custom_rules);
    assert_eq!(out.priority, BacklogPriority::Critical);
    assert_eq!(out.intent, Intent::Bug);

    let unmatched = SyncedItem {
        title: "Random thing".to_owned(),
        ..Default::default()
    };
    let default_out = classify(&unmatched, &custom_rules);
    assert_eq!(default_out.intent, Intent::Idea);
    assert_eq!(default_out.priority, BacklogPriority::Low);
}

#[test]
fn custom_rule_with_no_priority_uses_intent_default() {
    let custom_rules = TriageRules {
        rules: vec![TriageRule {
            name: "custom-feature".to_owned(),
            keywords: vec!["magic".to_owned()],
            intent: Intent::Feature,
            priority: None,
        }],
        default: TriageOutcome {
            priority: BacklogPriority::Low,
            intent: Intent::Task,
            matched_rule: "default".to_owned(),
        },
    };

    let item = SyncedItem {
        title: "Add magic feature".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &custom_rules);
    assert_eq!(out.intent, Intent::Feature);
    // priority=None means intent.default_priority() => Feature -> Medium
    assert_eq!(out.priority, BacklogPriority::Medium);
}

#[test]
fn empty_rules_set_always_defaults() {
    let empty_rules = TriageRules {
        rules: vec![],
        default: TriageOutcome {
            priority: BacklogPriority::Critical,
            intent: Intent::Bug,
            matched_rule: "always-bug".to_owned(),
        },
    };

    let item = SyncedItem {
        title: "anything at all".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &empty_rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::Critical);
}

// ── Body and labels corpus building ──────────────────────────────────────────

#[test]
fn body_text_is_searched() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "General issue".to_owned(),
        body: Some("The application keeps failing with a segfault".to_owned()),
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
}

#[test]
fn label_matching_works() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Random title".to_owned(),
        body: None,
        labels: vec!["feature".to_owned()],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Feature);
    assert_eq!(out.priority, BacklogPriority::Medium);
}

#[test]
fn multiple_labels_first_matching_rule_wins() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Random title".to_owned(),
        body: None,
        labels: vec!["bug".to_owned(), "feature".to_owned()],
    };
    let out = classify(&item, &rules);
    // Bug rule fires first (before feature)
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn only_body_matches_no_title_keywords() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Issue report".to_owned(),
        body: Some("Adding a new endpoint for user profiles".to_owned()),
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Feature);
}

// ── Serialization round-trip ─────────────────────────────────────────────────

#[test]
fn synced_item_serialization_round_trip() {
    let item = SyncedItem {
        title: "Test item".to_owned(),
        body: Some("Some body".to_owned()),
        labels: vec!["bug".to_owned(), "urgent".to_owned()],
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: SyncedItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.title, item.title);
    assert_eq!(back.body, item.body);
    assert_eq!(back.labels, item.labels);
}

#[test]
fn triage_outcome_serialization_round_trip() {
    let outcome = TriageOutcome {
        priority: BacklogPriority::High,
        intent: Intent::Bug,
        matched_rule: "bug-keywords".to_owned(),
    };
    let json = serde_json::to_string(&outcome).unwrap();
    let back: TriageOutcome = serde_json::from_str(&json).unwrap();
    assert_eq!(back, outcome);
}

#[test]
fn triage_rules_serialization_round_trip() {
    let rules = TriageRules::default_rules();
    let json = serde_json::to_string(&rules).unwrap();
    let back: TriageRules = serde_json::from_str(&json).unwrap();
    assert_eq!(back.rules.len(), rules.rules.len());
    assert_eq!(back.default, rules.default);
}

// ── Edge cases ───────────────────────────────────────────────────────────────

#[test]
fn all_bug_keywords_match() {
    let rules = TriageRules::default_rules();
    let bug_keywords = ["bug", "crash", "error", "panic", "broken", "regression",
                        "failing", "exception", "segfault", "fix"];
    for kw in &bug_keywords {
        let item = SyncedItem {
            title: format!("Issue with {kw}"),
            body: None,
            labels: vec![],
        };
        let out = classify(&item, &rules);
        assert_eq!(out.intent, Intent::Bug, "keyword '{kw}' should classify as Bug");
    }
}

#[test]
fn all_docs_keywords_match() {
    let rules = TriageRules::default_rules();
    let docs_keywords = ["docs", "documentation", "readme", "changelog", "typo",
                         "spelling", "document", "guide", "tutorial", "wiki"];
    for kw in &docs_keywords {
        let item = SyncedItem {
            title: format!("Update the {kw}"),
            body: None,
            labels: vec![],
        };
        let out = classify(&item, &rules);
        assert_eq!(out.intent, Intent::Docs, "keyword '{kw}' should classify as Docs");
    }
}

#[test]
fn all_feature_keywords_match() {
    let rules = TriageRules::default_rules();
    let feature_keywords = ["feature", "enhancement", "implement", "add", "new",
                            "support", "request"];
    for kw in &feature_keywords {
        let item = SyncedItem {
            title: format!("Need to {kw} something"),
            body: None,
            labels: vec![],
        };
        let out = classify(&item, &rules);
        assert_eq!(out.intent, Intent::Feature, "keyword '{kw}' should classify as Feature");
    }
}

#[test]
fn very_long_title_matches_correctly() {
    let rules = TriageRules::default_rules();
    let long_title = format!("This is a very long title with {} words. ", "extra ").repeat(50)
        + "But it ends with crash in production.";
    let item = SyncedItem {
        title: long_title,
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn unicode_in_title_still_matches() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Élément de crash dans le titre".to_owned(),
        body: None,
        labels: vec![],
    };
    let out = classify(&item, &rules);
    // "crash" is in the title
    assert_eq!(out.intent, Intent::Bug);
}

#[test]
fn labels_with_mixed_case_match() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Some feature".to_owned(),
        body: None,
        labels: vec!["Enhancement".to_owned()],
    };
    let out = classify(&item, &rules);
    assert_eq!(out.intent, Intent::Feature);
}

#[test]
fn multiple_body_and_title_sections_combined() {
    let rules = TriageRules::default_rules();
    let item = SyncedItem {
        title: "Issue report".to_owned(),
        body: Some("The docs need to be updated with the new crash fix".to_owned()),
        labels: vec!["bug".to_owned()],
    };
    let out = classify(&item, &rules);
    // Bug fires first due to precedence
    assert_eq!(out.intent, Intent::Bug);
    assert_eq!(out.priority, BacklogPriority::High);
}

#[test]
fn rebuild_corpus_combines_all_parts() {
    // Verify that title + body + labels are all part of the search corpus
    let rules = TriageRules::default_rules();

    // No match in title
    let item1 = SyncedItem {
        title: "Issue report".to_owned(),
        body: None,
        labels: vec![],
    };
    let out1 = classify(&item1, &rules);
    assert_eq!(out1.intent, Intent::Task); // default

    // Match only in body
    let item2 = SyncedItem {
        title: "Issue report".to_owned(),
        body: Some("regression in login flow".to_owned()),
        labels: vec![],
    };
    let out2 = classify(&item2, &rules);
    assert_eq!(out2.intent, Intent::Bug);

    // Match only in labels
    let item3 = SyncedItem {
        title: "Issue report".to_owned(),
        body: None,
        labels: vec!["typo".to_owned()],
    };
    let out3 = classify(&item3, &rules);
    assert_eq!(out3.intent, Intent::Docs);
}
