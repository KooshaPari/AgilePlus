# Work Package Index: eco-006 Governance Sync Automation

**Feature**: eco-006-governance-sync
**Total WPs**: 6 | **Total Subtasks**: 28
**MVP Scope**: WP01 → WP02 → WP03 = Inventory + Scripting + CI Trigger (3 WPs)
**Full Scope**: + WP04 (Notifications) + WP05 (Drift Resolution) + WP06 (Audit Logging)

---

## What / Why

The Phenotype/AgilePlus ecosystem spans 61+ Rust repos, multiple worktrees, and a growing body of governance docs (CLAUDE.md hierarchy, policy docs, worklogs, kitty-specs). Governance drift — stale CLAUDE.md files, missing policy requirements, orphaned worklog entries, or out-of-sync kitty-specs — erodes compliance and makes it impossible for agents to trust the workspace state.

Manual governance audits are error-prone and do not scale with the ecosystem. This spec implements an automated governance sync system that:

1. **Detects drift** — continuously inventories governance artifacts and compares them against canonical sources
2. **Notifies stakeholders** — routes alerts to the right channel (iMessege, GitHub PR, worklog entry) based on severity
3. **Resolves drift** — proposes or applies corrections automatically where safe, flags for human review where not
4. **Audits continuously** — maintains an immutable evidence ledger of all sync events and resolutions

The system runs as a scheduled agent workflow (`dispatch` skill) and as an on-demand CLI command (`agileplus governance sync`).

---

## Dependency Graph

```
WP01 (Governance Doc Inventory)
├── WP02 (Sync Automation Scripting)   [depends: WP01]
│   ├── WP03 (CI Trigger for Drift)    [depends: WP02]
│   ├── WP04 (Notification & Alerting) [depends: WP02]
│   └── WP05 (Drift Resolution)        [depends: WP03, WP04]
└── WP06 (Evidence Ledger & Audit)     [depends: WP01, WP02]
```

**Parallelizable**: WP03, WP04, WP06 all become available after WP02.

---

## Phase 1 — Inventory

### WP01: Governance Doc Inventory (5 subtasks, ~250 lines)

**Goal**: Build a machine-readable inventory of all governance artifacts across the Phenotype ecosystem, with canonical fingerprints and source-of-truth references.
**Priority**: P1 | **Dependencies**: none
**FRs**: FR-GOV-01, FR-GOV-02
**Prompt**: `tasks/WP01-governance-inventory.md`

Subtasks:
- [ ] T001: Define GovernanceArtifact schema (path, repo, checksum, canonical_url, last_verified, source_type)
- [ ] T002: Implement repo discovery scanner (walks repos/Phenotype/, repos/docs/, worktrees/)
- [ ] T003: Implement CLAUDE.md fingerprint generator (hash of effective CLAUDE.md + inherited hierarchy)
- [ ] T004: Implement kitty-spec indexer (parses all kitty-specs/*/spec.md into structured catalog)
- [ ] T005: Implement worklog scanner (validates worklogs/ entries against kitty-spec requirements)

---

## Phase 2 — Sync Automation

### WP02: Sync Automation Scripting (5 subtasks, ~400 lines)

**Goal**: Build the core sync engine that compares inventory snapshots, detects drift, and emits structured drift events.
**Priority**: P1 | **Dependencies**: WP01
**FRs**: FR-GOV-03, FR-GOV-04
**Prompt**: `tasks/WP02-sync-engine.md`

Subtasks:
- [ ] T006: Implement DriftDetector — compares current inventory snapshot against canonical baseline
- [ ] T007: Implement DriftClassifier — categorizes drift by severity (BREAKING/MINOR/DOCUMENTATION/ORPHANED)
- [ ] T008: Implement SyncPlanner — generates remediation action plan from drift events
- [ ] T009: Implement SafeApplicator — applies low-risk fixes automatically (whitespace, encoding, missing fields)
- [ ] T010: Implement dry-run mode + structured JSON/CSV output for audit trail

---

## Phase 3 — CI Integration

### WP03: CI Trigger for Governance Drift (4 subtasks, ~200 lines)

**Goal**: Wire governance sync into CI pipeline so every PR triggers a drift check and every failed check blocks merge.
**Priority**: P1 | **Dependencies**: WP02
**FRs**: FR-GOV-05, FR-GOV-06
**Prompt**: `tasks/WP03-ci-trigger.md`

Subtasks:
- [ ] T011: Create `.github/workflows/governance-sync.yml` with dispatch + schedule triggers
- [ ] T012: Implement `--ci` flag on `agileplus governance sync` for non-interactive CI mode
- [ ] T013: Add status check output compatible with GitHub commit status API (pass/fail with summary)
- [ ] T014: Implement PR comment reporter (posts drift summary as GitHub PR comment)

---

## Phase 4 — Notifications

### WP04: Notification & Alerting (5 subtasks, ~300 lines)

**Goal**: Route governance drift alerts to the right stakeholders based on drift type and severity.
**Priority**: P2 | **Dependencies**: WP02
**FRs**: FR-GOV-07, FR-GOV-08
**Prompt**: `tasks/WP04-notifications.md`

Subtasks:
- [ ] T015: Implement NotificationRouter — maps drift type to delivery channel (iMessege, Slack, GitHub)
- [ ] T016: Implement iMessege notifier via agent-imessage MCP (critical drift alerts to Koosha)
- [ ] T017: Implement GitHub issue creator for BREAKING drift (auto-files governance debt issue)
- [ ] T018: Implement worklog entry writer for all sync events (appends to worklogs/GOVERNANCE.md)
- [ ] T019: Implement severity-based deduplication (suppress repeat alerts for same drift within 24h window)

---

## Phase 5 — Drift Resolution

### WP05: Drift Resolution Workflow (5 subtasks, ~350 lines)

**Goal**: Implement safe, auditable drift correction workflows — from auto-fix to human-gated approval.
**Priority**: P2 | **Dependencies**: WP03, WP04
**FRs**: FR-GOV-09, FR-GOV-10, FR-GOV-11
**Prompt**: `tasks/WP05-drift-resolution.md`

Subtasks:
- [ ] T020: Implement ReviewQueue — holds BREAKING drift for human approval before applying
- [ ] T021: Implement auto-fix executor for MINOR/DOCUMENTATION drift (CLAUDE.md updates, meta.json patches)
- [ ] T022: Implement orphan detection + cleanup workflow (removes stale kitty-spec references)
- [ ] T023: Implement rollback capability (stores pre-fix snapshot; `agileplus governance rollback <event_id>`)
- [ ] T024: Implement resolution approval CLI (`agileplus governance approve <drift_id>`)

---

## Phase 6 — Evidence & Audit

### WP06: Evidence Ledger & Audit (4 subtasks, ~250 lines)

**Goal**: Maintain an immutable, hash-chained evidence ledger of all sync events for governance compliance.
**Priority**: P2 | **Dependencies**: WP01, WP02
**FRs**: FR-GOV-12, FR-GOV-13
**Prompt**: `tasks/WP06-evidence-ledger.md`

Subtasks:
- [ ] T025: Implement GovernanceEvent struct (timestamp, event_type, artifact_ref, diff, actor, hash)
- [ ] T026: Implement hash-chain ledger (each event references previous event hash; tamper-evident)
- [ ] T027: Implement audit query CLI (`agileplus governance audit --repo <name> --since <date>`)
- [ ] T028: Implement compliance report generator (exports ledger as JSON/CSV for external auditors)

---

## Subtask Index

| ID | Description | WP | Parallel |
|----|-------------|-----|----------|
| T001 | GovernanceArtifact schema | WP01 | |
| T002 | Repo discovery scanner | WP01 | |
| T003 | CLAUDE.md fingerprint generator | WP01 | |
| T004 | kitty-spec indexer | WP01 | |
| T005 | Worklog scanner | WP01 | |
| T006 | DriftDetector | WP02 | |
| T007 | DriftClassifier | WP02 | [P] T006 |
| T008 | SyncPlanner | WP02 | [P] T006 |
| T009 | SafeApplicator | WP02 | [P] T006 |
| T010 | Dry-run mode + structured output | WP02 | [P] T006 |
| T011 | GitHub Actions workflow | WP03 | |
| T012 | CI flag for governance sync | WP03 | |
| T013 | GitHub status check output | WP03 | [P] T011 |
| T014 | PR comment reporter | WP03 | [P] T011 |
| T015 | NotificationRouter | WP04 | |
| T016 | iMessege notifier | WP04 | |
| T017 | GitHub issue creator | WP04 | |
| T018 | Worklog entry writer | WP04 | [P] T015 |
| T019 | Severity deduplication | WP04 | [P] T015 |
| T020 | ReviewQueue for BREAKING drift | WP05 | |
| T021 | Auto-fix executor | WP05 | |
| T022 | Orphan detection + cleanup | WP05 | [P] T020 |
| T023 | Rollback capability | WP05 | [P] T020 |
| T024 | Resolution approval CLI | WP05 | |
| T025 | GovernanceEvent struct | WP06 | |
| T026 | Hash-chain ledger | WP06 | |
| T027 | Audit query CLI | WP06 | |
| T028 | Compliance report generator | WP06 | |

---

## Acceptance Criteria

| ID | Criterion | Verification |
|----|----------|--------------|
| AC01 | `agileplus governance sync --dry-run` produces valid JSON/CSV output without modifying any files | Manual: run command, inspect output |
| AC02 | CI workflow triggers on every PR and blocks merge on BREAKING drift | CI: PR with broken CLAUDE.md fails status check |
| AC03 | iMessege alert fires for BREAKING drift within 60s of detection | Manual: simulate BREAKING drift, verify notification |
| AC04 | ReviewQueue holds BREAKING drift until approved | Manual: approve drift, verify fix applied |
| AC05 | Hash-chain ledger is tamper-evident (modified event breaks chain) | Manual: tamper with ledger file, verify detection |
| AC06 | Audit query returns correct event history for a given repo | Manual: `agileplus governance audit --repo focalpoint --since 2026-04-01` |
| AC07 | Orphaned kitty-spec entries are detected and flagged | Manual: add stale spec entry, run sync, verify detection |

---

## Verification Checklist

- [ ] All 28 subtasks implemented and passing unit tests
- [ ] CI workflow passes in GitHub Actions (Linux runner only)
- [ ] Dry-run output schema validated against spec
- [ ] iMessege integration tested with agent-imessage MCP
- [ ] Hash-chain tamper detection verified
- [ ] Audit CLI tested against real worklog entries
- [ ] No governance artifacts modified by test runs (verified via git status)
- [ ] Compliance report exports valid JSON/CSV
