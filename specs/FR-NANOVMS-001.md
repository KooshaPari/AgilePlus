---
id: FR-NANOVMS-001
title: bare-cua Nanovms Integration — Sandboxed CUA Execution
status: specified
priority: P1
created: 2026-04-04
category: integration
owner: phenotype-org
source: kitty-specs/014-bare-cua-nanovms-integration
---

# FR-NANOVMS-001: bare-cua Nanovms Integration

## Description

Integrate bare-cua (Rust-based Computer-Use Agent) with nanovms 3-tier isolation architecture (WASM, gVisor, Firecracker) for secure, isolated CUA execution.

## Context

bare-cua enables AI agents to interact with computer systems through tool execution, browser automation, and file system operations. Currently runs on host with limited isolation.

## Problem Statement

- No isolation: Tool execution shares host resources
- Security risk: Untrusted code can access sensitive files/systems
- Platform inconsistency: Different isolation on macOS vs Linux vs Windows
- No recovery: Failed/corrupted operations require manual cleanup

## Goals

- Enable 3-tier sandboxing for bare-cua tool execution:
  - Tier 1 (WASM): Fast tool execution (~1ms startup, ~1MB memory)
  - Tier 2 (gVisor): Browser automation and semi-trusted code (~90ms startup)
  - Tier 3 (Firecracker): Full isolation for untrusted file operations (~125ms startup)

## Acceptance Criteria

- [ ] WASM sandbox for fast tool execution
- [ ] gVisor containers for browser automation
- [ ] Firecracker microVMs for full isolation
- [ ] Cross-platform support (macOS/Lima, Windows/WSL2, Linux/KVM)
- [ ] Tier selection based on trust levels
- [ ] Automatic recovery and cleanup

## Notes

Original: `kitty-specs/014-bare-cua-nanovms-integration/`
