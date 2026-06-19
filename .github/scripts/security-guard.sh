#!/usr/bin/env bash
# security-guard.sh — local pre-commit/pre-push security gate
# Implements the three-stage gatekeeper security stage:
#   1. Secret scanning  (ggshield / gitleaks)
#   2. Dependency audit  (cargo-deny)
#   3. SAST coverage    (semgrep)

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PASS=0
FAIL=0

run_check() {
    local label="$1"; shift
    echo "[security-guard] Running: $label"
    if "$@"; then
        echo "[security-guard]   PASS: $label"
        PASS=$((PASS+1))
    else
        echo "[security-guard]   FAIL: $label" >&2
        FAIL=$((FAIL+1))
    fi
}

# ── Stage 1: Secret scanning ────────────────────────────────────────────

if command -v ggshield >/dev/null 2>&1; then
    GGSHIELD=(ggshield)
elif command -v uvx >/dev/null 2>&1; then
    GGSHIELD=(uvx ggshield)
elif command -v uv >/dev/null 2>&1; then
    GGSHIELD=(uv tool run ggshield)
else
    echo "[security-guard]   WARN: ggshield not installed — skipping secret scan" >&2
    echo "[security-guard]   Install: pipx install ggshield or uv tool install ggshield" >&2
    GGSHIELD=()
fi

if [ -n "${GGSHIELD[*]:-}" ]; then
    run_check "ggshield secret scan" \
        "${GGSHIELD[@]}" secret scan pre-commit || true
fi

if command -v gitleaks >/dev/null 2>&1; then
    run_check "gitleaks detect" \
        gitleaks detect --source . --config gitleaks.toml --no-color 2>/dev/null || true
fi

# ── Stage 2: Dependency audit ─────────────────────────────────────────

if [ -f rust/Cargo.toml ] && command -v cargo >/dev/null 2>&1; then
    run_check "cargo-deny check" \
        cargo deny check 2>/dev/null || true
fi

# ── Stage 3: SAST coverage ─────────────────────────────────────────────

if command -v semgrep >/dev/null 2>&1; then
    run_check "semgrep SAST" \
        semgrep --config .semgrep.yml --error --no-color 2>/dev/null || true
elif [ -f .semgrep.yml ]; then
    echo "[security-guard]   WARN: semgrep not installed — install with: pip install semgrep" >&2
fi

# ── Summary ────────────────────────────────────────────────────────────

if command -v codespell >/dev/null 2>&1; then
    changed_files=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || true)
    [ -z "${changed_files}" ] && changed_files=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || true)
    if [ -n "${changed_files}" ]; then
        echo "[security-guard] Running codespell fast pass"
        echo "${changed_files}" \
            | grep -E '\.(md|txt|py|ts|tsx|js|go|rs|kt|java|yaml|yml)$' \
            | xargs -r codespell -q 2 -L "hte,teh" || true
    fi
fi

echo "[security-guard] Security gate complete: $PASS passed, $FAIL failed"
[ "$FAIL" -gt 0 ] && exit 1
exit 0
