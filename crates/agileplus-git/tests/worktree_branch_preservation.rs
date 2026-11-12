use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

#[tokio::test]
async fn existing_branch_keeps_its_tip_and_occupied_target_is_preserved() {
    let fixture = tempfile::tempdir().unwrap();
    let repo = fixture.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    git(&repo, &["config", "user.name", "fixture"]);
    git(&repo, &["config", "core.autocrlf", "false"]);
    std::fs::write(repo.join("tracked"), "old\n").unwrap();
    git(&repo, &["add", "tracked"]);
    git(&repo, &["commit", "-qm", "old"]);
    let old = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["branch", "feat/sample/wp-1"]);
    std::fs::write(repo.join("tracked"), "new\n").unwrap();
    git(&repo, &["commit", "-qam", "new"]);
    assert_ne!(old, git(&repo, &["rev-parse", "HEAD"]));

    let adapter = GitVcsAdapter::new(repo.clone());
    let target = adapter.worktree_path("sample", "wp-1");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("keep"), "sentinel").unwrap();
    assert!(adapter.create_worktree("sample", "wp-1").await.is_err());
    assert_eq!(
        std::fs::read_to_string(target.join("keep")).unwrap(),
        "sentinel"
    );
    assert_eq!(git(&repo, &["rev-parse", "feat/sample/wp-1"]), old);

    let second = GitVcsAdapter::new(repo.clone());
    git(&repo, &["branch", "feat/other/wp-2", &old]);
    let worktree = second.create_worktree("other", "wp-2").await.unwrap();
    assert_eq!(git(&worktree, &["rev-parse", "HEAD"]), old);
    assert_eq!(git(&repo, &["rev-parse", "feat/other/wp-2"]), old);
    assert_eq!(
        std::fs::read_to_string(worktree.join("tracked")).unwrap(),
        "old\n"
    );
}
