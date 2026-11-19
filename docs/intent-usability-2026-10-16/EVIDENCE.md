# Evidence, decisions and publication boundary

All links below resolve relative to this document at the assessment source base.
They support source/document claims; historical execution records are not current
runtime observations. Private prompt exports and raw logs are intentionally omitted.

| Claim | Source anchor |
|---|---|
| Product intent and ownership boundary | [Recovered intent](../../spec/product/RECOVERED_PRODUCT_INTENT.md), [authority](../../spec/product/AUTHORITY_INDEX.md) |
| Accepted semantic contract | [Mature contract](../../spec/product/mature-contract.v1.json), [journeys](../../spec/product/journeys.v1.json) |
| Current interface interpretation | [Interface profiles](../../spec/product/INTERFACE_PROFILE_CONTRACTS.md), [runtime mounting](../../spec/product/RUNTIME_MOUNTING_MAP.md) |
| Requirement/case reconciliation | [Reconciliation](../../verification/INTENT_RECONCILIATION_2026-10-14.md), [case catalogue](../../verification/semantic-verification-cases.v1.json) |
| Active real API-backed UI | [OperatorDashboard](../../crates/agileplus-dashboard/web/src/OperatorDashboard.tsx), [operator notes](../../verification/OPERATOR_DASHBOARD_2026-10-08.md) |
| MCP advertised stubs and health | [Status](../../agileplus-mcp/src/agileplus_mcp/tools/status.py), [features](../../agileplus-mcp/src/agileplus_mcp/tools/features.py), [governance](../../agileplus-mcp/src/agileplus_mcp/tools/governance.py), [client](../../agileplus-mcp/src/agileplus_mcp/grpc_client.py) |
| Prior recovery witnesses | [Forward delta](../../verification/FORWARD_DELTA_2026-10-04.md), [promotion recovery](../../verification/PROMOTION_RECOVERY_2026-10-08.md) |
| Release qualification still required | [Next deployable release](../../verification/NEXT_DEPLOYABLE_RELEASE_2026-10-04.md), [runtime evidence plan](../../verification/RUNTIME_EVIDENCE_PLAN.md) |
| Permissive sampled nightly/provenance | [Nightly workflow](../../.github/workflows/nightly.yml), [sampled run](https://github.com/KooshaPari/AgilePlus/actions/runs/38988833974) |

## Evidence limits

The earlier read-only local observation found canonical localhost:8765 unavailable
and the prescribed status wrapper absent. This does not establish every deployment
is down. No authoritative current engine feature/WP state is inferred from files.
No fresh suite, measured coverage, installed-artifact proof, browser journey,
device test or deployed acceptance was performed for this documentation set.

Existing PRs at the audit snapshot include recovery/concurrency, dashboard provenance,
desktop and fail-closed QE work. Their titles and presence do not prove merge or
acceptance. Reconcile current heads/checks in package A; do not recreate those lanes.
Current source raises Rust coverage to 85%; older 20% notes must not be presented as
current. Go report-only and echo aggregate gaps remain separately identified.

## Decisions required during execution

| Decision | Recommended bounded choice |
|---|---|
| First supported product profile | Declare the actual CLI/MCP/web/desktop support matrix; do not advertise stubbed surfaces as complete |
| First pilot subject | One unrelated repository with frozen intent and concrete useful outcome |
| MVP breadth | Freeze supported daily workflows and page inventory before estimating implementation detail |
| Deployment target | Qualify private tailnet first; separately qualify requested Access/public boundary |
| Mature finish denominator | Current 12 journeys/92 cases plus explicit release and recovery gates |
| Comparative claim | Separate matched study with frozen comparator tasks and outcome criteria |

Publication is a reviewed Markdown planning snapshot in the owned AgilePlus repo;
it does not merge fixes, alter product authority, certify release or deploy services.
The portfolio copy should preserve the exact document revision and publication receipt.
