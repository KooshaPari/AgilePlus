# Intent reconciliation documentation evidence

Scope: supplemental AgilePlus source reconciliation, not product/runtime acceptance.
Base: `45bb5c9fde4d15ee698f42a6e5f9304aae336ff5` (refreshed `origin/main`).
The revision containing this evidence identifies the reviewed documentation tree;
all four canonical input hashes are pinned in the
[crosswalk](../spec/product/recovery-2026-10-14/requirement-crosswalk.v1.json).

## Executed local checks

Python 3.12 with `jsonschema`; Node 24.19.0; npm 11.9.0; Prettier 3.6.2.
Run from repository root:

```sh
python3 verification/check_intent_reconciliation.py
npm exec --cache /tmp/agileplus-npm-cache --yes --package=prettier@3.6.2 -- prettier --ignore-path /dev/null --check spec/product/recovery-2026-10-14/*.md spec/product/recovery-2026-10-14/*.json docs/adr/proposed-product-work-simulation-boundary.md spec/product/AUTHORITY_INDEX.md spec/product/NON_CODE_FINALITY_LEDGER.md verification/INTENT_RECONCILIATION_2026-10-14.md
git diff --check
```

| Check                                    | Local result and limit                                                                                                                                                              |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crosswalk schema and duplicate JSON keys | Passed; schema validated with Draft 2020-12; no duplicate keys                                                                                                                      |
| Canonical byte/hash comparison           | Passed for requirements, journeys, oracles and mature manifest against base; none changed                                                                                           |
| Identifier and reference consistency     | Passed: 67 unique requirement rows, all 12 canonical journeys and 92 case identities checked; direct requirement witnesses distinguished from shared journey context                |
| Source references and links              | Passed: nine source anchors resolve; Markdown local targets exist and supplied source URLs have expected conversation structure. This is not remote source read-back.               |
| Exact excerpt review                     | Selected quoted spans manually compared with supplied text; original spelling retained; omissions and original-message update caveat explicit                                       |
| Diff privacy review                      | New excerpts limited to product intent; no personal employment context, raw transcripts, internal coordination, private provenance, execution identifiers or transfer data included |
| Formatting and whitespace                | Prettier check and `git diff --check` passed after formatting; no canonical JSON changed                                                                                            |

The checker is documentation validation, not any of the canonical runtime oracles.
The exact 92-case catalogue remains `specified_not_executed`. Source-theme mappings
express intent support; inherited-contract mappings do not fabricate literal owner
requirements or recover unavailable matrix rows.

## Failed checks

`node --test docs/tests/unit/*.test.mjs` exits 1 before assertions with
`ERR_MODULE_NOT_FOUND`: `@phenotype/docs` imported by
`docs/.vitepress/site-meta.mjs`. The same command fails identically at the unchanged
base checkout. No docs unit assertions passed.

`npm --prefix docs run docs:build` exits 1 while loading the VitePress configuration
because `@phenotype/docs` is missing, after local docs dependencies installed with
lifecycle scripts disabled. The base configuration and package manifest are
unchanged. No rendered-site result is claimed. No substitute dependency/configuration
repair is included.

The initial formatting invocation also failed to create the environment's default
npm cache; using an explicit disposable cache succeeded. That failed invocation
was not a formatting verdict.

## Unexecuted and unresolved

No Rust runtime, browser/component/e2e, simulation, ergonomic, deployment or live
federation checks were run for this documentation-only change. Docs component/e2e
checks remain unexecuted after build failure. No GitHub Actions execution is
requested; publication uses the standard CI skip marker to preserve zero requested
hosted execution spend. Skipped/pending remote checks do not constitute acceptance.

The archive remains unavailable and its size/digest/full matrix and proposed
14-package plan remain unverified. Source links were not independently retrieved.
The [dossier](../spec/product/recovery-2026-10-14/README.md) and
[future work](../spec/product/recovery-2026-10-14/FUTURE_WORK.md) bound those gaps.
Product/spatial owners retain scenario and product applicability decisions;
AgilePlus semantic reviewers retain work acceptance compatibility review. Existing
implementation and spatial-registration ownership remains unchanged.
