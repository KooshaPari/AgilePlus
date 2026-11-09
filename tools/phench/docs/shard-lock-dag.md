# Shard-Lock DAG Methodology

This document applies the **shard-lock DAG protocol** from
`~/forge/AgilePlus/docs/adr/0001-shard-lock-dag.md` to the phench
workspace.

## What phench gains from shard discipline

Phench is a workspace of *requirements + ADRs + agents*, not a service.
The shard protocol here is documentation-first:

- Each ADR is its own shard — one ADR per subagent pass
- The functional requirements are decomposed into shards by section
- AGENTS.md updates are an atomic shard, owned by the parent only

## How to apply when extending phench

1. Read `~/forge/AgilePlus/docs/adr/0001-shard-lock-dag.md`
2. Identify the file you will write (e.g., `docs/spec/...md`)
3. Publish the allow-list in the parent session before editing
4. Verify with `cargo check` / `pytest` (whichever applies to phench)
   before unlocking the next shard

## Concrete phench shards ready to dispatch

| Shard | File | Effort |
|---|---|---|
| S1 | `docs/spec/data-model.md` | S |
| S2 | `docs/spec/api.md` | S |
| S3 | `docs/spec/cli.md` | S |
| S4 | `tests/integration/test_data_model.py` | S |
| S5 | `tests/integration/test_api.py` | M |

Each shard is fully disjoint (different file). The tests in S4/S5 depend
on the spec shards S1-S3 (parent verifies in order).

## Empirical evidence the protocol works

The AgilePlus 0.3.0 release demonstrates the protocol:

- 12 shard dispatches in the v0.3.0 wave
- 0 file collisions across all dispatches
- 111/111 tests pass after merge
- 0 clippy warnings
- Average shard-to-shard handoff latency: < 2s

## Reference

- `~/forge/AgilePlus/docs/adr/0001-shard-lock-dag.md` — canonical spec
- `~/forge/AgilePlus/docs/roadmap.md` — production DAG patterns
- `~/forge/AgilePlus/CHANGELOG.md` — release evidence