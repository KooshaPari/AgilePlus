---
spec_id: eco-005
slug: eco-005-xdd-quality
title: XDD Quality Enforcement
phase: discovery
---

# XDD Quality Enforcement — Work Packages

## What / Why

AgilePlus is a 24-crate Rust workspace built on hexagonal architecture, event sourcing, and multi-VCS plugin support. As the ecosystem grows, code quality must be enforced programmatically — not by convention alone. XDD (eXtreme Design Discipline) encompasses the quality mandates already codified in the Phenotype governance layer:

- **Wrap-over-Handroll**: Prefer existing OSS libraries over hand-rolling utilities; fork-and-extend rather than rewrite
- **Rich-UI Mandate**: All UI work uses Radix/shadcn/Headless UI; no plain HTML forms
- **Quality Gates**: `task quality` / `task quality:full`, clippy, ruff, Vale on Markdown, Cargo-deny
- **Prose Quality**: Vale + markdownlint for all documentation
- **Manager Pattern**: Strategic parent-agents delegate exploration, audits, and multi-file work to subagents

Without automated enforcement, these policies degrade over time as new contributors arrive. This spec fills the gap between policy (CLAUDE.md, QA_GOVERNANCE.md, QA_GOVERNANCE.md) and enforcement (CI gates, pre-commit hooks, violation reporting).

---

## Work Package WP1: XDD Tooling Inventory

**Status**: pending
**Priority**: high
**Dependencies**: None
**Estimated**: ~3 tool calls

### Objective

Catalog every tool already available in the AgilePlus workspace for XDD enforcement. Identify gaps where tooling is referenced in governance but not yet wired into CI or pre-commit.

### Acceptance Criteria

- [ ] `cargo deny` config present and enforced in CI
- [ ] `cargo clippy --all -- -D warnings` wired as a CI gate
- [ ] `cargo fmt --check` wired as a CI gate
- [ ] Ruff configured for Python MCP server (`agileplus/`)
- [ ] Vale configured for Markdown linting in `docs/`
- [ ] `buf lint` and `buf breaking` wired for proto definitions (`proto/`)

### Verification Checklist

- [ ] `cargo deny check` runs without errors (advisories cleared or accepted)
- [ ] `cargo clippy --all -- -D warnings` passes locally
- [ ] `cargo fmt --check` passes locally
- [ ] `ruff check agileplus/` runs without errors
- [ ] `vale docs/` produces no errors (or only acknowledged warnings)
- [ ] `buf lint proto/` passes
- [ ] CI workflow includes all six gates above

---

## Work Package WP2: Pre-Commit Hooks for XDD Patterns

**Status**: pending
**Priority**: high
**Dependencies**: WP1 (tooling inventory)
**Estimated**: ~4 tool calls

### Objective

Install and configure pre-commit hooks that block commits violating XDD mandates before they reach CI.

### Acceptance Criteria

- [ ] `.husky/` or `pre-commit/` config present
- [ ] Pre-commit runs `cargo fmt` and `cargo clippy` on Rust code
- [ ] Pre-commit runs `ruff check` on Python files
- [ ] Pre-commit runs `markdownlint` on Markdown files
- [ ] Pre-commit is documented in `docs/guides/xdd-onboarding.md`
- [ ] Hook failures produce actionable error messages

### Verification Checklist

- [ ] `pre-commit run --all-files` passes locally
- [ ] A deliberate clippy violation is blocked by the pre-commit hook
- [ ] A deliberate format violation is blocked by the pre-commit hook
- [ ] A deliberate ruff violation is blocked by the pre-commit hook
- [ ] Docs reference the hook and link to `xdd-onboarding.md`

---

## Work Package WP3: CI Gate Enforcement

**Status**: pending
**Priority**: critical
**Dependencies**: WP2 (pre-commit hooks wired)
**Estimated**: ~3 tool calls

### Objective

Ensure all XDD quality gates are present in CI workflows and fail the build on violations.

### Acceptance Criteria

- [ ] `.github/workflows/ci.yml` (or equivalent) includes all quality gates from WP1
- [ ] CI fails on any clippy warning (`-D warnings`)
- [ ] CI fails on any format violation
- [ ] CI fails on any cargo-deny advisory (unless explicitly accepted)
- [ ] CI runs `cargo test --workspace` with no failures
- [ ] CI matrix covers Linux (primary); macOS/Windows skipped to conserve Actions billing

### Verification Checklist

- [ ] Push a branch with a clippy warning — CI job fails
- [ ] Push a branch with a format violation — CI job fails
- [ ] Push a branch with an unacknowledged cargo-deny advisory — CI job fails
- [ ] CI workflow file is present and validated
- [ ] No hardcoded secrets or sensitive data in CI config

---

## Work Package WP4: Violation Detection and Reporting

**Status**: pending
**Priority**: medium
**Dependencies**: WP1 (tooling inventory)
**Estimated**: ~3 tool calls

### Objective

Build a violation dashboard or reporting mechanism so the team can see XDD compliance status at a glance and track trends over time.

### Acceptance Criteria

- [ ] A nightly or on-push job collects clippy, deny, and ruff violations
- [ ] Violations are published as a GitHub Actions artifact or summary
- [ ] A `docs/reference/xdd-violations.md` tracker exists and is updated by CI
- [ ] Accepted/acknowledged violations are tracked with justification comments

### Verification Checklist

- [ ] Violation report is generated on a sample push
- [ ] Report lists all current clippy warnings (or confirms zero)
- [ ] Report lists all cargo-deny advisories (or confirms zero)
- [ ] `xdd-violations.md` exists and reflects current state
- [ ] Accepted advisories have inline comments citing rationale

---

## Work Package WP5: Team Onboarding Docs

**Status**: pending
**Priority**: high
**Dependencies**: WP1, WP2, WP3
**Estimated**: ~2 tool calls

### Objective

Create `docs/guides/xdd-onboarding.md` so every contributor (human or agent) understands the XDD quality mandates and how to comply.

### Acceptance Criteria

- [ ] `docs/guides/xdd-onboarding.md` exists with:
  - Explanation of why XDD matters for AgilePlus
  - List of all quality gates and what they check
  - Step-by-step setup instructions (tool installation, pre-commit enablement)
  - How to handle false positives and add accepted-violation comments
  - Links to relevant governance docs (CLAUDE.md, QA_GOVERNANCE.md)
- [ ] `AGENTS.md` references the onboarding doc for agent contributors
- [ ] Onboarding doc is tested by a new contributor (human or agent walkthrough)

### Verification Checklist

- [ ] File exists at `docs/guides/xdd-onboarding.md`
- [ ] File contains all four sections above
- [ ] Links to CLAUDE.md and QA_GOVERNANCE.md are valid
- [ ] A new agent (or agent-walkthrough simulation) can follow the setup steps and pass all gates

---

## Work Package WP6: Wrap-over-Handroll Audit

**Status**: pending
**Priority**: medium
**Dependencies**: WP1 (tooling inventory)
**Estimated**: ~4 tool calls

### Objective

Scan the codebase for hand-rolled utilities that could be replaced with existing OSS crates, enforcing the wrap-over-handroll mandate.

### Acceptance Criteria

- [ ] Audit identifies any hand-rolled utilities matching known OSS equivalents (e.g., `anyhow`/`eyre` vs custom error types, `tracing` vs custom logging, `serde` vs manual serialization)
- [ ] Each finding is filed as a GitHub issue or added to `docs/reference/xdd-handroll-audit.md`
- [ ] No new hand-roll patterns are introduced (enforced by code review + clippy)

### Verification Checklist

- [ ] Audit report exists at `docs/reference/xdd-handroll-audit.md`
- [ ] Each finding has a severity label and a recommendation
- [ ] A follow-up tracking issue or PR exists for high-severity findings
- [ ] Code review checklist includes "wrap vs hand-roll" question

---

## Work Package WP7: Rich-UI Mandate Enforcement (Python MCP Server)

**Status**: pending
**Priority**: medium
**Dependencies**: WP1 (tooling inventory)
**Estimated**: ~2 tool calls

### Objective

Ensure the Python MCP server (`agileplus/`) follows the rich-UI mandate — specifically, any CLI output or TUI that touches the Python layer uses rich/Textual rather than plain print statements.

### Acceptance Criteria

- [ ] `agileplus/` uses `rich` or `textual` for all structured output
- [ ] Ruff enforces consistent formatting and no bare `print()` statements in production code
- [ ] `docs/guides/xdd-onboarding.md` documents the rich-UI requirement for Python

### Verification Checklist

- [ ] `ruff check agileplus/` produces no errors
- [ ] No bare `print()` statements in `agileplus/` production modules
- [ ] Structured output uses `rich.console.Console()` or equivalent
- [ ] Onboarding doc references the rich-UI requirement

---

## Work Package WP8: Spec Completion and Handoff

**Status**: pending
**Priority**: high
**Dependencies**: WP1–WP7 (all prior work packages)
**Estimated**: ~2 tool calls

### Objective

Finalize the eco-005 spec, update meta.json status to `completed`, and hand off to the next sprint owner.

### Acceptance Criteria

- [ ] All 7 prior WPs are marked `completed` in this document
- [ ] `meta.json` updated: `status` → `"completed"`, `completed_at` added
- [ ] `spec.md` updated: state → `completed`, acceptance criteria filled in
- [ ] A GitHub issue exists for any open follow-up work
- [ ] Worklog entry written to `worklogs/XDD_QUALITY_ENFORCEMENT.md`

### Verification Checklist

- [ ] All checkboxes in WP1–WP7 are ticked
- [ ] `meta.json` status is `"completed"`
- [ ] `spec.md` state is `COMPLETED`
- [ ] Worklog file exists and summarizes findings
- [ ] Branch is merged to main (or PR ready for merge)
