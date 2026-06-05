# Traceability (Phenotype Org)

The Phenotype Org tracks FRs in a single canonical registry at
`AgilePlus/FUNCTIONAL_REQUIREMENTS.md`. Each FR has a `trace.json` at
`AgilePlus/traces/<fr_id>.json` that binds it to:

- the spec slug + anchor (one hop to the source of truth)
- the docs pages that describe it
- the test paths that verify it
- the code modules that implement it
- the journey stub (`docs/operations/journeys/<fr_id>.md`) that walks it

A `trace-validator` (eco-024, eco-026) walks every trace on every PR and
refuses merges that break a previously-`accepted` FR (eco-023).

See `AgilePlus/traces/SCHEMA.md` for the v1 shape.
See `AgilePlus/traces/MATRIX.md` for the per-FR green/yellow/red rollup.
