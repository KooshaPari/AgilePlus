# AgilePlus Dogfood Guide — Use It Right Now

**Status**: ✅ Fully functional — M1+ complete  
**Last verified**: 2026-08-21  
**What it is**: OpenSpec/SpecKit-like spec management tool with governance, traceability, and AI agent integration

---

## Quick Start (2 minutes)

### Build
```bash
cargo build --release
```

### Initialize a project
```bash
cd your-project
agileplus init
# Creates .agileplus/ directory with SQLite database
```

### Create a spec
```bash
# Interactive interview
agileplus specify

# From file
agileplus specify --from-file my-spec.md

# With feature name
agileplus specify --feature my-feature
```

### List specs
```bash
# All specs
agileplus list

# Filter by state
agileplus list --state draft
agileplus list --state approved
agileplus list --state implemented
```

---

## CLI Reference — All M1 Commands

### Core Commands (always available)

#### `specify` — Create/review specs
```bash
# Interactive interview (asks questions)
agileplus specify

# From file
agileplus specify --from-file my-spec.md

# With specific feature slug
agileplus specify --feature my-feature --from-file spec.md

# Force overwrite existing
agileplus specify --feature my-feature --from-file spec.md --force

# Target specific branch
agileplus specify --target-branch develop
```

#### `list` — List features
```bash
# All features
agileplus list

# Filter by state
agileplus list --state draft
agileplus list --state in-progress
agileplus list --state implemented

# JSON output
agileplus list --format json
```

#### `cycle` — Manage delivery cycles
```bash
# List cycles
agileplus cycle list

# Create cycle
agileplus cycle create --name "Sprint 1" --duration 14d
```

#### `queue` — Manage triage backlog
```bash
# List backlog
agileplus queue list

# Add to backlog
agileplus queue add --title "Bug fix" --description "..."
```

#### `module` — Manage product modules
```bash
# List modules
agileplus module list

# Create module
agileplus module create --name "Frontend" --description "UI components"
```

#### `platform` — Manage platform services
```bash
# Start all services
agileplus platform up

# Stop all services
agileplus platform down

# Check status
agileplus platform status

# View logs
agileplus platform logs
```

#### `dashboard` — Render DAG/status
```bash
# Show DAG status
agileplus dashboard

# JSON output
agileplus dashboard --format json
```

#### `cockpit` — Scorecards
```bash
# Show scorecard
agileplus cockpit

# Publish scorecard
agileplus cockpit publish
```

#### `rubric` — Governance scoring
```bash
# Score a repo
agileplus rubric --path ./my-repo
```

### Advanced Commands (behind `full-deps` feature)

```bash
# Enable all commands
cargo build --release --features full-deps

# Research a feature
agileplus research --feature my-feature

# Generate delivery plan
agileplus plan --feature my-feature

# Implement work packages
agileplus implement --feature my-feature

# Validate governance
agileplus validate --feature my-feature

# Ship a feature
agileplus ship --feature my-feature

# Generate retrospective
agileplus retrospective --feature my-feature

# Triage incoming items
agileplus triage
```

---

## How Agents Use AgilePlus

### Create a Spec (via MCP)
```json
{
  "tool": "create_spec",
  "arguments": {
    "title": "User Authentication",
    "description": "Implement JWT-based auth with refresh tokens",
    "requirements": [
      "FR-001: Login endpoint",
      "FR-002: Token refresh",
      "FR-003: Logout endpoint"
    ]
  }
}
```

### List Specs (via MCP)
```json
{
  "tool": "list_specs",
  "arguments": {
    "state": "draft",
    "format": "json"
  }
}
```

### Update Spec State (via MCP)
```json
{
  "tool": "update_spec_state",
  "arguments": {
    "feature": "user-auth",
    "state": "approved"
  }
}
```

---

## Database Location

By default, AgilePlus stores data in `.agileplus/agileplus.db` in your project root.

```bash
# Custom DB location
agileplus --db /path/to/custom.db list
```

---

## What's Already Working

- [x] Spec CRUD (create, read, update)
- [x] Interactive interview for spec creation
- [x] File-based spec import
- [x] Feature state machine (draft → in-progress → implemented → shipped)
- [x] Cycle management (sprint-like delivery units)
- [x] Queue/triage backlog
- [x] Module management (product area groupings)
- [x] Platform services (up, down, status, logs)
- [x] DAG/status dashboard
- [x] Cockpit scorecards
- [x] Governance rubric scoring
- [x] SQLite storage (zero infrastructure)
- [x] Git VCS integration
- [x] Audit trail (hash chain)
- [x] Governance validation
- [x] Intent graph (DAG of specs, code, tests)
- [x] Traceability links

## What's Next (M2)

- [ ] Spec validation (schema, completeness)
- [ ] Drift detection (spec vs implementation)
- [ ] Spec templates (reusable patterns)
- [ ] Governance scoring automation
- [ ] Multi-user collaboration
- [ ] GitHub sync (bidirectional)
- [ ] Plane.so integration
- [ ] Import/export (JSON, YAML, Markdown)

---

## Architecture

```
agileplus-cli
  ├── specify.rs     — Spec CRUD + interactive interview
  ├── cycle.rs       — Delivery cycle management
  ├── list.rs        — Feature listing/filtering
  ├── queue.rs       — Triage backlog
  ├── module.rs      — Product module management
  ├── dashboard.rs   — DAG/status rendering
  ├── cockpit.rs     — Scorecard publishing
  └── rubric.rs      — Governance scoring

agileplus-domain
  ├── domain/        — Core entities (Feature, StateMachine, Audit)
  ├── ports/         — StoragePort, VcsPort traits
  ├── config/        — Configuration management
  └── traceability/  — Intent graph, governance

agileplus-sqlite
  ├── repository/    — SQLite implementations
  ├── migrations/    — Schema migrations
  └── event_store.rs — Event sourcing
```

---

**AgilePlus is ready for dogfooding.** Initialize a project, create some specs, and start tracking your development lifecycle. The spec→code traceability is the killer differentiator.
