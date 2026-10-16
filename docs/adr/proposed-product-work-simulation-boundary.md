# Proposed amendment: product graph inputs and bounded work acceptance

Status: proposed, 2026-10-14. Unnumbered pending architecture review; does not
supersede any accepted ADR. Read
[ADR-0019](0019-acceptance-authority-terminal-gates.md), the
[canonical semantic decisions](../../spec/product/CANONICAL_SEMANTIC_DECISIONS.md)
and the [source dossier](../../spec/product/recovery-2026-10-14/README.md).

## Context and source basis

The [owner excerpts](../../spec/product/recovery-2026-10-14/OWNER_EXCERPTS.md)
S01–S03, S06 and S09 establish a multi-view editable product graph, not a ledger-only
product. S04 motivates adaptive specification with independent grading. S05 and
S07 motivate model-first journey and ergonomic simulation; S08 adds research
experimentation. These goals require explicit product/work and model/reality
boundaries. Their runtime realization is not demonstrated by this amendment.

## Proposed decisions

Tracera owns canonical typed product entities, typed relationships, profile
versions and product baselines across software/hardware/other declared domains.
Views project common identity and applicable facts. Missing types or import loss
must be explicit. Storage engine selection and migrations require workload and
compatibility evidence; this proposal does not prescribe a storage rewrite.

A product-changing gesture produces a proposal binding affected entities, base
revision, authority, preconditions and expected outputs. A supported deterministic
adapter may produce a candidate; other changes become bounded human or agent
Assignments through AgilePlus or a compatible executor. AgilePlus retains its
accepted identity model and standalone grading. Board or orchestration surfaces
remain projections/coordinators, not another permanent work authority.

The proposed federation envelope binds product baseline, accepted SpecRevision,
Assignment, Attempt, exact Candidate, evaluator/version, acceptance-relevant
configuration, criterion results and evidence references. Version compatibility,
unsupported capabilities and applicability are explicit. Identical receipt replay
must not duplicate effects. A stale baseline remains stale/pending review rather
than silently applying results to a newer product. Correctness and transition
permission remain separate under ADR-0019. Tracera independently admits product
applicability; work acceptance does not imply product satisfaction.

A proposed Tracera Scenario binds ProductBaseline, environment, actors,
devices/peripherals, initial state, ordered/branching actions, model/solver
versions, seed where relevant, assumptions and metrics. A Run preserves immutable
inputs and outputs. Rendering references canonical entities without inventing
implemented behavior. The following fidelity labels are engineering proposals,
not verbatim owner terminology:

| Proposed fidelity                 | Required validity declaration                                               |
| --------------------------------- | --------------------------------------------------------------------------- |
| Semantic playback                 | Sequence/relationship visualization; no physical accuracy claim             |
| Behavioral simulation             | Modeled state/timing and declared implementation correspondence             |
| Physical/ergonomic simulation     | Units, frames, geometry/pose assumptions, uncertainty and solver validation |
| Empirically calibrated prediction | Measured comparator, applicability range and error distribution             |

Per-variable provenance must distinguish measured/physical/semantic/illustrative
values and reconcile with existing spatial geometry-truth proposals. Unknown
motion, dimensions or timing remain unknown. Renderer output does not prove
realized behavior or ergonomic safety. Realization enters the reviewed
work/candidate/evaluation path. Physical actuation and deployment require their
own authority; editing a scene cannot authorize either.

Product-aware views and graph cards belong to Tracera. API lenses require typed
API entities and explicit not-modeled states; ordinary requirements cannot acquire
an invented HTTP method. Contract-defined, route-mounted, deployed and verified
are distinct claims. Rich previews and decomposition navigation need distinct
accessible actions, explicit absence/loading states and source/asset provenance.
No universal layout grid or fixed panel percentage is prescribed. Known zero,
missing, error, stale and invalid-denominator metrics remain distinct. Independent
user-task/design evidence and usability review accompany implementation evidence;
this proposal provides no legal determination.

## Alternatives and consequences

Separate truth stores per view risk conflicting identity. One untyped universal
row loses domain semantics. A full storage rewrite lacks current justification.
Duplicating a work authority inside Tracera, worker self-award or mandatory live
Tracera grading conflicts with accepted AgilePlus semantics. Treating animation as
validation or imposing one physics engine on every domain cannot establish model
validity. Incumbent cloning, superficial recoloring and generic tables do not
establish task fitness or independent design provenance.

Follow-ups require compatibility review, import-loss reporting, replay/stale
baseline oracles, scenario assumption registries, calibration datasets and
change-impact invalidation. Renderer/solver choices remain benchmarked proposals.
See [future witnesses](../../spec/product/recovery-2026-10-14/FUTURE_WORK.md).
No runtime migration, work registration or authority transfer occurs here.
