---
description: DEPRECATED — use `ap review-loop` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.review` | `ap review-loop <feature-id>` |

```bash
ap review-loop <feature-id>
```

## What it does

Drives the PR review cycle: dispatch reviewer agents, collect feedback, route to implementer.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap review-loop <feature-id>`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
