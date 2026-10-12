# Tracera Load Check -- AGILEPLUS project (2026-10-12)

> Read-only sanity check. Compares the 2,490 items in Tracera project "AGILEPLUS"
> against this AgilePlus repo. Reversible; do not merge.

## Summary

- **Tracera items checked:** 0 of 2,490 (auth blocked -- see below)
- **Repo paths/specs cross-referenced:** ~30 (paths, contracts, journeys, oracles, crates, workflows, ADRs)
- **False positives (Tracera item -> non-existent target):** UNKNOWN (auth blocked)
- **False negatives (repo spec -> missing Tracera item):** UNKNOWN (auth blocked)
- **Auth status:** **BLOCKED by Cloudflare Access** -- all API paths return a
  Cloudflare Access login interstitial (`floral-moon-c604.cloudflareaccess.com`,
  `auth_status: NONE`, `service_token_status: false`). No public read endpoint.
  Per task stop condition, no bypass attempted.

## 1. False positives

**Cannot be evaluated -- Tracera auth required.**

If/when access is restored, run cross-reference against the canonical repo
sources listed in section 6 below.

## 2. False negatives

**Cannot be evaluated -- Tracera auth required.**

If/when access is restored, verify Tracera covers every canonical item in
section 6 (each row is one item-class that the repo defines; an absent Tracera
item for that class is a false negative).

## 3. Notable observations (repo-side baseline)

### 3.1 The contract has two versions across the repo

The campaign brief assumed `spec/product/mature-contract.v1.json` is at
schema 2.1.0 source-backed. The repo has **two contract states**:

| Ref | Branch | schema_version | status | reqs | file size |
|-----|--------|----------------|--------|------|-----------|
| `git show main:spec/product/mature-contract.v1.json` | `main` (`2701f24e`) | **1.1.0** | `invalidated_requires_source_backed_replacement` | 0 | 1,670 B |
| Same on `refactor/release-lifecycle-v2-r1-r10` (`ba8b313c`) | current branch | **1.1.0** | `invalidated_requires_source_backed_replacement` | 0 | 1,670 B |
| `git show origin/spec/mature-product-contract-v2:spec/product/mature-contract.v1.json` | `origin/spec/mature-product-contract-v2` (`4e9f454b`) | **2.1.0** | `accepted_mature_semantic_contract_runtime_evidence_pending` | 0 (but `canonical_spine.derived_requirement_count = 67`) | larger envelope |

**Implication for Tracera.** If Tracera was last synced against `main`, its
contract item points at the invalidated 1.1.0 envelope. If synced against the
spec branch, its contract item should reflect the 2.1.0 source-backed version.
The snail memo says the A'' spec-chain merge is "in flight" (`8ce7a446`,
PR #1114 landed); until that merge lands, the two views disagree.

The spec branch's `canonical_spine` references:
- `spec/product/requirements.semantic.v1.json` (67 mature requirements)
- `verification/semantic-verification-cases.v1.json` (92 cases, `status: specified_not_executed`)
- `spec/product/journeys.v1.json` (12 journeys, status `accepted_mature_semantic_journeys`)

### 3.2 The 92 oracles do not exist on main / current branch

The task brief located the oracles at
`verification/semantic-verification-cases.v1.json`. That file **only exists on
the spec branch**, not on `main` or the current working branch. On the current
branch `verification/` contains only `AUTOGRADER_CONTRACT.md`.

| File | Current branch | `main` | Spec branch |
|------|----------------|--------|-------------|
| `verification/semantic-verification-cases.v1.json` | **missing** | **missing** | present (92 cases, schema 0.4.0) |
| `verification/AUTOGRADER_CONTRACT.md` | present | present | present |

**Possible Tracera false-negative candidate:** if Tracera contains any item
that references `verification/semantic-verification-cases.v1.json` as a target
path, the path resolves only on the spec branch. On `main` it is "missing"
until the A'' merge lands.

### 3.3 The 12 journeys exist in two places

| Where | Count | Format |
|-------|-------|--------|
| `docs/journeys/FR-024-*.md` (per-journey narrative pages) | **8** files (`FR-024-1.md`..`FR-024-8.md`) | markdown pages anchored to FR/NFR rows |
| `spec/product/journeys.v1.json` (machine-readable) on spec branch | **12** (`AP-J-01`..`AP-J-12`) | JSON records with `id`, `actor`, `preconditions`, `outcome`, `activities`, `failure_recovery` |
| `docs/journeys/` extras | 2 special pages (`dashboard-health-check.md`, `domain-models.md`) + `manifests/` + `assets/` | supplemental |

**Implication.** The journey narrative (`docs/journey.md`) count (8) and the
journey data count (12) diverge. If Tracera links against the `.md` files,
expect false-negatives for the 4 journeys that have data but no page yet (or
vice-versa: pages exist for journey IDs that may not be in the spec-branch
JSON if the spec branch is the source).

### 3.4 Crate count: 30, not 36

The task brief stated "36 active members". Actual workspace members = **30**
(per `Cargo.toml [workspace] members`, including `tests/{e2e,e2e-desktop,transport,bdd}`,
3 `agileplus-agents/crates/*`, `desktop/src-tauri`, and 24 named
`crates/agileplus-*` / `crates/traceability-core`). The difference may be
draft crates on other branches or pre-squash inventory.

### 3.5 STABLE.md, traceability-core, release workflows -- all present

| Repo asset | Path | Present? |
|------------|------|----------|
| Tier definition doc | `STABLE.md` | yes (S0/S1/S2 with criteria) |
| Tracera embed interface | `crates/traceability-core/` | yes (`artifact.rs`, `contract.rs`, `governance.rs`, `ids.rs`, `impact.rs`, `intent_graph.rs`, `lifecycle.rs`, `matrix.rs`, `requirement.rs`) |
| Release workflows | `.github/workflows/*.yml` | yes (9 files: `release.yml`, `agileplus-release.yml`, `release-crates.yml`, `release-plz.yml`, `release-attestation.yml`, `release-desktop.yml`, `release-drafter.yml`, `publish.yml`, `promote.yml`) |
| Spec layer | `spec/product/*.md` + `*.json` | yes (15+ files; the two `.v1.json` envelopes + 8+ ADRs/specs) |

The lifecycle audit (2026-10-12) found `promote.yml` references a missing
`gate-check.yml`. If Tracera has an item pointing at
`.github/workflows/gate-check.yml`, that path is genuinely absent on disk
(false positive candidate).

### 3.6 Campaign state relevant to Tracera freshness

From `docs/sessions/20261012-snail-unified-execution-memo.md`:
- main HEAD (memo time): `8ce7a446` (PR #1114 landed)
- Spec branch: `origin/spec/mature-product-contract-v2 @ 4e9f454b`
- Spec branch is **267 commits ahead** of main, source-backed (2.1.0), with
  92 oracles `specified_not_executed`
- A'' merge is "in flight" (Q1 lock-in per the matrix)
- Recommended execution: A'' x B (land contract, then execute oracles in same arc)
- Q2-B tier-0 pilot is queued: 5 cases, 13.5h (`AP-VC-001`, `AP-RVC-040`,
  `AP-RVC-041`, `AP-RVC-042`, `AP-RVC-043`)

This means any Tracera item that pre-dates the A'' merge is almost certainly
stale on contract shape (was 1.1.0 invalidated, should now be 2.1.0
source-backed).

### 3.7 The 2,490 item count, plausible breakdown

ADR-0017 lists Tracera contents as Requirements + TraceLinks + Oracles +
Journeys. With current repo data:

| Item class | Expected count |
|------------|----------------|
| Requirements (`AP-R-*`) | 67 |
| Journeys (`AP-J-*`) | 12 |
| Oracle cases (`AP-VC-*`) | 92 |
| TraceLinks (req<->oracle, req<->journey, etc.) | ~1,710 (every req linked to multiple oracles + journey) |
| Subtotal | ~1,881 |
| Other (crates, workflows, ADRs, audit-hash envelopes) | ~609 |
| **Plausible total** | **~2,490** |

This breakdown is a *plausibility estimate*, not a measurement. If Tracera
holds more or fewer TraceLinks per requirement the count will shift.

## 4. Auth/access notes

- **Auth method:** Cloudflare Access (Zero Trust). The application banner
  shows `floral-moon-c604.cloudflareaccess.com` and JWT metadata in the
  redirect URL.
- **Access state observed:** `auth_status: NONE`, `service_token_status: false`,
  `is_warp: false`, `is_gateway: false`, `cert_issuer_dn: ""`.
- **Probed endpoints (all return Cloudflare Access login HTML, HTTP 200, ~48 KB):**
  - `/`, `/robots.txt`, `/sitemap.xml` -- login page
  - `/api`, `/api/v1`, `/api/v1/health`, `/api/v1/auth`, `/api/v1/login` -- login page
  - `/api/v1/projects`, `/api/v1/projects/agileplus`, `/api/v1/projects/AGILEPLUS`,
    `/api/v1/agileplus`, `/api/v1/items`, `/api/v1/requirements`, `/api/v1/oracles`,
    `/api/v1/journeys`, `/api/v1/export` -- login page
  - `/api/v1/openapi.json`, `/api/v1/swagger.json`, `/openapi.json` -- login page
- **No public read path** -- the API requires a Cloudflare Access service
  token or interactive IdP login to even reach the application layer.
- **Stop condition hit** ("Tracera requires auth you don't have").
- **No bypass attempted.**

## 5. Recommendations

1. **Do not block the campaign on this check.** Tracera load-check is
   orthogonal to A'' merge; defer until auth is provisioned.
2. **Provision a Cloudflare Access service token** for the agent fleet, scoped
   to `tracera.pheno.studio` with read-only on the AGILEPLUS project. Once
   available, re-run this check end-to-end with the documented repo-side
   baseline (section 6).
3. **After re-run:** if false-positives > 0, the most likely targets are
   `.github/workflows/gate-check.yml` (does not exist per the lifecycle audit),
   `verification/semantic-verification-cases.v1.json` (only on spec branch),
   and any Tracera item that pre-dates the A'' merge (will reflect 1.1.0
   invalidated state, not 2.1.0 source-backed).
4. **If false-negatives > 0:** the most likely candidates are: `STABLE.md`
   tier items (S0/S1/S2), the 9 release workflows in `.github/workflows/`,
   the audit-hash envelope items, and `crates/traceability-core/` artifacts
   that ADR-0017 says Tracera depends on.
5. **Tie this into ADR-0017 Phase 4 (Data migration):** the Tracera-import
   CLI work needs this baseline as input. Treat this doc as the "import spec
   starting point" -- once auth is provisioned, the cross-reference becomes
   the actual migration test.
6. **Crates count discrepancy:** the task brief's "36 active members" does
   not match the workspace's 30. Investigate outside this check whether the
   brief refers to a different branch or pre-squash inventory; this matters
   if Tracera has crate-level items.

## 6. Cross-reference checklist (use after auth restored)

For each Tracera item that references an AgilePlus asset, verify:

| Reference type | Repo source of truth |
|----------------|----------------------|
| `spec/product/mature-contract.v1.json` schema 2.1.0 | exists on `origin/spec/mature-product-contract-v2` only |
| `spec/product/requirements.semantic.v1.json` (67 reqs) | exists on spec branch; `git show origin/spec/mature-product-contract-v2:...` |
| `verification/semantic-verification-cases.v1.json` (92 cases) | exists on spec branch; schema_version `0.4.0`; status `specified_not_executed` |
| `spec/product/journeys.v1.json` (12 journeys) | exists on spec branch; IDs `AP-J-01`..`AP-J-12` |
| `STABLE.md` | exists at repo root |
| `docs/adr/0017-tracera-embedding-migration-path.md` | exists (note: superseded by ADR-0018 per its own status header) |
| `docs/adr/0018-tracera-agileplus-sibling-runtimes.md` | not present in repo root scan; may be only in session folder |
| `crates/traceability-core/` | present (9 source files: artifact, contract, governance, ids, impact, intent_graph, lifecycle, matrix, requirement) |
| `crates/*` (30 workspace members) | present (see Cargo.toml) |
| `.github/workflows/*.yml` (9 files) | present; `promote.yml` refs missing `gate-check.yml` |
| `docs/journeys/FR-024-*.md` (8 files) | present on main; 12 journeys live in spec-branch JSON |
| `kitty-specs/eco-035-spec-oracle/{meta.json,plan.md,spec.md,tasks.md}` | present |
| `verification/AUTOGRADER_CONTRACT.md` | present |

For each Tracera item that should exist (per ADR-0017 model), verify a Tracera
entry covers it.

## 7. Auth/access notes (extended)

### 7.1 Cloudflare Access JWT claim observations

The redirect URL embeds a JWT with these claims (decoded from `meta` param):

```
{
  "type": "meta",
  "aud": "1502c30a5519bf4a7dbd7ad0148e5d2b9d951aa8371d64ff4858bdc468396b27",
  "hostname": "tracera.pheno.studio",
  "redirect_url": "/api/v1/projects",
  "service_token_status": false,
  "is_warp": false,
  "is_gateway": false,
  "exp": 1791839621,
  "nbf": 1791839321,
  "iat": 1791839321,
  "auth_status": "NONE",
  "mtls_auth": {
    "cert_issuer_dn": "",
    "cert_serial": "",
    "cert_issuer_ski": "",
    "cert_presented": false,
    "common_name": "",
    "auth_status": "NONE"
  },
  "real_country": "US",
  "app_session_hash": "77892ce89248cfaf3f1d635aa25c2c608b440529d5ce92d5731810219285694b3"
}
```

The 2-minute future window (nbf -> exp) suggests a short-lived challenge JWT.
The `auth_status: NONE` confirms the agent has no Cloudflare Access identity
in this session.

### 7.2 What we did NOT do

- Did not attempt interactive Cloudflare Access login (no IdP credentials).
- Did not try to brute-force API paths or guess a service token.
- Did not scrape cached/indexed pages for Tracera data (none found).
- Did not modify any Tracera-side state.
- Did not modify any AgilePlus repo state.
- Did not open a PR or push anything.

## 8. What this check IS vs IS NOT

**IS:** a documented repo-side baseline + an auth blocker report. The
checklist in section 6 is ready-to-run when auth is provisioned.

**IS NOT:** a complete cross-reference. False-positive and false-negative
counts are UNKNOWN, not zero. Anyone reading this doc should treat the
"checked" rows above as "verified to exist on disk", and the "checked"
columns as "the path is real and the file is non-empty" -- not "matches a
Tracera item".

---

_Ledger: tx-agent: jcode, tx-validated: manual, tx-task: tracera-load-check-20261012, tx-scope: docs/TRACERA-LOAD-CHECK.md, tx-intent: Read-only Tracera load check vs AgilePlus repo -- auth-blocked, repo-side baseline established_