# Architecture Decision Records -- phench

## ADR-001: Python with Typer CLI Framework

**Status:** Accepted
**Context:** phench needs a CLI with subcommands, rich terminal output, and TUI potential.
**Decision:** Use Python 3.11+ with typer for CLI and rich for terminal formatting.
**Alternatives:** Rust (clap) -- heavier for a project-state tool; Go (cobra) -- less TUI ecosystem.
**Consequences:** Fast iteration; `pyproject.toml` based packaging; editable installs for development.

## ADR-002: Dual-Store State Model

**Status:** Accepted
**Context:** Target state needs to persist both in the project directory (`.phench/`) and a home mirror (`~/phench/`).
**Decision:** Maintain dual stores with bidirectional sync to support both local and global views of project state.
**Alternatives:** Single store in home directory (loses locality), single store in project (loses global overview).
**Consequences:** Sync logic must handle conflicts; both stores must stay consistent.

## ADR-003: Lockfile-Based Reproducibility

**Status:** Accepted
**Context:** Materialized project directories must be reproducible from a target definition.
**Decision:** Lock targets by capturing exact commit SHAs for all bound repos in a JSON lockfile.
**Alternatives:** Branch-based pinning (non-deterministic), git submodules (complex UX).
**Consequences:** Lock/unlock lifecycle adds a step; materialization from lockfile is fully deterministic.

## ADR-004: orjson for Serialization

**Status:** Accepted
**Context:** State files (target metadata, lockfiles, timelines) need fast JSON serialization.
**Decision:** Use orjson for all JSON I/O (10x faster than stdlib json).
**Alternatives:** stdlib json (slower), msgpack (less human-readable).
**Consequences:** Binary dependency; lockfiles remain human-readable JSON.
