# ADR-0018: Tracera and AgilePlus Are Interoperable Sibling Runtimes

## Status

Accepted for the mature-product recovery program on 2026-09-30.

## Supersedes

This ADR supersedes the runtime-coupling and ownership conclusions of:
- ADR-0011 (Tracera embeds AgilePlus);
- ADR-0017 (embedding migration path);
- the embedding-specific portions of ADR-0015;
- the runtime-ownership table in ADR-0016 where it makes Tracera subordinate to AgilePlus.

Those documents remain historical design evidence and should not be deleted.

## Context

The harmonization generation correctly identified several problems:
- duplicate lifecycle/governance semantics can drift;
- work execution should have one authoritative process machine;
- graph color/checklist state should not independently override real process state;
- shared identifiers and evidence contracts are valuable.

However, later product recovery re-established a stronger product boundary:

### AgilePlus
Owns durable development/work execution:
- specification evolution;
- work packages;
- claims/leases;
- worktrees;
- attempts/workers;
- execution FSM;
- review/retry/replan;
- execution receipts.

### Tracera
Owns persistent product truth:
- product identity;
- accepted product intent;
- product configurations/baselines;
- multi-projection product graph;
- realized/observed state;
- evidence interpretation;
- dissatisfaction;
- product-level MACE assessment;
- lifecycle/product-state history.

A worker attempt is ephemeral.
A development effort survives workers.
Product truth survives development systems.

Therefore making Tracera unable to run without AgilePlus would couple product truth to one execution system and violate the product boundary.

## Decision

1. **Tracera and AgilePlus are sibling runtimes.**
   Neither is a mandatory embedded child of the other.

2. **Both may run independently.**
   - AgilePlus can manage development without Tracera.
   - Tracera can model/assess products whose work is performed by AgilePlus, another system, humans, CI, external vendors, or no active work system.

3. **AgilePlus is authoritative for development-process state.**
   Tracera must not reimplement AgilePlus claim/lease/work-attempt semantics and then treat the duplicate as authoritative.

4. **Tracera is authoritative for product-state interpretation.**
   AgilePlus cannot mark a product requirement/capability satisfied merely because a work item reached Done.

5. **Integration is contract/event/evidence based.**
   Candidate shared surfaces include:
   - ProductChange / dissatisfaction references from Tracera to AgilePlus;
   - DevelopmentId / WorkPackage / Attempt references from AgilePlus;
   - immutable artifact/evidence receipts back to Tracera;
   - shared identifiers and pure schemas where useful;
   - events/webhooks/MCP/API adapters;
   - optional in-process adapters when deployment topology makes them advantageous.

6. **No runtime dependency is globally mandatory.**
   A deployment may choose:
   - both systems;
   - AgilePlus only;
   - Tracera only;
   - either system integrated with another tool.

7. **Process machine over checklist survives, scoped correctly.**
   Within AgilePlus, execution progression is enforced by its process machine rather than an external graph color/checklist.
   Within Tracera, product assessment is independent and evidence-bound.
   The two states may legitimately disagree:
   - work can be Done while product proof is missing;
   - product can be Satisfied from evidence produced outside AgilePlus.

8. **Shared types do not imply shared authority.**
   A common schema/ID crate may be reused without forcing runtime ownership or deployment coupling.

## Consequences

### Positive
- Tracera remains usable for non-AgilePlus products/workers.
- AgilePlus remains usable without the product graph.
- Clear three-lifetime model.
- Less version/deployment coupling.
- External tools remain pluggable.
- Work completion cannot manufacture product green.

### Costs
- Integration contracts must be explicit.
- Cross-system consistency is eventual/evidence-based rather than one in-process state machine.
- Shared concepts need authority labels to avoid duplicate truth.

## Required follow-up

- Re-evaluate ADRs 0012–0016 for assumptions inherited from ADR-0011.
- Keep useful artifact/lifecycle/harmonization ideas where they do not violate this boundary.
- Replace embedding migration work with federation/integration contracts.
- Add shared E2E witness:
  Tracera ProductChange → AgilePlus DevelopmentId → worker replacement → artifacts/evidence → Tracera independent reconciliation.

## Invariant

**Development truth is not product truth. Integration connects them; it does not collapse them.**
