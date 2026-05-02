---
spec_id: eco-001
title: Worktree Remediation — Full Work Package Structure
created_at: 2026-05-02T00:00:00Z
priority: high
status: planned
type: operational
supersedes: eco-001 meta.json skeleton (2026-03-29)
---

# Work Packages: Worktree Remediation

**Inputs:** `meta.json`, `spec.md` in this directory; Phenotype/repos/CLAUDE.md "Worktree Rule"; `AgilePlus/docs/agents/governance-constraints.md`.
**Prerequisites:** Git worktree management, Bash shell, Python 3.10+ for governance scripts.
**Primary scope:** Canonical bare AgilePlus repo; Phenotype `repos/` worktree inventory.
**Secondary scope:** Per-repo worktree state, drift detection, remediation scripts.

> Per global governance: planner WPs equip implementers. No code in this document. Acceptance criteria and checklists drive implementation verification.

---

## What

The `eco-001` spec was opened 2026-03-29 to remediate legacy worktrees and establish worktree governance discipline. It was marked COMPLETED the same day — but the `meta.json` skeleton contains no tasks.md, no phased WPs, and no per-WP acceptance criteria. Subsequent sessions (April 2026) repeatedly hit worktree drift issues: bare canonical checkouts, orphan submodule entries, ghost branches from squash-merges, and `AgilePlus` canonical flagged as bare requiring push-from-worktree workarounds.

This tasks.md expands the skeleton into a fully traceable WP structure covering:
1. **Discovery** — inventory current worktree state across all Phenotype repos
2. **Drift detection** — detect canonical-to-worktree anomalies (bare repos, orphan refs, phantom branches)
3. **Remediation automation** — scripted fixes for detected drift classes
4. **Git housekeeping** — `git gc`, ref cleanup, stale pack pruning
5. **Monitoring & reporting** — governance dashboards, periodic drift scans

## Why

Worktree drift is an economic and operational concern (eco spec class):
- Agent sessions block on `git` commands failing against bare canonicals
- Squash-merge ghost branches inflate repo size and confuse `git log` audits
- Orphan submodule entries trigger false-positive `git fsck` failures
- Repeated remediation burns agent time; automated detection amortizes cost
- Without a monitoring loop, drift re-accumulates post-remediation

The WP structure here makes the remediation traceable, repeatable, and independently verifiable.

---

## Phase 1 — Discovery

### WP-001 — Worktree Inventory Audit

- **State:** pending
- **Phase:** 1 (Discovery)
- **Depends on:** —
- **Priority:** high
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (read-only):**
  - `repos/` subdirectories (list worktrees per repo)
  - `AgilePlus/kitty-specs/eco-001-worktree-remediation/spec.md`
  - `Phenotype/repos/CLAUDE.md`
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/worktree-inventory.md`
- **Acceptance criteria:**
  - Table of all Phenotype repos: canonical bare or working tree, list of worktrees (name, branch, last-activity date)
  - Flag any canonical repo that is bare (`core.bare=true`)
  - Flag any worktree whose branch is orphaned (no upstream, no associated PR)
  - Flag any repo with orphan submodule entries (`.gitmodules` ref exists but submodule dir missing)
- **Verification checklist:**
  - [ ] `git -C <repo> rev-parse --is-inside-work-tree` for each canonical repo
  - [ ] `git worktree list --porcelain` per repo
  - [ ] `grep -r "submodule" .gitmodules 2>/dev/null` per repo
  - [ ] `gh pr list --state all --author @me --json number,title,state,isDraft` cross-check for orphan branches
- **Handoff prompt:** "Audit all Phenotype `repos/` directories; produce `worktree-inventory.md` per WP-001 acceptance criteria. Use `git rev-parse --is-inside-work-tree` to detect bare canonicals."

---

### WP-002 — Ghost Branch & Squash-Merge Archaeology

- **State:** pending
- **Phase:** 1 (Discovery)
- **Depends on:** WP-001
- **Priority:** medium
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (read-only):**
  - `AgilePlus/.git/` (local reflog if available)
  - `gh api repos/KooshaPari/AgilePlus/pulls?state=merged&per_page=100`
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/ghost-branch-report.md`
- **Acceptance criteria:**
  - List of local branches whose commits are all present in an already-merged PR (ghost branches)
  - List of squash-merged PRs with orphaned SHA lineages (local SHA not reachable from origin/main)
  - For each ghost: repo, branch name, merge PR number, reason flagged
- **Verification checklist:**
  - [ ] `git log --oneline origin/main..HEAD` per worktree (compares to canonical)
  - [ ] `gh pr list --state merged --json number,title,mergeCommitOid` per repo
  - [ ] `git patch-id --stable` dedup to detect squash-merge SHA orphans
- **Handoff prompt:** "Run ghost-branch archaeology per WP-002. Cross-reference local branches against merged PRs via `gh api`. Produce `ghost-branch-report.md`."

---

## Phase 2 — Drift Detection

### WP-003 — Drift Detection Script

- **State:** pending
- **Phase:** 2 (Drift Detection)
- **Depends on:** WP-001
- **Priority:** high
- **Effort:** Medium (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/drift-detect.sh`
- **Acceptance criteria:**
  - Single Bash script `drift-detect.sh` that scans a target repo root and emits a structured report
  - Detects: bare canonicals, orphan submodules, ghost branches, stale worktrees (no commits in 90+ days)
  - Output format: machine-readable (JSON or key=value lines) for CI integration
  - Exit code 0 if clean, non-zero if drift detected
- **Verification checklist:**
  - [ ] Script runs against a clean canonical repo with exit 0
  - [ ] Script detects the bare AgilePlus canonical (exit non-zero)
  - [ ] Script detects orphan submodule entries (exit non-zero)
  - [ ] `bash -n drift-detect.sh` passes shellcheck (or documents required ignores)
- **Handoff prompt:** "Author `drift-detect.sh` per WP-003 acceptance. Emit JSON to stdout. Use `git rev-parse --is-inside-work-tree`, `git submodule status`, and `git branch --list` for detection."

---

## Phase 3 — Remediation

### WP-004 — Canonical-to-Worktree Remediation Playbook

- **State:** pending
- **Phase:** 3 (Remediation)
- **Depends on:** WP-002, WP-003
- **Priority:** high
- **Effort:** Medium (8-15 tool calls, ~5 min)
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/remediation-playbook.md`
- **Acceptance criteria:**
  - Step-by-step playbook for each drift class detected in WP-003
  - Canonical bare: create fresh worktree from bare, cherry-pick any uncommitted state, update remote URL
  - Ghost branches: `git branch -D` with confirmation checklist (PR already merged)
  - Orphan submodules: `git submodule deinit` + `git rm` + commit
  - Stale worktrees: archive to `archive/<name>-<date>-stale` preserving `.git/` and reflog
  - Each step includes rollback procedure
- **Verification checklist:**
  - [ ] Playbook covers all 4 drift classes
  - [ ] Each drift class has rollback procedure documented
  - [ ] Confirmation checklist exists before destructive operations (git branch -D, git rm)
- **Handoff prompt:** "Author `remediation-playbook.md` per WP-004. Reference ghost-branch-report.md from WP-002 and drift-detect.sh from WP-003. Include lane-decomposition framing (Lane A: user-gated, Lane B: structural one-shots, Lane C: mechanical recurring)."

---

### WP-005 — Git GC and Ref Cleanup Automation

- **State:** pending
- **Phase:** 3 (Remediation)
- **Depends on:** WP-004
- **Priority:** medium
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/git-housekeeping.sh`
- **Acceptance criteria:**
  - Single Bash script `git-housekeeping.sh` that runs `git gc --prune=now --aggressive` on target repos
  - Cleans stale pack files and loose objects
  - Reports before/after `.git/` size
  - Dry-run mode (`--dry-run`) that estimates cleanup without executing
  - Safe to run in CI or cron (idempotent, non-destructive to objects reachable from refs)
- **Verification checklist:**
  - [ ] `git-housekeeping.sh --dry-run` exits 0 with estimated savings
  - [ ] `git-housekeeping.sh` completes without corruption on a test repo
  - [ ] `git fsck --full` passes after housekeeping run
- **Handoff prompt:** "Author `git-housekeeping.sh` per WP-005. Use `git gc --prune=now --aggressive` and report `.git/` size delta. Include `--dry-run` flag."

---

## Phase 4 — Monitoring & Reporting

### WP-006 — Periodic Drift Scan Integration

- **State:** pending
- **Phase:** 4 (Monitoring)
- **Depends on:** WP-003
- **Priority:** medium
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (write):** `AgilePlus/.github/workflows/worktree-drift-scan.yml`
- **Acceptance criteria:**
  - GitHub Actions workflow triggered on `workflow_dispatch` and `schedule` (weekly cron)
  - Runs `drift-detect.sh` across all Phenotype repos
  - Posts results as a GitHub Actions summary table
  - Opens a GitHub Issue if drift detected, labels it `worktree-drift`
  - Uses standard Linux runner (macOS/Windows runners skipped per billing constraint)
- **Verification checklist:**
  - [ ] Workflow file is valid YAML with correct trigger syntax
  - [ ] `drift-detect.sh` is referenced by correct relative path
  - [ ] Linux runner label used (`ubuntu-latest`)
  - [ ] Billing constraint acknowledged in workflow comment
- **Handoff prompt:** "Create GitHub Actions workflow `worktree-drift-scan.yml` per WP-006. Reference `drift-detect.sh` from WP-003. Post results as Actions summary. Use `ubuntu-latest` runner."

---

### WP-007 — Governance Dashboard Entry

- **State:** pending
- **Phase:** 4 (Monitoring)
- **Depends on:** WP-001, WP-006
- **Priority:** low
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/governance-dashboard.md`
- **Acceptance criteria:**
  - Entry for AgilePlus spec kitty dashboard: worktree health score (last scan date, drift count, last remediation date)
  - Integration instructions linking drift-scan workflow results to the dashboard
  - Escalation path: drift count > 0 for 2+ consecutive weeks → auto-flag to governance board
- **Verification checklist:**
  - [ ] Dashboard entry has all required fields (last scan, drift count, status)
  - [ ] Escalation threshold defined (2 consecutive weeks)
  - [ ] Links to `worktree-drift-scan.yml` and `drift-detect.sh`
- **Handoff prompt:** "Author `governance-dashboard.md` per WP-007. Create a dashboard entry format for worktree health tracking. Include escalation logic."

---

## Phase 5 — Verification & Handoff

### WP-008 — End-to-End Verification Run

- **State:** pending
- **Phase:** 5 (Verification)
- **Depends on:** WP-001, WP-002, WP-003, WP-004, WP-005, WP-006, WP-007
- **Priority:** high
- **Effort:** Small (3-6 tool calls, ~2 min)
- **File scope (read-only):** All artifacts produced in Phases 1-4
- **File scope (write):** `kitty-specs/eco-001-worktree-remediation/research/verification-report.md`
- **Acceptance criteria:**
  - All 7 prior WPs verified complete (checklist items checked)
  - `drift-detect.sh` runs clean (exit 0) against the canonical AgilePlus repo after remediation
  - `git fsck --full` reports zero missing or unreachable objects in canonical
  - All ghost branches resolved (verified via `gh pr list --state merged`)
  - Workflow `worktree-drift-scan.yml` passes YAML validation
  - `spec.md` updated to reflect completion of all phases
- **Verification checklist:**
  - [ ] WP-001 inventory complete and reviewed
  - [ ] WP-002 ghost branches documented and resolved
  - [ ] WP-003 drift-detect.sh exits 0 on clean canonical
  - [ ] WP-004 playbook reviewed and approved
  - [ ] WP-005 git-housekeeping.sh tested on non-critical repo
  - [ ] WP-006 workflow file passes `actionlint` or equivalent
  - [ ] WP-007 dashboard entry populated with initial baseline
  - [ ] `meta.json` status updated to `completed`
  - [ ] `spec.md` updated with final completion date
- **Handoff prompt:** "Run end-to-end verification per WP-008. Verify all 7 prior WPs complete. Produce `verification-report.md`. Update `meta.json` status to `completed` and `spec.md` with final completion date."

---

## Cross-Cutting Concerns

- **Fork awareness:** Repos may be forks; check `gh api repos/.../fork` before destructive operations
- **Billing constraint:** All CI uses `ubuntu-latest`; macOS/Windows runners explicitly skipped
- **Dirty-tree discipline:** Separate commits by provenance (user-requested vs. pre-existing vs. generated artifacts)
- **AgilePlus integration:** All scripts must be runnable from within the AgilePlus worktree environment
- **Evidence ledger:** Link all verification reports and scripts into the AgilePlus spec kitty evidence chain
