#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Complete, production-only workspace coverage.
#
# Why this exists
# ---------------
# `cargo llvm-cov --workspace --summary-only` silently omits most of the
# workspace. It does not report over the test harnesses it just ran; it reports
# over the *non-test binaries* only. The command it issues is:
#
#   llvm-cov report -instr-profile=… \
#     -object target/llvm-cov-target/debug/agileplus \
#     -object target/llvm-cov-target/debug/agileplus-api \
#     -object target/llvm-cov-target/debug/agileplus-dashboard \
#     -object target/llvm-cov-target/debug/agileplus-grpc \
#     -object target/llvm-cov-target/debug/migrate \
#     -object target/llvm-cov-target/debug/seed_db \
#     -object target/llvm-cov-target/debug/seed_requirements \
#     -ignore-filename-regex '…'
#
# Only crates linked into those seven binaries can appear. Every other workspace
# member (agileplus-p2p, -nats, -telemetry, -proto, -github, -config, -graph,
# -sync, -import, -cache, -subcmds, the agileplus-agents/* crates, …) has real
# tests that run, real profile data on disk, and zero rows in the report. That
# is why the published workspace figure was ~48.7% on a 14-crate basis while
# ~25k lines of source (17% of the workspace) went unmeasured.
#
# The rows that *are* reported are production-only: cargo-llvm-cov passes
# non-test objects, so a file's own `#[cfg(test)]` module is not counted. E.g.
# crates/agileplus-events/src/bus.rs is 626 physical lines, 435 of them test
# code, and reports 79 lines. Reporting over the test harnesses instead gives
# 430 lines for the same file - a test-inclusive number that would credit test
# code as covered production code. That basis is not comparable and is never
# used here.
#
# What this script does
# ---------------------
#  1. Builds and runs the workspace suite under llvm-cov, writing profiles only
#     (`--no-report`), then leaves the merged profile at
#     `$CARGO_TARGET_DIR/AgilePlus.profdata`.
#  2. Reports over the *non-test* compilation units of every workspace member:
#     for each `debug/build/<crate>/<hash>/out` directory that contains a
#     `.rlib`, its `*.rcgu.o` coverage-mapped objects. Those are the same class
#     of object cargo-llvm-cov uses, verified to reproduce its per-file numbers
#     exactly (agileplus-events/src/bus.rs: 79 lines, 22 missed, 72.15% in both).
#  3. Chunks the object list so no single `llvm-cov report` invocation exceeds
#     ARG_MAX (~12k objects is ~1.6MB of arguments, and the modern macOS limit
#     is 1MB), then merges the chunk results per file by taking the larger line
#     count and the larger covered count. A file belongs to one crate, so the
#     merge is exact rather than a union that could double-count.
#  4. Applies the same `-ignore-filename-regex` cargo-llvm-cov uses, so the basis
#     is unchanged: test/example/bench sources, the registry, and the toolchain
#     stay out.
#  5. Builds subcmds with its real feature set: agileplus-subcmds declares
#     `default = []` and gates every module behind `#[cfg(feature = ...)]`, so
#     a default build compiles none of its ~2.2k production lines. The run uses
#     `--features agileplus-subcmds/full` (confirmed from its Cargo.toml; `full`
#     enables all five features and activates no dependencies).
#  6. Never publishes a partial number: a failed or truncated llvm-cov chunk
#     aborts the run instead of being swallowed by `|| true`, the target-path
#     part of -ignore-filename-regex is absolute (the old relative form never
#     matched), and COVERAGE_SKIP_BUILD=1 prints a prominent stale-profile
#     warning.
#  7. Ends with a completeness assertion: every production .rs file under the
#     workspace-owned roots must have a FILE_ROW in the merged report. Files
#     with no row are listed and the run fails (no percentage) unless they are
#     on the explicit, justified allowlist embedded below.
#
# Usage: scripts/coverage-complete.sh [--per-crate]
#   --per-crate   also print the per-crate table (default: totals + worst crates)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

target_dir="${CARGO_TARGET_DIR:-target/llvm-cov-target}"
work_dir="${JCODE_SCRATCH_DIR:-/tmp}/agileplus-coverage-complete"
mkdir -p "$work_dir"

toolchain_bin="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | awk '/host:/{print $2}')/bin"
llvm_cov="$toolchain_bin/llvm-cov"
[ -x "$llvm_cov" ] || { echo "missing $llvm_cov (install the llvm-tools component)" >&2; exit 1; }

# Identical to the regex cargo-llvm-cov composes for this workspace, plus the
# absolute form of the target-dir entry: llvm-cov matches against absolute (or
# repo-relative) source paths, so the old `^target/...`-only form never matched
# anything and the awk whitelist in coverage-merge.awk was doing all the real
# filtering. Both forms are kept; neither replaces the whitelist.
target_abs="$target_dir"
case "$target_abs" in /*) ;; *) target_abs="$repo_root/$target_abs" ;; esac
ignore_regex="$(printf '%s' \
    '/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/' \
    "|^${repo_root}(/.*)?/(tests|examples|benches)/" \
    "|^${target_abs//\//\\/}\$" \
    "|^${target_abs//\//\\/}/" \
    "|^${target_dir//\//\\/}\$" \
    "|^${target_dir//\//\\/}/" \
    '|^'"${HOME//\//\\/}"'/.cargo/(registry|git)/' \
    '|^'"${HOME//\//\\/}"'/.rustup/toolchains($|/)')"

profdata="$target_dir/AgilePlus.profdata"

if [ "${COVERAGE_SKIP_BUILD:-0}" = "1" ]; then
    # Reusing a profile must never be silent: the numbers below reflect the
    # state of the tree at the time the LAST full run happened, not now.
    echo "**********************************************************************" >&2
    echo "* WARNING: COVERAGE_SKIP_BUILD=1 - reusing existing profile data:" >&2
    echo "*   $profdata" >&2
    echo "* This profdata may be STALE. Any change to sources, features, or" >&2
    echo "* tests since the last full run is NOT reflected below. Only trust" >&2
    echo "* these numbers if nothing changed after that run." >&2
    echo "**********************************************************************" >&2
fi

if [ "${COVERAGE_SKIP_BUILD:-0}" != "1" ]; then
    echo "==> building and running the workspace suite"
    # --no-clean: without it cargo-llvm-cov wipes its target directory, and the
    # non-test compilation units this script reports over are exactly what
    # disappears. --no-fail-fast so one failing suite does not hide every other
    # member's profile data. This cargo-llvm-cov version rejects
    # `--no-report --no-clean` together, so the command runs its own (ignored)
    # seven-object report step on the way through; the merged profile it leaves
    # behind is what we report over below.
    #
    # --features agileplus-subcmds/full: subcmds' Cargo.toml declares
    # `default = []`, and lib.rs gates every module (audit, dashboard, events,
    # registry, sync) on #[cfg(feature = "...")], so the default build compiles
    # essentially none of the crate's ~2.2k production lines and it contributes
    # zero rows to the report. `full = [dashboard, events, sync, audit, registry]`
    # is confirmed from crates/agileplus-subcmds/Cargo.toml to cover all five
    # features, and all five are empty feature lists (no optional dependencies
    # activated), so enabling them adds only cfg-gated code that compiles
    # against the dependencies the crate already declares. (Compile success is
    # UNVERIFIED here - no build was run for this change.)
    cargo llvm-cov --workspace --features agileplus-subcmds/full --no-clean --no-fail-fast
fi

[ -s "$profdata" ] || { echo "no merged profile at $profdata" >&2; exit 1; }
echo "==> profile: $profdata ($(du -h "$profdata" | cut -f1))"

echo "==> collecting compilation units"
# Lib objects: only out directories that also hold a .rlib are the non-test
# lib build; the ones without an rlib are test harnesses. Bins are non-test by
# construction and are what cargo-llvm-cov already reports, so keeping them
# preserves comparability with the published figure.
#
# KNOWN LIMITATION (verified 2026-09-19): for some members (agileplus-nats,
# -events, ...) neither lib compilation's objects carry profile data - the
# test harness links a different compilation of the lib, and the rlib objects
# are keyed to a compilation that never executed. Reporting those objects
# against any profile yields 0.00%. The ONLY source of counts for such a crate
# is its test-harness binary object, whose rows are test-inclusive (a file's
# own #[cfg(test)] module is counted). Those crates are excluded from the
# production-only basis and listed separately below.
: > "$work_dir/objects.txt"
# One traversal for the rlibs, rather than an `ls` probe per out directory:
# the build directory holds an entry for every third-party dependency too, and
# probing them all dominates the runtime of this script.
find "$target_dir/debug/build" -name '*.rlib' -print0 2>/dev/null \
    | while IFS= read -r -d '' rlib; do dirname "$rlib"; done \
    | sort -u \
    | while IFS= read -r d; do
        find "$d" -maxdepth 1 -name '*.rcgu.o' -print0 2>/dev/null
    done >> "$work_dir/objects.txt"
find "$target_dir/debug" -maxdepth 1 -type f -perm -u+x -print0 2>/dev/null >> "$work_dir/objects.txt"

if [ ! -s "$work_dir/objects.txt" ]; then
    echo "no non-test compilation units found under $target_dir" >&2
    exit 1
fi

tr '\0' '\n' < "$work_dir/objects.txt" | grep -v '^$' | sort -u > "$work_dir/object-list.txt"
object_total="$(wc -l < "$work_dir/object-list.txt" | tr -d ' ')"
echo "    $object_total objects"

echo "==> reporting"
rm -f "$work_dir"/chunk-*
: > "$work_dir/chunks.txt"
: > "$work_dir/chunk-errors.log"
# 1200 objects per invocation keeps every argument vector near 160 KB, well
# inside the 1 MB ARG_MAX of this platform.
split -l 1200 "$work_dir/object-list.txt" "$work_dir/chunk-"
chunks=0
# `chunk-??` matches only split's two-letter chunk parts (its default suffix
# length). A bare `chunk-*` also matches the chunk-errors.log scratch file
# created above and feeds it to llvm-cov as a bogus object list.
for part in "$work_dir"/chunk-??; do
    args=()
    while IFS= read -r obj; do
        [ -n "$obj" ] || continue
        args+=("--object=$obj")
    done < "$part"
    [ ${#args[@]} -gt 0 ] || continue
    chunk_out="$work_dir/chunk-out.$chunks"
    # A failed chunk previously vanished via `|| true` with stderr discarded,
    # silently lowering the totals while a valid-looking percentage still
    # printed. Failure is now fatal: no partial number is ever published.
    if ! "$llvm_cov" report -use-color=0 -instr-profile="$profdata" \
        -ignore-filename-regex="$ignore_regex" "${args[@]}" \
        > "$chunk_out" 2>> "$work_dir/chunk-errors.log"; then
        echo "FAILED: llvm-cov report chunk $chunks ($(basename "$part")) - stderr follows" >&2
        sed 's/^/    /' "$work_dir/chunk-errors.log" >&2
        echo "ABORTING: a partial report must never be published as a percentage." >&2
        exit 1
    fi
    # A truncated report (killed invocation, broken pipe) has rows but no
    # TOTAL line; accepting it would silently lower the totals too.
    if ! grep -q '^TOTAL' "$chunk_out"; then
        echo "FAILED: llvm-cov report chunk $chunks ($(basename "$part")) produced no TOTAL row" >&2
        echo "       (empty or truncated output; see $work_dir/chunk-errors.log)" >&2
        echo "ABORTING: a partial report must never be published as a percentage." >&2
        exit 1
    fi
    cat "$chunk_out" >> "$work_dir/chunks.txt"
    rm -f "$chunk_out"
    chunks=$((chunks + 1))
done
echo "    $chunks report invocations"

if [ ! -s "$work_dir/chunks.txt" ]; then
    echo "every report invocation failed; nothing to summarise" >&2
    exit 1
fi

# Merge per file: a file belongs to one crate, so the largest line count is its
# own and the largest covered count is its coverage. Summing instead would
# double-count a file that appears in two compilation variants.
#
# llvm-cov prints the longest common prefix of the reported paths stripped, so
# the workspace root can arrive truncated to any depth (`CodeProjects/...`,
# `AgilePlus/...`, or an absolute path depending on the object mix). Anchor on
# the repository directory name rather than on an absolute prefix.
awk -f "$repo_root/scripts/coverage-merge.awk" -v ancestor="$(basename "$repo_root")" \
    "$work_dir/chunks.txt" > "$work_dir/merged.txt"
crate_table="$(awk '/^CRATE_TABLE$/{f=1;next} /^TOTAL$/{f=0} f' "$work_dir/merged.txt" | sort -k5 -n)"
totals="$(awk '/^WORKSPACE_TOTAL/{print $2, $3, $4}' "$work_dir/merged.txt")"
read -r file_count all_lines all_covered <<<"$totals"

if [ -z "${all_lines:-}" ] || ! [ "$all_lines" -gt 0 ] 2>/dev/null; then
    echo "merge produced no workspace totals; refusing to print a percentage" >&2
    exit 1
fi

# ── Completeness assertion (audit finding B) ─────────────────────────────────
# The report is only trustworthy if EVERY production file has a row. The object
# collection samples disk state at one instant (`find debug/build -name
# '*.rlib'`), so a crate whose rlib is absent or whose features are off would
# otherwise vanish silently; this check makes any such gap fatal.
#
# Expected set: every .rs file under the workspace-owned production roots that
# is not inside a tests/ examples/ or benches/ directory - the same basis the
# coverage-merge.awk whitelist enforces on the measured side. `libs/` is
# workspace-owned per the whitelist but no Cargo.toml references it, so nothing
# builds it; add it to this find if that ever changes.
#
# ALLOWLIST: only files that legitimately cannot carry a production coverage
# row, each with an inline reason. If the first run lists other files (a module
# declared only under #[cfg(test)], a platform-gated file, a .rs fixture that
# is never `mod`-declared), verify each one before adding it HERE - never pad
# the list just to make the check pass.
allowlist_file="$work_dir/completeness-allowlist.txt"
cat > "$allowlist_file" <<'ALLOWLIST'
agileplus-agents/crates/agileplus-agent-service/build.rs  # build-script executable; never a reported compilation unit
crates/agileplus-grpc/build.rs                             # build-script executable; never a reported compilation unit
crates/agileplus-proto/build.rs                            # build-script executable; never a reported compilation unit
desktop/src-tauri/build.rs                                 # build-script executable; never a reported compilation unit
crates/agileplus-integration-tests/build.rs                # build-script executable; never a reported compilation unit
ALLOWLIST
awk '{print $1}' "$allowlist_file" > "$work_dir/allowlist-paths.txt"

find crates agileplus-agents desktop/src-tauri libs -type f -name '*.rs' 2>/dev/null \
    | awk -F/ '{
        skip = 0
        for (i = 1; i <= NF; i++)
            if ($i == "tests" || $i == "examples" || $i == "benches") { skip = 1; break }
        if (!skip) print
      }' | LC_ALL=C sort -u > "$work_dir/expected-files.txt"
awk '$1 == "FILE_ROW" { print $2 }' "$work_dir/merged.txt" \
    | LC_ALL=C sort -u > "$work_dir/measured-files.txt"
missing="$work_dir/missing-files.txt"
LC_ALL=C comm -23 "$work_dir/expected-files.txt" "$work_dir/measured-files.txt" > "$missing"
awk 'NR == FNR { drop[$1]; next } !($0 in drop)' \
    "$work_dir/allowlist-paths.txt" "$missing" > "$missing.filtered"
mv "$missing.filtered" "$missing"
extra="$work_dir/extra-files.txt"
LC_ALL=C comm -13 "$work_dir/expected-files.txt" "$work_dir/measured-files.txt" > "$extra"
expected_count="$(wc -l < "$work_dir/expected-files.txt" | tr -d ' ')"
measured_count="$(wc -l < "$work_dir/measured-files.txt" | tr -d ' ')"
missing_count="$(wc -l < "$missing" | tr -d ' ')"
extra_count="$(wc -l < "$extra" | tr -d ' ')"
allowlisted_count="$(wc -l < "$work_dir/allowlist-paths.txt" | tr -d ' ')"
echo "==> completeness: $expected_count production files expected ($allowlisted_count allowlisted), $measured_count rows produced, $missing_count missing, $extra_count outside the expected set"
if [ "$missing_count" -ne 0 ]; then
    echo "FAILED: production files with NO coverage row (the denominator is incomplete):" >&2
    sed 's/^/    /' "$missing" >&2
    echo "No percentage will be printed. Likely causes: a crate not built (features)," >&2
    echo "a failed report chunk, or a file that is cfg'd out. Fix the cause, or -" >&2
    echo "only after verifying a file genuinely cannot be instrumented - add it to" >&2
    echo "the allowlist in scripts/coverage-complete.sh with its reason." >&2
    exit 1
fi

echo
echo "COVERAGE — full workspace, production lines only"
echo "============================================================"
if [ "${1:-}" = "--per-crate" ]; then
    printf '%s\n' "$crate_table"
    echo "============================================================"
else
    printf '%s\n' "$crate_table" | head -8
    echo "    … (pass --per-crate for the full table)"
    echo "============================================================"
fi
awk -v files="$file_count" -v l="$all_lines" -v c="$all_covered" '
BEGIN { printf "%d files, %d lines, %d covered = %.2f%%\n", files, l, c, c * 100 / l }'

echo
echo "raw chunk reports: $work_dir/chunks.txt"
echo "merged per-file:   $work_dir/merged.txt"

# ── Test-inclusive supplement ─────────────────────────────────────────────────
#
# For each workspace member, report its test-harness binary object. If the
# production-only basis above carries rows for that crate, the supplement adds
# nothing (the merge takes the larger covered count and lib rows already
# dominate). If the basis has no rows for the crate (the lib objects carry no
# data), the supplement is the only measurement available - and it is
# TEST-INCLUSIVE: a file's own #[cfg(test)] module counts as covered lines.
# Those crates are printed in their own section and must not be summed into
# the production-only figure.

echo "==> collecting test-harness binary objects"
: > "$work_dir/harness-chunks.txt"
# One invocation per member's out directory that holds a harness binary but no
# rlib (the lib-only dirs are already in the production basis).
find "$target_dir/debug/build" -mindepth 1 -maxdepth 1 -type d -print0 2>/dev/null \
    | while IFS= read -r -d '' crate_dir; do
        # Any out dir under this crate that has an executable but no rlib.
        for out in "$crate_dir"/*/out; do
            [ -d "$out" ] || continue
            has_rlib=$(find "$out" -maxdepth 1 -name '*.rlib' | head -1)
            [ -n "$has_rlib" ] && continue
            find "$out" -maxdepth 1 -type f -perm -u+x ! -name '*.so' -print0 2>/dev/null
        done
    done > "$work_dir/harness-objects.txt"
harness_total="$(tr '\0' '\n' < "$work_dir/harness-objects.txt" | grep -v '^$' | sort -u | wc -l | tr -d ' ')"
echo "    $harness_total harness objects"
if [ "$harness_total" != "0" ]; then
    tr '\0' '\n' < "$work_dir/harness-objects.txt" | grep -v '^$' | sort -u > "$work_dir/harness-object-list.txt"
    # Drop stale chunk parts from a previous run; leftover parts would be
    # re-processed below against objects that may no longer exist (and would
    # now abort the run instead of being harmlessly re-merged). The glob does
    # not match harness-chunks.txt (no hyphen after "chunk" in that name).
    rm -f "$work_dir"/harness-chunk-*
    split -l 300 "$work_dir/harness-object-list.txt" "$work_dir/harness-chunk-"
    for part in "$work_dir"/harness-chunk-*; do
        args=()
        while IFS= read -r obj; do
            args+=(-object "$obj")
        done < "$part"
        # Same rule as the production chunks: a failed or truncated chunk is
        # fatal, never swallowed into a partial supplement.
        harness_out="$work_dir/harness-chunk-out"
        if ! "$llvm_cov" report -use-color=0 -instr-profile="$profdata" -ignore-filename-regex="$ignore_regex" "${args[@]}" > "$harness_out" 2>> "$work_dir/chunk-errors.log"; then
            echo "FAILED: llvm-cov harness chunk ($(basename "$part")) - stderr follows" >&2
            sed 's/^/    /' "$work_dir/chunk-errors.log" >&2
            echo "ABORTING: a partial supplement must never be published." >&2
            exit 1
        fi
        if ! grep -q '^TOTAL' "$harness_out"; then
            echo "FAILED: harness chunk ($(basename "$part")) produced no TOTAL row (truncated or empty output)" >&2
            echo "ABORTING: a partial supplement must never be published." >&2
            exit 1
        fi
        cat "$harness_out" >> "$work_dir/harness-chunks.txt"
    done
    awk -f "$repo_root/scripts/coverage-merge.awk" -v ancestor="$(basename "$repo_root")" \
        "$work_dir/harness-chunks.txt" > "$work_dir/harness-merged.txt"
    echo
    echo "TEST-INCLUSIVE SUPPLEMENT — crates with no production-basis rows"
    echo "(their #[cfg(test)] modules count as covered lines; do not sum into the figure above)"
    echo "============================================================"
    awk '/^CRATE_TABLE$/{f=1;next} /^TOTAL$/{f=0} f' "$work_dir/harness-merged.txt" | sort -k5 -n
    echo "============================================================"
fi
