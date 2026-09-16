//! Integration tests for the SubCommandRegistry.
//!
//! Covers: CLI subcommand construction, lookup, categories, display.

#![cfg(feature = "registry")]

use agileplus_subcmds::{SubCommand, SubCommandCategory, SubCommandRegistry};

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[test]
fn registry_new_has_nonzero_count() {
    let reg = SubCommandRegistry::new();
    let all = reg.list(None);
    assert!(
        all.len() >= 20,
        "Expected at least 20 built-in commands, got {}",
        all.len()
    );
}

#[test]
fn registry_lists_all_eight_categories() {
    let reg = SubCommandRegistry::new();
    let cats = reg.categories();
    assert_eq!(cats.len(), 8, "Expected 8 categories");
}

#[test]
fn registry_category_counts_sum_to_total() {
    let reg = SubCommandRegistry::new();
    let total = reg.list(None).len();
    let cat_total: usize = reg.categories().iter().map(|(_, n)| n).sum();
    assert_eq!(total, cat_total);
}

// ---------------------------------------------------------------------------
// Lookup
// ---------------------------------------------------------------------------

#[test]
fn lookup_known_commands() {
    let reg = SubCommandRegistry::new();
    for name in &[
        "triage:classify",
        "triage:file-bug",
        "governance:check-gates",
        "sync:push-plane",
        "git:create-worktree",
        "devops:lint-and-format",
        "context:load-spec",
        "escape:quick-fix",
        "meta:generate-router",
        "meta:list-commands",
    ] {
        let cmd = reg.get(name).unwrap_or_else(|| panic!("Missing command: {name}"));
        assert_eq!(cmd.name, *name);
        assert!(!cmd.description.is_empty());
        assert!(!cmd.usage.is_empty());
    }
}

#[test]
fn lookup_unknown_returns_none() {
    let reg = SubCommandRegistry::new();
    assert!(reg.get("no:such:command").is_none());
}

// ---------------------------------------------------------------------------
// Category filtering
// ---------------------------------------------------------------------------

#[test]
fn list_by_category_triage() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Triage));
    assert_eq!(cmds.len(), 3);
    assert!(cmds.iter().all(|c| c.category == SubCommandCategory::Triage));
}

#[test]
fn list_by_category_governance() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Governance));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn list_by_category_sync() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Sync));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn list_by_category_git() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Git));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn list_by_category_devops() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::DevOps));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn list_by_category_context() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Context));
    assert_eq!(cmds.len(), 4);
}

#[test]
fn list_by_category_escape() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Escape));
    assert_eq!(cmds.len(), 3);
}

#[test]
fn list_by_category_meta() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(Some(SubCommandCategory::Meta));
    assert_eq!(cmds.len(), 3);
}

// ---------------------------------------------------------------------------
// Sorted output
// ---------------------------------------------------------------------------

#[test]
fn list_returns_sorted_by_name() {
    let reg = SubCommandRegistry::new();
    let cmds = reg.list(None);
    for window in cmds.windows(2) {
        assert!(
            window[0].name <= window[1].name,
            "Not sorted: {} > {}",
            window[0].name,
            window[1].name
        );
    }
}

// ---------------------------------------------------------------------------
// Display trait
// ---------------------------------------------------------------------------

#[test]
fn category_display() {
    assert_eq!(SubCommandCategory::Triage.to_string(), "triage");
    assert_eq!(SubCommandCategory::Governance.to_string(), "governance");
    assert_eq!(SubCommandCategory::Sync.to_string(), "sync");
    assert_eq!(SubCommandCategory::Git.to_string(), "git");
    assert_eq!(SubCommandCategory::DevOps.to_string(), "devops");
    assert_eq!(SubCommandCategory::Context.to_string(), "context");
    assert_eq!(SubCommandCategory::Escape.to_string(), "escape");
    assert_eq!(SubCommandCategory::Meta.to_string(), "meta");
}

// ---------------------------------------------------------------------------
// Serialization round-trip
// ---------------------------------------------------------------------------

#[test]
fn subcommand_serde_round_trip() {
    let cmd = SubCommand {
        name: "test:cmd".to_string(),
        category: SubCommandCategory::Triage,
        description: "A test command".to_string(),
        usage: "test:cmd --foo".to_string(),
        hidden: false,
    };
    let json = serde_json::to_string(&cmd).unwrap();
    let restored: SubCommand = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.name, "test:cmd");
    assert_eq!(restored.category, SubCommandCategory::Triage);
    assert!(!restored.hidden);
}

#[test]
fn category_serde_round_trip() {
    let cats = [
        SubCommandCategory::Triage,
        SubCommandCategory::Governance,
        SubCommandCategory::Sync,
        SubCommandCategory::Git,
        SubCommandCategory::DevOps,
        SubCommandCategory::Context,
        SubCommandCategory::Escape,
        SubCommandCategory::Meta,
    ];
    for cat in cats {
        let json = serde_json::to_string(&cat).unwrap();
        let restored: SubCommandCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, cat);
    }
}

// ---------------------------------------------------------------------------
// No hidden commands in built-in set
// ---------------------------------------------------------------------------

/// All built-in commands are hidden by default (per design).
/// They are listed via `meta:list-commands` but not surfaced directly.
#[test]
fn all_builtin_commands_are_hidden() {
    let reg = SubCommandRegistry::new();
    for cmd in reg.list(None) {
        assert!(
            cmd.hidden,
            "Command '{}' should be hidden by default",
            cmd.name
        );
    }
}

// ---------------------------------------------------------------------------
// Prefix convention: every command has category prefix
// ---------------------------------------------------------------------------

#[test]
fn every_command_has_category_prefix() {
    let reg = SubCommandRegistry::new();
    for cmd in reg.list(None) {
        let prefix = format!("{}:", cmd.category);
        assert!(
            cmd.name.starts_with(&prefix),
            "Command '{}' does not start with expected prefix '{}'",
            cmd.name,
            prefix
        );
    }
}
