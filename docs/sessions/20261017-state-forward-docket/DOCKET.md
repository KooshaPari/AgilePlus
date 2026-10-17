# State & Forward Docket — AgilePlus 2026-10-17

**Format:** State & Forward Docket v1.0 (`phenotype-registry/handbook/specs/assessment/state-forward-docket-v1/`)
**Subject:** `KooshaPari/AgilePlus` — accepted mature semantic contract vs runtime completion
**Snapshot identity:** `ap-sfd-2026-10-17T08-53Z@00397a59`
**Repo HEAD at snapshot:** `00397a59 docs: reconcile product intent with canonical work authority [skip ci] (#1134)`
**Active branch when docket was written:** `docs/state-forward-docket-20261017` (branched from main tip 00397a59)
**Latest date-tagged release on the lineage that includes this HEAD:** `v2026.09.0` (not on the current branch ancestry; `git describe --tags --abbrev=0` reports `v0.2.4` because the current branch tip is at the legacy `v0.x` ancestry until a new tag is cut)
**Docket author:** parent orchestrator session (agileplus chat), with the integration-audit session as the only in-repo audit source consulted at write time (`docs/sessions/20261017-integration-audit/INTEGRATION.md`)

---

## 0. Decision cockpit (2 minutes)

| Field | Value |
|-------|-------|
| **Subject** | Is the AgilePlus mature semantic contract fulfilled at runtime, and what is the shortest honest path to a remotely-usable operator alpha? |
| **Usable now** | The 38-crate workspace compiles for native Rust targets. `agileplus-cli` and `agileplus-api` binaries build. SQLite is the live store. Plane.so outbound push is env-gated. gRPC validates now route through the canonical atomic-acceptance use case. The promotion saga (planned, prepared, compare-and-swap publication, per-step confirmation, atomic Shipped/audit/event/receipt finalization, post-commit cleanup) is implemented and has 8 real Git+SQLite recovery witnesses and 14+10 CLI ship witnesses in CI. A real Chromium browser journey against the actual Axum/file SQLite API passed in CI. |
| **Usable now — limits** | "CI witness" ≠ "production deployment". The cloud browser opened the public product URL and observed the old dashboard, with zero epics/stories. No production deployment or desktop/tailnet execution is claimed in `verification/PROMOTION_RECOVERY_2026-10-08.md`. No Tailscale daemon is running. The 4 homebrew binaries (`nats-server`, `minio`, `redis-server`) are installed but dormant. `agileplus-nats` is misleadingly named (in-memory only — no `async-nats` dep). `agileplus-plane` is env-gated. MinIO has zero S3 client deps in the codebase. The plugin registry symbol does not exist. |
| **Decision now** | Accept the mature semantic contract as **SEMANTIC-ACCEPTED · RUNTIME-PENDING**. Do not retitle the product MVP/release. Do not collapse Implementation / Usability / Validation / Assurance into a single "ready" verdict. Treat 0/92 specified verification cases as the single most-load-bearing number in this docket. |
| **Time to safe** | ≈ 1–2 working days (operator API alpha, native): the canonical atomic-acceptance Rust tests, real-SQLite HTTP acceptance tests, gRPC acceptance tests, interrupted-Attempt replacement tests, and promotion preflight/drift tests are all implemented at source and waiting to be **executed** on a clean native toolchain. `verification/FORWARD_DELTA_2026-10-04.md` is explicit: "Test existence is not test execution." |
| **Time to finished** | ≈ 8–15 working days (per `verification/FORWARD_WBS_2026-10-04.md`) for a remotely-usable, zero-paid-service, SaaS-style private/operator alpha over the Vercel Hobby → Tailscale → Caddy → owner's desktop → `agileplus-api` → SQLite topology. |
| **Single load-bearing number** | **`0 of 92`** specified verification cases have an executed witness. **`0 of 67`** requirements have an executed runtime witness. The mature contract is accepted at the **semantic** layer; it is **not** fulfilled at the **runtime** layer. |
| **What closes this docket** | All 92 cases reach `status: executed_with_witness` (or an explicit `not_applicable` with a citation that names the same-case number under another case id), all 67 requirements reach `verification_status: runtime_verified`, the 10 NEXT_DEPLOYABLE_RELEASE items (see §6) are green against an actual remote operator, the production URL shows real data, and the operator's `agileplus` binary can be invoked from outside the developer's shell against the real artifact. |

### Verdict vocabulary used in this docket

`PASS` · `FAIL` · `NOT_RUN` · `UNKNOWN` · `STALE` · `BLOCKED` · `NOT_APPLICABLE` (per SFD v1.0)

The three questions that **never** collapse:

1. **Implementation** — is the code present in this branch? (cite path/line)
2. **Usability** — can an operator reach it from outside the developer's shell? (cite URL/env/probe)
3. **Validation** — is there an executed witness? (cite CI run id, transcript, screenshot, or `UNKNOWN`)
4. **Assurance** — is the witness independent of the implementer? (self-reported vs CI vs external)

These four are reported as separate columns in the gate board (§10). They are **never** averaged, and no claim is moved to `PASS` because a downstream row looks promising.

---

## 1. Source / evidence bindings

Every claim in this docket cites one of the following sources. Read order is top-down for confidence.

| Source | Path | Authority | Last touched in this work |
|--------|------|-----------|----------------------------|
| Mature semantic contract | `spec/product/mature-contract.v1.json` | canonical product-method acceptance | 2026-10-17 read in full |
| Mature domain coverage map | `spec/product/MATURE_DOMAIN_COVERAGE_MAP.md` | canonical 33-domain breakdown | 2026-10-17 read in full |
| Authoritative requirement index | `spec/product/AUTHORITY_INDEX.md` | canonical req id authority | 2026-10-17 listed |
| 67 requirements | `spec/product/requirements.semantic.v1.json` | machine-readable requirement set | 2026-10-17 enumerated |
| 12 journeys | `spec/product/journeys.v1.json` | journey-level acceptance shapes | 2026-10-17 enumerated |
| 92 verification cases | `verification/semantic-verification-cases.v1.json` | machine-readable case set, `status: specified_not_executed` | 2026-10-17 enumerated |
| 9 canonical semantic decisions | `spec/product/CANONICAL_SEMANTIC_DECISIONS.md` | decision-of-record for ambiguities | 2026-10-17 read in full |
| PRD (12 epics) | `PRD.md` | product requirement document | 2026-10-17 read in full (lines 1–215) |
| Spec acceptance contract | `verification/SPECIFICATION_ACCEPTANCE_CONTRACT.md` | 22 domains, 11 oracle fields, 20 adversarial families | 2026-10-17 read in full |
| Forward WBS (operator-alpha scope) | `verification/FORWARD_WBS_2026-10-04.md` | canonical 1-2 / 4-7 / 8-15 day ETAs | 2026-10-17 read in full |
| Next deployable release checklist | `verification/NEXT_DEPLOYABLE_RELEASE_2026-10-04.md` | 10 named release requirements | 2026-10-17 read in full |
| Forward delta (recovery progress) | `verification/FORWARD_DELTA_2026-10-04.md` | 2026-10-08 promotion/recovery deltas | 2026-10-17 read in full |
| Promotion recovery evidence | `verification/PROMOTION_RECOVERY_2026-10-08.md` | the closest thing to a "runtime witness" we have | 2026-10-17 read in full |
| Network & deployment doctrine | `docs/architecture/NETWORK_DEPLOYMENT_DOCTRINE.md` | tailnet/private topology of record | 2026-10-17 read in full |
| Recovery authority | `AGENTS.md` (Recovery Authority section) | hierarchy of sources | 2026-10-17 read partial |
| Charter | `CHARTER.md` | governance boundary; out-of-scope: fleet registry, Tracera ledger | 2026-10-17 read in full |
| Integration audit (in-repo) | `docs/sessions/20261017-integration-audit/INTEGRATION.md` | per-integration verdict for 12 audited surfaces | 2026-10-17 read in full |
| Scout (intent & state baseline) | PR #1147 on `docs/scout-20261017-agileplus-intent-state` (SCOUT.md) | intent vs. state delta baseline | 2026-10-17 referenced (not re-read in this turn) |
| Repository state | `cargo metadata` (parent ctx) | 38 workspace members | 2026-10-17 08:53Z |

What this docket **does not** cite (and why): a fresh native `cargo test --workspace` execution, a fresh `cargo clippy --workspace`, a fresh `cargo deny`, a fresh `cargo audit`, a fresh `cargo mutants`, a fresh `gh run list` on the current HEAD, a fresh probe of the live Vercel project, a fresh probe of Tailscale on the owner's desktop. The integration-audit INTEGRATION.md cites many of these but is dated 2026-10-17T01:02Z, which is 7–8 hours before the snapshot time. **All current-state claims are STALE-ELIGIBLE by the SFD v1.0 staleness rule**; the dossier flags the staleness window per row.

---

## 2. What changed in the repo between 2026-10-16 and 2026-10-17 (the live delta)

| Date | Event | Source | Maturity in this docket |
|------|-------|--------|--------------------------|
| 2026-10-08 | Promotion saga shipped: planned, prepared, compare-and-swap, per-step confirmation, atomic Shipped/audit/event/receipt finalization, post-commit cleanup. 8 Git+SQLite recovery witnesses + 14 CLI ship + 10 side-effect tests passed. | `verification/PROMOTION_RECOVERY_2026-10-08.md` | **PASS (Implementation + in-CI Validation) / NOT_RUN (Production Validation)** |
| 2026-10-08 | Real Chromium browser journey passed in CI against Axum/file SQLite + production Caddy fragment at commit `f959fad5390b8ff5e016be38cc2caedd27121597`. CI run [37718656528](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656528) + artifacts [11524233847](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656528/artifacts/11524233847). | same | **PASS (in-CI) / NOT_RUN (production)** |
| 2026-10-08 | Recovery Acceptance Witnesses CI run [37718656424](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656424) passed SQLite, application, migration, HTTP, origin, authenticated gRPC, CLI recovery gates on tested commit. | same | **PASS (in-CI) / STALE (today's HEAD is `00397a59` not `f959fad53`)** |
| 2026-10-08 | NATS dependency on the workspace now uses the 0.50 series already used by the Git adapter. The locked vulnerable rustls-webpki 0.102.8 chain is removed. Config macro uses maintained `pastey` replacement. `neo4rs` pinned to 0.9.0-rc.10. | same | **PASS (Implementation) / NOT_RUN (live broker probe today)** |
| 2026-10-08 | Linux Tauri GTK/GLib/Unicode chain still has upstream findings; no blanket advisory ignore was added. Security OSV scanning now verifies and scans the committed lockfile. | same | **FAIL (Advisories & OSV are blocking on the Linux desktop GTK/GLib/Unicode chain)** |
| 2026-10-14 (intent-reconciliation branch) | Multiple `chore/intent-reconciliation-20261014` PRs merged; intent-archeologist (`docs/intent-usability-20261016`) and scout (`docs/scout-20261017-agileplus-intent-state`) PRs opened (#1143, #1147). | `git log --oneline -1` of branch tips; PR list | **NOT_APPLICABLE for this docket's runtime question (documentation work)** |
| 2026-10-17 01:02Z | Integration audit (`docs/sessions/20261017-integration-audit/INTEGRATION.md`) produced per-integration verdicts for 12 surfaces. | in-repo audit file | **STALE-ELIGIBLE; row-level staleness applied** |
| 2026-10-17 01:14Z | Audit swarm proposed 12 subagents; 1 produced `SCOUT.md`, 2 created empty folders, 4 were never spawned. Noted for evidence that "dispatched" ≠ "ran" in this session. | session journals | **NOT_RUN (claim-of-12 was a plan-proposal, not a runtime)** |
| 2026-10-17 08:53Z | Snapshot taken at this docket's authoring time. | parent ctx | **snapshot identity `ap-sfd-2026-10-17T08-53Z@00397a59`** |

---

## 3. Twelve epics × 38 crates (PRD ↔ repo reality)

PRD §1–§12 names 12 epics. The PRD also reports 22 crates. **`cargo metadata --no-deps` reports 38 workspace members** — a 16-crate drift from the PRD claim. The drift is the first instance of "implementation yes, claim no" we have to reconcile.

| Epic | PRD title | Crates that visibly serve it | PRD-cited crate count | Repo-crate count | Implementation | Usability | Validation |
|------|-----------|------------------------------|----------------------|------------------|----------------|-----------|------------|
| E1 | Feature Lifecycle (CRUD, state machine, hashing) | `agileplus-domain`, `agileplus-application`, `agileplus-sqlite`, `agileplus-events`, `agileplus-api`, `agileplus-cli` | 2 | 6 | PASS (sources present: `crates/agileplus-domain/src/backlog_item.rs`, `agileplus-application/src/use_cases/feature_lifecycle`) | PASS (CLI subcommands `feature` exist; `agileplus` binary built locally) | NOT_RUN (none of the 92 cases have a runtime witness; the live `.agileplus/agileplus.db` has 47 tables per integration-audit, but no case id is bound to a specific table row) |
| E2 | Work Package Decomposition | `agileplus-domain`, `agileplus-application`, `agileplus-sqlite` | 2 | 3 | PASS | NOT_RUN (no executable fixture observed) | NOT_RUN |
| E3 | Attempt + Worker Lifecycle | `agileplus-domain`, `agileplus-application`, `agileplus-agent-dispatch`, `agileplus-agent-service`, `agileplus-agent-review`, `agileplus-cli` | 2 | 6 | PASS (dispatch + service + review crates exist) | UNKNOWN (no test invoked against the live binary in this turn) | NOT_RUN |
| E4 | Independent Evaluator | `agileplus-application`, `agileplus-agent-review`, `agileplus-triage`, `agileplus-governance` | 1 | 4 | PASS (governance crate has 20+ test targets including `scoring_engine_tests`, `policy_tests`, `code_scanner_tests`) | UNKNOWN | NOT_RUN (no runtime witness) |
| E5 | Authorized Acceptance | `agileplus-application`, `agileplus-governance`, `agileplus-cli` | 1 | 3 | PASS (`atomic_acceptance` test target in `agileplus-application`, `atomic_acceptance_http` in `agileplus-api`, `atomic_acceptance_rpc` in `agileplus-grpc`) | UNKNOWN (FORWARD_DELTA says "the exact current candidate still requires authoritative native execution of canonical atomic acceptance tests") | NOT_RUN |
| E6 | Audit Trail | `agileplus-events`, `agileplus-sqlite`, `agileplus-telemetry` | 1 | 3 | PASS (chained `event_store_tests`, `replay_tests`, `proptest_audit`) | UNKNOWN | NOT_RUN |
| E7 | External Integrations | `agileplus-plane` (Plane.so), `agileplus-github`, `agileplus-import` | 3 | 3 | PASS for code presence; **maturity diverges per integration** (see §3a) | per-integration (see §3a) | per-integration (see §3a) |
| E8 | Deployment (release/installable) | `agileplus-cli`, `agileplus-api`, `agileplus-desktop`, `agileplus-dashboard` | 1 | 4 | PASS for binary build; FAIL for live deployment (no production deployment is claimed) | FAIL (cloud browser observed old dashboard, zero data) | FAIL (no production deployment evidence) |
| E9 | Quality Gates (coverage≥85%, e2e 100%, perf 100%, qe 100% — self-reported) | `agileplus-benchmarks`, `agileplus-benchmarks/api_response_times`, `agileplus-integration-tests`, `agileplus-bdd`, `agileplus-e2e-roundtrip`, `agileplus-e2e-desktop` | 1 | 6 | PASS for source; **self-reported, not independently verified** | UNKNOWN | UNKNOWN (the 100% gates are claim, not measurement, and `.cargo-test-result.txt` in repo root is a 2026-09-01 Windows snapshot, not a current measurement) |
| E10 | Dashboard/Desktop UI | `agileplus-dashboard`, `agileplus-desktop`, `agileplus-telemetry` | 1 | 3 | PASS (CI witnessed dashboard typecheck + 9 operator component tests + prod build + JSX a11y lint) | FAIL (public URL shows old dashboard, no data) | PASS in-CI / FAIL in production |
| E11 | Federation (Tracera) | `agileplus-p2p` (Tailscale + vector clock), `agileplus-nats` (misnamed in-memory bus) | 0 (Tracera is out-of-scope per CHARTER, p2p is the in-scope analog) | 2 | PASS (p2p crate exists with `tailscale_local_api`, `vector_clock`, `git_merge_tests`); `agileplus-nats` is in-memory, not NATS | NOT_RUN (no Tailscale daemon running) | NOT_RUN |
| E12 | Operator/CLI | `agileplus-cli` (40+ subcommand test targets) | 1 | 1 | PASS (extensive CLI coverage: `cli_fix_list`, `cli_integration`, `cockpit_command_tests`, `queue_import_flow`, `ship_command_flow`, etc.) | PASS (CLI binary builds and runs) | NOT_RUN (no public execution witness; FORWARD_DELTA says "Test existence is not test execution") |

**Crate count reconciliation:** PRD names 22; cargo metadata names 38. The 16-crate delta is partly explained by E9 (benchmarks, e2e, BDD) and E10 (desktop, dashboard) being counted as quality/deployment rather than product. This is a PRD-writing gap, not a code-writing gap. The PRD should be updated to name 38; the docket does not fix that here.

### 3a. E7 External Integrations (the integration-audit verdict, transposed)

Verdicts in the third column are taken directly from `docs/sessions/20261017-integration-audit/INTEGRATION.md` §3. SFD v1.0 requires us to repeat the citation rather than paraphrase, and to mark the staleness window.

| # | Component | Repo code | Verdict from integration-audit (2026-10-17T01:02Z) | Staleness window (≈7h45m) | Disposition in this docket |
|---|-----------|-----------|---------------------------------------------------|---------------------------|----------------------------|
| E7.1 | Plane.so sync (`agileplus-plane/`) | 14 files, 4000+ lines, real reqwest + HMAC + axum; 207 unit tests + 1 contract test | 🟡 Implemented, env-gated (`PLANE_API_KEY` / `PLANE_WORKSPACE` / `PLANE_PROJECT` / `PLANE_API_URL`); outbound push only wired into CLI; no top-level `plane` subcommand; daemon only startable via dashboard HTTP | STALE-ELIGIBLE; re-probe recommended | **Implementation PASS, Usability PARTIAL (env-gated), Validation NOT_RUN** |
| E7.2 | GitHub Issues (`agileplus-github/`) | 4 files, 250+ lines; reqwest push with 60 req/min token bucket; **`LiveGhDataSource::list_issues` returns `Ok(vec![])` — pull is stub** | 🔴 Pull stubbed, push unused; no `GITHUB_TOKEN` wiring into CLI; no `agileplus-github` references in CLI | STALE-ELIGIBLE | **Implementation PARTIAL, Usability FAIL, Validation NOT_RUN** |
| E7.3 | NATS event bus (`agileplus-nats/`) | 7 files, 2800+ lines; **misleadingly named — no `async-nats` dep; in-memory `EventBus` trait + `InMemoryBus`** | 🔴 Stub; no code path constructs a NATS connection; `NatsConfig::with_auth` exists but unused | STALE-ELIGIBLE | **Implementation PARTIAL (bus exists, not NATS), Usability FAIL, Validation NOT_RUN** |
| E7.4 | Import from external (`agileplus-import/`) | 13 lines `lib.rs` + `manifest.rs`, `report.rs`, `importer.rs`; pure data structures; **no plane/github dep; no HTTP clients** | 🔴 Stub (manifest only) | STALE-ELIGIBLE | **Implementation PARTIAL, Usability FAIL, Validation NOT_RUN** |
| E8 | MCP server (`agileplus-mcp/`) | Python package, FastMCP 3.0; 3 tool modules (`features`, `governance`, `status`) all return `{"error": "not_implemented"}` | 🔴 Stub; would need `agileplus-grpc` server on `:50051` (not built, not running) | STALE-ELIGIBLE | **Implementation PARTIAL, Usability FAIL, Validation NOT_RUN** |
| E9.1 | SQLite (`agileplus-sqlite/`) | 6 files, 1700+ lines; real `rusqlite 0.40 bundled`, WAL + FK + `MigrationRunner` | ✅ Implemented & live; 47 tables in `agileplus.db`; WAL mode; 24KB `governance.db` separate | LOW staleness; 7h45m | **Implementation PASS, Usability PASS, Validation PARTIAL (file exists, not a case id witness)** |
| E9.2 | Cache layer (`agileplus-cache/`) | 6 files, 580 lines; real `bb8-redis` + `redis` client; **NOT LRU** despite name | 🟡 Implemented, unused; no `agileplus_cache::` references in CLI; no `bb8`/`redis` import in CLI `Cargo.toml` | STALE-ELIGIBLE | **Implementation PARTIAL, Usability FAIL, Validation NOT_RUN** |
| E9.3 | Plugin registry | ❌ No `plugin_registry` / `runtime_backend` / `backend_discovery` symbol | ⚪ Unknown / absent | STALE-ELIGIBLE | **Implementation FAIL, Usability FAIL, Validation NOT_RUN** |
| E10 | Neo4j (`agileplus-graph/`) | 4 files; `neo4rs = "0.9.0-rc.10"` behind `neo4j` feature (off by default); `InMemoryGraphStore` is default | 🟡 Optional, off by default; 5 test functions | STALE-ELIGIBLE | **Implementation PARTIAL, Usability FAIL, Validation NOT_RUN** |
| E11 | P2P (`agileplus-p2p/`) | 9 files, 2200+ lines; real `async-nats` for replication, real hyper UNIX-socket client for Tailscale local API; 182 unit tests | 🟡 Implemented, requires Tailscale daemon + NATS broker; neither running | STALE-ELIGIBLE | **Implementation PASS, Usability FAIL (no Tailscale), Validation NOT_RUN** |
| — | MinIO artifact archival | ❌ `rg "minio" --type toml` returns 0; `rg "s3::"` returns 0; no client dep | ⚪ Absent | STALE-ELIGIBLE | **Implementation FAIL, Usability FAIL, Validation NOT_RUN** |
| — | `.agileplus/` runtime state | 7 files; `agileplus.db` 671KB modified Oct 17 01:16; `governance.db` 24KB; `logs/nats.log` 0 bytes (never initialized) | 🟡 Partial (SQLite only) | LOW staleness | **Implementation PARTIAL, Usability PASS for SQLite, Validation PARTIAL** |

**Net verdict for E7:** 1 of 11 surfaces is live (SQLite). 1 of 11 is implemented and env-gated but not used at runtime (Plane.so). 9 of 11 are stub, absent, or behind a feature flag. **E7 is not "wired" at the level the PRD claims.**

---

## 4. Twelve journeys × 92 verification cases (the 2-minute number that matters)

Each journey is mapped to its cases and to its requirements. The 92/92 `specified_not_executed` count is the dossier's load-bearing fact.

| Journey | Title | Reqs | Cases | Requirement IDs (first 5) | Case verdict (all 92) |
|---------|-------|------|-------|----------------------------|------------------------|
| AP-J-01 | Shape Contract | 4 | 6 | AP-R-DEV-ID, AP-R-SPEC-REV, AP-R-AMBIGUITY, AP-R-ARTIFACT-ROLE | `specified_not_executed` × 6 |
| AP-J-02 | Decompose | 5 | 7 | AP-R-WP-DAG, AP-R-ASSIGNMENT, AP-R-CRITERIA-NONEMPTY, AP-R-CONTEXT, AP-R-SCHED-DEPS | `specified_not_executed` × 7 |
| AP-J-03 | Execute | 11 | 14 | AP-R-ATTEMPT-ID, AP-R-CLAIM, AP-R-CLAIM-NOINHERIT, AP-R-ISOLATION, AP-R-CANDIDATE | `specified_not_executed` × 14 |
| AP-J-04 | Grade | 6 | 9 | AP-R-EVALUATION, AP-R-NO-SELF-GRADE, AP-R-EVAL-FAIL, AP-R-EVIDENCE, AP-R-CONFIG-REV | `specified_not_executed` × 9 |
| AP-J-05 | Authorize | 5 | 8 | AP-R-ACCEPT-BOTH, AP-R-REVIEW-NOT-DONE, AP-R-POLICY-SEPARATE, AP-R-PR-CONTEXT, AP-R-REVIEW-ESC | `specified_not_executed` × 8 |
| AP-J-06 | Promote | 5 | 7 | AP-R-PROMOTE-EXACT, AP-R-PROMOTE-CLOSED, AP-R-RELEASE-ID, AP-R-RELEASE-HITL, AP-R-RELEASE-ROLLBACK | `specified_not_executed` × 7 |
| AP-J-07 | Recover (and Audit) | 11 | 12 | AP-R-RECOVERY, AP-R-HISTORY, AP-R-OPTIONAL-FAIL, AP-R-PERSIST-DURABLE, AP-R-PERSIST-TXN | `specified_not_executed` × 12 |
| AP-J-08 | Adapt methodology | 2 | 3 | AP-R-PROFILE, AP-R-CONFIG-PREC | `specified_not_executed` × 3 |
| AP-J-09 | Import external | 3 | 5 | AP-R-IMPORT, AP-R-NO-DUP-AUTH, AP-R-TRIAGE-PROV | `specified_not_executed` × 5 |
| AP-J-10 | Federate | 3 | 5 | AP-R-FEDERATION, AP-R-STANDALONE, AP-R-PRODUCT-BOUNDARY | `specified_not_executed` × 5 |
| AP-J-11 | Surface parity | 7 | 8 | AP-R-INTERFACE-PARITY, AP-R-ERROR-TRUTH, AP-R-IFACE-CANON, AP-R-IFACE-MACHINE, AP-R-MUTATE-PRECOND | `specified_not_executed` × 8 |
| AP-J-12 | Audit | 5 | 8 | AP-R-AUDIT-V1, AP-R-AUDIT-V2, AP-R-TRACE, AP-R-OBS-ATTRIB, AP-R-SEC-REDACT | `specified_not_executed` × 8 |
| **TOTAL** | — | **67** | **92** | — | **`0 of 92 executed`** |

**`0 of 92` is the single number that closes the mature-complete gate.** All 67 requirements share `verification_status: specified_not_executed` (confirmed by enumeration in `spec/product/requirements.semantic.v1.json`).

The mature contract is explicit: `mature_complete: false`, `grading_eligible_for_runtime_completion: false`. Until at least one case transitions from `specified_not_executed` to `executed_with_witness` (the schema's other valid status) and at least one requirement transitions from `specified_not_executed` to `runtime_verified`, the contract remains accepted at the semantic layer and unfulfilled at the runtime layer.

---

## 5. Claim-quality findings table

The 12 findings below are the most load-bearing disagreements between what the repo claims (PRD, README, charter) and what the runtime shows (integration-audit, `cargo metadata`, session journals). The SFD v1.0 template requires this table to be the central evidence-binding surface.

| # | Claim (source) | Implementation | Usability | Validation | Assurance | Required next evidence |
|---|----------------|----------------|-----------|------------|-----------|------------------------|
| F1 | "PRD names 12 epics across 22 crates" (PRD §1) | **PARTIAL** — 38 workspace members per `cargo metadata`; 16-crate drift | n/a | n/a | n/a | PRD amendment listing the 16 missing crates (benchmark, e2e, BDD, desktop, dashboard, p2p, nats, fixtures, subcmds, transport-tests, contract-tests, agent-*). |
| F2 | "Mature semantic contract accepted" (`mature-contract.v1.json`) | **PASS** — schema 0.3.0, 67 reqs, 12 journeys, 9 canonical decisions | n/a | n/a | n/a | None at the semantic layer. The contract is accepted; this is a stable claim. |
| F3 | "Mature contract fulfilled" (implied by `mature_complete: true` reading of PRD §9 quality gates) | **NOT_RUN** — `mature_complete: false` per `mature-contract.v1.json`; `grading_eligible_for_runtime_completion: false` | NOT_RUN | 0 of 92 cases executed | n/a | Execute the P0 cohort of 92 cases: canonical atomic-acceptance Rust tests, real-SQLite HTTP acceptance tests, gRPC acceptance tests, interrupted-Attempt replacement/restart tests, promotion preflight/drift tests. Each must produce an artifact (CI run id, transcript, or executed command + observed value). |
| F4 | "Plane.so is wired" (PRD §7.1, integration-audit §4) | **PASS** — 14-file `agileplus-plane` crate, real reqwest + HMAC + axum, 207 unit tests | **PARTIAL** — env-gated; no top-level `plane` subcommand; daemon only startable via dashboard HTTP | NOT_RUN (no live integration test) | LOW (no live witness; only contract test) | Set `PLANE_API_KEY` / `PLANE_WORKSPACE` / `PLANE_PROJECT` and execute one round-trip push from `agileplus module add` to `https://api.plane.so`; capture the returned issue id. |
| F5 | "GitHub Issues sync" (PRD §7.2) | **FAIL** — push client exists, **`LiveGhDataSource::list_issues` returns `Ok(vec![])` — pull is stub**; no `GITHUB_TOKEN` wiring in CLI | FAIL | NOT_RUN | n/a | Either implement the live pull (≥3 lines: real reqwest with token) or remove the claim from the PRD. The current state is a partial implementation that looks complete. |
| F6 | "NATS event bus" (`agileplus-nats` crate, 7 files, 2800 lines) | **PARTIAL** — files exist; **no `async-nats` dep**; the crate is an in-memory bus with a misleading name | FAIL | NOT_RUN | n/a | Rename the crate to `agileplus-inproc-bus` or wire `async-nats` against the running broker. The current name is a 38-crate line item that will mislead future contributors. |
| F7 | "MinIO artifact archival" (PRD §7.x implied by CHARTER's hexagonal adapters) | **FAIL** — zero S3/MinIO client deps; no `minio` references; `.agileplus/minio-data/` does not exist | FAIL | NOT_RUN | n/a | Either add a `agileplus-storage-s3` crate with `s3::Client` or remove MinIO from the deployment doctrine. |
| F8 | "Cache layer" (PRD §9.2) | **PARTIAL** — `bb8-redis` + `redis` deps; real client; **NOT LRU** despite being named "cache" in a system that has both "cache" and "Lru" elsewhere | FAIL (no `agileplus_cache::` in CLI) | NOT_RUN | n/a | Either wire the cache into the CLI or document it as a future-leaving adapter. The current state is dead code. |
| F9 | "Plugin registry" (PRD §9.3) | **FAIL** — `rg "plugin_registry\|backend_discovery\|runtime_backend"` returns 0 | FAIL | NOT_RUN | n/a | Either scaffold `agileplus-plugins` with a real `PluginRegistry` trait + a `WasmPlugin` and `SubprocessPlugin` adapter, or drop the claim. |
| F10 | "Production deployment" (CHARTER §1, README §1) | **PARTIAL** — Vercel project exists; production URL renders the old dashboard with zero data | FAIL (cloud browser observed old dashboard, no epics/stories) | NOT_RUN | n/a | Either deploy the current tip `00397a59` to Vercel and verify the dashboard shows real data, or document the gap in the README. |
| F11 | "Quality gates ≥85% coverage, 100% e2e / perf / qe" (PRD §9) | **PARTIAL** — coverage reports exist as source targets (`agileplus-benchmarks`, `agileplus-e2e-roundtrip`, `agileplus-bdd`); `agileplus-integration-tests` has 9+ test targets | UNKNOWN | **FAIL** (gates are self-reported; `.cargo-test-result.txt` in repo root is a 2026-09-01 Windows snapshot, not a current measurement; the FORWARD_DELTA explicitly says "Test existence is not test execution") | LOW (self-reported) | Run `cargo test --workspace` + `cargo llvm-cov --workspace --summary-only` + `cargo clippy --workspace -- -D warnings` and store the result as `verification/QUALITY_GATES_2026-10-17.md` with SHA. |
| F12 | "Audit-trail with 8 Git+SQLite recovery witnesses" (`PROMOTION_RECOVERY_2026-10-08.md`) | **PASS** — the saga is implemented at source; 8 real recovery tests passed | NOT_RUN (no production deployment) | **PASS in-CI** (CI run [37718656424](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656424) on commit `f959fad53`); **STALE** for current HEAD `00397a59` (different SHA, different code lineage) | MEDIUM (CI run id is independent of the implementer, but the run is on a different commit) | Re-run the recovery suite on the current HEAD `00397a59` and produce `verification/PROMOTION_RECOVERY_REPLAY_2026-10-17.md`. |

---

## 6. NEXT_DEPLOYABLE_RELEASE — the 10 release requirements

Per `verification/NEXT_DEPLOYABLE_RELEASE_2026-10-04.md`. Verdicts in the third column are this docket's best reading, citing the source for each.

| # | Requirement (paraphrased) | Verdict | Evidence (or absence) | Disposition |
|---|---------------------------|---------|------------------------|-------------|
| NDR-1 | Canonical atomic acceptance Rust tests green | **NOT_RUN** | FORWARD_DELTA: "the exact current candidate still requires authoritative native execution of canonical atomic acceptance tests" | Run `cargo test -p agileplus-application --test atomic_acceptance` on current HEAD; record CI run id. |
| NDR-2 | Real-SQLite HTTP acceptance tests green | **NOT_RUN** | Same FORWARD_DELTA paragraph; `agileplus-api/tests/atomic_acceptance_http.rs` exists as source but is not bound to an executed case | Run on current HEAD; record CI run id. |
| NDR-3 | Desktop API stack restartable | **UNKNOWN** | `agileplus-desktop` is a separate crate; no live restart witnessed | Restart the desktop API, observe health probes (`agileplus-api/tests/health_probes.rs`), record transcript. |
| NDR-4 | Tailnet service naming and routing | **FAIL** | No Tailscale daemon running on this host; `agileplus-p2p` requires it | Document the named service in the deployment doctrine; provision a Tailscale service. |
| NDR-5 | Browser credentials do not require an operator key | **UNKNOWN** | No live browser test observed today | Run a real browser journey against the public URL and observe that the operator does not need to paste a key. |
| NDR-6 | Production auth + same-origin works | **NOT_RUN** | FORWARD_DELTA: "The cloud browser opened the public product URL and observed the old dashboard, with zero epics/stories" | Deploy the current HEAD; run a real browser journey; observe auth flow + same-origin. |
| NDR-7 | Dashboard API origin correct | **PARTIAL** | Dashboard typecheck + 9 component tests + prod build + JSX a11y lint passed in CI; production dashboard shows old | Re-run the dashboard suite on the deployed current HEAD. |
| NDR-8 | 1 browser journey reads real backend data | **FAIL** | Cloud browser observed zero epics/stories | Replay the in-CI journey on the production URL. |
| NDR-9 | 1 browser acceptance journey reaches the atomic endpoint | **PARTIAL** | The in-CI journey did reach the atomic endpoint on commit `f959fad53`; production has not | Replay on current HEAD and on production. |
| NDR-10 | SQLite backup/restore + desktop reboot recovery tested | **NOT_RUN** | Backup is mentioned in `agileplus-sqlite/src/migrate.rs` but no test target names backup | Add a test target, run it, store the result. |

**`0 of 10` NDR items have a current-HEAD production-quality witness.**

---

## 7. Estimate reconciliation (do not average, do not pick optimistic)

Three independent time estimates exist in the SSOT. SFD v1.0 says: do not average, do not pick optimistic, record the conflict.

| Estimate source | P0 (native atomic-acceptance green) | P1 (HTTP acceptance as production contract) | P2 (gRPC validation parity) | Notes |
|-----------------|-----------------------------------|--------------------------------------------|----------------------------|-------|
| `verification/FORWARD_WBS_2026-10-04.md` | 1–2 days | 4–7 days (browser alpha) | 8–15 days (acceptance UI + worker/recovery visibility) | top-down WBS from P0 → P1 → P2 |
| `verification/FORWARD_DELTA_2026-10-04.md` | "authoritative native execution still required" (no time) | n/a | n/a | focuses on what is not yet executed, not on ETA |
| `verification/PROMOTION_RECOVERY_2026-10-08.md` | "the exact current candidate still requires authoritative native execution" (no time) | n/a | n/a | focus on promotion-saga closure, not on full release |
| Integration-audit (this snapshot, 7h45m older) | "no production deployment" (no time) | n/a | n/a | focus on integration reality, not on ETA |

**Conflict:** the only ETA is FORWARD_WBS (1–2 / 4–7 / 8–15 days). The other two sources explicitly say the work is not yet executed without giving a new ETA. **P0 estimate is therefore 1–2 days but only against the assumption that the canonical atomic-acceptance tests will pass on first execution.** If the tests reveal new findings (e.g. a forgotten schema migration, a NATS dep that won't compile on the operator's host), the estimate slides.

**The dossier does not average, does not pick optimistic, and does not declare a single number.** The forward path in §9 reports the WBS estimate and the conditions that could shorten or lengthen it.

---

## 8. Counterexamples (the things the SFD v1.0 example section calls out by name)

The SFD v1.0 example explicitly asks for the "Counterexample" section: the things that look like evidence for the optimistic claim and are not.

1. **"8 recovery witnesses passed"** — these are in-CI at commit `f959fad53`, **not on the current HEAD `00397a59`**. The 8 witnesses are evidence the saga implementation is sound; they are not evidence the saga works today on the line the operator would actually run.
2. **"Real Chromium browser journey passed"** — same in-CI at the same commit. The CI run is real; the production is empty.
3. **"207 unit tests in agileplus-plane"** — unit tests on an env-gated client do not validate the live plane.so round trip. A unit test on a stub `InMemoryPlaneClient` is not the same as a `LivePlaneClient::create_issue` call.
4. **"38 crates, 4000+ lines of Plane code"** — line count and crate count are not behavior. MinIO has zero code, GitHub has 250 lines, NATS has 2800 lines that don't talk to NATS.
5. **"`agileplus ship` is implemented"** — the saga is implemented; the Git+SQLite ACID boundary is explicitly not transactionally coupled (PROMOTION_RECOVERY §1). A real promotion with a partial failure is observable; recovery exists but it is not equivalent to "ship is safe".
6. **"Coverage ≥85%, e2e 100%, perf 100%, qe 100%"** — self-reported. The integration-audit calls this out by implication. The `.cargo-test-result.txt` in the repo root is a 2026-09-01 Windows snapshot. The dossier treats these gates as **NOT_RUN** until re-measured.
7. **"`agileplus-nats` is implemented"** — implemented as an in-memory bus, not as a NATS client. The name is the counterexample.
8. **"PRD names 22 crates"** — `cargo metadata` says 38. The PRD is a counterexample to the repository state.
9. **This docket's own §9a first draft named four of six packages incorrectly.** The 2026-10-17 first draft of §9a said `cargo test -p agileplus-application --test atomic_acceptance`, `-p agileplus-application --test wp_dependency_graph_integration_test`, `-p agileplus-application --test acceptance_traceability_integration_test`, and `-p agileplus-application --test promotion_recovery`. A cross-check against `cargo metadata --no-deps` showed that `atomic_acceptance` lives in `agileplus-sqlite`, that `wp_dependency_graph_integration_test` and `acceptance_traceability_integration_test` live in `agileplus-domain`, and that `promotion_recovery` lives in `agileplus-cli`. `agileplus-application` has only four test targets (`domain_use_case_tests`, `dto_serialization`, `error_types`, `triage_app_state_tests`) and none of them is an acceptance witness. **This is logged deliberately**: a forward path that names nonexistent targets would have produced four spurious `TARGET_NOT_FOUND` results and looked like a verification attempt while closing zero cases. The corrected list is what §9a now shows.

---

## 9. Forward path (30 minutes / 1 hour / 4 hours / 24 hours / CVP / MVP / mature)

CVP, MVP, and mature are **projections of the mature contract**, not separate products. The SFD v1.0 example is explicit about this.

### 9a. 30 minutes (the operator alpha's first witness)

Run on the current HEAD `00397a59` (no code change). **Package names below were verified against `cargo metadata --no-deps` on 2026-10-17 after the first draft of this docket named four of them incorrectly (see §8 counterexample 9).**

- `cargo test -p agileplus-sqlite --test atomic_acceptance` (P0 cohort #1 — the canonical atomic-acceptance native suite)
- `cargo test -p agileplus-api --test atomic_acceptance_http` (P0 cohort #2)
- `cargo test -p agileplus-api --test atomic_acceptance_sqlite` (P0 cohort #2b — real-SQLite HTTP variant)
- `cargo test -p agileplus-grpc --test atomic_acceptance_rpc` (P0 cohort #3)
- `cargo test -p agileplus-domain --test wp_dependency_graph_integration_test` (AP-R-WP-DAG)
- `cargo test -p agileplus-domain --test acceptance_traceability_integration_test` (AP-R-TRACE)

If the suite passes, transition at least 6 cases from `specified_not_executed` to `executed_with_witness` and record the CI run id. If the suite fails, record the failure and move to the 4-hour path.

### 9b. 1 hour (P0 close)

After the 30-minute run:

- Re-run the 8 Git+SQLite recovery witnesses on the current HEAD; transition 8 more cases.
- Run the 14 CLI ship + 10 side-effect tests; transition 24 more cases.
- Cross-check the 33 mature domains against the test targets that name them; identify which domains have no executable test and add a `not_applicable` annotation with a citation, where appropriate.

**Net: 0 of 92 → up to 37 of 92 in 1 hour, if everything passes on first execution.** The dossier does not claim this happens; it claims the work is *defined*.

### 9c. 4 hours (P1 close, browser alpha)

- Deploy the current HEAD to Vercel production; observe the dashboard at `https://agileplus.pheno.studio/` (or the current public URL).
- Open a real Chromium instance against the deployed URL; walk the journey in `PROMOTION_RECOVERY_2026-10-08.md` (gateway auth, axe a11y, lost response after committed acceptance, API restart, receipt replay, exactly one persisted event).
- Capture the run as a CI artifact; bind it to NDR-6, NDR-8, NDR-9.

### 9d. 24 hours (P2 close, gRPC parity, operator-deployable alpha)

- Stand up a Tailscale service on the operator's desktop.
- Bring up `agileplus-api` against the tailnet, fronted by Caddy.
- Run a real operator from a second device on the tailnet; verify NDR-1 through NDR-10 are all green.
- Cut tag `v2026.10.0-rc.1`; run `cargo publish --dry-run` against the `agileplus-cli` and `agileplus-api` libraries (registry semver is currently blocked on unpublished `agileplus-agent-dispatch` baseline — this is a known release blocker in `PROMOTION_RECOVERY_2026-10-08.md` §"Registry-based semver checks").

### 9e. CVP (Customer Validation Projection) — what a CVP looks like in this contract

A CVP is the projection of the mature contract that an external operator can pick up and use without operator help. In the AgilePlus topology, a CVP is "the operator's laptop, with Tailscale running, can sign in from a second device and execute a real promotion that survives a desktop reboot." A CVP requires:

- All NDR items green.
- All 92 cases at `executed_with_witness` or `not_applicable` with citation.
- A versioned Tailscale service name in the deployment doctrine.

**Today, no external operator has a CVP.** The cloud-browser public URL has zero data.

### 9f. MVP — what an MVP looks like in this contract

An MVP is the projection that "early operator" use is feasible. The MVP bar is lower than CVP: a single operator on the tailnet, with the operator as the only user. **Today, no operator has an MVP either** because the tailnet service is not provisioned.

### 9g. mature — what mature looks like in this contract

`mature_complete: true` is the verdict in `mature-contract.v1.json`. It requires:

- All 67 requirements at `runtime_verified`.
- All 92 cases at `executed_with_witness` (or `not_applicable` with citation).
- A 30-day production witness with no P0 regressions.
- A public status page that an outside operator can read.

**None of these are met today.**

---

## 10. Gate board (one row per gate; do not collapse Implementation / Usability / Validation / Assurance)

| Gate | Implementation | Usability | Validation | Assurance | Source / evidence |
|------|----------------|-----------|------------|-----------|-------------------|
| G1: Mature contract accepted | PASS | n/a | n/a | n/a | `mature-contract.v1.json` |
| G2: Mature contract fulfilled | NOT_RUN | NOT_RUN | 0/92 executed | n/a | `verification/semantic-verification-cases.v1.json` |
| G3: All 67 reqs runtime-verified | n/a | n/a | 0/67 | n/a | `spec/product/requirements.semantic.v1.json` |
| G4: P0 atomic-acceptance tests green | NOT_RUN | n/a | NOT_RUN | n/a | FORWARD_DELTA §"Immediate native gate" |
| G5: P1 HTTP acceptance as production contract | NOT_RUN | NOT_RUN | NOT_RUN | n/a | FORWARD_WBS P1 |
| G6: P2 gRPC validation parity | NOT_RUN | UNKNOWN | NOT_RUN | n/a | FORWARD_WBS P2 |
| G7: Production dashboard shows real data | PARTIAL (Vercel project exists) | FAIL (old dashboard, zero data) | NOT_RUN | n/a | FORWARD_DELTA §"Deployment evidence" |
| G8: Plane.so live round-trip | PASS (env-gated) | PARTIAL | NOT_RUN | n/a | integration-audit §4 |
| G9: GitHub pull implemented | FAIL (stub returns `Ok(vec![])`) | FAIL | NOT_RUN | n/a | integration-audit §3 |
| G10: NATS bus actually uses NATS | FAIL (in-memory despite name) | FAIL | NOT_RUN | n/a | integration-audit §3 |
| G11: MinIO storage | FAIL (no S3 client dep) | FAIL | NOT_RUN | n/a | integration-audit §3 |
| G12: Plugin registry | FAIL (no symbol) | FAIL | NOT_RUN | n/a | integration-audit §3 |
| G13: Tailscale service running | NOT_RUN (no daemon) | NOT_RUN | NOT_RUN | n/a | parent ctx probe |
| G14: Quality gates independently measured | NOT_RUN (self-reported) | n/a | NOT_RUN | LOW | FORWARD_DELTA §"Remaining release checks" |
| G15: 8 Git+SQLite recovery witnesses on current HEAD | PASS in-CI (commit `f959fad53`) | NOT_RUN | PASS in-CI / STALE today | MEDIUM (CI independent) | `PROMOTION_RECOVERY_2026-10-08.md` |
| G16: 14+10 CLI ship witnesses on current HEAD | PASS in-CI (commit `f959fad53`) | NOT_RUN | PASS in-CI / STALE today | MEDIUM | same |
| G17: Real Chromium browser journey on production | PARTIAL (CI run) | FAIL (prod shows old) | PARTIAL in-CI / FAIL in-prod | MEDIUM | same |
| G18: `agileplus ship` survives Git+SQLite boundary | PASS (saga exists) | NOT_RUN | PASS in-CI | MEDIUM | same |
| G19: Coverage ≥85% on current HEAD | UNKNOWN | n/a | NOT_RUN | LOW | repo `.cargo-test-result.txt` is 2026-09-01 |
| G20: `cargo deny` advisories & OSV clean on current HEAD | UNKNOWN | n/a | FAIL (Linux GTK/GLib/Unicode chain) | n/a | PROMOTION_RECOVERY §"Remaining release checks" |

**Net at snapshot time: 1 PASS at semantic acceptance; 19 of 20 Implementation/Usability/Validation rows are NOT_RUN / FAIL / UNKNOWN / STALE.**

---

## 11. Visual review (the parts that do not fit in tables)

- **Architecture diagram** lives in `DOCKET.architecture.mmd` in this folder. It shows: Plane.so, GitHub, NATS, MinIO, Neo4j, Cache/Redis, MCP, P2P/Tailscale, Vercel, the owner's desktop, the operator's device, with each link labeled by its F-row verdict.
- **Delta tree** lives in `DOCKET.delta-tree.mmd` in this folder. It shows the 2026-10-08 recovery branch and the 2026-10-14 intent-reconciliation branch with the gap from each to today's HEAD `00397a59`.
- **HTML render** lives in `DOCKET.html`.
- **PDF** lives in `DOCKET.pdf`. Open in Safari via `open DOCKET.pdf` from the docket folder (Safari will refuse if the file is not present; the dossier will report `NOT_OPENED` in that case rather than claim success).

---

## 12. What closes this docket

A single 3-line statement, as the SFD v1.0 example requires:

1. `0 of 92` becomes `92 of 92` (or `92 - N` with N named and cited as `not_applicable`).
2. `0 of 67` requirements at `runtime_verified` becomes `67 of 67` (or a smaller number with the same naming rule).
3. The 10 NEXT_DEPLOYABLE_RELEASE items are green against an actual remote operator on a tailnet, not a CI witness at a prior commit.

Until those three lines are observed, the docket remains open. The next docket in this lineage will inherit the snapshot identity `ap-sfd-2026-10-17T08-53Z@00397a59` and update it to the new HEAD's SHA. The SFD v1.0 contract forbids treating a forward path as closure.

---

## 13. Operator-only items (decisions, access, feedback)

These are the only items in the docket that require operator authority. Everything else in §9 can move without an operator decision.

- **Tailscale provisioning** — the operator must provision a Tailscale service on the owner's desktop before NDR-4 and the 24-hour path can start. The deployment doctrine says Tailscale Services are preferred, but no service is currently named.
- **Vercel team access** — `FORWARD_DELTA` reports "scoped deployment inspection returned 403 for the owning team." Without team scope, NDR-6, NDR-7, NDR-8, NDR-9 cannot be re-verified on the current HEAD.
- **`kooshapari-desk` host** — named in `PROMOTION_RECOVERY_2026-10-08.md` §"Latest candidate and access continuation" as the deployment host. SSH details and account not yet established. Required for the 24-hour path.
- **crates.io publication** — the registry semver checks are blocked on the unpublished `agileplus-agent-dispatch` baseline. The operator must decide whether to publish pre-release tags or to vendor the dependency for the first release.
- **PRD amendment** — the 22-vs-38-crate drift is a documentation question, not a code question. The operator can defer it to a follow-up PR.

---

## 14. Citation discipline (what this dossier does not do)

- It does not claim the 92 cases are passing.
- It does not claim the 67 requirements are runtime-verified.
- It does not claim the production deployment is current.
- It does not claim an operator can use the product today.
- It does not claim CI witnesses at a prior commit are evidence for the current HEAD.
- It does not average the 1-2/4-7/8-15 day estimates.
- It does not retitle the product MVP, release, or CVP.
- It does not invent external adoption, operator quotes, or user feedback.
- It does not collapse Implementation / Usability / Validation / Assurance into a single verdict.
- It does not borrow a passing in-CI row to mark a failing in-prod row as `PASS`.

The dossier is honest about what it knows and what it does not. The next dossier inherits this one and updates it.
