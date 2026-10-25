# GitHub JWT Dependency Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the unused Octocrab JWT dependency chain so Cargo-deny passes the Marvin Attack advisory gate without suppression.

**Architecture:** `agileplus-github` retains its raw `reqwest` client in `src/client.rs` and synchronization in `src/sync.rs`. The unconsumed Octocrab read facade is removed rather than reimplemented or switched to another JWT backend.

**Tech Stack:** Rust, Cargo.lock, cargo-deny, reqwest.

---

### Task 1: Prove and remove the unused JWT facade

**Files:**

- Delete: `crates/agileplus-github/src/octo.rs`
- Modify: `crates/agileplus-github/src/lib.rs:1-14`
- Modify: `crates/agileplus-github/Cargo.toml:22`
- Modify: `Cargo.lock`

- [x] **Step 1: Record the pre-change dependency proof**

Run: `cargo tree --locked -i rsa@0.9.10`

Expected: `agileplus-cli -> agileplus-github -> octocrab -> jsonwebtoken -> rsa`.

- [x] **Step 2: Verify the facade has no workspace consumer**

Run: `rg -n 'GitHubClient|list_open_issues|list_open_prs|octocrab' --glob '*.rs' crates tests`

Expected: Octocrab-specific references are limited to `crates/agileplus-github/src/octo.rs`, its module declaration, and its dependency declaration.

- [x] **Step 3: Remove the unused facade and dependency**

Delete `src/octo.rs`; remove `pub mod octo;` and its `GitHubClient` re-export from `src/lib.rs`; remove `octocrab = "0.54"` from the crate manifest. Do not modify `client.rs`, `sync.rs`, or add a Cargo-deny ignore.

- [x] **Step 4: Regenerate the locked graph and prove retained behavior**

Run: `cargo check -p agileplus-github && cargo test --locked -p agileplus-github`

Expected: compilation and GitHub synchronization tests pass using the raw client.

- [x] **Step 5: Prove security closure**

Run: `if cargo tree --locked -i rsa@0.9.10; then print -u2 'rsa remains in the graph'; exit 1; fi; cargo deny check advisories`

Expected: the inverse tree reports no package and advisory checking passes without an exception.

- [ ] **Step 6: Commit only the security slice**

Run: `git add crates/agileplus-github/Cargo.toml crates/agileplus-github/src/lib.rs crates/agileplus-github/src/octo.rs Cargo.lock docs/superpowers/specs/2026-08-29-github-jwt-removal-design.md docs/superpowers/plans/2026-08-29-github-jwt-removal.md && git commit -m "fix(security): remove unused GitHub JWT facade"`

Expected: one focused commit with no Cargo-deny suppression.

### Task 2: Verify the complete recovery boundary

**Files:**

- Verify only: workspace manifests and source

- [ ] **Step 1: Run the local full suite**

Run: `cargo fmt --check && cargo test --workspace --locked && cargo check --workspace --locked`

Expected: all local checks pass.

- [ ] **Step 2: Separate remaining policy work**

Run: `cargo deny check bans`

Expected: the 13 wildcard path-dependency manifest groups remain recorded for a separate manifest-only PR; do not weaken `deny.toml`.
