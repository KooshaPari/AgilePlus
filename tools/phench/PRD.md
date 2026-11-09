# Product Requirements Document -- phench

## Product Vision

phench is a standalone control-plane for Phenotype project-state runtime orchestration. It manages "targets" (project configurations) with repository references, lock files, materialization, and environment health checks. It provides a Python CLI/TUI for day-to-day operations.

## E1: Target Lifecycle

### E1.1: Target Initialization

As a developer, I can initialize a named target that defines a project-state configuration.

**Acceptance Criteria:**
- `phench target init <name>` creates target metadata in `.phench/`
- Target includes: name, repos list, refs, lock state, materialization status

### E1.2: Repository Binding

As a developer, I can bind git repositories to a target with pinned refs.

**Acceptance Criteria:**
- `phench target add-repo <name> --repo <path> --ref <ref>` adds repo binding
- Multiple repos per target supported
- Ref can be branch, tag, or commit SHA

### E1.3: Target Locking

As a developer, I can lock a target to snapshot its current state for reproducibility.

**Acceptance Criteria:**
- `phench target lock <name>` produces a lockfile capturing all repo SHAs
- Locked targets are immutable until explicitly unlocked
- Lock metadata includes timestamp and author

## E2: Materialization

### E2.1: Target Materialization

As a developer, I can materialize a target to produce a working project directory.

**Acceptance Criteria:**
- `phench target materialize <name>` clones/checks out repos at locked refs
- Materialized root: `$HOME/CodeProjects/Phenotype/projects/<name>`
- Precondition checks run before materialization (disk space, git availability)

## E3: Runtime Operations

### E3.1: Status and Timeline

As a developer, I can view target status and change history.

**Acceptance Criteria:**
- `phench status <name>` shows current lock state, repo refs, materialization status
- `phench timeline <name>` shows chronological events (init, lock, materialize, sync)

### E3.2: Run and Sync

As a developer, I can run commands in a target's context and sync state.

**Acceptance Criteria:**
- `phench run <name>` executes the target's configured run command
- `phench sync <name>` pulls latest refs and updates lock state

### E3.3: Environment Doctor

As a developer, I can diagnose environment issues for a target.

**Acceptance Criteria:**
- `phench env doctor <name>` checks: git, Python, disk space, repo accessibility
- Reports pass/fail per check with actionable messages
