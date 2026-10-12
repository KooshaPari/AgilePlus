# AgilePlus Mature Domain Coverage Map

**Status:** canonical mature-spec coverage denominator  
**Date:** 2026-10-01

The 35 `AP-R-*` semantic requirements form the accepted **work-control spine**. Mature specification finality additionally requires explicit disposition/decomposition of these domains.

| Domain | State | Mature scope |
|---|---|---|
| Development/spec identity | DECOMPOSED | Development, immutable SpecRevision, supersession |
| Work decomposition/DAG | DECOMPOSED | WP identity, dependencies, cycle handling |
| Assignment/context/criteria | DECOMPOSED | frozen assignment, bounded context, non-vacuous criteria |
| Attempts/claims/isolation | DECOMPOSED | replacement, TTL/authority, worktree/sandbox |
| Candidate/evaluation/evidence | DECOMPOSED | exact candidate, independent evaluator, receipts |
| Acceptance/governance | DECOMPOSED | correctness + policy, no self-award |
| Promotion/ship | DECOMPOSED | exact accepted candidate, fail closed |
| Recovery/history | DECOMPOSED | interruption/resumption/supersession |
| Adaptive methodology | DECOMPOSED at semantic level | profiles vary ceremony, not acceptance prerequisites |
| Framework harmonization | DECOMPOSED at semantic level | typed roles, aliases, provenance, conflicts |
| Tracera federation | DECOMPOSED | standalone work truth + product boundary |
| Audit/trace | DECOMPOSED at semantic level | v1 honest envelope, v2 intent, bidirectional work trace |
| CLI | DECOMPOSED | operation-by-operation mature command semantics and machine-readable output contracts |
| HTTP API | DECOMPOSED | resource/action/version/auth/error/idempotency contracts |
| MCP | DECOMPOSED | tool/resource/prompt semantics, parity and authority |
| gRPC | OPTIONAL-PROFILE | if retained, parity/version/TLS contracts |
| Human dashboard/UI | OPTIONAL-PROFILE | inspection/control surfaces without competing truth |
| Git/GitHub | DECOMPOSED | candidate/worktree/PR operations, base drift/conflicts, provenance |
| Triage/intake | DECOMPOSED | classification/routing/priority, override/provenance |
| Scheduling/concurrency | DECOMPOSED | ready-work selection, resource constraints, claims, fairness/priority |
| Review/HITL | DECOMPOSED | escalation, reviewer authority, comment disposition, no review→accept shortcut |
| Release/versioning | DECOMPOSED | promotion profiles, prerelease/stable/HITL policy, rollback/receipts |
| External PM sync | OPTIONAL-PROFILE | Plane/GitHub/etc never canonical authority |
| Event bus | OPTIONAL-PROFILE | NATS/other transport replaceable |
| Graph backend | OPTIONAL-PROFILE | dependency queries without mandatory Neo4j |
| Object storage | OPTIONAL-PROFILE | archival/artifacts without mandatory MinIO |
| P2P/multi-device | OPTIONAL-PROFILE | replication/conflict semantics if product-retained |
| Observability/health | DECOMPOSED | metrics/traces/logs/health must not affect correctness semantics |
| Security/secrets | DECOMPOSED | authn/authz, credential delegation, least privilege, redaction |
| Persistence/migration | DECOMPOSED | local-first durability, migrations, transactions, backup/recovery |
| Configuration/profiles | DECOMPOSED | repository/workflow/tool/provider profiles and precedence |
| Extensibility/plugins | DECOMPOSED | replaceable worker/evaluator/storage/integration boundaries |
| Documentation/fresh-context recovery | DECOMPOSED | dossier/source/decision recovery quality |

## Closure rule

Final mature non-code specification requires every PARTIAL row to become DECOMPOSED or an explicit OPTIONAL/N/A disposition with concrete contracts. OPTIONAL-PROFILE domains remain represented but do not burden the core stage unless the profile is selected.
