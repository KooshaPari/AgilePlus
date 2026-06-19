# OCI Always-Free A1.Flex Lottery Daemon

## Meta

- **ID**: TBD (assigned by agileplus specify)
- **Title**: OCI Always-Free A1.Flex Lottery Daemon
- **Created**: 2026-04-25
- **State**: shipped (backfill spec)
- **Scope**: phenotype-infra (`iac/oci-lottery/` Rust crate)
- **Reference PR**: https://github.com/KooshaPari/phenotype-infra/pull/15
- **Branch**: `feat/oci-lottery-daemon` (merged)

## Context

Oracle Cloud Infrastructure (OCI) Always-Free tier offers ARM A1.Flex compute (up to 4 OCPU / 24 GB RAM) but capacity is intermittently unavailable across regions. Acquiring an instance requires repeated polling — a "lottery" — until OCI reports capacity in one of the eligible regions. Manual polling is impractical; a long-running daemon is required to claim capacity the moment it becomes available.

This spec backfills the design intent for the Rust daemon shipped in PR #15, which automates the lottery across 5 OCI regions and persists state across restarts.

## Problem Statement

OCI A1.Flex Always-Free capacity is scarce and stochastic. Manual `oci compute instance launch` invocations miss most availability windows, and shell-loop polling lacks:
- Multi-region fan-out
- Persistent state across SIGTERM / reboot / launchd respawn
- Clean shutdown semantics
- A failsoft hook chain that runs only on success
- Observable, structured logging suitable for long-running daemons

## Goals

- Async tokio polling loop targeting A1.Flex 4 OCPU / 24 GB shape
- Fan-out across 5 OCI regions in parallel
- Persistent state file so restarts resume rather than restart
- SIGTERM-clean shutdown (no half-launched instances)
- On-success hook chain (failsoft — failures in hooks do not undo acquisition)
- launchd unit for macOS daemon supervision
- Rust 2024 edition; minimal external surface (8 files, ~914 LOC)

## Non-Goals

- Provisioning beyond compute (no DB, networking-as-service, etc.)
- Auto-scaling once acquired (one-shot lottery)
- Cross-cloud failover (OCI-only)
- Replacing the post-acquire hook chain (delegated to sibling `oci-post-acquire` crate; see PR #14)

## Architecture

```
iac/oci-lottery/
├── Cargo.toml          # Rust 2024 edition
├── src/
│   ├── main.rs         # tokio entry, signal handling
│   ├── poller.rs       # per-region async poller
│   ├── state.rs        # persistent state (JSON on disk)
│   ├── shapes.rs       # A1.Flex target spec
│   ├── hooks.rs        # success hook chain dispatch
│   └── ...
└── launchd/com.kooshapari.oci-lottery.plist
```

- 5 regions polled concurrently via `tokio::spawn`
- State file written atomically; daemon resumes from last known position
- SIGTERM handler awaits in-flight launches before exit
- Success hook invokes the `oci-post-acquire` hook chain (PR #14)

## Functional Requirements

| FR ID | Requirement |
|-------|-------------|
| FR-OCI-LOT-001 | Poll all 5 OCI regions concurrently for A1.Flex capacity |
| FR-OCI-LOT-002 | Target shape: A1.Flex with 4 OCPU and 24 GB memory |
| FR-OCI-LOT-003 | Persist state to disk across restarts |
| FR-OCI-LOT-004 | Handle SIGTERM cleanly (no orphaned launches) |
| FR-OCI-LOT-005 | Trigger failsoft success hook chain on acquisition |
| FR-OCI-LOT-006 | Run under launchd as a managed daemon |
| FR-OCI-LOT-007 | Structured logging suitable for long-lived process |

## Implementation Status

**Shipped** in PR #15 (merged 2026-04-25). This spec is a retroactive backfill per AgilePlus mandate — "no code without corresponding AgilePlus spec." Subsequent work touching `iac/oci-lottery/` MUST reference this spec.

## Cross-Project Reuse Opportunities

- Hook-chain dispatcher pattern is shared with `oci-post-acquire` crate (PR #14) — candidate for extraction into a generic `phenotype-hookchain` crate if a third caller appears.
- Multi-region polling abstraction may generalize to AWS spot / GCP preemptible lotteries.
