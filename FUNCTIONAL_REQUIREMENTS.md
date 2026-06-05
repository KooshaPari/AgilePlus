# Functional Requirements (Canonical Registry)

Status: PROPOSED (eco-034 in flight)
Source of truth: every FR in any active kitty-spec under `AgilePlus/kitty-specs/*/spec.md`.
Schema: `AgilePlus/traces/SCHEMA.md`
Validator: `tooling/trace-validator/`

## FR Index

| FR ID | Title | Spec | Anchor | Status | Trace |
|-------|-------|------|--------|--------|-------|
| FR-024-1 | Per-FR `trace.json` mandatory | eco-024 | #fr-1 | proposed | [trace](traces/FR-024-1.json) |
| FR-024-2 | `trace.json` schema (5 layers) | eco-024 | #fr-2 | proposed | [trace](traces/FR-024-2.json) |
| FR-024-3 | `trace-validator` binary | eco-024 | #fr-3 | proposed | [trace](traces/FR-024-3.json) |
| FR-024-4 | CI gate on every PR | eco-024 | #fr-4 | proposed | [trace](traces/FR-024-4.json) |
| FR-024-5 | `MATRIX.md` generated | eco-024 | #fr-5 | proposed | [trace](traces/FR-024-5.json) |
| FR-024-6 | Journey stubs under `docs/operations/journeys/<fr_id>.md` | eco-024 | #fr-6 | proposed | [trace](traces/FR-024-6.json) |
| FR-024-7 | `--check-anchors` mode | eco-024 | #fr-7 | proposed | [trace](traces/FR-024-7.json) |
| FR-024-8 | `SCHEMA.md` versioning | eco-024 | #fr-8 | proposed | [trace](traces/FR-024-8.json) |

> The remaining FRs from eco-001..033 will be backfilled by the eco-026 autograder's first pass.

## Process

1. Open a PR titled `feat(fr): <id> <title>`.
2. Add the row to this table with `status: proposed` and a stub trace.
3. Implement spec → code → tests → journey stub in one PR.
4. Update status to `accepted` (MVP shipped) or `mature` (post-MVP hardening).
5. Autograder (eco-026) refuses merges that break an `accepted` FR (eco-023).
