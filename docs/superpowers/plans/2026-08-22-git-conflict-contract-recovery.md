# Git Conflict Contract Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recover exact Git merge-conflict paths from preserved PR #1022 without carrying unrelated configuration or lockfile churn.

**Architecture:** `GitVcsAdapter` has a pure parser for version-dependent
`git merge-tree` output. The live merge path uses unresolved index entries when
they exist, then uses the parser as a fallback. The parser is unit-tested; a
real divergent merge verifies the observable `MergeResult` contract.

**Tech Stack:** Rust, Cargo, Git CLI, git2, tokio, GitHub Actions.

---

### Task 1: Define exact conflict-path behavior

**Files:**
- Modify: `crates/agileplus-git/src/lib.rs`
- Test: `crates/agileplus-git/src/lib.rs`

- [x] **Step 1: Write failing parser tests**

```rust
assert_eq!(conflicts[0].file_path, "docs/legacy conflict.txt");
assert_eq!(conflicts[0].file_path, "docs/space conflict.txt");
assert_eq!(conflicts[0].file_path, "docs/quote\"name.txt");
assert_eq!(conflicts[0].file_path, "docs/café.txt");
```

- [x] **Step 2: Confirm the old adapter has no parser contract**

Run: `cargo test -p agileplus-git --lib parse_conflicts`

Expected: the new test cannot compile until `GitVcsAdapter::parse_conflicts` exists.

- [x] **Step 3: Implement the minimum parser**

```rust
fn parse_conflicts(raw: &str) -> Vec<ConflictInfo> {
    // Deduplicate exact paths from legacy rows, structured rows,
    // diagnostics, and quoted diff headers.
}
```

- [x] **Step 4: Verify the parser cases**

Run: `cargo test -p agileplus-git --lib parse_conflicts`

Expected: four passing tests.

### Task 2: Verify the live failed-merge contract

**Files:**
- Modify: `crates/agileplus-git/src/lib.rs`
- Modify: `crates/agileplus-git/tests/integration.rs`

- [x] **Step 1: Strengthen the divergent merge assertion**

```rust
assert!(result.conflicts.iter().any(|conflict| {
    conflict.file_path == "conflict.txt"
}));
```

- [x] **Step 2: Prefer the unresolved Git index after merge failure**

```rust
let conflicts = Self::unresolved_conflicts_in(dir);
let conflicts = if conflicts.is_empty() {
    Self::parse_conflicts(&format!("{stdout}\n{stderr}"))
} else {
    conflicts
};
```

- [x] **Step 3: Run the live flows**

Run: `cargo test -p agileplus-git test_merge_with_conflict` and
`cargo test -p agileplus-git test_detect_conflicts_divergent_branches`

Expected: both pass and report exact conflicted paths.

### Task 3: Promotion evidence and isolation

**Files:**
- Create: `kitty-specs/eco-048-git-conflict-contract-recovery/{spec.md,plan.md,tasks.md,meta.json}`
- Modify: `kitty-specs/INDEX.md`

- [x] **Step 1: Record scope and excluded deltas**

The spec must name #1022 as preserved evidence and explicitly exclude the
hook/configuration and lockfile changes.

- [ ] **Step 2: Resolve external baseline gates in separate semantic PRs**

Run: `cargo fmt --all -- --check`, `cargo test -p agileplus-git`, and
`cargo clippy -p agileplus-git --all-targets --all-features -- -D warnings`.

Expected: current failures are classified and repaired without broadening this
parser PR.

- [ ] **Step 3: Complete PR review and hosted gates**

Run: `gh pr checks 1030 --repo KooshaPari/AgilePlus`.

Expected: only after all required checks are green and review threads are
resolved may the PR be made ready and merged.
