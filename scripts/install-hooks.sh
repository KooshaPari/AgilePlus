#!/usr/bin/env bash
# 5-line glue: point git at canonical .githooks/ tree (Rust replacement is overkill for one git-config call).
# Justification: per Phenotype scripting policy, ≤5-line shell glue is acceptable when a real-language
# replacement is a net loss. This is a one-shot `git config` invocation per checkout.
set -euo pipefail
git -C "$(git rev-parse --show-toplevel)" config core.hooksPath .githooks
echo "[install-hooks] core.hooksPath -> .githooks (canonical, staged-only secret scan)"
