//! Tests for `agileplus branch` CLI argument parsing.
//!
//! Covers BranchArgs / BranchCommand clap parsing, default values,
//! and all subcommand variants.

use clap::Parser;

use agileplus_cli::commands::branch::{BranchArgs, BranchCommand};

/// Helper: parse args from string slices.
fn parse_branch(args: &[&str]) -> BranchArgs {
    #[derive(Parser)]
    #[command(name = "test")]
    struct Wrapper {
        #[command(subcommand)]
        cmd: BranchArgs,
    }
    // Prepend a dummy program name + "branch" subcommand prefix.
    let mut full = vec!["test", "branch"];
    full.extend(args);
    Wrapper::parse_from(full).cmd
}

// ── Create ──────────────────────────────────────────────────────────────────

#[test]
fn branch_create_parses_name() {
    let args = parse_branch(&["create", "--name", "feat/auth"]);
    match args.command {
        BranchCommand::Create { name, base } => {
            assert_eq!(name, "feat/auth");
            assert_eq!(base, "main"); // default
        }
        _ => panic!("expected Create variant"),
    }
}

#[test]
fn branch_create_custom_base() {
    let args = parse_branch(&["create", "--name", "fix/bug", "--base", "develop"]);
    match args.command {
        BranchCommand::Create { name, base } => {
            assert_eq!(name, "fix/bug");
            assert_eq!(base, "develop");
        }
        _ => panic!("expected Create variant"),
    }
}

// ── Checkout ────────────────────────────────────────────────────────────────

#[test]
fn branch_checkout_parses_name() {
    let args = parse_branch(&["checkout", "--name", "feat/auth"]);
    match args.command {
        BranchCommand::Checkout { name } => {
            assert_eq!(name, "feat/auth");
        }
        _ => panic!("expected Checkout variant"),
    }
}

// ── Delete ──────────────────────────────────────────────────────────────────

#[test]
fn branch_delete_defaults() {
    let args = parse_branch(&["delete", "--name", "feat/old"]);
    match args.command {
        BranchCommand::Delete {
            name,
            force,
            remote,
        } => {
            assert_eq!(name, "feat/old");
            assert!(!force); // default
            assert!(remote.is_none()); // default
        }
        _ => panic!("expected Delete variant"),
    }
}

#[test]
fn branch_delete_force_with_remote() {
    let args = parse_branch(&[
        "delete",
        "--name",
        "feat/old",
        "--force",
        "--remote",
        "upstream",
    ]);
    match args.command {
        BranchCommand::Delete {
            name,
            force,
            remote,
        } => {
            assert_eq!(name, "feat/old");
            assert!(force);
            assert_eq!(remote.as_deref(), Some("upstream"));
        }
        _ => panic!("expected Delete variant"),
    }
}

// ── List ────────────────────────────────────────────────────────────────────

#[test]
fn branch_list_defaults() {
    let args = parse_branch(&["list"]);
    match args.command {
        BranchCommand::List {
            pattern,
            remote,
            output,
        } => {
            assert!(pattern.is_none());
            assert!(!remote);
            assert_eq!(output, "table");
        }
        _ => panic!("expected List variant"),
    }
}

#[test]
fn branch_list_with_pattern() {
    let args = parse_branch(&["list", "--pattern", "feat/*", "--remote", "--output", "json"]);
    match args.command {
        BranchCommand::List {
            pattern,
            remote,
            output,
        } => {
            assert_eq!(pattern.as_deref(), Some("feat/*"));
            assert!(remote);
            assert_eq!(output, "json");
        }
        _ => panic!("expected List variant"),
    }
}

// ── Sync ────────────────────────────────────────────────────────────────────

#[test]
fn branch_sync_defaults() {
    let args = parse_branch(&["sync"]);
    match args.command {
        BranchCommand::Sync {
            source,
            target,
            output,
        } => {
            assert_eq!(source, "main");
            assert_eq!(target, "canary");
            assert_eq!(output, "table");
        }
        _ => panic!("expected Sync variant"),
    }
}

#[test]
fn branch_sync_custom_refs() {
    let args = parse_branch(&[
        "sync",
        "--source",
        "develop",
        "--target",
        "release",
        "--output",
        "json",
    ]);
    match args.command {
        BranchCommand::Sync {
            source,
            target,
            output,
        } => {
            assert_eq!(source, "develop");
            assert_eq!(target, "release");
            assert_eq!(output, "json");
        }
        _ => panic!("expected Sync variant"),
    }
}

// ── BranchInfo Serialize (domain type) ───────────────────────────────────────

#[test]
fn branch_info_serialization() {
    // BranchInfo is private in branch.rs, but we can test the domain port type
    use agileplus_domain::ports::BranchInfo;
    let info = BranchInfo {
        name: "feat/auth".to_string(),
        commit: "abc1234".to_string(),
        is_remote: false,
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("feat/auth"));
    assert!(json.contains("abc1234"));

    let back: BranchInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "feat/auth");
    assert!(!back.is_remote);
}

#[test]
fn branch_info_remote_serialization() {
    use agileplus_domain::ports::BranchInfo;
    let info = BranchInfo {
        name: "origin/feat/auth".to_string(),
        commit: "def5678".to_string(),
        is_remote: true,
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: BranchInfo = serde_json::from_str(&json).unwrap();
    assert!(back.is_remote);
    assert_eq!(back.name, "origin/feat/auth");
}
