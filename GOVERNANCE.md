# AgilePlus repository governance

## Scope

This document governs `KooshaPari/AgilePlus`, the AgilePlus tooling/product codebase.

AgilePlus is not a repository shelf. Sibling products, organization plans, personal tasks, and their canonical specifications remain in their owning systems and repositories. AgilePlus may reference or operate on external Git repositories without absorbing their source or authority.

## Product authority

| Domain | Canonical authority |
|---|---|
| Code-project specifications, WorkPackages, execution records, evidence, evaluation, audit, and development acceptance | AgilePlus |
| Persistent product/system records and product-state interpretation | Tracera |
| Organization/personal hierarchy, schedule, due date, and coordination status, when adopted | Plane or Paperclip |
| Repository/release approval | Koosha/operator and configured repository protections |

No external product, planning system, dashboard, import, or generic state-transition API may grant AgilePlus terminal acceptance.

## Decision making

| Decision | Authority and process |
|---|---|
| Product scope or ecosystem boundary | Operator ruling, recorded in an issue/ADR when material |
| Architecture | Evidence-backed proposal and ADR; operator accepts material trade-offs |
| Requirement or semantic-contract change | Preserve source intent and supersession; review against authority index and executable acceptance |
| Dependency change | Focused PR with lockfile/source impact, advisories, compatibility, and relevant tests |
| Database migration | Additive/reversible where possible; backup, compatibility, negative controls, and migration receipt |
| Release/deployment | Evidence gates in `STABLE.md` and release records; no badge-only approval |
| Merge | Required review/protection plus explicit authorized merge action |
| Removal/retirement | Proven ownership and replacement/retirement decision; preserve provenance |

Agents research, implement bounded changes, and report evidence. They do not manufacture human approval.

## Repository boundary

### Allowed

- AgilePlus source, tests, schemas, migrations, documentation, packaging, and product-owned adapters.
- Typed references to external repositories and records.
- Fixtures or vendored dependencies with explicit provenance, license, update, and build boundaries.
- Historical/recovery evidence retained with clear status.

### Not allowed by default

- Sibling repository clones or worktrees placed under AgilePlus.
- Canonical sibling product specifications copied into AgilePlus.
- Organization/personal planning records treated as AgilePlus WorkPackages.
- Plane-derived product forks implemented as AgilePlus native clients.
- Tracera product records duplicated as an AgilePlus product SSOT.
- Unclassified foreign trees presented as active AgilePlus implementation.

Suspected shelf-era or misplaced material is preserved and classified through issue #1128 rather than silently deleted or accepted.

## Change workflow

1. Identify the owning domain and source of intent.
2. Inspect current implementation, tests, packaging, and relevant history.
3. State observed behavior separately from proposed target behavior.
4. Open or reference a bounded issue/spec/ADR.
5. Work on a focused branch or isolated worktree.
6. Add positive and negative tests that can distinguish the intended behavior from trivial/stub behavior.
7. Run applicable checks and retain exact evidence.
8. Open a draft PR while evidence, review, migration, or rollout gates remain incomplete.
9. Merge only through an authorized action after required evidence and review.
10. Deploy/release separately; a merged PR is not deployment evidence.

## Quality and acceptance gates

Applicable changes should address:

- formatting, lint, build, unit/integration tests;
- domain and lifecycle invariants;
- evidence and audit-chain integrity;
- migration and rollback safety;
- platform and packaging behavior;
- authorization, credentials, and secret handling;
- accessibility and operator error handling;
- performance/resource bounds where material;
- failure propagation and negative controls;
- exact artifact/revision provenance.

Skipped, waived, advisory, environment-blocked, and not-run checks remain distinct from passes. Baseline CI defects are isolated rather than hidden inside product PRs.

Terminal development acceptance follows ADR-0019. It requires exact-candidate correctness evidence plus applicable governance/policy. Review approval, external task completion, or evidence presence alone is insufficient.

## Client and writer governance

- CLI, MCP, native web, and tray/desktop converge on one application/writer boundary.
- Raw client SQL must not bypass lifecycle, audit, event, or acceptance semantics.
- During daemon migration, direct SQLite and daemon-backed mutation paths must be explicitly fenced; no silent dual-writer operation.
- External projections use typed, idempotent, asynchronous contracts and cannot control local acceptance.

## Release governance

The authoritative release posture is evidence-bound and tiered in `STABLE.md`.

A release claim must identify:

- exact source revision;
- artifacts built from that revision;
- platforms and signing state;
- checks actually executed;
- installer/upgrade/rollback/uninstall results;
- known waivers and residual advisories;
- deployed or installed instances actually verified.

Static badges, file presence, historical releases, and workflow definitions do not establish current rollout completion.

## Agent authority

| Action | Default agent authority |
|---|---|
| Read/search/history inspection | Allowed when access is authorized |
| Isolated reproduction and bounded validation | Allowed with safety/resource limits |
| Documentation, diagnostics, focused branch commits, draft PRs | Allowed when the task authorizes repository work |
| Modify unrelated concurrent work or active user worktree | Not allowed |
| Merge, force-push, delete refs/history, deploy, publish, DNS/auth/billing changes | Not allowed without explicit authorization |
| Move/delete sibling or historical material | Not allowed without provenance and approved disposition |

## Historical governance material

Shelf-era governance, portfolio specifications, and embedded project instructions remain historical evidence pending disposition. They are non-authoritative when they conflict with this document, accepted ADRs, or explicit operator rulings. Supersession should be recorded; history should not be rewritten to hide the conflict.

## Change log

| Date | Change |
|---|---|
| 2026-03-29 | Shelf-governance document introduced, displacing AgilePlus-specific governance. |
| 2026-10-13 | Repository governance restored to the AgilePlus product boundary; shelf-era material retained as historical evidence pending issue #1128. |
