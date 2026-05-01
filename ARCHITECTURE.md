# Architecture

## Overview
- AgilePlus is a polyglot workspace with Rust, Python, Go, docs, and supporting automation.
- The workspace centers on shared libraries, CLI surfaces, and platform-level orchestration crates.
- This document is a skeleton for the repo's main boundaries and runtime paths.

## Components
## agileplus-agents
- Rust crate for agent orchestration and workspace automation helpers.

## agileplus-mcp
- Python MCP service surface for integrations and tool exposure.

## rust workspace
- Shared Rust crates and services under the root workspace members.

## python
- Python packages and support code for runtime automation and integration.

## pheno-cli
- Go CLI surface for user-facing commands and workflow automation.

## docs and specs
- Architecture, governance, and product specification material that drives implementation.

## Data flow
```text
user input -> CLI / MCP surface -> shared workspace logic -> adapters and services -> external systems
```

## Key invariants
- Keep workspace-level contracts aligned across Rust, Python, and Go entrypoints.
- Favor shared libraries over duplicate logic in app shells.
- Treat docs and specs as first-class sources for implementation intent.

## Cross-cutting concerns (config, telemetry, errors)
- Config: make workspace settings explicit and environment-driven.
- Telemetry: standardize tracing and logging across language boundaries.
- Errors: normalize error reporting so cross-language calls remain debuggable.

## Future considerations
- Replace grouped placeholders with crate-by-crate ownership notes.
- Add diagrams for orchestration, persistence, and release flows.
- Capture how docs/spec changes map to code-generation or runtime behavior.
