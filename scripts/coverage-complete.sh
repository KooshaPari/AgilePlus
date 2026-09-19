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

# Identical to the regex cargo-llvm-cov composes for this workspace.
ignore_regex="$(printf '%s' \
    '/rustc/([0-9a-f]+|[0-9]+\.[0-9]+\.[0-9]+)/' \
    "|^${repo_root}(/.*)?/(tests|examples|benches)/" \
    "|^${target_dir//\//\\/}\$" \
    "|^${target_dir//\//\\/}/" \
    '|^'"${HOME//\//\\/}"'/.cargo/(registry|git)/' \
    '|^'"${HOME//\//\\/}"'/.rustup/toolchains($|/)')"

profdata="$target_dir/AgilePlus.profdata"

if [ "${COVERAGE_SKIP_BUILD:-0}" != "1" ]; then
    echo "==> building and running the workspace suite"
    # --no-clean: without it cargo-llvm-cov wipes its target directory, and the
    # non-test compilation units this script reports over are exactly what
    # disappears. --no-fail-fast so one failing suite does not hide every other
    # member's profile data. This cargo-llvm-cov version rejects
    # `--no-report --no-clean` together, so the command runs its own (ignored)
    # seven-object report step on the way through; the merged profile it leaves
    # behind is what we report over below.
    cargo llvm-cov --workspace --no-clean --no-fail-fast
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
: > "$work_dir/chunks.txt"
rm -f "$work_dir"/chunk-*
# 1200 objects per invocation keeps every argument vector near 160 KB, well
# inside the 1 MB ARG_MAX of this platform.
split -l 1200 "$work_dir/object-list.txt" "$work_dir/chunk-"
chunks=0
for part in "$work_dir"/chunk-*; do
    args=()
    while IFS= read -r obj; do
        [ -n "$obj" ] || continue
        args+=("--object=$obj")
    done < "$part"
    [ ${#args[@]} -gt 0 ] || continue
    "$llvm_cov" report -use-color=0 -instr-profile="$profdata" \
        -ignore-filename-regex="$ignore_regex" "${args[@]}" \
        >> "$work_dir/chunks.txt" 2>/dev/null || true
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
    split -l 300 "$work_dir/harness-object-list.txt" "$work_dir/harness-chunk-"
    for part in "$work_dir"/harness-chunk-*; do
        args=()
        while IFS= read -r obj; do
            args+=(-object "$obj")
        done < "$part"
        "$llvm_cov" report -use-color=0 -instr-profile="$profdata" -ignore-filename-regex="$ignore_regex" "${args[@]}" >> "$work_dir/harness-chunks.txt" 2>/dev/null || true
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
