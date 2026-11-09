---
description: DEPRECATED — use `ap dashboard` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.dashboard` | `ap dashboard` |

```bash
ap dashboard
```

## What it does

Opens the in-flight work visualization (Dioxus web/desktop viewer per memory: project_session_delivery).

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap dashboard`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
