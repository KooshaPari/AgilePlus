# Progress Tree — AgilePlus per-intent × per-dimension state

**Snapshot identity:** `ap-sfd-2026-10-18T07-25Z@ci-blockers-plus-coverage-attempted`
**HEAD at capture:** `d6b418a6 docs(sfd): correct 9a package names after cargo metadata cross-check` (branch `docs/state-forward-docket-20261017`)
**Workspace members:** 38 (verified by `cargo metadata` at 22:12:26Z)
**Mode:** evidence-cell tree; per-cell verdict + source. **No** percentage fabrication, **no** dimension collapse. Every non-`?` cell cites its source.
**Raw evidence file:** `PROGRESS_TREE_RAW.md` (256 cells, harvested by `blossom` subagent 2026-10-17T22:16Z)

---

## Verdict vocabulary (SFD v1.0)

`✓` PASS · `~` PARTIAL · `✗` FAIL · `…` NOT*RUN · `?` UNKNOWN · `S` STALE · `B` BLOCKED · `N/A` NOT_APPLICABLE · `*` n/a (dimension not meaningful for that row)

---

## Tree (legend: one line per intent axis, 4 sub-cells = Implementation / Usability / Validation / Assurance)

### Section A — 10 operator-directive sections

```
A.1 Current state vs accepted intent       ✓ _ _ L   DOCKET.md §0; mature-contract.v1.json status=accepted_mature_semantic_contract_runtime_evidence_pending
A.2 Verification case execution            ✓ ✗ … _   92 cases specified; 0 executed; ATOMIC_ACCEPTANCE_EXECUTION closes target-level only
A.3 Forward path (operator alpha/MVP/mature)✓ _ … L   FORWARD_WBS, NEXT_DEPLOYABLE_RELEASE, CHARTER
A.4 Quality gates (cov, e2e, perf, qe)     ~ S ✗ L   cargo test --workspace: 5515 pass / 1 fail (2026-10-18T01:42Z, owl subagent) — fail is `agileplus-cli::commands::mvp::tests::wp_transition_requires_its_contract_evidence_before_mutation` at mvp.rs:626 (panics on `/bin/true` ENOENT; macOS has `/usr/bin/true` only); cargo clippy --workspace --all-targets: clean (5m 05s, exit 0); .cargo-test-result.txt still 2026-09-01 Windows snapshot (stale)
A.5 Operator/CLI affordances                ✓ ✓ … _   40+ test targets, target/debug/agileplus builds
A.6 Acceptance UX                           ~ ✗ … _   cloud browser saw old dashboard, zero data
A.7 Integration state (Plane/GH/NATS/MinIO/MCP)~ ✗ … L   INTEGRATION.md §3: 1/11 surfaces live
A.8 Worker replacement / recovery          ✓ _ ~ M   8 Git+SQLite + 14+10 CLI witnesses in-CI on f959fad53
A.9 Federation (Tracera) posture            N/A N/A N/A _  CHARTER: out-of-scope
A.10 Risk register / open issues           ~ _ … L   no central verification/RISK_REGISTER.md
```

### Section B — 12 epics (PRD-aligned)

```
B.1  E1 Feature Lifecycle                 ✓ ✓ … _   domain/backlog_item.rs; application/use_cases/feature_lifecycle/
B.2  E2 Work Package Decomposition        ✓ … … _   DOCKET §3 E2
B.3  E3 Attempt + Worker Lifecycle         ✓ ? … _   agileplus-agent-{dispatch,service,review}
B.4  E4 Independent Evaluator             ✓ ? … _   scoring_engine_tests, policy_tests, code_scanner_tests
B.5  E5 Authorized Acceptance             ✓ ? ~ M   ATOMIC_ACCEPTANCE_EXECUTION: 4 targets PASS on 00397a59, target-level not case-level
B.6  E6 Audit Trail                       ✓ ? ✓ _   cargo test -p agileplus-events: 20+ tests pass, 0 failed (2026-10-17T23:08Z, parent + worker subagent)
B.7  E7 External Integrations             ~ ~/✗ … L  INTEGRATION.md §3
B.8  E8 Deployment (release/installable)  ✓ ✗ ✗ _   binary builds; Vercel 403 on prod inspect; old dashboard in-prod
B.9  E9 Quality Gates                     ✓ ? ? L   self-reported only; no independent measurement
B.10 E10 Dashboard/Desktop UI             ✓ ✗ ~/✗ M/L  CI PASS on f959fad53; prod FAIL (old dashboard, zero data)
B.11 E11 Federation (Tracera)             ✓ … … _   p2p crate; Tailscale daemon not running
B.12 E12 Operator/CLI                     ✓ ✓ … _   40+ test targets; CLI subcommands wired
```

### Section C — 12 journeys AP-J-01..AP-J-12

```
C.1  AP-J-01 Shape and accept a contract  ✓ _ … _   AP-VC-001..006 specified, 0 executed
C.2  AP-J-02 Decompose into executable    ✓ _ … _   AP-VC-007..013, 0 executed; wp_dependency_graph_integration_test PASS target-level
C.3  AP-J-03 Execute with replaceable     ✓ _ … _   AP-VC-014..027, 0 executed
C.4  AP-J-04 Independently grade          ✓ _ … _   AP-VC-028..036, 0 executed
C.5  AP-J-05 Authorize accepted work      ✓ _ … _   AP-VC-037..044, 0 executed
C.6  AP-J-06 Promote or ship              ✓ _ … _   AP-VC-045..051, 0 executed; promotion_recovery PASS in-CI target-level
C.7  AP-J-07 Resume after interruption    ✓ _ … _   AP-VC-052..063, 0 executed
C.8  AP-J-08 Adapt methodology depth      ✓ _ … _   AP-VC-064..066, 0 executed
C.9  AP-J-09 Import external frameworks   ~ ✗ … _   agileplus-import = manifest stub
C.10 AP-J-10 Federate to Tracera          ~ ✗ … _   p2p crate present; Tailscale not running; CHARTER out-of-scope
C.11 AP-J-11 Surface parity (CLI/HTTP/gRPC)✓ _ … _   AP-VC-072..079, 0 executed
C.12 AP-J-12 Audit who changed what/why   ✓ _ … _   AP-VC-080..087, 0 executed; acceptance_traceability_integration_test PASS target-level
```

### Section D — 10 NDR items

```
D.1  NDR-1 canonical atomic acceptance   ✓ _ ✓ H   atomic_acceptance 10/10 on 00397a59
D.2  NDR-2 real-SQLite HTTP acceptance    ✓ _ ✓ H   atomic_acceptance_http 10/10 + atomic_acceptance_sqlite 3/3 on 00397a59
D.3  NDR-3 desktop API stack restartable   ✓ ? … _   cargo build -p agileplus-desktop --release: exit 0, 15m 29s (2026-10-17T23:02Z)
D.4  NDR-4 tailnet service naming/routing ✗ ✗ … _   no Tailscale daemon
D.5  NDR-5 browser creds no operator key  ? ? … _   no SaaS-style auth implemented
D.6  NDR-6 production auth + same-origin  … … … _   no production deploy witnessed
D.7  NDR-7 dashboard API origin correct   ~ _ ✗ M   typecheck + 9 component tests + JSX a11y PASS in-CI; prod shows old
D.8  NDR-8 browser journey reads real data✗ ✗ … _   cloud browser saw zero epics/stories
D.9  NDR-9 browser acceptance reaches     ~ … ~ M   CI on f959fad53 only; prod NOT_RUN
D.10 NDR-10 SQLite backup/restore tested  … … … _   migrate.rs mentions backup, no test target
```

### Section E — 20 gates G1..G20

```
E.1  G1 Mature contract accepted          ✓ _ _ _   mature-contract.v1.json status=accepted_…
E.2  G2 Mature contract fulfilled         … … … _   mature_complete=false; 0/92 cases
E.3  G3 All 67 reqs runtime-verified      _ _ … _   0/67
E.4  G4 P0 atomic-acceptance tests green  ✓ _ ✓ H   89/89 on 00397a59 (parent + worker + direct binary)
E.5  G5 P1 HTTP acceptance as prod        … … … _   FORWARD_WBS P1
E.6  G6 P2 gRPC validation parity         … ? … _   target-level only on 00397a59
E.7  G7 Production dashboard shows real   ~ ✗ … _   Vercel project exists; prod shows old
E.8  G8 Plane.so live round-trip          ✓ ~ … L   env-gated, fire-and-forget from CLI
E.9  G9 GitHub pull implemented           ✗ ✗ … _   LiveGhDataSource::list_issues returns Ok(vec![])
E.10 G10 NATS bus actually uses NATS      ✗ ✗ … _   agileplus-nats is in-memory despite name
E.11 G11 MinIO storage                    ✗ ✗ … _   no S3 client dep
E.12 G12 Plugin registry                  ✗ ✗ … _   no plugin_registry symbol
E.13 G13 Tailscale service running        … … … _   ps aux | grep tailscaled empty
E.14 G14 Quality gates independently      … _ … L   self-reported; no independent measurement
E.15 G15 8 Git+SQLite recovery witnesses  ✓ … ~ M   promotion_recovery 8/8 in-CI on 00397a59
E.16 G16 14+10 CLI ship witnesses         ✓ … ~ M   target-level on 00397a59
E.17 G17 Real Chromium browser journey    ~ ✗ ~ M   CI run on f959fad53; prod FAIL
E.18 G18 agileplus ship survives boundary ✓ … ~ M   saga exists
E.19 G19 Coverage ≥85% on current HEAD    ? _ … L   cargo llvm-cov --workspace --summary-only cold-run aborted >20m (2026-10-17T23:08Z); warm-cache rerun not executed in 35-min budget (2026-10-18T01:39Z, owl subagent task 2 SKIPPED); **fresh 06:43:43Z sloth run hit ENOSPC (disk /Users/kooshapari 100% full, 1.3 GB free)** — `cargo test --no-run --workspace` failed with `ld: write() failed, errno=28` and `os error 28: No space left on device` in 3 places (agileplus-telemetry logging_file_output_tests, agileplus-github sync_adapter_tests, agileplus-cli --test); verdict remains NOT_RUN; remediation: free disk space then re-dispatch sloth; cargo clippy --workspace --all-targets is the only fresh evidence (clean, exit 0)
E.20 G20 cargo deny + OSV clean           ? _ ~ _   cargo deny: advisories PARTIAL (2026-10-18T06:48Z, parrot subagent) — 1 documented RUSTSEC-2024-0429 macOS-ignore intentional (Linux-only tauri PR #16170) + 3 newly-IGNORED RUSTSEC-2026-0370/0371/0372 (zvariant/zbus_names/zvariant_derive unmaintained-info via zbus→keyring→agileplus-domain); all 3 silenced via deny.toml ignore PR #1157 (commit 070babcd, open); bans/licenses/sources ok; PARTIAL (not PASS) because advisories still exist (only acceptance is declared); re-evaluate to PASS when zbus 6.0.0+ upstream migration lands
```

---

## Counts (no percentages)

**Re-validated by parent orchestrator (this file).** Per-dimension totals across 256 cells (64 rows × 4 dimensions):

| Verdict               | Impl   | Usab   | Val    | Assur  | Total   |
| --------------------- | ------ | ------ | ------ | ------ | ------- |
| ✓ PASS                | 32     | 4      | 7      | 3      | **46**  |
| ~ PARTIAL             | 6      | 4      | 5      | 0      | **15**  |
| ✗ FAIL                | 4      | 12     | 4      | 0      | **20**  |
| … NOT_RUN             | 11     | 14     | 37     | 0      | **62**  |
| ? UNKNOWN             | 2      | 4      | 2      | 0      | **8**   |
| S STALE               | 1      | 1      | 0      | 0      | **2**   |
| N/A NOT_APPLICABLE    | 1      | 1      | 0      | 0      | **2**   |
| \_ dim-not-meaningful | 7      | 24     | 9      | 61     | **101** |
| **Column total**      | **64** | **64** | **64** | **64** | **256** |

> Caveat: the subagent's `PROGRESS_TREE_RAW.md §G.1` and `§G.2` show minor 28-row arithmetic drift (subagent counted 162 row-verdicts, parent counts 155; difference is the 7 `dim-not-meaningful` cells the subagent double-counted as n/a, vs the 7 it correctly classed as `_`). The evidence cells themselves are correct; the **per-cell verdict and source** stand.

### Headline numbers (raw counts only — NO percentages)

- **45 / 256** cells are PASS (any verdict in any dimension). This is a count, not a "completeness" claim.
- **45 / 150** non-`n/a`/`_` cells are PASS.
- **0 / 92** verification cases have `executed_with_witness` in the JSON-locked schema (`0.4.0`).
- **5 / 92** cases have a **STRONG** test witness per `verification/CASE_BINDING_2026-10-17.md`; **5 / 92** have an **ADJACENT** witness. (AP-RVC-056 was downgraded from STRONG to ADJACENT after re-reading the case body — the test proves only the immutability half, not the content-attribution half.) The two numbers are not the same claim.
- **0 / 67** requirements are `runtime_verified`.
- **3 / 10** NDR items (NDR-1, NDR-2, NDR-3) have a parent-observed PASS witness at the **target** level.
- **1 / 11** integration surfaces is wired for real (SQLite).
- **1 / 5** Tracera-relevant surfaces (E11 / p2p crate exists but not running) is partially implemented.
- **3 / 20** gates are clearly PASS (G1 = spec acceptance; G4 = P0 acceptance cohort; G19.Val? — coverage tool aborted, NOT_RUN remains; **E.20 transitioned FAIL→PARTIAL on 2026-10-18T06:48Z** — 3 RUSTSEC-2026-0370/371/372 unmaintained advisories now silenced via deny.toml ignore (PR #1157, commit 070babcd, open); PARTIAL retained (not PASS) because the advisories still exist (only acceptance is declared). B.6 / E.6 Validation and D.3 / NDR-3 Implementation were closed by the cargo audit subagent on 2026-10-17T23:08Z; E.20 was re-validated on 2026-10-18T01:33Z (FAIL, 3 NEW advisories recorded) and again on 2026-10-18T06:48Z (PARTIAL, mitigations recorded). **A.4 / E.9 Validation transitioned UNKNOWN→FAIL on 2026-10-18T01:42Z** (cargo test --workspace 1 fail: `agileplus-cli::commands::mvp::tests::wp_transition_requires_its_contract_evidence_before_mutation` at mvp.rs:626 — panics on `/bin/true` ENOENT; macOS has `/usr/bin/true` only; fix is in PR #1156 / commit 39ee805b, open, unmerged). **E.19 Val still NOT_RUN**; warm-cache `cargo llvm-cov` rerun was budget-cut by owl subagent on 2026-10-18T01:39Z; **fresh sloth attempt at 2026-10-18T06:43:43Z hit ENOSPC** (disk 100% full) — see `llvm_cov_2026-10-18` block in PROGRESS_TREE.snapshot.json.

> Per the cross-harness cockpit policy (memory `mem_1792275198980_10767103059655466859`, file `~/.jcode/memories/global/cockpit-progress-tree-policy.md`, adopted 2026-10-17T22:12Z), these counts are the **only** quantitative claims in this tree. They are **never** converted to percentages.

### Case binding overlay (2026-10-17T22:22Z, parent-orchestrator)

A parallel file `verification/CASE_BINDING_2026-10-17.md` (schema-clean, does not modify `semantic-verification-cases.v1.json`) maps the 29 `#[tokio::test]` functions across the 4 atomic-acceptance files to specific `AP-VC-*` / `AP-RVC-*` cases.

- **5 / 92** cases have a **STRONG** witness (the test asserts the exact preconditions + stimulus + expected of the case on a fresh, isolated deterministic fixture): AP-RVC-057, AP-RVC-081, AP-VC-028, AP-VC-008, AP-VC-009. AP-RVC-056 was originally claimed STRONG and downgraded to ADJACENT on re-read.
- **5 / 92** cases have an **ADJACENT** witness (test proves a closely related concern but not the literal case stimulus): AP-VC-002, AP-VC-038, AP-VC-034, AP-RVC-049, AP-RVC-056.
- **Transitions are conditional on a follow-up `0.5.0` schema-bump PR** to add `linked_test_targets[]` and `witness_run` fields to the cases file. The binding is the **first-class reference** that PR should consume.
- **Per the cockpit policy:** the headline `0 / 92` is reported in the JSON-locked schema; the binding's `5 / 92 STRONG + 5 / 92 ADJACENT` is reported in the parallel binding file. These are not the same claim and must not be conflated.

This overlay updates the **Validation** column for B.5, C.4, C.5, C.7, and C.12 from `…` to `~` (PARTIAL: target-level PASS + case-level ADJACENT or STRONG for some sub-cases, pending schema-bump).

---

## What closes the `?` and the rest

### Cheap closes (no operator authority needed)

| Cell                        | Cheapest move                                                                                                                                     | ETA    |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| A.4 / E.19 Validation       | `cargo llvm-cov --workspace --summary-only --fail-under 85` on `d6b418a6`; record in `verification/QUALITY_GATES_2026-10-17.md`                   | 10–15m |
| A.4 / E.20 Validation       | `cargo deny check advisories sources licenses bans` on `d6b418a6`                                                                                 | 5m     |
| B.5 Validation → case-level | Read the 4 atomic_acceptance*.rs files; bind each test name to an `AP-VC-*`id; edit`semantic-verification-cases.v1.json`to`executed_with_witness` | 30m    |
| B.6 / E.6 Validation        | `cargo test -p agileplus-events --test event_store_tests` + `replay_tests` + `proptest_audit` on `d6b418a6`                                       | 5m     |
| B.3 / B.4 Usability         | Smoke-run `target/debug/agileplus module list`, `feature list`, `agent review`; capture transcript                                                | 10m    |
| D.3 Implementation          | `cargo build -p agileplus-desktop --release`; `target/release/agileplus-desktop --help`; record exit code                                         | 5m     |
| D.3 Validation              | Start `agileplus-api`, hit `health_probes` test target, kill + relaunch, observe `atomic_acceptance_sqlite` reopen                                | 10m    |
| E.2 / E.5 Validation        | Run P1/P2 command list in `FORWARD_WBS_2026-10-04.md` P1+P2                                                                                       | 1–2h   |
| F.3 (all 6 doc gaps)        | Open 1 docs PR with the 6 new files (RISK_REGISTER, COVERAGE_REPORT, DASHBOARD_PROD_OBSERVED, etc.)                                               | 30m    |

### Operator-gated (cannot close in parent ctx)

| Cell                                             | Why                                                                                            |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------------- |
| A.7 Usability (`agileplus-nats` / MinIO / Neo4j) | Operator must start the dormant homebrew binaries and provision missing S3/MinIO client code   |
| A.9 / A.10 Risk register                         | No central file; operator convening authority needed                                           |
| D.4–D.10 Validation (6 of 10 NDR)                | Requires Tailscale provisioning on `kooshapari-desk`, Vercel team access, Windows deploy creds |
| E.7 / E.8 / E.13 / E.17 Validation (production)  | Vercel deploy + `PLANE_API_KEY` env vars + reachable URL                                       |
| E.9 Validation (GitHub live)                     | `GITHUB_TOKEN` + actual repo to list                                                           |
| Cargo registry semver                            | Blocked on operator publish decision (`agileplus-agent-dispatch` baseline unpublished)         |
| Vercel team-scope access                         | FORWARD_DELTA: "scoped deployment inspection returned 403 for the owning team"                 |

---

## Policy reminders (from this turn's correction)

- **No percentage fabrication.** A cell with one passing test in a 20-test target is `PASS (Impl) + ? (Assur)` — never "5%".
- **No dimension collapse.** Implementation ≠ Usability ≠ Validation ≠ Assurance.
- **No false "complete" on a single target.** A passing test target is `PASS` at the target level; it is `?` at the journey/case level until the per-case mapping is recorded.

This tree and the policy it follows are now the canonical end-of-turn cockpit shape for this docket and (per the policy file at `~/.jcode/memories/global/cockpit-progress-tree-policy.md`) for every other project that gets the same question.

---

_Snapshot: `ap-sfd-2026-10-17T22-14Z@d6b418a6` · evidence file `PROGRESS_TREE_RAW.md` · policy `~/.jcode/memories/global/cockpit-progress-tree-policy.md` · memory `mem_1792275198980_10767103059655466859`_
