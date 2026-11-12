use agileplus_domain::ports::VcsPort;
use agileplus_git::GitVcsAdapter;
use std::path::Path;
use std::process::Command;

fn git(repo: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(name);
        }
    }
    let output = command
        .current_dir(repo)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", repo.join("fixture-no-global-config"))
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

#[test]
fn poisoned_git_environment_cannot_redirect_fixture_or_worktree() {
    let fixture = tempfile::tempdir().unwrap();
    let decoy = fixture.path().join("decoy");
    std::fs::create_dir(&decoy).unwrap();
    git(&decoy, &["init", "-q", "-b", "main"]);
    git(
        &decoy,
        &[
            "config",
            "core.hooksPath",
            decoy.join(".git/hooks").to_str().unwrap(),
        ],
    );
    git(&decoy, &["config", "user.email", "fixture@example.invalid"]);
    git(&decoy, &["config", "user.name", "fixture"]);
    std::fs::write(decoy.join("keep"), "sentinel").unwrap();
    git(&decoy, &["add", "keep"]);
    git(&decoy, &["commit", "-qm", "decoy"]);
    let refs = git(&decoy, &["show-ref"]);
    let worktrees = git(&decoy, &["worktree", "list", "--porcelain"]);
    let gitdir = decoy.join(".git");
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "existing_branch_keeps_its_tip_and_occupied_target_is_preserved",
        ])
        .env("GIT_DIR", &gitdir)
        .env("GIT_WORK_TREE", &decoy)
        .env("GIT_COMMON_DIR", &gitdir)
        .env("GIT_INDEX_FILE", gitdir.join("index"))
        .env("GIT_OBJECT_DIRECTORY", gitdir.join("objects"))
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.worktree")
        .env("GIT_CONFIG_VALUE_0", &decoy)
        .env("GIT_LFS_SKIP_SMUDGE", "preserved")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(git(&decoy, &["show-ref"]), refs);
    assert_eq!(git(&decoy, &["worktree", "list", "--porcelain"]), worktrees);
    assert_eq!(
        std::fs::read_to_string(decoy.join("keep")).unwrap(),
        "sentinel"
    );
}

#[tokio::test]
async fn existing_branch_keeps_its_tip_and_occupied_target_is_preserved() {
    let fixture = tempfile::tempdir().unwrap();
    let repo = fixture.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(
        &repo,
        &[
            "config",
            "core.hooksPath",
            repo.join(".git/hooks").to_str().unwrap(),
        ],
    );
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
    let hook = repo.join(".git/hooks/post-checkout");
    std::fs::write(
        &hook,
        "#!/bin/sh\nprintf '%s' \"${GIT_LFS_SKIP_SMUDGE-unset}\" > .compat-env\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

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
    assert_eq!(
        std::fs::read_to_string(worktree.join(".compat-env")).unwrap(),
        std::env::var("GIT_LFS_SKIP_SMUDGE").unwrap_or_else(|_| "unset".into())
    );
    assert_eq!(git(&worktree, &["rev-parse", "HEAD"]), old);
    assert_eq!(git(&repo, &["rev-parse", "feat/other/wp-2"]), old);
    assert_eq!(
        std::fs::read_to_string(worktree.join("tracked")).unwrap(),
        "old\n"
    );

    let missing = adapter.create_worktree("fresh", "wp-3").await.unwrap();
    assert_eq!(
        git(&missing, &["rev-parse", "HEAD"]),
        git(&repo, &["rev-parse", "main"])
    );
    let before = git(&repo, &["worktree", "list", "--porcelain"]);
    let error = adapter
        .create_worktree("bad\0slug", "wp-4")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("find branch"), "{error}");
    assert_eq!(git(&repo, &["worktree", "list", "--porcelain"]), before);
}
