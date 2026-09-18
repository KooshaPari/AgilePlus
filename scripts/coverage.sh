#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Workspace coverage that does not lose the agileplus-cli crate.
#
# Why this exists
# ---------------
# `cargo llvm-cov --workspace --summary-only` reports 46 of the 55
# agileplus-cli source files at 0.00%, which makes the workspace look far
# less covered than it is. The data is not missing: the cli library's
# compilation unit records *absolute* source paths in its coverage profile,
# while every other unit records paths relative to the workspace. The
# cargo-llvm-cov report emits relative rows, so the cli files appear twice -
# once absolute and covered, once relative and empty - and only the empty
# relative rows survive into the summary.
#
# Evidence: running the instrumented cli test binary directly and reporting
# it by hand shows commands/module/tag.rs at 89.7% lines and cli overall at
# 74.2% (14,348 lines), against the 14.8% the cargo-llvm-cov summary claims.
#
# Reproduce the cli half on its own:
#   bin=target/llvm-cov-target/debug/build/agileplus-cli/*/out/agileplus_cli-*
#   LLVM_PROFILE_FILE=/tmp/cli.profraw $bin
#   "$toolchain_bin/llvm-profdata" merge -sparse /tmp/cli.profraw -o /tmp/cli.profdata
#   "$toolchain_bin/llvm-cov" report $bin -instr-profile=/tmp/cli.profdata | grep tag.rs
# The report's paths are absolute for this crate, which is exactly why the
# relative-path cargo-llvm-cov rows come out empty.
#
# What this script does
# ---------------------
#  1. Runs the workspace suite under llvm-cov as usual.
#  2. Additionally runs each instrumented agileplus-cli test binary by hand
#     with an explicit LLVM_PROFILE_FILE, merges those profiles, and reports
#     them, so the cli crate is measured from its own profiles.
#  3. Prints both the raw summary and a corrected workspace figure that
#     replaces the cli rows with the hand-measured ones.
#
# Usage: scripts/coverage.sh [--corrected-only]

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

target_dir="${CARGO_TARGET_DIR:-target/llvm-cov-target}"
work_dir="${JCODE_SCRATCH_DIR:-/tmp}/agileplus-coverage"
mkdir -p "$work_dir"

toolchain_bin="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | awk '/host:/{print $2}')/bin"
llvm_cov="$toolchain_bin/llvm-cov"
llvm_profdata="$toolchain_bin/llvm-profdata"
for tool in "$llvm_cov" "$llvm_profdata"; do
    [ -x "$tool" ] || { echo "missing $tool (install the rustfilt/llvm-tools component)" >&2; exit 1; }
done

echo "==> running the workspace suite under llvm-cov"
cargo llvm-cov --workspace --summary-only > "$work_dir/workspace.txt" 2>&1 || true
grep -E '^TOTAL' "$work_dir/workspace.txt" | tail -1 || echo "workspace summary unavailable"

echo "==> hand-measuring agileplus-cli from its own test binaries"
rm -f "$work_dir"/cli-*.profraw "$work_dir"/cli.profdata

shopt -s nullglob
bins=("$target_dir"/debug/build/agileplus-cli/*/out/agileplus_cli-*)
if [ ${#bins[@]} -eq 0 ]; then
    echo "no instrumented agileplus-cli test binary found under $target_dir" >&2
    exit 1
fi

for bin in "${bins[@]}"; do
    [ -x "$bin" ] || continue
    echo "    running $(basename "$bin")"
    LLVM_PROFILE_FILE="$work_dir/cli-%p.profraw" "$bin" --test-threads=4 >/dev/null 2>&1 || true
    "$llvm_profdata" merge -sparse "$work_dir"/cli-*.profraw -o "$work_dir/cli.profdata" 2>/dev/null
    "$llvm_cov" report "$bin" -instr-profile="$work_dir/cli.profdata" \
        > "$work_dir/cli-report.txt" 2>/dev/null
done

# llvm-cov prints paths without a leading '/', so match on the fragment.
cli_root="${repo_root#/}"
cli_total="$(awk -v root="$cli_root" '
    index($1, root "/crates/agileplus-cli/src/") == 1 && NF >= 13 { lines += $8; missed += $9; files++ }
    END { if (lines > 0) printf "%d %d %d", files, lines, missed; else print "0 0 0" }
' "$work_dir/cli-report.txt")"
read -r cli_files cli_lines cli_missed <<<"$cli_total"

if [ "$cli_lines" -eq 0 ]; then
    echo "could not measure agileplus-cli from its own profiles; see $work_dir" >&2
    exit 1
fi

printf 'agileplus-cli (hand-measured): %d files, %d lines, %.1f%% covered\n' \
    "$cli_files" "$cli_lines" \
    "$(awk -v l="$cli_lines" -v m="$cli_missed" 'BEGIN { printf "%.1f", (l - m) * 100 / l }')"

echo "==> corrected workspace total"
awk -v cli_lines="$cli_lines" -v cli_missed="$cli_missed" '
    NF >= 13 && $1 == "TOTAL" { ws_lines = $8; ws_missed = $9 }
    NF >= 13 && $1 ~ /^agileplus-cli/ { lost_lines += $8; lost_missed += $9 }
    END {
        total = ws_lines - lost_lines + cli_lines
        missed = ws_missed - lost_missed + cli_missed
        printf "  raw     : %d lines, %.2f%% covered\n", ws_lines, (ws_lines - ws_missed) * 100 / ws_lines
        printf "  corrected: %d lines, %.2f%% covered (cli replaced with its hand-measured rows)\n",
            total, (total - missed) * 100 / total
    }
' "$work_dir/workspace.txt"

echo "reports: $work_dir/workspace.txt, $work_dir/cli-report.txt"
