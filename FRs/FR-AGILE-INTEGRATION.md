# FR-AGILE-INTEGRATION: Integration Tests

## Overview

Full workflow integration tests for AgilePlus. These tests exercise the complete specify -> research -> plan -> implement -> validate -> ship lifecycle against a real running stack.

## Tests (FR-AGILE-065 to FR-AGILE-068)

| FR ID | Description | Test Function |
|-------|-------------|---------------|
| FR-AGILE-065 | Full specify to ship workflow | `test_full_specify_to_ship_workflow` |
| FR-AGILE-066 | Specify rejects invalid state transition | `test_specify_rejects_invalid_state_transition` |
| FR-AGILE-067 | Validate blocks on missing evidence | `test_validate_blocks_on_missing_evidence` |
| FR-AGILE-068 | Audit chain integrity after full lifecycle | `test_audit_chain_integrity_after_full_lifecycle` |

## Source

- File: `tests/integration/test_full_workflow.rs`
- Traceability: WP16-T096
