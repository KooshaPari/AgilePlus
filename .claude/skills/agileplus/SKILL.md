---
name: agileplus-ops
description: Use for AgilePlus project-management and project-ops tasks that need the AgilePlus CLI: seeding requirements, listing projects/epics/stories, inspecting features, syncing a repository, or working against the local SQLite database.
---

# AgilePlus Ops

Use the released CLI at `E:/agileplus-target/release/agileplus-cli.exe` for operational work.
Prefer the seeded database at `./agileplus.db` unless the user explicitly points at another database.

## Core commands

Seed the requirement catalogs into SQLite:

```powershell
E:/agileplus-target/release/agileplus-cli.exe seed-requirements --db ./agileplus.db
```

List projects from the local database:

```powershell
E:/agileplus-target/release/agileplus-cli.exe list-projects
```

List epics, optionally scoped to a project:

```powershell
E:/agileplus-target/release/agileplus-cli.exe list-epics
E:/agileplus-target/release/agileplus-cli.exe list-epics --project <project-id>
```

List stories, optionally scoped to an epic and/or status:

```powershell
E:/agileplus-target/release/agileplus-cli.exe list-stories
E:/agileplus-target/release/agileplus-cli.exe list-stories --epic <epic-id>
E:/agileplus-target/release/agileplus-cli.exe list-stories --status <todo|in_progress|review|done|blocked|cancelled>
```

Inspect features from the in-memory feature surface:

```powershell
E:/agileplus-target/release/agileplus-cli.exe feature list
E:/agileplus-target/release/agileplus-cli.exe feature show <feature-id>
```

Sync a GitHub repository into AgilePlus:

```powershell
E:/agileplus-target/release/agileplus-cli.exe sync <owner/repo> --project <project-id> --epic <epic-id> --token <github-token>
```

## Operational rules

- Use `seed-requirements --db ./agileplus.db` before listing project data when the database may be empty.
- Use `list-projects` first to discover project IDs, then `list-epics --project <id>`, then `list-stories --epic <id>` when narrowing scope.
- Use `feature list` and `feature show <id>` for the mock feature surface exposed by the CLI.
- Use `sync` only when you have a valid GitHub repository, project ID, epic ID, and token.
