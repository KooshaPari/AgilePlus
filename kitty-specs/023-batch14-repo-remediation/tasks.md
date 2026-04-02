# Work Packages: Batch 14 Repo Remediation

**Inputs**: Audit report from batch-14-audit
**Prerequisites**: spec.md
**Scope**: 4 repos remediation

---

## WP-001: Remove clikit

- **State:** planned
- **Sequence:** 1
- **File Scope:** clikit/ directory
- **Acceptance Criteria:**
  - clikit directory removed
- **Estimated Effort:** XS

Remove empty clikit directory (not a git repo, contains only SECURITY.md and TEST_COVERAGE_MATRIX.md).

### Subtasks
- [ ] T001 Remove clikit directory

### Dependencies
- None

---

## WP-002: Scaffold bare-cua

- **State:** planned
- **Sequence:** 2
- **File Scope:** bare-cua/ — CI/CD, CHANGELOG, VERSION, AgilePlus
- **Acceptance Criteria:**
  - .github/workflows/ci.yml with Rust fmt, clippy, test
  - CHANGELOG.md created
  - VERSION file created
  - .agileplus/worklog.md created
- **Estimated Effort:** S

Add CI/CD and basic scaffolding to bare-cua.

### Subtasks
- [ ] T002 Create .github/workflows/ci.yml
- [ ] T003 Create CHANGELOG.md
- [ ] T004 Create VERSION file
- [ ] T005 Create .agileplus/worklog.md

### Dependencies
- WP-001 (can start in parallel)

---

## WP-003: Add CHANGELOG/VERSION to colab and devenv-abstraction

- **State:** planned
- **Sequence:** 2
- **File Scope:** colab/, devenv-abstraction/
- **Acceptance Criteria:**
  - colab/ has CHANGELOG.md and VERSION
  - devenv-abstraction/ has CHANGELOG.md and VERSION
- **Estimated Effort:** XS

Add CHANGELOG.md and VERSION to projects that already have good CI/CD.

### Subtasks
- [ ] T006 Create colab/CHANGELOG.md
- [ ] T007 Create colab/VERSION
- [ ] T008 Create devenv-abstraction/CHANGELOG.md
- [ ] T009 Create devenv-abstraction/VERSION

### Dependencies
- WP-001 (can start in parallel)

---

## Dependency & Execution Summary

```
WP-001 (Remove clikit) ───────────── first, no deps
WP-002 (Scaffold bare-cua) ───────── parallel with WP-001
WP-003 (CHANGELOG/VERSION) ─────────── parallel with WP-001
```
