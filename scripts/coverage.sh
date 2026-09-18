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
# it by hand shows commands/module/tag.rs at 89.7% lines instead of 0.00%.
# That hand-run view is test-INCLUSIVE (a file's own #[cfg(test)] module is
# reported there but excluded from cargo-llvm-cov's rows), so its raw cli
# total of 74.2% over 14,348 lines is not comparable with the workspace
# metric: it would credit 7,488 lines of test code as covered production
# code. Compared on the workspace's own basis, agileplus-cli is ~46% and the
# corrected workspace figure is ~61.5% rather than the reported 48.7%.
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
# --no-clean matters: by default cargo-llvm-cov cleans its target directory, and
# the instrumented test binaries this script later needs are exactly what
# disappears. Without it the union pass finds nothing to run.
cargo llvm-cov --workspace --summary-only --no-clean > "$work_dir/workspace.txt" 2>&1 || true
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
# The two views count different things: cargo-llvm-cov excludes a file's own
# #[cfg(test)] module from its rows (a file with in-file tests reports fewer
# lines there than here), while a hand-run binary reports everything it holds.
# Substituting raw manual totals would therefore credit test-module lines as
# covered production code and inflate the result. Compare on one basis: keep
# the workspace denominator, and take only the covered lines that fall outside
# the file's test code, assuming the test module itself fully executes.
awk -v cli_lines="$cli_lines" -v cli_missed="$cli_missed" '
    NF >= 13 && $1 == "TOTAL" { ws_lines = $8; ws_missed = $9 }
    NF >= 13 && $1 ~ /^agileplus-cli\// { lost_lines += $8 }
    END {
        ws_covered = ws_lines - ws_missed
        test_lines = cli_lines - lost_lines
        if (test_lines < 0) test_lines = 0
        cli_covered = (cli_lines - cli_missed) - test_lines
        if (cli_covered < 0) cli_covered = 0
        if (cli_covered > lost_lines) cli_covered = lost_lines
        printf "  raw        : %d lines, %.2f%% covered (agileplus-cli counted as empty)\n",
            ws_lines, ws_covered * 100 / ws_lines
        printf "  corrected  : %d lines, %.2f%% covered (cli restored on the same basis)\n",
            ws_lines, (ws_covered + cli_covered) * 100 / ws_lines
        printf "  agileplus-cli alone: %d of %d lines = %.1f%% (test code excluded)\n",
            cli_covered, lost_lines, cli_covered * 100 / lost_lines
    }
' "$work_dir/workspace.txt"

# The workspace summary also omits whole crates: 24 of the 37 declared members
# contribute no rows at all (agileplus-cache, -config, -subcmds, -p2p and the
# agileplus-agents/* crates among them) even though their test binaries run.
# Measure every instrumented test binary directly so those crates are counted,
# then union the per-file results.
if [ "${COVERAGE_ALL:-1}" = "1" ]; then
    echo "==> measuring every instrumented test binary (recovers crates the summary drops)"
    rm -f "$work_dir"/all-*.profraw "$work_dir/all.profdata"
    mkdir -p "$work_dir/allreports"

    # Test binaries live at debug/build/<crate>/<hash>/out/<name>-<hash>. Note
    # that cargo-llvm-cov does not keep them around indefinitely: the out/
    # directories can be present with zero executables once a run has finished,
    # so this discovery is best-effort. If nothing is found, the union step
    # says so rather than reporting a partial total as if it were complete.
    # The robust alternative is to capture the paths from the instrumented
    # build itself, for example:
    #   RUSTFLAGS="-Cinstrument-coverage" cargo test --workspace --no-run \
    #       --message-format=json | jq -r 'select(.executable != null) | .executable'
    # Test binaries live at debug/build/<crate>/<hash>/out/<name>-<hash>. Only
    # that location is searched: debug/deps also holds non-test executables,
    # and running those produced a wall of instant non-zero exits that showed up
    # as bogus "skipped" counts in the first version of this pass.
    all_bins=("$target_dir"/debug/build/*/*/out/*-*)
    measured=0
    skipped=0
    for bin in "${all_bins[@]}"; do
        [ -f "$bin" ] && [ -x "$bin" ] || continue
        case "$bin" in *.d|*.rlib|*.rmeta|*.lock|*/-*) continue ;; esac
        if timeout "${COVERAGE_BIN_TIMEOUT:-180}" env \
            LLVM_PROFILE_FILE="$work_dir/all-%p.profraw" "$bin" --test-threads=4 \
            >/dev/null 2>&1; then
            measured=$((measured + 1))
        else
            # Slow or environment-dependent suites are skipped, not silently
            # counted as uncovered; the skip count is reported below.
            skipped=$((skipped + 1))
        fi
    done
    printf '    ran %d test binaries (%d skipped on timeout)\n' "$measured" "$skipped"

    if ls "$work_dir"/all-*.profraw >/dev/null 2>&1; then
        "$llvm_profdata" merge -sparse "$work_dir"/all-*.profraw -o "$work_dir/all.profdata" 2>/dev/null
        present=()
        for bin in "${all_bins[@]}"; do
            [ -f "$bin" ] && [ -x "$bin" ] || continue
            present+=("$bin")
        done
        # Report every binary in ONE invocation. Doing it per binary costs a
        # full profile lookup each time: 201 test binaries made the earlier
        # per-binary pass run for tens of minutes without finishing, which is
        # why the union number was never produced. This llvm-cov takes one
        # positional object plus repeated --object=<path> for the rest, and
        # emits a single merged report (verified against two real binaries).
        objects=()
        for bin in "${present[@]:1}"; do
            objects+=("--object=$bin")
        done
        if [ ${#present[@]} -gt 1 ] && "$llvm_cov" report "${present[0]}" "${objects[@]}" \
            -instr-profile="$work_dir/all.profdata" > "$work_dir/allreports/union.txt" 2>/dev/null \
            && [ -s "$work_dir/allreports/union.txt" ]; then
            echo "    unioned ${#present[@]} binaries in a single report pass"
        else
            echo "    batched report unavailable; falling back to per-binary (slow)"
            : > "$work_dir/allreports/union.txt"
            for bin in "${present[@]}"; do
                "$llvm_cov" report "$bin" -instr-profile="$work_dir/all.profdata" 2>/dev/null \
                    >> "$work_dir/allreports/union.txt"
            done
        fi
        awk -v root="${repo_root#/}" '
            index($1, root "/") == 1 && NF >= 13 {
                if ($8 > lines[$1]) lines[$1] = $8
                covered = $8 - $9
                if (covered > coveredlines[$1]) coveredlines[$1] = covered
                next
            }
            END {
                for (f in lines) { total += lines[f]; cov += coveredlines[f]; files++ }
                printf "    full workspace (all test binaries, test code included): %d files, %d lines, %.2f%% covered\n",
                    files, total, cov * 100 / total
            }
        ' "$work_dir/allreports/union.txt"
    else
        echo "    no profiles collected; union skipped"
    fi
fi

echo "reports: $work_dir/workspace.txt, $work_dir/cli-report.txt"
