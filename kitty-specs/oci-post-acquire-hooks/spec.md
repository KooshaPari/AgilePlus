# OCI Post-Acquire Universal Hook Chain

## Meta

- **ID**: TBD (assigned by agileplus specify)
- **Title**: OCI Post-Acquire Universal Hook Chain
- **Created**: 2026-04-25
- **State**: shipped (backfill spec)
- **Scope**: phenotype-infra (`iac/oci-post-acquire/` Rust crate)
- **Reference PR**: https://github.com/KooshaPari/phenotype-infra/pull/14
- **Branch**: `feat/oci-acquire-hooks` (merged)

## Context

Once the OCI lottery daemon (PR #15, sibling spec) acquires an A1.Flex instance, that bare VM must be brought into the Phenotype mesh: SSH reachable, on Tailscale, baseline-configured, DNS-routable, recorded in mesh state, and visible to the operator. Doing this by hand is error-prone and not idempotent.

This spec backfills the design intent for the universal post-acquire hook chain shipped in PR #14, an 8-step idempotent pipeline that fully integrates a freshly acquired host.

## Problem Statement

A newly acquired OCI host has no identity, no overlay-network membership, no baseline config, and no presence in operator-facing surfaces. Without an idempotent, observable post-acquire chain:
- Manual onboarding diverges from machine to machine
- Re-running steps risks duplicate DNS records, duplicate Tailscale enrollments, etc.
- Failures mid-chain leave the host in an undefined partial state
- The operator may not learn the host is ready

## Goals

- 8-step idempotent post-acquire chain runnable from any caller (lottery daemon, manual `oci compute launch`, future provisioner)
- Each step safe to re-run with no side effects on already-completed state
- Clear ordering with explicit dependencies (encoded as Mermaid flowchart)
- Operator notification on completion (iMessage)
- Plug-in extension directory for project-specific follow-ups
- Followup-trigger hands off to the next stage of the mesh pipeline

## Non-Goals

- Lottery / acquisition logic (delegated to sibling `oci-lottery` crate; see PR #15)
- Long-running daemon behavior (chain is one-shot per host)
- Multi-cloud abstraction (OCI-only initial scope)
- GUI/dashboard for chain progress (CLI + logs only)

## The 8-Step Chain

| # | Step | Idempotency Strategy |
|---|------|----------------------|
| 1 | SSH wait | Poll until SSH responds; no state mutation |
| 2 | Tailscale enroll | Skip if already in tailnet (check `tailscale status`) |
| 3 | Ansible baseline | Ansible is idempotent by design |
| 4 | Cloudflare DNS A record | Upsert (PUT) — same record name → same record |
| 5 | mesh-state commit | Git commit only if diff present |
| 6 | iMessage notify | Always send (notification, not state) |
| 7 | Plug-in extension dir | Iterate `extensions/*.sh` — each plugin owns its own idempotency |
| 8 | Followup-trigger | Fire downstream pipeline event |

Full Mermaid flowchart and idempotency table live in `docs/governance/oci-acquire-hook-chain.md` (added in PR #14).

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-OCI-ACQ-001 | All 8 steps execute in declared order |
| FR-OCI-ACQ-002 | Every step is idempotent and safe to re-run |
| FR-OCI-ACQ-003 | SSH-wait blocks until host is reachable |
| FR-OCI-ACQ-004 | Tailscale enrollment skipped if already enrolled |
| FR-OCI-ACQ-005 | Cloudflare DNS A record upserted (no duplicates) |
| FR-OCI-ACQ-006 | mesh-state commit only when diff is non-empty |
| FR-OCI-ACQ-007 | Operator notified via iMessage on completion |
| FR-OCI-ACQ-008 | Plug-in directory loaded and each extension invoked |
| FR-OCI-ACQ-009 | Followup-trigger fires downstream event |

## Implementation Status

**Shipped** in PR #14 (merged 2026-04-25). This spec is a retroactive backfill per AgilePlus mandate — "no code without corresponding AgilePlus spec." Subsequent work touching `iac/oci-post-acquire/` MUST reference this spec.

## Cross-Project Reuse Opportunities

- The hook-chain dispatcher is reused by the OCI lottery daemon (PR #15) — both crates share the pattern. If a third caller appears, extract into a generic `phenotype-hookchain` crate.
- The plug-in extension directory pattern is a candidate template for any future "post-X" pipeline (post-deploy, post-build, post-merge).
