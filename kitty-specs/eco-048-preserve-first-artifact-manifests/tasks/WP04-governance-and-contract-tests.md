---
work_package_id: WP04
title: Governance and Contract Tests
feature: Preserve-First Artifact Manifests
feature_slug: eco-048-preserve-first-artifact-manifests
sequence: 4
state: planned
created_at: 2026-08-16T01:51:48Z
depends_on:
  - WP01
  - WP02
  - WP03
---

# Work Package: Governance and Contract Tests

## Objective

Bind manifest, rendering, preservation, and recovery proof to fail-closed review and
completion transitions.

## File Scope

- `kitty-specs/eco-048-preserve-first-artifact-manifests/contracts/governance-v1.json`
- `crates/agileplus-governance/src/contracts/preserve_first_artifacts.rs`
- `crates/agileplus-governance/tests/preserve_first_artifacts_contract_test.rs`
- `.github/workflows/preserve-first-artifact-manifest.yml`

## Dependencies

- WP01 schema and custody-manifest validation evidence.
- WP02 deterministic rendering and digest-parity evidence.
- WP03 independent restore, ref-parity, and `git fsck` evidence.

## Acceptance Criteria

- The contract requires schema/fixture proof for WP01 and digest parity for WP02 before
  review transitions.
- The contract requires independent restore, successful ref parity, and successful `git
  fsck` before accepting WP03 review evidence.
- Final completion requires CI output, approval, valid audit-chain verification, and a
  no-secret check.
- Missing, stale, or failed evidence blocks the affected transition.
- CI validates packet JSON and the focused domain, dashboard, and governance tests.

## Test-First Commands

1. Add a contract test with missing audit-chain evidence that expects the final transition
   to be denied.
2. Run: `cargo test -p agileplus-governance preserve_first_artifacts_denies_missing_audit_chain -- --exact`
   Expected before implementation: fail because this contract evaluator is absent.
3. Add the minimal evaluator that consumes `governance-v1.json` and fails closed.
4. Run: `cargo test -p agileplus-governance preserve_first_artifacts -- --nocapture`
   Expected after implementation: required-evidence and denial cases pass.
5. Run: `python3 -m json.tool kitty-specs/eco-048-preserve-first-artifact-manifests/contracts/governance-v1.json >/dev/null`

## Preserve-First Prohibitions

- Do not allow CI or governance automation to invoke destructive Git or archive actions.
- Do not waive restore, ref-parity, `git fsck`, review, CI, or audit requirements through
  generated HTML or a manifest digest alone.
- Do not add secrets to workflows, contracts, test fixtures, or logs.
