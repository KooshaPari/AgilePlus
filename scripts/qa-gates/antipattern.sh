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

  # Scan files for the pattern, excluding lines inside test modules/functions.
  # We exclude: files under /tests/ directories and any line that the awk
  # test-block tracker marks as being inside a #[cfg(test)] mod block.
  matches=""
  while IFS= read -r file; do
    [ -f "$file" ] || continue
    # Skip integration-test files
    echo "$file" | grep -q '/tests/' && continue
    # Use awk to skip lines inside #[cfg(test)] mod blocks
    result="$(awk -v pat="$pattern" -v fname="$file" '
      /^#\[cfg\(test\)\]/ { skip_next=1; next }
      skip_next && /^[[:space:]]*mod / { in_test_mod=1; depth=0; skip_next=0 }
      skip_next { skip_next=0 }
      in_test_mod {
        for(i=1;i<=length($0);i++){
          c=substr($0,i,1)
          if(c=="{") depth++
          else if(c=="}") { depth--; if(depth<=0){ in_test_mod=0; next } }
        }
        next
      }
      { if($0 ~ pat) printf "%s:%d:%s\n", fname, NR, $0 }
    ' "$file" 2>/dev/null || true)"
    [ -n "$result" ] && matches="${matches}${result}"$'\n'
  done < <(printf '%s\n' "$files")

  matches="${matches%$'\n'}"
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
