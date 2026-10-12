# AgilePlus Mature Source Disposition Ledger

**Status:** semantic source review resolved  
**Date:** 2026-10-01

This ledger prevents repository age, file status labels or implementation existence from deciding product authority.

| Source class | Examples | Mature disposition |
|---|---|---|
| Direct current owner intent | mature-first/MACE/adaptive-method/boundary decisions | CONTROLLING; A0 in authority index |
| Canonical mature contract | manifest, canonical semantic decisions, requirements, journeys, verification catalogues | NORMATIVE |
| Accepted ADRs consistent with mature contract | ADR-0019 and explicitly retained decisions | NORMATIVE-SUPPORTING |
| March PRD / FUNCTIONAL_REQUIREMENTS | `PRD.md`, `FUNCTIONAL_REQUIREMENTS.md` | HISTORICAL IMPLEMENTATION/PRODUCT LINEAGE; 90 FR IDs dispositioned |
| Spec-driven engine specs | `docs/specs/001-...` | HISTORICAL/ADAPT; useful Feature/WP/governance/worktree semantics |
| Release-governance specs | `docs/specs/002-...` | ADAPT/PROFILE; release automation semantics where consistent |
| Platform-completion specs | `docs/specs/003-...` | HISTORICAL/OPTIONAL-PROFILE for service-heavy architecture |
| Module/cycle specs | `docs/specs/004-...` | ADAPT; organizational/cadence projections, not universal lifecycle |
| Helios/Thegent specs in repo | `docs/specs/005-007` | FOREIGN/PORTFOLIO HISTORICAL; not AgilePlus product scope |
| Harmonization analysis | FRAMEWORK_ANALYSIS / UNIFIED_PM_MODEL | RESEARCH/PROPOSAL provenance |
| ADR-0012 | kitty-specs canonical root | SUPERSEDED by typed artifact-role authority |
| ADR-0013 | fixed eight-stage lifecycle | SUPERSEDED as mandatory FSM by adaptive partial-order lifecycle |
| ADR-0014 | RequirementId universal trace namespace | SUPERSEDED/ADAPTED by typed identity graph |
| Runtime Feature/WP FSM | domain implementation | CURRENT IMPLEMENTATION; compatibility surface, not mature ontology authority |
| Claim/lease runtime | triage/application/sqlite | KEEP/ADAPT; Attempt authority semantics governed by mature contract |
| Execution-record migration | SpecRevision/Assignment/Attempt/Evaluation records | CURRENT IMPLEMENTATION EXPERIMENT aligned to mature model; runtime evidence pending |
| Governance evaluator | CLI/shared/HTTP | CURRENT IMPLEMENTATION; semantic target governed by ADR-0019 + verification plan |
| Audit implementation/tests | v1 hash chain | CURRENT IMPLEMENTATION; exact v1 envelope documented, v2 future contract |
| CLI/API/gRPC/MCP | transport implementations/specs | PROJECTIONS/PROFILES; must preserve canonical semantics |
| Plane/NATS/Neo4j/MinIO/P2P | adapters/infrastructure | OPTIONAL-PROFILE unless later explicitly elevated |
| Tests/CI/benchmarks | runtime evidence | EVIDENCE, never product authority |
| Worklogs/session reports/scorecards | historical process evidence | HISTORICAL EVIDENCE |
| Shared agent-lab dossier material | PhenoRegistry/portfolio prior art | ADAPT/MERGE CANDIDATE; not AgilePlus runtime authority by provenance alone |
| Tracera integration docs | federation prior art | ADAPT to explicit work/product authority split |

## Resolution rule

Every known major source generation now has a semantic disposition. Further discovered documents inherit the nearest class above until explicitly reviewed.

A source marked historical/optional/superseded remains valuable evidence and Git history. It is not deleted merely to make documentation look clean.

## Runtime boundary

This ledger closes source **meaning**, not implementation evidence. Runtime reconciliation follows `verification/RUNTIME_EVIDENCE_PLAN.md`.
