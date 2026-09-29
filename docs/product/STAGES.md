# AgilePlus VP Stages

Stages are projections over `AGP-MATURE-V1`.

## CVP

The smallest real encapsulated AgilePlus should close:

```text
intake
 -> specification
 -> plan/work-package DAG
 -> scoped resource claim/worktree
 -> agent or human execution
 -> review
 -> governance/evidence gate
 -> ship/close work
 -> immutable receipt/audit
```

The spine must be real. Optional external sync, distributed operation and richer interfaces may be stubbed or deferred.

## MVP

MVP makes the same execution spine repeatedly useful across multiple features/repos/agents, with durable recovery, stronger CLI/API/MCP/dashboard surfaces, reliable governance, external references/sync and reciprocal Tracera federation.

## GA

GA requires supported security, recovery, packaging/runtime, compatibility, release proof, external integration behavior and broad accepted journey closure.

## Grade shape

Never report only "AgilePlus is N% done." Report mature completeness, stage readiness, structural shape, closed core features/journeys, survivability, mature-contract compatibility, transition burden and blockers.

Shape vocabulary: `scaffold | primitive_system | product_husk | vertical_slice | narrow_functional_product | broad_product`.

A stage may be blocked despite high numerical completion when an essential execution/gating journey is open.
