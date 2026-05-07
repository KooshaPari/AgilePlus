# AgilePlus Worktree Usage Audit — 2026-05-04

Found 50 markdown files mentioning worktree/working tree.

## `001-spec-driven-development-engine/data-model.md`
- 136: | worktree_path | TEXT | nullable | Absolute path to worktree |

## `001-spec-driven-development-engine/plan.md`
- 134: │   │       ├── worktree.rs
- 296: 1. Create worktree: .worktrees/001-feature-WP01/
- 314: 2. WPs with no overlapping files → parallel worktrees
- 368: │                   │    │ git:create-worktree          │
- 434: | Roots | Workspace boundaries: feature dirs, worktree paths, config dirs |

## `001-spec-driven-development-engine/research.md`
- 17: **Rationale**: git2-rs is the standard Rust binding for libgit2. Supports worktree operations (create, list, prune), branch management, and commit inspection. Used by cargo itself. Covers all our needs: worktree creation (FR-010), artifact read/write (FR-014), branch merging (FR-006).
- 19: - gitoxide (pure Rust, faster for some operations, but worktree API is still maturing)
- 43: **Rationale**: Both agents support non-interactive/headless modes. Claude Code supports `--print` mode and can be invoked with system prompts. Codex supports similar batch execution. AgilePlus constructs the prompt (WP goal, FR references, governance rules) and invokes the agent CLI in a worktree directory. Agent output (commits, PRs) is observed via GitHub API polling.

## `001-spec-driven-development-engine/spec.md`
- 10: AgilePlus is a local, git+SQLite-backed spec-driven development engine that runs as a CLI sidecar alongside Claude Code and Codex. It harmonizes the best of OpenSpec (simplicity, ~3 commands to plan), spec-kitty (structured granularity, worktree isolation, kanban tracking), bmad (enterprise depth, role-based agents), and GSD (automation, parallel execution) into a streamlined 7-command workflow.
- 12: AgilePlus does not build a custom agent engine. It orchestrates existing AI coding agents (Claude Code, Codex) through slash commands, dispatching work to subagents in isolated worktrees. A Plane.so-based (or equivalent OSS) web UI provides visual project management, auditing, and dashboards — not built from scratch.
- 57: - Q: How should resource conflicts between parallel WPs be handled? → A: Worktree isolation by default + dependency-aware scheduling when WPs declare shared files.
- 87: A developer runs `implement` on a planned feature. AgilePlus spawns 1-3 subagents per worktree (using Claude Code/Codex), each working on assigned work packages in isolated git worktrees. Each agent creates a PR with the original goal/prompt in the PR description and detailed commit messages. The system awaits Coderabbit auto-review, then loops agents on review comments and CI fixes until the PR is green. Once a WP's PR passes, the system moves to the next WP.
- 91: **Independent Test**: Can be tested by running `implement` on a single WP and verifying: worktree created, subagent dispatched, PR created with goal context, Coderabbit review awaited, agent loops on feedback, PR merges when green.
- 95: 1. **Given** a feature with 3 planned WPs, **When** the user runs `implement`, **Then** the system creates isolated worktrees for each WP, spawns subagents (1-3 per worktree), and each agent begins working on its assigned WP.
- 135: The developer runs `ship` to merge the feature into the target branch, clean up worktrees, archive the feature, and record completion in the audit log. Optionally, they run `retrospective` which auto-generates learnings from the feature's history (time per WP, review cycles, common issues) and feeds insights back into the constitution/governance for future features.
- 139: **Independent Test**: Can be tested by running `ship` on a validated feature and verifying: feature branch merged, worktrees cleaned, SQLite records updated, audit log finalized. Then `retrospective` generates a summary with actionable learnings.

## `001-spec-driven-development-engine/tasks/WP00-proto-scaffold.md`
- 107: - **Purpose**: Establish the repository skeleton so all subsequent subtasks have a clean working tree with proper metadata and ignore rules.
- 124: - **Validation**: `git status` shows clean working tree; `ls -la` shows all three files.

## `001-spec-driven-development-engine/tasks/WP01-rust-workspace-scaffold.md`
- 209: - `agileplus-git/src/lib.rs`: `pub mod worktree;`, `pub mod repository;`, `pub mod artifact;` stubs

## `001-spec-driven-development-engine/tasks/WP03-domain-feature-state-machine.md`
- 265: pub worktree_path: Option<String>,

## `001-spec-driven-development-engine/tasks/WP04-domain-governance-audit.md`
- 632: 2. Extract reusable patterns: worktree discipline, quality gates, agent workflow rules

## `001-spec-driven-development-engine/tasks/WP05-port-traits.md`
- 129: **Worktree operations** (FR-010):
- 130: - `async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> Result<PathBuf, DomainError>` -- returns worktree absolute path
- 131: - `async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>, DomainError>`
- 132: - `async fn cleanup_worktree(&self, worktree_path: &Path) -> Result<(), DomainError>`
- 149: - `WorktreeInfo { path: PathBuf, branch: String, feature_slug: String, wp_id: String }`
- 158: - Worktree path convention matches plan.md: `.worktrees/<feature-slug>-<WP-id>/`.
- 172: - `AgentTask { wp_id: String, feature_slug: String, prompt_path: PathBuf, worktree_path: PathBuf, context_files: Vec<PathBuf> }`
- 189: - `AgentTask` includes all context an agent needs: WP prompt, worktree path, context files (spec.md, plan.md, data-model.md).

## `001-spec-driven-development-engine/tasks/WP07-git-adapter.md`
- 40: Implement the git adapter in `crates/agileplus-git/` that fulfills the `VcsPort` trait from WP05. This adapter uses `git2` (libgit2 Rust bindings) for all git operations -- no shelling out to the `git` CLI. It provides worktree management, branch operations, artifact read/write, and git history scanning for `rebuild_from_git` support.
- 45: 2. Worktree operations create/list/cleanup worktrees at `.worktrees/<feature-slug>-<WP-id>/`.
- 55: - **Worktree convention**: `.worktrees/<feature-slug>-<WP-id>/` relative to repo root. See `plan.md` section 3.
- 104: ### T039: Implement worktree operations
- 106: **Purpose**: Manage isolated worktrees for parallel WP implementation. Each WP gets its own worktree so agents don't conflict.
- 109: 1. Create `crates/agileplus-git/src/worktree.rs` (or implement in lib.rs).
- 111: 2. **create_worktree**:
- 113: async fn create_worktree(&self, feature_slug: &str, wp_id: &str) -> Result<PathBuf, DomainError>

## `001-spec-driven-development-engine/tasks/WP08-agent-dispatch-adapter.md`
- 51: 5. Dispatch logic selects agent from config, creates worktree (via VcsPort), injects prompt, spawns 1-3 subagents.
- 66: - **Worktree dependency**: This adapter calls `VcsPort.create_worktree()` to set up the agent's working directory.
- 142: - `send_instruction`: Write instruction to agent's stdin (if supported) or create instruction file in worktree.
- 175: .current_dir(&task.worktree_path);
- 197: 7. Implement `extract_commits()`: parse git log in worktree for new commits since dispatch.
- 232: .current_dir(&task.worktree_path);
- 236: - Write combined prompt to a temp file in the worktree.
- 254: ### T047: Implement `dispatch.rs`: agent selection, worktree setup, multi-agent spawn

## `001-spec-driven-development-engine/tasks/WP12-cli-plan-implement.md`
- 72: 4. **`agileplus implement`** takes a feature slug (and optionally a WP ID), creates worktrees for ready WPs, dispatches agents, creates PRs with structured descriptions, and orchestrates the review-fix loop until PRs are green.
- 94: - Agent dispatch must be cancellable. If the user presses Ctrl+C, the system should attempt to clean up worktrees and record a partial audit entry.
- 232: ### Subtask T069 -- Implement `commands/implement.rs`: Worktree creation, agent dispatch, PR creation
- 234: - **Purpose**: Implement the implement command that orchestrates the full agent workflow: check dependencies, create worktrees, dispatch agents, create PRs, and manage the review-fix loop.
- 244: - Create worktree via VcsPort: `.worktrees/{slug}-{wp_id}/`.
- 280: - For `doing`: check if worktree exists, re-attach to agent output.
- 285: - Clean up worktrees.
- 294: - Worktree cleanup happens after PR merge, not after agent completes. The worktree is needed during the review-fix loop.

## `001-spec-driven-development-engine/tasks/WP13-cli-validate-ship-retro.md`
- 50: worktrees, archives the feature directory, and finalizes the audit chain.
- 85: - WP07 provides: Git adapter (worktree ops, branch merge, artifact read/write).
- 229: worktrees, archives the feature spec directory, and writes the final audit entry.
- 266: - Clean up worktrees via VcsPort: `cleanup_worktree(feature, wp)` for each WP.
- 294: - Worktree already cleaned up (manual deletion): skip cleanup, log info.
- 297: operations, verify worktree cleanup and audit finalization.

## `001-spec-driven-development-engine/tasks/WP14-grpc-mcp-integration.md`
- 659: 2. Declare per-feature roots: feature dir, worktree paths, config dirs

## `001-spec-driven-development-engine/tasks/WP16-bdd-integration-tests.md`
- 160: Scenario: FR-004 - Implement dispatches agent to worktree
- 163: Then a worktree is created for WP01
- 170: And the agent has committed code in the WP01 worktree
- 806: 4. Clean up test data after each test: delete test features, remove test worktrees.

## `001-spec-driven-development-engine/tasks/WP20-hidden-subcommands.md`
- 37: # Start work in the designated worktree for this package
- 164: | `git:create-worktree` | Create a new git worktree for a WP via VcsPort |
- 166: | `git:merge-and-cleanup` | Merge a worktree branch into main and remove the worktree |
- 332: `git:create-worktree`:
- 334: - Call `VcsPort.create_worktree(wp_id, branch_name)`.
- 335: - Return the worktree path.
- 346: - Remove the worktree via `VcsPort.remove_worktree()`.

## `001-spec-driven-development-engine/tasks/WP21-cli-triage-queue.md`
- 54: # Start work in the designated worktree for this package
- 154: git-create-worktree.md

## `001-spec-driven-development-engine/tasks.md`
- 216: - [x] T026 [P] Define `VcsPort` trait in `ports/vcs.rs`: worktree create/cleanup, branch ops, artifact read/write
- 261: - rebuild_from_git reads: meta.json, audit/chain.jsonl, evidence/** from git working tree
- 277: **Goal**: Implement git adapter for worktree management, branch ops, and artifact read/write.
- 278: **Independent Test**: Integration tests pass for worktree create/cleanup, artifact read/write, branch merge in a temp repo.
- 287: - [x] T039 Implement worktree operations: create_worktree(feature, wp), list_worktrees, cleanup_worktree
- 294: - Worktree paths: `.worktrees/<feature-slug>-<WP-id>/`
- 305: - git2 worktree API quirks: test on macOS + Linux, handle case-insensitive filesystems
- 325: - [x] T047 Implement `dispatch.rs`: select agent (from config), create worktree, inject prompt, spawn 1-3 subagents

## `002-org-wide-release-governance-dx-automation/spec.md`
- 148: - **FR-007**: System MUST prevent publishing from dirty working trees — all automation publishes from clean, committed state only.

## `002-org-wide-release-governance-dx-automation/tasks/WP01-cli-scaffold-adapter-interface.md`
- 222: ErrDirtyWorkTree    = errors.New("work tree contains uncommitted changes")

## `002-org-wide-release-governance-dx-automation/tasks/WP04-crates-adapter.md`
- 384: Err:          ErrDirtyWorkTree,
- 492: - If any output, return `ErrDirtyWorkTree`

## `002-org-wide-release-governance-dx-automation/tasks/WP08-cli-audit-matrix.md`
- 224: - Worktree directories (match pattern `*-wtrees/*`)
- 329: 4. **Repo Discovery**: Create test directory structure with hidden dirs, node_modules, worktrees, and verify correct repos are found. Test monorepo detection.

## `002-org-wide-release-governance-dx-automation/tasks/WP13-pilot-rollout.md`
- 59: - No merges to canonical repos during pilot; use worktrees if testing is needed
- 69: - Ensure repo is on canonical `main` branch (from governance rules, work in worktree if modifying)

## `002-org-wide-release-governance-dx-automation/tasks/WP15-documentation-polish.md`
- 202: - Pheno checks: working tree clean, lint passes, tests pass
- 529: - Dirty working tree (uncommitted changes)
- 544: - **After:** `Git error: working tree has uncommitted changes. Commit or stash changes before promoting. Run 'git status' to see what's pending.`
- 556: - Run pheno on dirty working tree → check error message

## `002-org-wide-release-governance-dx-automation/tasks.md`
- 139: - Never `--allow-dirty` — fail if working tree is dirty

## `013-phenotype-infrakit-stabilization/plan.md`
- 13: | P0 Discovery | Pin down current state — open PRs, dirty worktrees, stale branches, broken builds. | Live audit memo (counts of PRs/worktrees/branches/disk), per-crate maturity matrix, duplicate-functionality matrix, dependency graph. | `cargo metadata --workspace` clean; PR/worktree/branch counts recorded; matrix lists each of 19 crates. |
- 15: | P2 Build | Land the consolidation — close P0 backlog, migrate crates, merge duplicates, normalize errors/MSRV. | Single workspace builds clean; duplicates removed; preflight backlog (10 PRs + 2 worktrees + ~20 stale branches) drained. | `cargo build --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --check` all green on `main`. |
- 25: | P0 | WP-000 | Preflight: drain 10 open PRs (`#544–#563`), resolve 2 dirty worktrees (`cache-adapter-impl`, `phenotype-crypto-complete-v2`), prune ~20 stale branches, run `cargo clean` to reclaim ~1.3 GB. | — |
- 61: | WP-000 | 15–25 | 1 batch (3 agents: PR drain / worktree resolve / branch prune) | 8–15 min |
- 119: - Spec context: [`spec.md`](./spec.md) §"Audit Update — 2026-04-02" for live counts (10 PRs, 2 worktrees, ~20 stale branches, 1.8 GB → 0.5 GB after `cargo clean`).

## `013-phenotype-infrakit-stabilization/spec.md`
- 149: - **Worktrees**: 2 active (cache-adapter-impl detached HEAD, phenotype-crypto-complete-v2)
- 165: 2. **Resolve worktrees**:

## `013-phenotype-infrakit-stabilization/tasks.md`
- 19: - **File scope:** `repos/phenotype-infrakit/` (entire repo); GitHub PRs `#544–#563`; worktrees `cache-adapter-impl`, `phenotype-crypto-complete-v2`; ~20 stale branches.
- 22: - Both noted worktrees rebased onto `main` and either merged or closed with rationale.

## `014-observability-stack-completion/plan.md`
- 161: Phase outputs feed `tasks.md` (work-package detail). Implementer agents read tasks.md and produce code in topic worktrees under `repos/PhenoObservability-wtrees/<topic>/`. Quality gates run via `cargo test --workspace` + `cargo clippy --workspace -- -D warnings` + `cargo fmt --check` + `cargo deny check advisories`. No human review checkpoints; review is agent-driven via `/review` and `/security-review`.

## `014-observability-stack-completion/tasks.md`
- 148: - **Effort:** Cross-stack (8-15 tool calls, ~6 min planning; implementation in worktree)
- 149: - **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-006-tracely-w3c/`
- 172: - **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-007-logging-traceid/`
- 194: - **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-008-metrics-corr/`
- 216: - **Implementation worktree:** `repos/Phench-wtrees/wp-009-obs-emission/` (or PhenoObservability if Phench already absorbed).
- 237: - **Implementation worktree:** `repos/PhenoObservability-wtrees/wp-010-profile-integration/`

## `015-plugin-system-completion/plan.md`
- 19: | WP-002 | Git VCS adapter implementing the trait: clone/fetch/commit/push/branch/worktree + auth + retries, integration tests with temp repos (T013–T024) | WP-001 | agileplus-plugin-git | 12–16 tool calls, 8–12 min |

## `015-plugin-system-completion/tasks.md`
- 59: - Git operations: clone, fetch, commit, push, branch management, worktree operations
- 75: - [ ] T019 Implement worktree operations: create, list, cleanup

## `017-cli-tools-consolidation/tasks.md`
- 147: - Worktree management for parallel development
- 152: Complete forgecode as the git workflow framework, integrated with Cmdra. forgecode handles branch management, PR creation, review loops, and worktree operations — all accessible through Cmdra-based CLI commands.
- 160: - [ ] T044 Implement worktree management: create, list, cleanup worktrees

## `021-polyrepo-ecosystem-stabilization/plan.md`
- 17: ├── P1.7: Worktree discipline ──────────────────┤
- 134: - [ ] Worktree discipline documented

## `021-polyrepo-ecosystem-stabilization/research.md`
- 75: - **Active worktrees**: 8 worktrees (some empty, some detached HEAD)
- 81: - Delete empty worktree directories
- 87: - heliosCLI: 4 active worktrees need finish/close decisions
- 149: 4. **Governance**: AgilePlus spec completion, worktree discipline, branch hygiene

## `021-polyrepo-ecosystem-stabilization/spec.md`
- 11: - **Depends On**: 012-github-portfolio-triage, 013-phenotype-infrakit-stabilization, eco-001-worktree-remediation, eco-002-branch-consolidation
- 19: - **Stale artifacts**: 22 GB in build artifacts, 50+ stale branches, empty worktree directories
- 30: 2. **Local state degraded**: 89 GB disk, 7/9 repos dirty, worktrees in disarray
- 47: - Establish **worktree discipline** with documented rules
- 88: | P1.7: Establish worktree discipline — document in WORKTREES.md | 2h | None |
- 143: - **Worktree discipline**: Documented and enforced
- 163: | WP015 | Worktree remediation and discipline | All | specified |
- 186: - Related: eco-001-worktree-remediation

## `021-polyrepo-ecosystem-stabilization/tasks.md`
- 155: ## WP-07: Establish worktree discipline
- 158: - Read: [`WORKTREES.md`, worktree directories `docs/`, `infrastructure/`, `phenotype-errors/`, `cache-adapter-impl`, `phenotype-crypto-complete`]
- 159: - Write: [`WORKTREES.md`, worktree directories `docs/`, `infrastructure/`, `phenotype-errors/`, `cache-adapter-impl`, `phenotype-crypto-complete`]
- 165: - [ ] Worktree rules and maximum concurrent worktree guidance are documented.
- 166: - [ ] Empty, detached, or stale worktrees are cleaned, investigated, merged, or closed as appropriate.
- 170: - [ ] T052 — Document worktree rules in WORKTREES.md — `WORKTREES.md`
- 171: - [ ] T053 — Clean empty worktree directories (docs/, infrastructure/, phenotype-errors/) — `docs/`, `infrastructure/`, `phenotype-errors/`
- 172: - [ ] T054 — Investigate cache-adapter-impl worktree (detached HEAD?) — `cache-adapter-impl/`

## `022-batch13-repo-remediation/spec.md`
- 18: - All repos share the same orphaned worktree branch (chore/gitignore-and-test-infra)
- 27: - **Orphaned worktrees** — All share same stale branch with deleted archive files
- 62: 3. Remove orphaned worktree references
- 93: | Losing worktree references | Low | Document before archival |

## `022-batch13-repo-remediation/tasks.md`
- 21: - Orphaned worktree references cleaned up
- 24: Move 5 empty repos to .archive/ directory. Update INDEX.md to reflect archival. Clean up any worktree references.
- 33: - [ ] T007 Clean up orphaned worktree references
- 39: - Losing worktree references: Document before archival

## `archive/006-helioscli-completion/spec.md`
- 71: - **Worktrees**: 4 active (governance-migration, codex-rs-core WIP, ci-failures, decompose-key-router)
- 72: - **Stale worktrees**: 1 (dep-drift-python — prunable)
- 84: - Worktree `chore-govern-pi`, branch `chore/governance-migration-hc`
- 88: - Parked worktree `wip/codex-rs-core`
- 92: - Worktree `fix-ci-failures`
- 96: - Worktree `decompose-key-router`
- 103: - [ ] Decide on 4 worktrees: finish or close each
- 104: - [ ] Delete prunable worktree (dep-drift-python)

## `archive/007-thegent-completion/spec.md`
- 94: - **Worktrees**: 3 checked out (bun-migrate, dotagents + primary)
- 130: - [ ] Merge or close sibling worktrees (bun-migrate, dotagents)

## `eco-001-worktree-remediation/spec.md`
- 8: Live worktree governance now embedded in:
- 9: - Phenotype/CLAUDE.md "Worktree Rule"
- 10: - repos/CLAUDE.md worktree discipline
- 11: - repos/.worktrees/ + repos/<repo>-wtrees/<topic> conventions
- 15: # Specification: Worktree Remediation
- 16: **Slug**: worktree-remediation | **Date**: 2026-03-29 | **State**: completed
- 19: Archived legacy worktrees - completed 2026-03-28/29
- 26: - [x] Implement worktree_governance_inventory.py with conformance checks

## `eco-012-orgops-capital-ledger/spec.md`
- 13: 5. **Worktree chaos** — no systematic gix-based worktree management; canonical repo is mixed with development
- 15: The result: agents waste cycles on failed API calls, duplicate secrets across worktrees, and have no concept of organizational "capital."
- 21: - **CI/CD**: Needs secrets injected into worktrees without manual .env management
- 47: The system SHALL export validated secrets from SQLite to `.env` files in project worktrees. The `.env` file is generated (gitignored), not hand-maintained.
- 52: ### FR-GIT-001: Worktree Management
- 53: The `phenotype-git-core` crate SHALL provide gix-based worktree creation, listing, and pruning. Agents SHALL always work in worktrees; canonical pulls SHALL target release branches only.
- 56: The system SHALL track which release branch each project's canonical repo is on, so agents can show users new features without polluting development worktrees.
- 81: - `gix 0.71`: Already in workspace deps — use for worktree management

## `eco-012-orgops-capital-ledger/tasks.md`
- 13: | Git & Worktrees | WP05 | P1 | After WP01 |
- 138: ## WP05 — phenotype-git-core Worktree Extension
- 141: **Goal**: Extend the stub `phenotype-git-core` with gix-based worktree management.
- 144: - Write: [crates/phenotype-git-core/src/lib.rs, crates/phenotype-git-core/src/worktree.rs, phenotype-git-core, worktrees/<project>/<branch>, worktrees/]
- 146: - [ ] T033: Implement `create_worktree(project, branch)` — gix worktree creation at `.worktrees/<project>/<branch>`
- 147: - [ ] T034: Implement `list_active()` — scan `.worktrees/` and parse gix state
- 148: - [ ] T035: Implement `prune_stale(max_age_days)` — remove worktrees older than threshold
- 150: - [ ] T037: Add tests for worktree lifecycle (create, list, prune, release tracking)

## `phenosdk-wave-a-contracts/plan.md`
- 13: - `worktrees/phenoSDK/main/src/pheno/ports`

## `phenosdk-wave-a-contracts/spec.md`
- 4: Extract stable ports and public DTOs from `worktrees/phenoSDK/main/src/pheno/ports` (and related schemas) into versioned contract artifacts consumable by Phenotype polyglot clients.

## `phenosdk-wave-a-contracts/tasks/WP01-initial-implementation.md`
- 20: - `worktrees/phenoSDK/main/src/pheno/ports`

## `portfolio-audit-kooshapari-2026/plan.md`
- 16: - `Phenotype/repos/worktrees/phenoSDK/main`

## `portfolio-audit-kooshapari-2026/spec.md`
- 9: - Canonical SDK tree: `Phenotype/repos/worktrees/phenoSDK/main` (clone of `github.com/KooshaPari/phenoSDK`). Legacy `pheno-sdk` remote is empty; do not block on it.

## `portfolio-audit-kooshapari-2026/tasks/WP01-initial-implementation.md`
- 23: - `Phenotype/repos/worktrees/phenoSDK/main`
