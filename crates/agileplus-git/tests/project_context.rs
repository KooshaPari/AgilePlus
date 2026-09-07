use agileplus_git::ProjectContext;

#[test]
fn discovers_root_and_local_database_from_nested_directory() {
    let repo = tempfile::tempdir().expect("tempdir");
    git2::Repository::init(repo.path()).expect("git init");
    let nested = repo.path().join("src").join("nested");
    std::fs::create_dir_all(&nested).expect("create nested directory");

    let context = ProjectContext::discover(&nested).expect("discover project context");

    let canonical_root = repo
        .path()
        .canonicalize()
        .expect("canonical repository root");
    assert_eq!(context.repo_root(), canonical_root);
    assert_eq!(
        context.database_path(),
        canonical_root.join(".agileplus").join("agileplus.db")
    );
}

#[test]
fn rejects_a_path_outside_a_git_worktree() {
    let outside = tempfile::tempdir().expect("tempdir");

    assert!(ProjectContext::discover(outside.path()).is_err());
}
