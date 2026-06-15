#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-.}"
status=0

target_files() {
  if [ -n "${CHANGED_FILES:-}" ]; then
    printf '%s\n' "$CHANGED_FILES"
  elif [ -n "${CHANGED_FILES_FILE:-}" ] && [ -f "$CHANGED_FILES_FILE" ]; then
    cat "$CHANGED_FILES_FILE"
  else
    find "$ROOT" \
      -path '*/.git' -prune -o \
      -path '*/target' -prune -o \
      -path '*/node_modules' -prune -o \
      -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' -o -name '*.js' -o -name '*.py' \) \
      -print
  fi
}

files="$(
  target_files |
    sed '/^[[:space:]]*$/d' |
    grep --color=never -E '\.(rs|ts|tsx|js|py)$' |
    while IFS= read -r file; do
      [ -f "$file" ] && printf '%s\n' "$file"
      [ -f "$ROOT/$file" ] && printf '%s\n' "$ROOT/$file"
    done |
    sort -u
)"

if [ -z "$files" ]; then
  echo "No source files to scan for governance anti-patterns."
  exit 0
fi

scan() {
  local label="$1"
  local pattern="$2"
  local matches

  matches="$(printf '%s\n' "$files" | xargs grep --color=never -InE "$pattern" || true)"
  if [ -n "$matches" ]; then
    printf '%s\n' "$matches"
    echo "anti-pattern detected: $label" >&2
    status=1
  fi
}

scan "Rust unwrap/expect/panic" '(^|[^[:alnum:]_])(unwrap|expect|panic!)\s*\('
scan "SQL string concatenation" 'SELECT .*\+|INSERT .*\+|UPDATE .*\+|DELETE .*\+|format!\s*\([^\n]*(SELECT|INSERT|UPDATE|DELETE)'

if [ "$status" -ne 0 ]; then
  echo "Governance anti-pattern gate failed." >&2
  exit "$status"
fi

echo "Governance anti-pattern gate passed."
