# Functional Requirements -- phench

## FR-TGT: Target Lifecycle

### FR-TGT-001: Target Init
The CLI SHALL create a target with `phench target init <name>`, writing metadata to `.phench/<name>/`.
**Traces to:** E1.1

### FR-TGT-002: Repo Binding
The CLI SHALL bind repositories to targets via `phench target add-repo <name> --repo <path> --ref <ref>`, supporting branch, tag, or SHA refs.
**Traces to:** E1.2

### FR-TGT-003: Target Lock
The CLI SHALL lock a target via `phench target lock <name>`, producing a JSON lockfile with exact commit SHAs for all bound repos.
**Traces to:** E1.3

### FR-TGT-004: Lock Immutability
A locked target SHALL reject modifications until explicitly unlocked.
**Traces to:** E1.3

## FR-MAT: Materialization

### FR-MAT-001: Materialize Target
The CLI SHALL materialize a locked target via `phench target materialize <name>`, cloning repos at locked SHAs to the materialized root directory.
**Traces to:** E2.1

### FR-MAT-002: Precondition Checks
Materialization SHALL validate preconditions (disk space, git availability, repo accessibility) and fail with actionable messages if any check fails.
**Traces to:** E2.1

## FR-OPS: Runtime Operations

### FR-OPS-001: Status Display
`phench status <name>` SHALL display lock state, repo refs, and materialization status.
**Traces to:** E3.1

### FR-OPS-002: Timeline
`phench timeline <name>` SHALL display chronological events for the target.
**Traces to:** E3.1

### FR-OPS-003: Sync
`phench sync <name>` SHALL pull latest refs and update the target's lock state.
**Traces to:** E3.2

### FR-OPS-004: Environment Doctor
`phench env doctor <name>` SHALL check environment prerequisites and report pass/fail per check.
**Traces to:** E3.3
