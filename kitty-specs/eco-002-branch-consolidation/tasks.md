---
spec_id: eco-002
title: "Branch Consolidation — Full Work Package Structure"
created_at: "2026-05-02T00:00:00Z"
priority: high
status: planning
type: operational
---

## What

AgilePlus maintains a polyrepo workspace with multiple VCS integration points (Plane.so, GitHub, NATS). Over time, feature branches, worktrees, and merged/closed branches accumulate without a systematic lifecycle policy, leading to:

- **Cluttered namespace** — stale branches obscure active work and inflate CI trigger noise.
- **Inconsistent protection rules** — `main` and `develop` lack uniform enforcement across all repos.
- **Orphaned worktrees** — feature branches merged via squash leave local worktrees dangling.
- **No automation** — branch lifecycle is manual, error-prone, and not self-healing.

## Why

Operational efficiency and developer experience degrade without branch hygiene. Eco-002 directly reduces:

- CI noise from zombie branch pipelines
- Cognitive load on contributors navigating branch lists
- Risk of accidentally pushing to protected branches
- Cost and latency from orphaned build triggers

This spec implements a closed-loop branch lifecycle: inventory → categorize → prune → enforce → automate.

---

## Work Package Index

| # | WP ID | Title | Priority | Dependencies |
|---|-------|-------|----------|--------------|
| 1 | eco-002-WP01 | Branch Inventory & Categorization | high | — |
| 2 | eco-002-WP02 | Stale & Merged Branch Cleanup | high | WP01 |
| 3 | eco-002-WP03 | Branch Naming Convention Enforcement | medium | WP01 |
| 4 | eco-002-WP04 | Branch Protection Rules Consistency | high | WP01 |
| 5 | eco-002-WP05 | Automation for Branch Lifecycle (Git Hooks + Scripts) | medium | WP02, WP03 |
| 6 | eco-002-WP06 | GitHub Actions Workflow: Branch Sentinel | medium | WP04 |
| 7 | eco-002-WP07 | Worktree Coherence Check | low | WP02 |
| 8 | eco-002-WP08 | Governance Documentation & Runbook | medium | WP01–WP07 |

---

## WP-01 — Branch Inventory & Categorization

**WP ID:** eco-002-WP01
**Title:** Branch Inventory & Categorization
**Status:** pending
**Priority:** high
**Dependencies:** —
**File Scope:** `kitty-specs/eco-002-branch-consolidation/`, `.github/workflows/branch-inventory.yml`

### Narrative

Before any cleanup or enforcement can happen, the full landscape of branches must be mapped. This WP produces a canonical inventory report that categorizes every branch across all tracked remotes by state, age, and type.

### Acceptance Criteria

- [ ] Inventory script enumerates all branches across `origin` remote (local + remote).
- [ ] Each branch is tagged with: last-commit date, author, merged-status, protection-status.
- [ ] Output is structured (JSON or CSV) and committed to `kitty-specs/eco-002-branch-consolidation/artifacts/`.
- [ ] Branches are categorized into at least 4 buckets: `active`, `stale`, `merged`, `orphaned`.
- [ ] Age threshold for `stale` is configurable via `BRANCH_STALE_DAYS` env (default: 90 days).
- [ ] Inventory runs on-demand and via scheduled GitHub Actions (weekly).

### Verification Checklist

- [ ] Script exits 0 with no unhandled errors.
- [ ] Output file is valid JSON/CSV and parseable.
- [ ] At least 4 categorization buckets are represented in output.
- [ ] `BRANCH_STALE_DAYS` override is tested (set to 1, verify stale detection).
- [ ] GitHub Actions workflow file is valid YAML and passes `actionlint`.

---

## WP-02 — Stale & Merged Branch Cleanup

**WP ID:** eco-002-WP02
**Title:** Stale & Merged Branch Cleanup
**Status:** pending
**Priority:** high
**Dependencies:** WP01
**File Scope:** `kitty-specs/eco-002-branch-consolidation/artifacts/`, `.github/workflows/`

### Narrative

Branches that are already merged to `main` or have been abandoned for >90 days consume CI resources and create noise. This WP defines a safe, audited cleanup procedure with manual review gates before destructive operations.

### Acceptance Criteria

- [ ] Merged branch detection uses `git branch -r --merged origin/main` plus cross-check with GitHub PR API.
- [ ] Stale branch detection uses last-commit age from WP-01 inventory.
- [ ] Cleanup is dry-run by default; dry-run output lists all branches slated for deletion with rationale.
- [ ] Actual deletion requires `--confirm` flag or `CLEANUP_CONFIRM=true` env.
- [ ] Deleted branches are logged to `artifacts/cleanup-log.md` with: branch name, reason, deleted-at, actor.
- [ ] Local worktrees associated with deleted branches are listed (not auto-deleted) in cleanup report.
- [ ] GitHub branch protection is checked before deletion — protected branches are never deleted.

### Verification Checklist

- [ ] Dry-run produces a non-empty list for a repo with known stale/merged branches.
- [ ] `--confirm` run on a test branch deletes it and logs correctly.
- [ ] Protected branch deletion attempt is blocked and returns error.
- [ ] Cleanup log is append-only and valid Markdown.
- [ ] Local worktree association is accurate (test with a known worktree branch).

---

## WP-03 — Branch Naming Convention Enforcement

**WP ID:** eco-002-WP03
**Title:** Branch Naming Convention Enforcement
**Status:** pending
**Priority:** medium
**Dependencies:** WP01
**File Scope:** `docs/`, `.git/hooks/`

### Narrative

A consistent naming convention enables at-a-glance branch type identification, automated tooling (linting, tagging), and better PR routing. AgilePlus adopts a `type/description-kebab-case` pattern.

### Convention

```
<type>/<ticket-id>-<short-description>
```

| Type Prefix | Meaning |
|-------------|---------|
| `feat/` | New feature development |
| `fix/` | Bug fix |
| `chore/` | Tooling, deps, CI, config |
| `docs/` | Documentation only |
| `refactor/` | Code refactoring (no behavior change) |
| `test/` | Test-only changes |
| `spec/` | Spec / kitty-spec work |
| `wip/` | Work-in-progress (short-lived, discouraged) |
| `hotfix/` | Urgent production fix |

Examples:
- `feat/42-user-authentication`
- `fix/99-cli-crash-on-missing-config`
- `chore/update-axum-0.8`

### Acceptance Criteria

- [ ] Branch naming convention is documented in `docs/guides/branch-convention.md`.
- [ ] A `branch-lint` script validates branch names against the convention.
- [ ] Script exits `0` for compliant names, `1` for non-compliant, and prints offending branch names.
- [ ] Git hook (pre-push or commit-msg) runs the linter locally.
- [ ] GitHub Actions workflow runs the linter on every push and fails non-compliant branches.
- [ ] Legacy branches that predate the convention are grandfathered in a `legacy-branches.txt` allowlist (must be reviewed annually).

### Verification Checklist

- [ ] `feat/42-add-user` → passes linter.
- [ ] `user-feature` → fails linter with descriptive error.
- [ ] Hook fires correctly on `git push`.
- [ ] GitHub Actions workflow fails on non-compliant push.
- [ ] `legacy-branches.txt` allowlist is documented and reviewed.

---

## WP-04 — Branch Protection Rules Consistency

**WP ID:** eco-002-WP04
**Title:** Branch Protection Rules Consistency
**Status:** pending
**Priority:** high
**Dependencies:** WP01
**File Scope:** `.github/`, `docs/`

### Narrative

Inconsistent branch protection across repos leads to accidental force-pushes to `main`, unprotected `develop` branches, and missing required reviewers. This WP audits and enforces a baseline protection policy.

### Baseline Protection Policy

| Branch Pattern | Required Checks |
|---------------|-----------------|
| `main` | Require 1+ reviewers, dismiss stale reviews, require status checks, block force-push, require signed commits |
| `develop` | Require 1+ reviewers, block force-push, require linear history (optional) |
| `release/*` | Require 1+ reviewers, block force-push |
| `feat/*`, `fix/*`, etc. | No protection (contributor branches) |

### Acceptance Criteria

- [ ] Current protection rules for all repos are audited via GitHub API and documented in `artifacts/protection-audit.json`.
- [ ] Any repo deviating from baseline policy is flagged with: repo name, branch, missing rule, current state.
- [ ] A remediation script (`apply-protection-rules.sh`) applies the baseline policy to a target repo.
- [ ] Dry-run mode (`--dry-run`) shows what would change without modifying anything.
- [ ] Protection rules are enforced via GitHub API (not just documentation).
- [ ] `CODEOWNERS` file is present on `main` and defines minimum review requirements.

### Verification Checklist

- [ ] `protection-audit.json` lists all repos and their current protection config.
- [ ] Deviation report correctly identifies at least one deviation (or confirms none exist).
- [ ] Dry-run on a test repo shows diff without applying.
- [ ] `--apply` run correctly updates protection rules on a test repo.
- [ ] `CODEOWNERS` is valid (no duplicate entries, no invalid paths).

---

## WP-05 — Automation for Branch Lifecycle (Git Hooks + Scripts)

**WP ID:** eco-002-WP05
**Title:** Automation for Branch Lifecycle (Git Hooks + Scripts)
**Status:** pending
**Priority:** medium
**Dependencies:** WP02, WP03
**File Scope:** `scripts/`, `.git/hooks/`, `kitty-specs/eco-002-branch-consolidation/`

### Narrative

Manual branch management does not scale. This WP bundles reusable automation scripts and Git hooks that run at key lifecycle events: pre-push (lint + protect), post-merge (detect cleanup opportunities), and post-checkout (notify stale).

### Scripts to Implement

1. **`branch-lint.sh`** — validates branch name against WP-03 convention.
2. **`branch-inventory.sh`** — runs WP-01 inventory (executable, no args for default run).
3. **`branch-cleanup.sh`** — dry-run + confirm cleanup (WP-02).
4. **`worktree-audit.sh`** — lists orphaned worktrees with no corresponding remote branch.

### Git Hooks

- **`pre-push`** — runs `branch-lint.sh`. Non-zero exit blocks push.
- **`post-checkout`** — warns if checking out a branch older than `BRANCH_STALE_DAYS`.

### Acceptance Criteria

- [ ] All scripts are executable, have `--help` flags, and use `set -euo pipefail`.
- [ ] `branch-lint.sh` exits 0 for compliant names, 1 for non-compliant.
- [ ] `branch-cleanup.sh` defaults to dry-run; `--confirm` enables deletion.
- [ ] `worktree-audit.sh` lists orphaned worktrees (worktree dir exists, remote branch gone).
- [ ] `pre-push` hook is installed via `git config core.hooksPath scripts/hooks`.
- [ ] Scripts are documented with shebang, usage, and examples at the top of each file.
- [ ] Scripts work from any subdirectory of the repo (use `git rev-parse --show-toplevel`).

### Verification Checklist

- [ ] `branch-lint.sh feat/42-test` exits 0.
- [ ] `branch-lint.sh badbranch` exits 1 and prints offending name.
- [ ] `branch-cleanup.sh` without `--confirm` produces dry-run output.
- [ ] `branch-cleanup.sh --confirm` on a test branch deletes it and logs.
- [ ] `worktree-audit.sh` finds known orphaned worktrees.
- [ ] `pre-push` hook fires and blocks non-compliant names.
- [ ] All scripts pass `shellcheck`.

---

## WP-06 — GitHub Actions Workflow: Branch Sentinel

**WP ID:** eco-002-WP06
**Title:** GitHub Actions Workflow: Branch Sentinel
**Status:** pending
**Priority:** medium
**Dependencies:** WP04
**File Scope:** `.github/workflows/`

### Narrative

An automated GitHub Actions workflow that runs on a schedule (weekly) and on every push event, maintaining branch hygiene without human intervention. The sentinel produces artifacts and raises issues for human review when destructive action is required.

### Workflow: `branch-sentinel.yml`

**Triggers:**
- `push` (all branches)
- `schedule: cron: "0 0 * * 0"` (weekly, Sunday midnight UTC)
- `workflow_dispatch` (manual trigger)

**Jobs:**

1. **`inventory`** — runs `branch-inventory.sh`, uploads `inventory.json` as artifact.
2. **`lint`** — checks all branch names against convention, fails workflow if any non-compliant.
3. **`stale-check`** — flags branches older than `BRANCH_STALE_DAYS`, writes to `stale-branches.md`.
4. **`protection-audit`** — fetches branch protection rules, compares to baseline, outputs diff.
5. **`report`** — creates or updates a GitHub Issue with the weekly hygiene summary.

### Acceptance Criteria

- [ ] Workflow file is valid YAML, passes `actionlint`.
- [ ] Scheduled run executes within 5 minutes of cron time.
- [ ] `inventory` job produces a valid JSON artifact.
- [ ] `lint` job fails (non-zero exit) when non-compliant branch names exist.
- [ ] `stale-check` job produces a `stale-branches.md` file with branch names and last-commit dates.
- [ ] `report` job creates an issue titled `[Branch Sentinel] Hygiene Report — YYYY-MM-DD`.
- [ ] If an issue already exists from a prior run, it is updated (not duplicated).

### Verification Checklist

- [ ] `actionlint` passes on the workflow file.
- [ ] `workflow_dispatch` trigger works in a test run.
- [ ] `lint` job fails intentionally when a non-compliant branch exists.
- [ ] Issue creation is confirmed (new issue or updated existing issue).
- [ ] Artifact upload is confirmed from the `inventory` job.

---

## WP-07 — Worktree Coherence Check

**WP ID:** eco-002-WP07
**Title:** Worktree Coherence Check
**Status:** pending
**Priority:** low
**Dependencies:** WP02
**File Scope:** `scripts/`, `kitty-specs/eco-002-branch-consolidation/artifacts/`

### Narrative

AgilePlus uses git worktrees as the primary authoring environment (per CLAUDE.md). Orphaned worktrees — those whose branches have been deleted or renamed — accumulate silently and waste disk space. This WP provides tooling to detect and report them.

### Acceptance Criteria

- [ ] `worktree-audit.sh` (from WP-05) is extended to detect orphaned worktrees.
- [ ] An orphaned worktree is defined as: worktree directory exists at `../<repo>-wtrees/<name>`, but the corresponding `<name>` branch no longer exists on any remote.
- [ ] Report is written to `artifacts/worktree-audit.md` with: worktree path, missing branch, last-accessed time.
- [ ] Optional: `worktree-prune.sh --dry-run` lists worktrees eligible for removal.
- [ ] A GitHub Actions job (`worktree-coherence.yml`) runs monthly to catch orphaned worktrees.

### Verification Checklist

- [ ] `worktree-audit.sh` correctly identifies a test orphaned worktree.
- [ ] `artifacts/worktree-audit.md` is valid Markdown and lists orphaned worktrees.
- [ ] `worktree-prune.sh --dry-run` produces a safe (no-op) list.
- [ ] `worktree-coherence.yml` passes `actionlint`.

---

## WP-08 — Governance Documentation & Runbook

**WP ID:** eco-002-WP08
**Title:** Governance Documentation & Runbook
**Status:** pending
**Priority:** medium
**Dependencies:** WP01–WP07
**File Scope:** `docs/guides/`, `docs/reference/`, `kitty-specs/eco-002-branch-consolidation/`

### Narrative

All operational change must be documented and self-serviceable. This WP produces the runbook and reference documentation so any contributor can understand, execute, and extend the branch lifecycle system without tribal knowledge.

### Deliverables

1. **`docs/guides/branch-lifecycle-runbook.md`** — step-by-step runbook covering: creating a branch, naming it, getting it reviewed, merging it, and what happens post-merge.
2. **`docs/guides/branch-convention.md`** — naming convention reference (from WP-03).
3. **`docs/reference/branch-protection-reference.md`** — protection policy table (from WP-04), how to modify it, escalation path.
4. **`kitty-specs/eco-002-branch-consolidation/README.md`** — spec index linking all WPs, status dashboard, and decision log.
5. **`artifacts/decision-log.md`** — chronological record of all significant decisions made during implementation.

### Acceptance Criteria

- [ ] All 5 deliverables exist and are valid Markdown.
- [ ] Runbook includes a "Quick Reference" section with the 3 most common commands.
- [ ] Branch convention doc includes a regex for validation (for tool implementers).
- [ ] Protection reference doc explains how to request a policy exception.
- [ ] `README.md` has a WP status table with live links to each WP's checklist.
- [ ] Decision log has at least one entry (implementation decision at spec-close).
- [ ] All docs pass `vale` and `markdownlint`.

### Verification Checklist

- [ ] `vale lint docs/guides/branch-lifecycle-runbook.md` → no errors.
- [ ] `markdownlint docs/guides/branch-lifecycle-runbook.md` → no errors.
- [ ] Runbook's quick reference commands are tested and produce expected output.
- [ ] Branch convention regex matches `feat/42-add-auth` and rejects `my-branch`.
- [ ] `README.md` WP table is complete (all 8 WPs listed).
- [ ] Decision log is non-empty (at least one decision recorded).

---

## Verification Gate

All WPs must pass their individual verification checklists before the spec is closed. The final sign-off checklist for the spec lead:

- [ ] WP-01: Inventory artifact committed and parseable.
- [ ] WP-02: Cleanup log has ≥1 entry from dry-run + confirm cycle.
- [ ] WP-03: `branch-lint.sh` shipped, documented, and passing actionlint.
- [ ] WP-04: Protection audit artifact committed; remediation script shipped.
- [ ] WP-05: All 4 scripts shipped, `shellcheck`-clean, hook installed.
- [ ] WP-06: `branch-sentinel.yml` shipped, `actionlint`-clean, manual dispatch confirmed.
- [ ] WP-07: `worktree-audit.sh` extended, monthly workflow shipped.
- [ ] WP-08: All 5 docs shipped and lint-clean.
- [ ] All artifacts committed under `kitty-specs/eco-002-branch-consolidation/artifacts/`.
- [ ] This `tasks.md` updated: all WPs marked `completed`, status → `completed`.

---

*Generated: 2026-05-02 | eco-002 | AgilePlus kitty-spec system*
