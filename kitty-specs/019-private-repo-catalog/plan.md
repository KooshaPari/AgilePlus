# Plan: Private Repo Catalog — 19 Private Repositories

> Phased WBS with DAG. Agent-led discovery and remediation; no human approval gates. Per-WP subtasks in `tasks.md`.

## Phase 0: Discovery

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-000 | `gh api` walk of all 19 private repos: metadata, default branch, last commit, languages, size, collaborators, deploy keys. Output: `research/019-private-inventory.json` + `research/019-private-inventory.md` | — | 6–8 tool calls, ~4 min |

## Phase 1: Catalog + Map

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-101 | Per-repo card (purpose, owner, language, sensitivity tier, dependencies, exposed surface). Driven from WP-000 inventory + targeted file-tree probes. | WP-000 | 4 parallel subagents (~5 repos each), ~6 min |
| WP-102 | Map each private repo against public-repo equivalents (table in spec.md). Resolve ambiguity by reading READMEs + module names. Output: duplicate matrix + override notes. | WP-000 | 1 subagent, ~5 min |

## Phase 2: Duplicate Resolution

For each duplicate pair flagged in WP-102 (currently: `template-lang-go`, `template-commons`, `phenotype-docs-engine`, `phenotype-agent-core`, `phenotype-config`, `phenotype-agents`):

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-201 | Pick canonical (default: public unless private contains hard-secret material) for each pair, port unique commits forward, archive the loser with redirect README | WP-101, WP-102 | 6 parallel subagents, ~6 min total |
| WP-202 | Cross-link: public repo gains a `docs/private-overlay.md` listing internal-only modules; private repo gains a `PUBLIC.md` pointer | WP-201 | 4–6 tool calls, ~3 min |

## Phase 3: Governance

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-301 | Define sensitivity tiers (T1 secrets, T2 internal IP, T3 incubation) and required controls per tier (branch protection, required reviewers, secret scanning, SAST). Author `docs/governance/private-repo-policy.md` | WP-101 | 6–8 tool calls, ~5 min |
| WP-302 | Apply policy: enable branch protection + secret scanning on every cataloged repo via `gh api`; emit drift report for any repo that cannot be auto-configured | WP-301, WP-201 | 8–12 tool calls, ~6 min |
| WP-303 | Standing maintenance contract: each private repo gets a quarterly health-check entry in the org dashboard | WP-301 | 4–6 tool calls, ~3 min |

## Phase 4: Documentation + Verification

| WP | Description | Predecessors | Est. effort |
|----|-------------|--------------|-------------|
| WP-401 | Final catalog (`docs/registry/private-repos.md`) — table view + per-repo card pages | WP-101, WP-201, WP-301 | 6–10 tool calls, ~4 min |
| WP-402 | Verify: re-run WP-000 inventory, diff against WP-401 catalog; any unlisted repo blocks completion | WP-401 | 3–5 tool calls, ~2 min |

## DAG

```
WP-000 ─┬─► WP-101 ─┬─► WP-201 ─► WP-202 ─┐
        │           │                       │
        └─► WP-102 ─┘                       │
                                            │
        WP-101 ─► WP-301 ─┬─► WP-302 ───────┤
                          └─► WP-303 ───────┤
                                            │
                                            ▼
                                          WP-401 ─► WP-402
```

## Cross-Project Reuse Opportunities

- Reuse `gh api` walker pattern from spec 012 (github-portfolio-triage) and spec `kooshapari-stale-repo-triage`. Extract into a shared `phenotype-repo-inventory` Rust binary if not already done.
- Branch-protection apply step in WP-302 should reuse the policy bundle defined in `phenotype-infrakit` rather than per-repo bespoke configuration.
- Sensitivity-tier policy should compose with the secret-scanning + SAST baseline from the Phase 1 Security work (already deployed across 30 repos per session memory).

## Risks

| Risk | Mitigation |
|------|-----------|
| Accidental exposure during duplicate resolution | WP-201 mandates secret-scan run before any public-bound commit move |
| Policy apply (WP-302) breaks existing automation | Drift report — fail loudly, do not silently strip protection rules |
| Stale catalog after merge | WP-402 hard-gate; recurring re-run of WP-000 scheduled quarterly via WP-303 |

## Total estimated effort

~5–8 orchestrator-hours wall-clock; dominated by per-repo card writing in WP-101 (parallel) and policy apply in WP-302. No human checkpoints. All gates are mechanical (inventory diff, gh api response codes, secret-scan pass).
