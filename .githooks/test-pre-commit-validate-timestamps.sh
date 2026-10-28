#!/usr/bin/env bash
# .githooks/test-pre-commit-validate-timestamps
#
# Regression suite for the RFC3339 UTC timestamp pre-commit guard
# (.githooks/pre-commit-validate-timestamps) and for the two-step
# .githooks/pre-commit chain.
#
# Run:  .githooks/test-pre-commit-validate-timestamps
#
# Every fixture (throwaway git repositories, SQLite databases, temporary
# files) is created under $JCODE_SCRATCH_DIR. Nothing is written inside the
# repository and nothing is written to /tmp.
#
# Covered defects (from the PR #1161 review):
#   1. the secret scan used to short-circuit the timestamp check
#   2. the hook scanned the working tree instead of the staged blob
#   3. the row query required an `id` column
#   4. sqlite3 query errors were swallowed into "no rows"
#   5. a T-only value filter could hide corrupt rows
#   6. a missing sqlite3 silently skipped the check
set -uo pipefail

hook_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ts_hook="$hook_dir/pre-commit-validate-timestamps"
pre_commit_hook="$hook_dir/pre-commit"
validator="$hook_dir/rfc3339-utc-validate.py"

scratch_base="${JCODE_SCRATCH_DIR:-${TMPDIR:-/tmp}}"
work="$(mktemp -d "${scratch_base%/}/ts-hook-test.XXXXXX")"
export TMPDIR="$work/tmp"
mkdir -p "$TMPDIR"

cleanup() {
  rm -rf "$work"
}
trap cleanup EXIT

passed=0
failed=0
out=""
rc=0

report_ok() {
  printf 'ok   %s\n' "$1"
  passed=$((passed + 1))
}

report_fail() {
  printf 'FAIL %s\n' "$1"
  failed=$((failed + 1))
}

check_rc() { # check_rc <name> <expected-rc> <actual-rc>
  if [ "$2" = "$3" ]; then
    report_ok "$1 (exit $3)"
  else
    report_fail "$1 (expected exit $2, got $3)"
    printf '     output: %s\n' "${out:-<none>}"
  fi
}

check_nonzero() { # check_nonzero <name> <actual-rc>
  if [ "$2" -ne 0 ]; then
    report_ok "$1 (exit $2, non-zero as required)"
  else
    report_fail "$1 (expected a non-zero exit, got 0)"
    printf '     output: %s\n' "${out:-<none>}"
  fi
}

check_contains() { # check_contains <name> <needle>
  case "${out:-}" in
    *"$2"*) report_ok "$1 (output contains '$2')" ;;
    *)
      report_fail "$1 (output does not contain '$2')"
      printf '     output: %s\n' "${out:-<none>}"
      ;;
  esac
}

# ---------------------------------------------------------------- fixtures
repo_seq=0

# new_repo — a throwaway git repository carrying copies of the hooks.
# Each case gets its own repository so that staged fixtures never leak
# from one case into the next.
new_repo() {
  repo_seq=$((repo_seq + 1))
  local d="$work/repo-$repo_seq"
  mkdir -p "$d/.githooks" "$d/.agileplus"
  cp "$ts_hook" "$pre_commit_hook" "$validator" "$d/.githooks/"
  chmod +x "$d/.githooks/pre-commit" "$d/.githooks/pre-commit-validate-timestamps"
  git -C "$d" init -q -b main
  git -C "$d" config user.email "hook-test@example.invalid"
  git -C "$d" config user.name "hook test"
  git -C "$d" config core.hooksPath .githooks
  printf 'seed\n' >"$d/README.md"
  git -C "$d" add README.md
  git -C "$d" commit -q --no-verify -m "seed"
  REPO="$d"
}

# db_with_rows <path> <sql>
db_with_rows() {
  local path="$1"
  rm -f "$path"
  sqlite3 "$path" "$2"
}

# run_ts [ENV=VALUE ...] — run the timestamp hook inside $REPO.
run_ts() {
  out="$(cd "$REPO" && env "$@" "$ts_hook" 2>&1)"
  rc=$?
}

# run_pre_commit [ENV=VALUE ...]
run_pre_commit() {
  out="$(cd "$REPO" && env "$@" "$pre_commit_hook" 2>&1)"
  rc=$?
}

NO_ID_SCHEMA="CREATE TABLE channel_iterations (
  channel_id TEXT NOT NULL,
  iteration INTEGER NOT NULL DEFAULT 1,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (channel_id)
);"

WITH_ID_SCHEMA="CREATE TABLE work_items (
  id TEXT PRIMARY KEY,
  updated_at TEXT
);"

echo "== fixture setup =="

echo
echo "== 1. table with updated_at but no id column =="
new_repo
db_with_rows "$REPO/.agileplus/noid.db" "$NO_ID_SCHEMA
INSERT INTO channel_iterations(channel_id, iteration, updated_at)
  VALUES ('c1', 1, '2026-10-18T09:46:39Z');
INSERT INTO channel_iterations(channel_id, iteration, updated_at)
  VALUES ('c2', 2, 'not-a-timestamp');"
git -C "$REPO" add -f .agileplus/noid.db
run_ts
check_rc "no-id table with a corrupt row is rejected" 1 "$rc"
check_contains "the strict validator reported the corrupt row" "not-a-timestamp"

echo
echo "== 2. corrupt values that merely contain a 'T' =="
new_repo
db_with_rows "$REPO/.agileplus/tslike.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', 'definitelyTnot-rfc3339');
INSERT INTO work_items(id, updated_at) VALUES ('b', '2026-10-18Tgarbage');"
git -C "$REPO" add -f .agileplus/tslike.db
run_ts
check_rc "T-containing corrupt values are rejected" 1 "$rc"
check_contains "validator reached the first T-value" "definitelyTnot-rfc3339"
check_contains "validator reached the second T-value" "2026-10-18Tgarbage"

echo
echo "== 2b. the validator itself rejects those values =="
for bad in "definitelyTnot-rfc3339" "2026-10-18Tgarbage"; do
  printf 'updated_at|%s\n' "$bad" | python3 "$validator" >/dev/null 2>&1
  vrc=$?
  if [ "$vrc" -eq 1 ]; then
    report_ok "validator rejects '$bad' (exit 1)"
  else
    report_fail "validator rejects '$bad' (expected exit 1, got $vrc)"
  fi
done

echo
echo "== 3. all-valid UTC timestamps are accepted =="
new_repo
db_with_rows "$REPO/.agileplus/valid.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', '2026-10-18T09:46:39Z');
INSERT INTO work_items(id, updated_at) VALUES ('b', '2026-10-18T09:46:39.123456789+00:00');"
git -C "$REPO" add -f .agileplus/valid.db
run_ts
check_rc "valid database is accepted" 0 "$rc"

echo
echo "== 4. the staged blob is validated, not the working tree =="
# 4a: valid blob staged, working tree corrupted afterwards -> accepted.
new_repo
db_with_rows "$REPO/.agileplus/blob.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', '2026-10-18T09:46:39Z');"
git -C "$REPO" add -f .agileplus/blob.db
db_with_rows "$REPO/.agileplus/blob.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', 'corrupted-in-worktree');"
run_ts
check_rc "valid staged blob + corrupted worktree copy is accepted" 0 "$rc"

# 4b: corrupt blob staged, working tree repaired afterwards -> rejected.
new_repo
db_with_rows "$REPO/.agileplus/blob2.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', 'corrupt-in-index');"
git -C "$REPO" add -f .agileplus/blob2.db
db_with_rows "$REPO/.agileplus/blob2.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', '2026-10-18T09:46:39Z');"
run_ts
check_rc "corrupt staged blob + valid worktree copy is rejected" 1 "$rc"
check_contains "the corrupt staged value is reported" "corrupt-in-index"

# 4c: a staged file deleted from the working tree is still validated.
new_repo
db_with_rows "$REPO/.agileplus/blob3.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', 'corrupt-and-removed');"
git -C "$REPO" add -f .agileplus/blob3.db
rm -f "$REPO/.agileplus/blob3.db"
run_ts
check_rc "staged blob removed from the working tree is still rejected" 1 "$rc"

echo
echo "== 5. sqlite3 missing from PATH fails closed =="
new_repo
fakebin="$work/fakebin"
mkdir -p "$fakebin"
for tool in bash git grep python3 mktemp rm sh; do
  src="$(command -v "$tool")"
  if [ -z "$src" ]; then
    report_fail "fake PATH setup: '$tool' not found on this machine"
    break
  fi
  ln -s "$src" "$fakebin/$tool"
done
db_with_rows "$REPO/.agileplus/nosqlite.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', '2026-10-18T09:46:39Z');"
git -C "$REPO" add -f .agileplus/nosqlite.db
out="$(cd "$REPO" && PATH="$fakebin" "$ts_hook" 2>&1)"
rc=$?
check_nonzero "missing sqlite3 rejects the commit" "$rc"
check_contains "missing sqlite3 explains the install step" "sqlite3"

echo
echo "== 6. PRE_COMMIT_SKIP_SECRETS=1 does not disable the timestamp check =="
new_repo
db_with_rows "$REPO/.agileplus/skipped-secrets.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', 'corrupt-despite-skip');"
git -C "$REPO" add -f .agileplus/skipped-secrets.db
run_pre_commit PRE_COMMIT_SKIP_SECRETS=1
check_nonzero "pre-commit rejects a corrupt db with the secret scan skipped" "$rc"
check_contains "the timestamp check still ran" "corrupt-despite-skip"

echo
echo "== 6b. PRE_COMMIT_SKIP_TIMESTAMPS=1 skips only the timestamp check =="
run_pre_commit PRE_COMMIT_SKIP_SECRETS=1 PRE_COMMIT_SKIP_TIMESTAMPS=1
check_rc "the timestamp check can be skipped on its own" 0 "$rc"

echo
echo "== 6c. HOOKS_SKIP=1 remains the global bypass =="
run_pre_commit HOOKS_SKIP=1
check_rc "HOOKS_SKIP=1 skips everything" 0 "$rc"

echo
echo "== 7. all-NULL updated_at values are accepted =="
new_repo
db_with_rows "$REPO/.agileplus/nulls.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', NULL);
INSERT INTO work_items(id, updated_at) VALUES ('b', NULL);"
git -C "$REPO" add -f .agileplus/nulls.db
run_ts
check_rc "all-NULL updated_at database is accepted" 0 "$rc"

echo
echo "== 8. corrupt / unreadable database files fail closed =="
new_repo
printf 'this is definitely not a sqlite database\n' >"$REPO/.agileplus/corrupt.db"
git -C "$REPO" add -f .agileplus/corrupt.db
run_ts
check_nonzero "corrupt database file is rejected" "$rc"

new_repo
db_with_rows "$REPO/.agileplus/truncated.db" "$WITH_ID_SCHEMA
INSERT INTO work_items(id, updated_at) VALUES ('a', '2026-10-18T09:46:39Z');"
# Overwrite the SQLite header with zeros so the file is no longer a valid
# database (an empty file would be a perfectly valid empty database).
dd if=/dev/zero of="$REPO/.agileplus/truncated.db" bs=100 count=1 \
  conv=notrunc 2>/dev/null
git -C "$REPO" add -f .agileplus/truncated.db
run_ts
check_nonzero "truncated database file is rejected" "$rc"

echo
echo "== 9. a table without an updated_at column is ignored =="
new_repo
db_with_rows "$REPO/.agileplus/noupdated.db" "CREATE TABLE plain_rows (label TEXT);
INSERT INTO plain_rows(label) VALUES ('x');"
git -C "$REPO" add -f .agileplus/noupdated.db
run_ts
check_rc "database without updated_at columns is accepted" 0 "$rc"

echo
echo "== summary =="
printf 'passed: %s\nfailed: %s\n' "$passed" "$failed"
if [ "$failed" -ne 0 ]; then
  exit 1
fi
exit 0
