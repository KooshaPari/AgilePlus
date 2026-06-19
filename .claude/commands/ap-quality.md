---
description: Run the repository quality gate
---

# ap-quality

Run the canonical quality gate checks used by AgilePlus workstreams.

## What this does

1. Runs Rust formatting check and lint.
2. Runs workspace tests.
3. Runs Python checks if `python/` exists and contains pyproject tooling.
4. Runs Cargo deny/license/OS policy checks if configured.

## Steps

```pwsh
$root = Resolve-Path $PSScriptRoot/../..

Push-Location $root
try {
    cargo fmt --all -- --check
    cargo clippy --workspace -- -D warnings
    cargo test --workspace
    if (Test-Path "python/pyproject.toml") {
        python -m pip install --upgrade uv | Out-Null
        uv sync --project python
        uvx ruff format --check .
        uvx ruff check .
    }
    if (Test-Path "deny.toml") {
        cargo deny check licenses
    }
} finally {
    Pop-Location
}
```

## Notes

- This mirrors the repo gates in `AGENTS.md` and CI.
- It may fail if optional Python tooling is unavailable locally.

