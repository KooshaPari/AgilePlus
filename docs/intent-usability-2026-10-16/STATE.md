# State and usability

Source snapshot: `00397a59`; observations dated 2026-10-16. UNKNOWN means the
assessment did not obtain the needed runtime evidence, not that a feature is absent.

| Goal or surface | Evidence and implication | Acceptance state |
|---|---|---|
| Canonical work semantics | Authority index, mature contract, requirement crosswalk and recovery records exist | Documented; not full current execution proof |
| Real operator dashboard | Active OperatorDashboard uses same-origin `/api/v1`; rejects HTTP/non-JSON errors; persists acceptance requestID before POST and historical receipts | Implemented source flow; current deployed UX unqualified |
| Every named page useful | Existing dashboard/provenance work and stub inventory require exact-candidate reconciliation | Page-by-page runtime UNKNOWN |
| Agent-facing MCP | Dashboard/metrics, features and governance implementations contain explicit stubs; gRPC client raises NotImplementedError | Advertised implementation incomplete |
| Truthful health/status | MCP health hardcodes healthy alongside unreachable core; canonical localhost:8765 refused connection; prescribed wrapper absent | Local canonical status unavailable; health source defect |
| Durable acceptance/recovery | Historical records describe real Git/file-SQLite witnesses, replay, recovery and promotion | Recorded prior evidence; candidate requalification open |
| Assurance | Manifest says mature_complete=false; 12 journeys/92 cases specified_not_executed | Maturity NOT accepted |
| Coverage gates | Current Rust floor is 85%; Go report-only and aggregate echo remain assurance gaps | Neither measured coverage nor complete enforcement established |
| Source-bound distribution | Sampled nightly allows test failure; packaged crates.io CLI is not demonstrated checkout-built | Release provenance gap |
| Install, desktop and unrelated repo | No current clean-install/restart/user lifecycle witness obtained | UNKNOWN |
| Deployment and telemetry | Exact private/Access deployment, auth, traces and backup/reboot receipts not qualified | UNKNOWN |
| Competitive usefulness | Comparator intent exists; no matched outcome study reviewed | UNPROVEN |

## What a user can reasonably rely on

A developer can inspect substantial source and documented contracts, including a
real API-backed operator flow. That is a useful development candidate. The available
evidence does not yet justify telling an operator that installation, advertised MCP
status, all pages, recovery and distribution form one supported end-to-end product.

The first usability test must begin from a clean unrelated repository, using the
actual distributed artifact and supported commands. Record installation friction,
command discoverability, task completion, error clarity, time and recovery behavior.
Use text/CLI/log evidence only; screenshots and recordings are prohibited.

## Main workflow friction to resolve

1. Establish which installed CLI/MCP/dashboard profile is supported and actually mounted.
2. Make missing dependencies and service failures explicit, with an actionable recovery path.
3. Expose traceable IDs, evidence provenance and durable acceptance receipts consistently.
4. Prevent wrong-root, stale-evidence, concurrent-writer and restart failures from appearing successful.
5. Ensure each advertised page has useful loading, empty, error and success behavior.

Historical pass counts and successful hosted jobs remain separate from a current
installed-product witness. No completion percentage is inferred from source volume,
marker counts, planned cases or green jobs with permissive gates.
