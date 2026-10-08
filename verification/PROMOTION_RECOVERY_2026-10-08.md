# Promotion recovery and release closure

## Implemented behavior

`agileplus ship` now delegates mutation to the application-owned promotion saga. A Validated label alone is insufficient: the SQLite immutable acceptance receipt must match every planned work package and exact candidate.

The saga prepares immutable Git merge objects without changing a target ref, index or worktree. SQLite records the expected target and resulting commit before publication. Git publication compares the exact target identity, recognizes an exact already-published result, and rejects any other movement. SQLite separately confirms each publication, then commits Shipped state, chained audit, chained event and promotion receipt in one immediate transaction. Cleanup starts only after that transaction commits; failed cleanup remains pending for a later ship invocation. Promotion receipts and journal history have database immutability guards.

A target checked out in any worktree must be freed before publication. The saga fails closed rather than update its ref underneath local files. A changed target or changed target override requires operator reconciliation; the journal is not silently replaced. Git and SQLite do not share an ACID transaction, and external Git writers must coordinate with promotion.

## Executed evidence

- Eight real Git + file SQLite recovery tests passed: exact receipt replay after reopen; publication before lost confirmation; late receipt transaction failure rollback; checked-out target preservation and retry; target drift rejection; missing immutable acceptance receipt rejection; linked primary-worktree protection; conflict preparation without target mutation.
- Fourteen CLI ship gate/flow tests and ten ship side-effect tests passed.
- Mounted HTTP acceptance suite: ten passed after the schema addition.
- Full dashboard typecheck, nine operator component tests, production build and JSX accessibility lint passed.
- Real Chromium browser journey passed against the actual Axum/file SQLite API, production Caddy fragment and built frontend at commit `f959fad5390b8ff5e016be38cc2caedd27121597`: [CI run 37718656528](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656528). It includes gateway authentication, axe accessibility, a lost response after committed acceptance, API restart, receipt replay and exactly one persisted event. [HTML report and sandbox screenshot](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656528/artifacts/11524233847) were uploaded. This is a disposable CI environment witness, not production deployment evidence.
- Recovery Acceptance Witnesses [CI run 37718656424](https://github.com/KooshaPari/AgilePlus/actions/runs/37718656424) passed all SQLite, application, migration, HTTP, origin, authenticated gRPC and CLI recovery gates on the tested commit.
- Focused CLI/API/application/Git/SQLite Clippy with all targets passed. Full workspace tests were attempted but could not compile the Linux desktop dependency because GLib development prerequisites are absent locally.

## Dependency and CI repair

The workspace NATS dependency now uses the 0.50 series already used by the Git adapter. The locked vulnerable rustls-webpki 0.102.8 chain is removed. The config macro uses the maintained pastey replacement. Cargo Deny metadata errors (one test crate license and three unversioned desktop path dependencies) are repaired without loosening policy.

The optional Neo4j adapter is pinned to upstream 0.9.0-rc.10, replacing its unmaintained backoff/instant/paste/PEM chain and adapting the new synchronous constructor. This is an explicitly pinned prerelease, not a live Neo4j validation claim. Linux Tauri GTK dependencies still have upstream findings; no blanket advisory ignore was added. The yanked yoke-derive release is replaced. Security OSV scanning now verifies and scans the committed lockfile instead of regenerating a different dependency graph.

The accessibility workflow now uses the root committed npm lockfile, Node 24, installed axe dependencies and a valid TypeScript/JSX lint configuration. Color contrast is enabled. A separate operator-browser workflow builds a real file SQLite Axum fixture and checks the production Caddy fragment against a disposable static frontend. The browser test drops an acceptance response after a real commit, restarts the backend, reloads and inspects the original receipt and single persisted event.

## Deployment evidence and blockers

The cloud browser opened the public product URL and observed the old dashboard, with zero epics/stories. That is not evidence that the private desktop backend works. Vercel project metadata listed a READY deployment, but scoped deployment inspection returned 403 for the owning team. No new preview or production deployment was created from this patch. No desktop/tailnet host execution capability was established.

The source candidate is reviewable; the remotely usable operator release remains blocked by actual desktop/tailnet execution and scoped Vercel deployment access, the retained desktop dependency findings and unpublished crates.io workspace dependencies. No incremental paid service was provisioned.

## Remaining release checks

Cargo Deny licenses, bans and sources passed; advisories and OSV remain blocking on the Linux desktop GTK/GLib/Unicode chain. Cargo audit, secret scans, workspace audit, Python security and fuzz CI passed on the tested commit. Registry-based semver checks cannot find the unpublished agileplus-agent-dispatch baseline; CLI publish dry-run cannot resolve unpublished agileplus-application. These are release preparation blockers, not waived checks. The release workflow was regenerated with cargo-dist 0.33.0; its generation check passes. The newer faster-hex finding is fixed by locking 0.10.1. Accessibility path and Prettier errors found by CI are corrected in the follow-up; their rerun must be observed before claiming those CI gates passed.
