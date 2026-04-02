# Tasks: Polyrepo Mixed Tranche Wave 1

## Wave Summary

This replaces the older quarter-scale execution list as the active task surface for spec `021`.
The current execution unit is one manager-controlled mixed tranche with six worker-owned work
packages.

- **Feature slug**: `polyrepo-mixed-tranche-wave-1`
- **Topology**: `1 manager + 6 workers`
- **Manager role**: dispatch, rebalance, dependency enforcement, evidence rollup, tranche closeout
- **Worker model**: one worker per work package, no overlapping ownership

## Dependency Graph

```text
Wave Bootstrap
├── WP01 AgilePlus PR lane completion
├── WP02 Helios family PR convergence
├── WP03 Secondary PR lane cleanup
├── WP04 AgilePlus runtime and local deploy evidence
├── WP06 Intake and boundary normalization
└── WP05 Governance enforcement rollout
     ├── depends on WP01
     ├── depends on WP02
     └── depends on WP03
```

## Manager Control Loop

### M0: Create and dispatch wave
- [ ] set feature `polyrepo-mixed-tranche-wave-1` to `planned`
- [ ] create `WP01` through `WP06`
- [ ] assign one worker owner to each WP
- [ ] confirm file-surface boundaries before dispatch
- [ ] dispatch all ready workers in parallel

### M1: Poll and rebalance
- [ ] poll worker status every `30s`
- [ ] classify worker outcome as `done`, `blocked:<class>`, or `needs-rebalance`
- [ ] rebalance only blocked substeps, not whole WP ownership, unless a worker is fully stalled
- [ ] maintain tranche ledger and blocker taxonomy

### M2: Validate and close
- [ ] verify evidence attached for each completed WP
- [ ] keep `WP05` blocked until `WP01`, `WP02`, and `WP03` are done or blocker-finalized
- [ ] publish tranche summary with exact next-wave recommendation

## Work Packages

### WP01: AgilePlus PR lane completion
- [ ] work only on the active AgilePlus split follow-up lanes: `#274`, `#275`, `#276`, `#278`, `#279`
- [ ] advance each lane with one clean follow-up commit or classify it as blocked with an exact blocker type
- [ ] keep fixes scoped to the lane’s original purpose
- [ ] record per-PR status, changed-surface summary, and blocker state

### WP02: Helios family PR convergence
- [ ] work only on `heliosCLI` active governance/review lanes and `heliosApp` PR `#362`
- [ ] classify each lane as `advanced`, `re-scoped`, or `blocked`
- [ ] keep canonical branch ownership explicit for each lane
- [ ] record branch-safe next actions and any root-drift exclusions

### WP03: Secondary PR lane cleanup
- [ ] work only on `cliproxyapi-plusplus` PR `#942`, `phenodocs` PR `#119`, and `agentapi-plusplus` draft PR `#438`
- [ ] separate authored branch-local work from generated or root-checkout noise
- [ ] reduce each repo to one clean lane decision
- [ ] record CI/blocker split and next branch action

### WP04: AgilePlus runtime and local deploy evidence
- [ ] bring up the local AgilePlus runtime using repo-standard local scripts
- [ ] run health verification and capture the first durable local boot evidence set
- [ ] capture event snapshot and classify runtime-event sparsity precisely if it persists
- [ ] update the local deploy evidence baseline

### WP05: Governance enforcement rollout
- [ ] begin only after `WP01`, `WP02`, and `WP03` are complete or blocker-finalized
- [ ] derive enforcement-ready governance state from stabilized active lanes
- [ ] classify active repos as `ready`, `hold`, or `blocked`
- [ ] record required-check names and ruleset baseline status per active repo

### WP06: Intake and boundary normalization
- [ ] classify external intake wave 1 targets: `phenotype-xdd`, `phenotype-docs-engine`, `agileplus-plugin-core`
- [ ] classify boundary surfaces such as `koosha-portfolio` and similar non-repo edges
- [ ] reduce each target to `import-now`, `watch`, `archive`, or `boundary`
- [ ] record the required next artifact for each target

## Acceptance Gate

- [ ] every WP has exact repo/file-surface ownership
- [ ] every WP can finish as `done` or blocker-finalized without new implementer decisions
- [ ] `WP05` remains dependency-gated on `WP01`, `WP02`, and `WP03`
- [ ] tranche summary can be generated from WP outputs and attached evidence alone
