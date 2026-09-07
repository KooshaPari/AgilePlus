# Repository-Local State Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make AgilePlus a repository-local tool: CLI, core, MCP, hooks, and materialization resolve one Git repository and its `docs/` plus `.agileplus/` state, rather than a daemon-wide database.

**Architecture:** A canonical Rust `ProjectContext` resolves a non-bare Git worktree, derives `<repo-root>/.agileplus/agileplus.db`, and supplies a canonical root to every persistent-state operation. The gRPC protocol carries this context explicitly, and MCP obtains it only from an approved client workspace root; it never falls back to `file:///`, a process working directory, or a global database. Git-tracked human artifacts live under `docs/agileplus/`; mutable local SQLite, locks, and runtime data live under `.agileplus/`.

**Tech Stack:** Rust, git2, SQLite/rusqlite, tonic/protobuf, Python 3/FastMCP, pytest, Cargo tests.

---

## Non-negotiable migration rules

- Do not delete, overwrite, or mutate `~/Library/Application Support/AgilePlus/state/core.db`.
- Do not use the current central records as an authoritative owner map. The central database has zero `projects` and zero `modules` rows.
- Keep the verified source database and each exported artifact immutable; record SHA-256 digests.
- Do not release, install, restart the LaunchAgents, or deploy to any harness until all implementation and acceptance gates below pass.

## Planned file structure

- Create: `crates/agileplus-git/src/project_context.rs` - canonical Git-root and repository-local path resolution.
- Modify: `crates/agileplus-git/src/lib.rs` - export `ProjectContext` and make VCS root canonical.
- Create: `crates/agileplus-git/tests/project_context.rs` - Git-root, nested-CWD, bare-repo, and path-escape tests.
- Modify: `crates/agileplus-grpc/src/runtime.rs` - reject implicit daemon-wide database defaults; accept an explicit project context only for one-shot/project-scoped startup.
- Modify: `crates/agileplus-grpc/src/main.rs` - initialize storage and VCS from one `ProjectContext`.
- Modify: `crates/agileplus-grpc/tests/runtime_config.rs` - prove no CWD/global default is accepted.
- Modify: `proto/agileplus/v1/common.proto` and each stateful request in `proto/agileplus/v1/core.proto` - carry a validated project-root identity.
- Modify: `crates/agileplus-proto/build.rs` and generated bindings only through the existing protobuf generation command - regenerate Rust/Python bindings; never hand-edit generated files.
- Modify: `python/src/agileplus_mcp/server.py` - bind each MCP session to a client-approved Git root, return only that root's URIs, and create a scoped gRPC client.
- Modify: `python/src/agileplus_mcp/grpc_client.py` - attach the project scope to every stateful request.
- Modify: `python/tests/test_server.py` - replace the `file:///` root expectation with absolute, session-scoped roots.
- Modify: `crates/agileplus-cli/src/main.rs` and `crates/agileplus-cli/src/commands/trace.rs` - use `ProjectContext` instead of relative/CWD database defaults.
- Modify: `crates/agileplus-mcp-intent/src/storage.rs` - remove `agileplus.db` CWD fallback.
- Modify: `crates/agileplus-git/src/materialize.rs` - materialize human-readable status/spec/audit artifacts below `docs/agileplus/<feature-slug>/`; retain machine state below `.agileplus/`.
- Modify: `scripts/seed-projects.sh` - replace central fleet seeding with a command that requires an explicit single `--repo` root; it must refuse a parent directory containing multiple repositories.
- Create: `scripts/export-central-state.py` - read-only, hash-recording export from a supplied database path.
- Create: `scripts/import-repo-state.py` - imports one reviewed manifest entry into one explicit Git repository and refuses an unknown owner.
- Create: `docs/agileplus/repo-state-contract.md` - tracked-state layout, ignore rules, migration runbook, and rollback procedure.
- Create: `docs/agileplus/central-state-owner-map.example.yaml` - schema-only example; no project records.

### Task 1: Define and test the canonical repository context

**Files:**
- Create: `crates/agileplus-git/src/project_context.rs`
- Modify: `crates/agileplus-git/src/lib.rs`
- Test: `crates/agileplus-git/tests/project_context.rs`

- [ ] **Step 1: Write failing context tests.**

```rust
#[test]
fn discovers_root_and_local_database_from_nested_directory() {
    let repo = tempfile::tempdir().unwrap();
    git2::Repository::init(repo.path()).unwrap();
    let nested = repo.path().join("src/nested");
    std::fs::create_dir_all(&nested).unwrap();
    let context = ProjectContext::discover(&nested).unwrap();
    assert_eq!(context.repo_root(), repo.path());
    assert_eq!(context.database_path(), repo.path().join(".agileplus/agileplus.db"));
}

#[test]
fn rejects_a_path_outside_a_git_worktree() {
    let outside = tempfile::tempdir().unwrap();
    assert!(ProjectContext::discover(outside.path()).is_err());
}
```

- [ ] **Step 2: Run the new test target and verify failure.**

Run: `cargo test -p agileplus-git --test project_context`

Expected: FAIL because `ProjectContext` does not exist.

- [ ] **Step 3: Implement the minimal context type.**

```rust
pub struct ProjectContext {
    repo_root: PathBuf,
    state_dir: PathBuf,
}

impl ProjectContext {
    pub fn discover(start: &Path) -> Result<Self, DomainError> {
        let repo = git2::Repository::discover(start)
            .map_err(|error| DomainError::Vcs(format!("not inside a git worktree: {error}")))?;
        let repo_root = repo.workdir()
            .ok_or_else(|| DomainError::Vcs("bare repositories cannot own AgilePlus state".into()))?
            .canonicalize()
            .map_err(|error| DomainError::Vcs(format!("canonicalize repository root: {error}")))?;
        Ok(Self { state_dir: repo_root.join(".agileplus"), repo_root })
    }

    pub fn repo_root(&self) -> &Path { &self.repo_root }
    pub fn database_path(&self) -> PathBuf { self.state_dir.join("agileplus.db") }
}
```

- [ ] **Step 4: Run the context tests.**

Run: `cargo test -p agileplus-git --test project_context`

Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/agileplus-git/src/project_context.rs crates/agileplus-git/src/lib.rs crates/agileplus-git/tests/project_context.rs
git commit -m "feat(git): resolve repository-local AgilePlus context"
```

### Task 2: Remove implicit core and CLI database selection

**Files:**
- Modify: `crates/agileplus-grpc/src/runtime.rs`
- Modify: `crates/agileplus-grpc/src/main.rs`
- Modify: `crates/agileplus-grpc/tests/runtime_config.rs`
- Modify: `crates/agileplus-cli/src/main.rs`
- Modify: `crates/agileplus-cli/src/commands/trace.rs`
- Modify: `crates/agileplus-mcp-intent/src/storage.rs`

- [ ] **Step 1: Write the failing core configuration test.**

```rust
#[test]
fn rejects_startup_without_an_explicit_project_root() {
    let error = CoreConfig::from_values(None, None, None).unwrap_err();
    assert!(error.contains("project root is required"));
}
```

- [ ] **Step 2: Run the test and verify failure.**

Run: `cargo test -p agileplus-grpc --test runtime_config rejects_startup_without_an_explicit_project_root`

Expected: FAIL because the current default is `.agileplus/agileplus.db`.

- [ ] **Step 3: Change configuration to accept a project root, discover `ProjectContext`, and derive the database only from that context.**

```rust
pub struct CoreConfig {
    pub bind: SocketAddr,
    pub project_root: PathBuf,
}

pub fn context(&self) -> Result<ProjectContext, String> {
    ProjectContext::discover(&self.project_root).map_err(|error| error.to_string())
}
```

Remove the CWD fallbacks in `trace.rs` and `agileplus-mcp-intent/src/storage.rs`; each caller must pass a `&ProjectContext` or an explicit `--repo` path. Preserve an explicit `--db` only for the read-only export tool, never as a daemon default.

- [ ] **Step 4: Run focused Rust tests.**

Run: `cargo test -p agileplus-grpc --test runtime_config && cargo test -p agileplus-cli && cargo test -p agileplus-mcp-intent`

Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/agileplus-grpc crates/agileplus-cli crates/agileplus-mcp-intent
git commit -m "fix(runtime): require repository-local state context"
```

### Task 3: Scope MCP and gRPC requests to one repository

**Files:**
- Modify: `proto/agileplus/v1/common.proto`
- Modify: `proto/agileplus/v1/core.proto`
- Modify: `crates/agileplus-proto/build.rs`
- Modify: `python/src/agileplus_mcp/grpc_client.py`
- Modify: `python/src/agileplus_mcp/server.py`
- Test: `python/tests/test_server.py`

- [ ] **Step 1: Add a failing MCP test for a session root.**

```python
async def test_workspace_roots_are_scoped_to_the_client_repository(monkeypatch, tmp_path):
    repo = tmp_path / "project"
    (repo / ".git").mkdir(parents=True)
    monkeypatch.setattr(server, "_session_project_root", repo)
    assert await server.get_workspace_roots() == {
        "roots": [
            {"uri": repo.as_uri(), "name": "project-root"},
            {"uri": (repo / ".agileplus").as_uri(), "name": "agileplus-data"},
        ]
    }
```

- [ ] **Step 2: Run the test and verify failure.**

Run: `uv --project python run pytest python/tests/test_server.py -q`

Expected: FAIL because the server returns `file:///` and relative paths.

- [ ] **Step 3: Add `ProjectScope { string canonical_repo_root = 1; }` to `common.proto`, embed it in every stateful core request, and regenerate bindings through the repository's protobuf generation command.**

The server must accept exactly one client-provided file root, canonicalize it as a non-bare Git worktree, retain it per MCP session, and pass its canonical absolute path in `ProjectScope`. The core must rediscover and compare the canonical root before opening the repository-local database. Reject missing, relative, non-file, and multi-root scopes.

- [ ] **Step 4: Run protocol and Python tests.**

Run: `cargo test -p agileplus-proto && uv --project python run pytest python/tests/test_server.py -q`

Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add proto crates/agileplus-proto python/src/agileplus_mcp python/tests/test_server.py
git commit -m "feat(mcp): bind stateful requests to one repository"
```

### Task 4: Materialize project documentation and isolate machine state

**Files:**
- Modify: `crates/agileplus-git/src/materialize.rs`
- Test: `crates/agileplus-git/src/materialize.rs`
- Create: `docs/agileplus/repo-state-contract.md`

- [ ] **Step 1: Write a failing materialization assertion.**

```rust
assert!(repo_root.join("docs/agileplus/my-feature/status.md").exists());
assert!(repo_root.join("docs/agileplus/my-feature/audit.jsonl").exists());
assert!(!repo_root.join("kitty-specs/my-feature/status.md").exists());
```

- [ ] **Step 2: Run the materialization test and verify failure.**

Run: `cargo test -p agileplus-git materialize_feature`

Expected: FAIL because the current writer targets `kitty-specs/<slug>/`.

- [ ] **Step 3: Change feature and WP artifact paths.**

```rust
let feature_dir = repo_root.join("docs").join("agileplus").join(&feature.slug);
let wp_path = repo_root.join("docs").join("agileplus").join(feature_slug)
    .join("work-packages").join(format!("{}.json", wp.id));
```

Write the layout and ignored mutable paths into `docs/agileplus/repo-state-contract.md`.

- [ ] **Step 4: Run materialization tests.**

Run: `cargo test -p agileplus-git`

Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/agileplus-git/src/materialize.rs docs/agileplus/repo-state-contract.md
git commit -m "feat(artifacts): materialize AgilePlus records in project docs"
```

### Task 5: Create a reviewed, lossless migration workflow

**Files:**
- Create: `scripts/export-central-state.py`
- Create: `scripts/import-repo-state.py`
- Create: `docs/agileplus/central-state-owner-map.example.yaml`
- Modify: `scripts/seed-projects.sh`
- Test: `python/tests/migration/test_export_central_state.py`

- [ ] **Step 1: Write failing migration tests.**

```python
def test_import_refuses_an_owner_not_in_the_reviewed_manifest(tmp_path):
    result = run_import(tmp_path, owner_map={})
    assert result.returncode != 0
    assert "missing reviewed owner" in result.stderr

def test_export_records_the_source_database_sha256(tmp_path):
    result = run_export(tmp_path / "core.db")
    assert re.fullmatch(r"[0-9a-f]{64}", result.json["source_db_sha256"])
```

- [ ] **Step 2: Run the migration tests and verify failure.**

Run: `uv --project python run pytest python/tests/migration/test_export_central_state.py -q`

Expected: FAIL because neither tool exists.

- [ ] **Step 3: Implement export/import safeguards.**

`export-central-state.py` opens SQLite using `mode=ro`, emits JSON containing database SHA-256, each selected feature, its WPs, and its audit rows. `import-repo-state.py` requires `--repo`, `--manifest`, and `--export`; it verifies that the manifest's canonical repo path equals the discovered Git root, validates the export digest, verifies the source audit chain, and refuses to create anything when the feature owner is absent.

Replace `seed-projects.sh` with a compatibility wrapper that exits nonzero unless `--repo <git-root>` identifies exactly one Git repository; it must never enumerate or seed a fleet.

- [ ] **Step 4: Run migration tests.**

Run: `uv --project python run pytest python/tests/migration/test_export_central_state.py -q`

Expected: PASS.

- [ ] **Step 5: Commit.**

```bash
git add scripts/export-central-state.py scripts/import-repo-state.py scripts/seed-projects.sh tests/migration docs/agileplus
git commit -m "feat(migration): require reviewed repository ownership"
```

### Task 6: Verify two-repository isolation and document the final fleet gate

**Files:**
- Create: `tests/e2e/repository_isolation.rs`
- Modify: `docs/agileplus/repo-state-contract.md`

- [ ] **Step 1: Write the end-to-end isolation test.**

```rust
#[test]
fn repository_a_cannot_read_or_write_repository_b_state() {
    let repo_a = test_git_repo("a");
    let repo_b = test_git_repo("b");
    create_feature(&repo_a, "only-a");
    assert!(list_features(&repo_b).unwrap().is_empty());
    assert!(!repo_b.path().join(".agileplus/agileplus.db").exists());
}
```

- [ ] **Step 2: Run it and verify failure before wiring all callers.**

Run: `cargo test -p agileplus-integration-tests --test repository_isolation`

Expected: FAIL until CLI, core, and MCP use `ProjectContext` end-to-end.

- [ ] **Step 3: Run complete local verification.**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && uv --project python run pytest python/tests -q`

Expected: PASS.

- [ ] **Step 4: Add fleet gate documentation.**

Document that installer/release work begins only after all local checks pass and the two-repository test passes. The fleet acceptance matrix must independently verify install, upgrade, daemon restart, MCP root binding, hook execution, and repository-local round trip for every approved harness, kooshapari-desk, and this device.

- [ ] **Step 5: Commit.**

```bash
git add tests/e2e/repository_isolation.rs docs/agileplus/repo-state-contract.md
git commit -m "test: prove repository-local AgilePlus isolation"
```

## Self-review

- The plan covers core, CLI, MCP, hooks/materialization, migration, documentation, and fleet-gate sequencing.
- It does not authorize deletion, central-data retirement, daemon restart, build promotion, push, PR creation, or deployment.
- It separates the independent release/fleet distribution work from the repository-state migration. The latter must pass first.
