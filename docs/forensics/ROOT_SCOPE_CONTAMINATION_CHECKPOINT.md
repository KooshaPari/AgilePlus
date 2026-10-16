# Root scope-contamination checkpoint

## Status

**Resumable forensic checkpoint, observed at `main@61f0aadec2dca0a40281c49e2bd22d0aa13c81b1` on 2026-10-13.**

This document records confirmed active-authority contamination and a bounded first-pass disposition. It is not a complete path-by-path census and does not authorize deletion, movement, or restoration. Issue [#1128](https://github.com/KooshaPari/AgilePlus/issues/1128) tracks completion.

## Human scope ruling

`KooshaPari/AgilePlus` is the AgilePlus tooling/product codebase.

AgilePlus owns governed code-project specifications, WorkPackages, execution records, evidence, evaluation, audit, and development acceptance. It may manage external Git roots at runtime, but it does not become the source repository or product authority for those projects.

Sibling product source, worktrees, product specifications, organization plans, and personal tasks belong in their owning repositories/systems.

## Confirmed active-authority contamination

These files are consumed as root onboarding/governance and therefore directly affect agent behavior:

| Path | Observed contradiction | Disposition in the correction slice |
|---|---|---|
| `README.md` | Declared `NOT AgilePlus`, called the repo a shelf, directed agents into a nested `AgilePlus/`, advertised static completion/deployment badges, and named Electrobun as the desktop implementation. | Replace with an evidence-bound AgilePlus product README and explicit repository boundary. |
| `AGENTS.md` | Declared a lowercase nested worktree, uppercase bare repository, sibling project paths, and DINO-specific operations as current root instructions. | Replace with repository-root, authority, work, safety, and evidence rules for AgilePlus. |
| `GOVERNANCE.md` | Declared the repository a polyrepo shelf and defined portfolio project lifecycle rather than AgilePlus product governance. | Replace with AgilePlus repository/product governance while preserving the supersession history. |

These corrections do not establish that the rest of the tree is clean. They prevent the highest-authority files from producing further contamination.

## Initial path disposition

### A. Active AgilePlus-owned implementation and contracts

The following families are presumptively AgilePlus-owned because they are active workspace/product surfaces, but each material capability still requires reachability and test evidence:

- `Cargo.toml`, `Cargo.lock`, Rust toolchain/configuration files;
- `crates/` AgilePlus workspace members;
- `agileplus-agents/` AgilePlus agent adapters/services;
- `python/` FastMCP and gRPC bridge;
- `desktop/` active Tauri desktop/tray client;
- `spec/product/` product contracts and authority records;
- `verification/` execution, acceptance, recovery, and release evidence contracts;
- `.github/workflows/` CI/security/release/deployment definitions;
- current AgilePlus architecture, operations, SDK, and ADR material under `docs/`.

“Presumptively owned” is not a completion claim. Disconnected, stubbed, superseded, generated, or historical files may exist inside these families.

### B. Historical specification/recovery lineage requiring per-bundle status

- `kitty-specs/`
- `.archive/`
- `findings/`
- historical planning/recovery/session documents under `docs/`
- old root architecture/requirements/worklog material

These paths may contain valuable intent, implementation lineage, migration evidence, or obsolete claims. They remain evidence. A bundle becomes active only through accepted authority/traceability, not by directory name or recency.

Known shelf/portfolio examples include:

- `kitty-specs/012-github-portfolio-triage/`
- `kitty-specs/018-template-repo-cleanup/`
- `kitty-specs/019-private-repo-catalog/`
- `kitty-specs/021-polyrepo-ecosystem-stabilization/`
- `kitty-specs/022-batch13-repo-remediation/`

Their cross-repository/shelf framing is not the current AgilePlus product scope. Preserve them as historical evidence pending ownership migration or supersession records.

### C. Sibling-owned or foreign implementation candidates

The following observed families require provenance and ownership review before they may be considered AgilePlus dependencies or deleted/moved:

- `Dino/` and DINO-specific instructions/specifications;
- `thegent-shm/`;
- `apps/byteport/`;
- any nested full-project tree, worktree, checkout, or project-specific AGENTS/README that describes a product other than AgilePlus;
- Plane-derived product/client implementations that are not merely AgilePlus adapters;
- Tracera, Grapheon, Planify, or other sibling product documents/source copied into this tree.

Proposed ownership direction from current human rulings:

| Candidate | Direction | Current status |
|---|---|---|
| Plane.so-derived product forks/variants | Planify | Provenance and lossless migration still required. |
| Tracera product model/source/specifications | Tracera | Separate forensic recovery controls exact canonical boundary. |
| Grapheon graph/product source/specifications | Grapheon | Separate scope recovery required; do not assume it is merely a Tracera alias. |
| DINO, thegent, Byteport, and other named products | Their owning repositories | Ownership and historical reason for presence remain to be evidenced. |

This table is a routing hypothesis grounded in explicit product boundaries, not proof that every matching path is safe to move.

### D. Vendored, generated, cache, or build-output candidates

Examples may include generated SDKs, lockfiles, frontend build artifacts, `.hutch` content, fixtures, or vendored source. Classification requires:

- build-manifest reachability;
- regeneration source and command;
- license/provenance;
- whether the artifact is intentionally distributed;
- whether tests/package steps consume it;
- whether deleting it would alter supported behavior.

Do not classify a large or duplicated tree as disposable merely because it looks generated.

### E. Unknown

Every unreviewed path remains `unknown`, not implicitly AgilePlus-owned or foreign. Unknowns require a path-level evidence record before movement, deletion, or revival.

## Ownership tests

For each suspicious path, record:

1. first and last accessible commits and rename/copy lineage;
2. introducing issue/PR/commit and human intent, if available;
3. current build/workspace/package inclusion;
4. runtime entrypoint or lack thereof;
5. tests and CI jobs that execute it;
6. release/distribution inclusion;
7. references from active AgilePlus contracts;
8. sibling repository provenance and matching commits;
9. license/security implications;
10. proposed disposition and rollback.

Possible dispositions:

- retain as active AgilePlus implementation;
- retain as explicit adapter/dependency;
- preserve as historical evidence;
- hand off/migrate to owning repository;
- replace with typed external reference;
- archive with successor link;
- intentionally retire;
- investigate.

## Non-negotiable migration rules

- Do not copy a working tree into another repository and call that provenance-preserving migration.
- Record source and destination immutable revisions.
- Preserve useful history, issue/PR references, licenses, tests, and release implications.
- Do not delete the source until the destination is verified and rollback is defined.
- Do not make AgilePlus depend on a sibling runtime merely to remove copied source.
- Do not create another frontend while migrating historical client lineages.
- Do not infer that a historical product implementation belongs in AgilePlus because an integration was once planned.
- Do not import organization/personal tasks as accepted code WorkPackages.

## Required follow-up work packages

### SC-01 — complete path census

Inventory top-level and nested product-like roots; classify build/release reachability and provenance. Output a machine-readable path disposition registry.

### SC-02 — root-document migration map

Identify all documents that still call AgilePlus a shelf or direct work into nested/sibling paths. Mark active successor, historical status, or owning-repository handoff.

### SC-03 — embedded source lineage

Deeply inspect DINO, thegent, Byteport, Plane-derived, Tracera/Grapheon/Planify, and other full-project candidates. Compare with accessible sibling repositories and nominate lossless migrations.

### SC-04 — client lineage disposition

Map Askama/Axum, Tauri, Electrobun, other desktop/web variants, and Plane-derived clients by revision and capability. Retain one coherent AgilePlus client family; route product forks to their owners.

### SC-05 — packaging and CI scope audit

Determine which suspicious paths affect workspace membership, CI duration, release artifacts, dependency scanning, coverage, and installers. Prevent “cleanup” from silently changing distribution.

### SC-06 — guarded migration execution

For each approved migration, use a dedicated PR with source/destination revisions, tests, compatibility evidence, rollback, and explicit operator approval. No bulk sweep.

## Negative controls

The scope correction is incomplete if any of these remain possible:

1. A fresh agent reading root authority files concludes this is a multi-project shelf.
2. An agent is instructed to create or enter a nested `AgilePlus/` or `agileplus/` source root.
3. Registering an external project copies its source/specification into AgilePlus.
4. A Plane-derived product client is added as another AgilePlus native frontend.
5. A sibling task marked done grants AgilePlus acceptance.
6. Historical shelf content silently becomes active intent.
7. Cleanup deletes build/release inputs without executable comparison.
8. A migration loses provenance or leaves two canonical copies.
9. Unknown paths are reported as resolved merely because root docs were fixed.

## Evidence and limitations

This checkpoint deeply inspected only the three root authority files and a bounded set of search-discovered shelf/sibling candidates. It did not execute a full Git-history index, enumerate every path, compare every sibling repository, or run product tests. Those remain explicit follow-up work.

The immediate correction is therefore **authority containment**, not a claim that repository contamination has been fully removed.
