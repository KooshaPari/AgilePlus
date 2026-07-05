---
description: DEPRECATED — use `ap governance constitution` instead. See docs/design/SPECKITTY-MIGRATION.md.
---

> ⚠️ **Deprecated cursor shim.** SpecKitty is being migrated to `agileplus-cli` (`ap`).
> Use the canonical `ap` command below. See `docs/design/SPECKITTY-MIGRATION.md`.

## Migration

| SpecKitty | AgilePlus (`ap`) |
|-----------|-------------------|
| `/spec-kitty.constitution` | `ap governance constitution` |

```bash
ap governance constitution
```

## What it does

Manages the project constitution (immutable principles). Roadmap: full feature parity in agileplus-governance.

## Backward compat

The cursor command remains for legacy callers; new agents should use `ap governance constitution`. The `ap` command is owned by [crates/agileplus-cli/src/commands/](crates/agileplus-cli/src/commands/) and is the canonical entrypoint.
