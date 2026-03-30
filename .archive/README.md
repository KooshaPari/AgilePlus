# Archive: Duplicate Test Files

This directory contains duplicate files that were archived to reduce disk space waste and prevent agent confusion.

## Rationale

During a workspace LOC audit (2026-03-29), approximately 35,000 lines of duplicated test files were identified across worktrees. These duplicates waste disk space and create confusion when multiple agents edit the same logic in different locations.

Per the **Phenotype Long-Term Stability and Non-Destructive Change Protocol**, duplicates are moved to `.archive/` rather than deleted, preserving them for reference while eliminating active confusion.

## Contents

### duplicate-tests/
- **test_phench_runtime.py** (2,120 lines)
  - Duplicate of canonical copy at: `/platforms/thegent/tests/test_phench_runtime.py`
  - Tests phench runtime behavior
  - Archived from worktree to prevent duplicate editing

## Total LOC Archived

- Python test files: 2,120 lines

## Canonical Locations

All active development should reference these canonical locations:
- Test files: `/platforms/thegent/tests/`

## Recovery

If you need to reference or recover archived files, they remain intact in this directory. To restore, copy back to the original location with appropriate rebasing against the canonical version.

---
**Archived:** 2026-03-29
**Policy:** Phenotype Long-Term Stability and Non-Destructive Change Protocol
**Related Issue:** Wave 93 LOC audit and cleanup
