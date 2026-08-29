// SPDX-License-Identifier: MIT OR Apache-2.0

use agileplus_triage::repo_introspect::{RepoState, inspect_repo};
use std::process::Command;

#[test]
fn linked_worktree_is_inspected_as_git_repository() {
    let root = tempfile::tempdir().unwrap();
    let repo = root.path().join("repo");
    let worktree = root.path().join("linked-worktree");

    assert!(
        Command::new("git")
            .args(["init", "--initial-branch", "main"])
            .arg(&repo)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&repo)
            .args([
                "-c",
                "user.name=AgilePlus Test",
                "-c",
                "user.email=agileplus-test@example.invalid",
                "commit",
                "--allow-empty",
                "-m",
                "initial",
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .current_dir(&repo)
            .args([
                "worktree",
                "add",
                "-b",
                "linked",
                worktree.to_str().unwrap()
            ])
            .status()
            .unwrap()
            .success()
    );

    let info = inspect_repo(&worktree);

    assert_eq!(info.state, RepoState::Git);
    assert_eq!(info.current_branch.as_deref(), Some("linked"));
    assert_eq!(info.hygiene_score, 100);
}
