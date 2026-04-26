# AgilePlus

**AgilePlus is a local-first, spec-driven agile work-tracking CLI for AI-agent and human teams.** It models features, work packages, and acceptance criteria as versioned specs on disk, with optional sync to GitHub Issues, dashboards, and P2P merge for multi-actor collaboration.

## Quick Start

Install (from this repo):

```bash
cargo install --path crates/agileplus-cli
```

Three-step flow:

```bash
# 1. Create a feature spec (slug-based, kebab-case)
agileplus specify --feature my-feature-slug

# 2. Generate work packages and tasks for the feature
agileplus tasks --feature my-feature-slug

# 3. Update work-package state as you progress
agileplus status my-feature-slug --wp wp-001 --state doing
```

See **[`docs/guide/quick-start.md`](docs/guide/quick-start.md)** for the full quickstart, including project init, dashboard, and GitHub sync.

## Documentation

The canonical user guide lives under **[`docs/guide/`](docs/guide/)**:

| Topic | File |
|-------|------|
| Quick start | [`docs/guide/quick-start.md`](docs/guide/quick-start.md) |
| Getting started (concepts) | [`docs/guide/getting-started.md`](docs/guide/getting-started.md) |
| `agileplus init` | [`docs/guide/init.md`](docs/guide/init.md) |
| Workflow | [`docs/guide/workflow.md`](docs/guide/workflow.md) |
| Configuration | [`docs/guide/configuration.md`](docs/guide/configuration.md) |
| Sync (GitHub) | [`docs/guide/sync.md`](docs/guide/sync.md) |
| Triage | [`docs/guide/triage.md`](docs/guide/triage.md) |
| Local-first deployment | [`docs/guide/local-first-deployment.md`](docs/guide/local-first-deployment.md) |

> **Note on `docs/guide/` vs `docs/guides/`:** `docs/guide/` (singular) is the **canonical** product user guide. `docs/guides/` (plural) holds supplementary contributor/developer references (e.g. `DEV_STACK.md`, `FR_ANNOTATION_GUIDE.md`). When linking from external docs, prefer `docs/guide/`.

## Repository Layout

```
crates/                  # Rust workspace (CLI, dashboard, sqlite, p2p, grpc, ...)
docs/
  guide/                 # Canonical user guide (start here)
  guides/                # Contributor/developer references
  adr/                   # Architecture Decision Records
  reference/             # API and CLI reference
kitty-specs/             # Live AgilePlus feature specs (eat-our-own-dogfood)
.work-audit/             # Worklog and audit trail
```

## Governance

- `GOVERNANCE.md` — project governance
- `AGENTS.md` — agent interaction rules
- `CLAUDE.md` — Claude Code settings

## Contributing

1. Read [`docs/guide/quick-start.md`](docs/guide/quick-start.md).
2. Create a feature spec: `agileplus specify --feature <your-slug>`.
3. Open a PR referencing the spec.

## License

See [`LICENSE`](LICENSE).
