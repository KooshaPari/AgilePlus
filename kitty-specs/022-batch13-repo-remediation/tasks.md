# Work Packages: Batch 13 Repo Remediation

**Inputs**: Audit report from batch-13-audit
**Prerequisites**: spec.md, access to all 7 repos in batch 13
**Scope**: Shelf-level (cross-repo) — 5 repos archived, 2 repos scaffolded

---

## WP-001: Archive Empty Repos (KaskMan, kits, packs, portage, zen)

- **State:** planned
- **Sequence:** 1
- **File Scope:** 5 empty/near-empty repos → .archive/
- **Acceptance Criteria:**
  - KaskMan moved to .archive/KaskMan/
  - kits moved to .archive/kits/
  - packs moved to .archive/packs/
  - portage moved to .archive/portage/
  - zen moved to .archive/zen/
  - projects/INDEX.md updated with archival status
  - Orphaned worktree references cleaned up
- **Estimated Effort:** S

Move 5 empty repos to .archive/ directory. Update INDEX.md to reflect archival. Clean up any worktree references.

### Subtasks
- [ ] T001 Move KaskMan to .archive/KaskMan/
- [ ] T002 Move kits to .archive/kits/
- [ ] T003 Move packs to .archive/packs/
- [ ] T004 Move portage to .archive/portage/
- [ ] T005 Move zen to .archive/zen/
- [ ] T006 Update projects/INDEX.md archival table
- [ ] T007 Clean up orphaned worktree references

### Dependencies
- None (starting WP for this spec)

### Risks & Mitigations
- Losing worktree references: Document before archival
- Breaking existing references: Update INDEX.md

---

## WP-002: Populate dotfiles Scaffolding

- **State:** planned
- **Sequence:** 2
- **File Scope:** dotfiles/ repo — README.md, .gitignore, .agileplus/, AGENTS.md
- **Acceptance Criteria:**
  - README.md with project description, structure, and usage guidance
  - .gitignore covering governance/ and hooks/ patterns
  - .agileplus/ directory with worklog.md
  - AGENTS.md with agent guidance for dotfiles project
- **Estimated Effort:** S

Create basic scaffolding for dotfiles repo which already has governance/ and hooks/ content.

### Subtasks
- [ ] T008 Create README.md with project description and structure
- [ ] T009 Create .gitignore for dotfiles patterns
- [ ] T010 Create .agileplus/ directory with worklog.md
- [ ] T011 Create AGENTS.md for agent guidance

### Dependencies
- WP-001 (can start in parallel)

### Risks & Mitigations
- Over-scoping: Strict scope — scaffolding only, no new functionality

---

## WP-003: Populate harnesses Scaffolding

- **State:** planned
- **Sequence:** 2
- **File Scope:** harnesses/ repo — README.md, .gitignore, .agileplus/, AGENTS.md
- **Acceptance Criteria:**
  - README.md explaining harness purpose and available configs
  - .gitignore for harness patterns
  - .agileplus/ directory with worklog.md
  - AGENTS.md for agent guidance
- **Estimated Effort:** S

Create basic scaffolding for harnesses repo which already has agent config files.

### Subtasks
- [ ] T012 Create README.md explaining harness purpose
- [ ] T013 Create .gitignore for harness patterns
- [ ] T014 Create .agileplus/ directory with worklog.md
- [ ] T015 Create AGENTS.md for agent guidance

### Dependencies
- WP-001 (can start in parallel)

### Risks & Mitigations
- Over-scoping: Strict scope — scaffolding only, no new functionality

---

## WP-004: Update INDEX.md and Write Worklog

- **State:** planned
- **Sequence:** 3
- **File Scope:** projects/INDEX.md, worklogs/GOVERNANCE.md
- **Acceptance Criteria:**
  - projects/INDEX.md updated with archival status for 5 repos
  - Worklog entry written documenting the remediation
  - All changes committed and documented
- **Estimated Effort:** XS

Final documentation updates.

### Subtasks
- [ ] T016 Update projects/INDEX.md archival table
- [ ] T017 Write worklog entry at worklogs/GOVERNANCE.md
- [ ] T018 Commit all changes with proper messages

### Dependencies
- WP-001 (archival complete)
- WP-002 (dotfiles scaffolded)
- WP-003 (harnesses scaffolded)

---

## Dependency & Execution Summary

```
WP-001 (Archive empty repos) ──────────── first, no deps
WP-002 (Populate dotfiles) ────────────── parallel with WP-001
WP-003 (Populate harnesses) ───────────── parallel with WP-001
WP-004 (Update INDEX.md + worklog) ────── depends on WP-001, WP-002, WP-003
```

**Parallelization**: WP-001, WP-002, and WP-003 can run in parallel. WP-004 is the final step.
