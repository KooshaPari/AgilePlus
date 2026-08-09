# Isolated AgilePlus Recovery

## Goal

Establish a clean recovery boundary from live `origin/main` without mutating the dirty
checkout at `../AgilePlus`.

## Boundaries

- Recovery checkout: `AgilePlus-recovery-20260714`
- Recovery branch: `recovery/isolated-20260714`
- Baseline: `a83a7677ecacac0a3080e41da312d80def74fee5`
- Source remote: `git@github.com:KooshaPari/AgilePlus.git`
- Evidence archive: `../AgilePlus-recovery-evidence-20260714`

The dirty source checkout remains evidence-only. No reset, clean, checkout, stash mutation,
prune, or garbage collection was performed there.
