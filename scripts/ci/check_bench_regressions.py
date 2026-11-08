#!/usr/bin/env python3
"""Compare cargo bench output against a stored baseline.

Usage:
    check_bench_regressions.py <bench.txt> <baseline.json> [--noise-budget PCT] [--max-regress N]

Reads Criterion's human-readable `cargo bench` output (including its
two-line benchmark-name format) and compares the median ns/iter for each
benchmark against the JSON baseline:

    {
      "name::bench_name": {"median_ns": 12345, "stddev_ns": 678},
      ...
    }

Fails (exit 1) if any benchmark regressed by more than its noise budget.

Traces to: FR-CI-01 (infrastructure), pillar L27 (Infrastructure CI).
"""

from __future__ import annotations

import argparse
import json
import re
import statistics
import sys
from pathlib import Path

# ── Noise budget ─────────────────────────────────────────────────────────────
#
# WHY this replaced the old flat threshold
# --------------------------------------
# The gate used to compare every benchmark's median against the stored
# baseline with ONE flat absolute percentage (50 % in CI). The stored baseline
# has `stddev_ns: 0` for every entry, so the gate had no idea how noisy any
# benchmark was, and a flat absolute threshold cannot tell "this code got
# slower" from "this runner is slower".
#
# On 2026-11-08 that misfired on main: a runner ~1.61x slower than the machine
# the baseline was captured on inflated 19 of 36 benchmarks by 57 %-116 % and
# failed the gate. The same head SHA passed on a different runner minutes
# later. The inflation was uniform across unrelated subsystems, which is the
# signature of the machine, not of a code change.
#
# The threshold is now derived per benchmark instead:
#
#   sigma_rel = stddev_ns / median_ns        (measured, from the baseline)
#   budget    = NOISE_SIGMA_K * sigma_rel    (3 sigma of the benchmark's own
#                                             run-to-run dispersion)
#
# When the baseline entry carries NO measured stddev (`stddev_ns` is 0 or
# absent) we fall back to DEFAULT_NOISE_BUDGET_PCT: a documented default
# relative noise budget, not a flat absolute threshold. The default is a floor
# rather than a replacement because the stored baseline was captured on a
# different machine than the CI runner, so a benchmark's absolute timing also
# carries machine-to-machine transfer error that no amount of within-run
# measurement noise describes. A measured stddev can therefore only *raise* a
# benchmark's budget above that floor - it is evidence that the benchmark is
# noisier than the default assumption, never evidence that the transfer error
# is smaller than it is.
#
# The comparison itself is made against the run's COMMON-MODE factor: the
# median of the per-benchmark ratios. A slowdown shared by the whole suite is
# the runner; only the deviation from that shared factor is a regression.
#
# KNOWN LIMITATION: a regression that slows down MORE THAN HALF the suite by
# the same amount is indistinguishable from a slower runner and is normalised
# away. The report surfaces the runner factor so this stays visible.
NOISE_SIGMA_K = 3.0
DEFAULT_NOISE_BUDGET_PCT = 40.0
MIN_REGRESS_PCT = 10.0
MAX_NOISE_BUDGET_PCT = 150.0
COMMON_MODE_MIN_BENCHES = 5
COMMON_MODE_SANE_RANGE = (0.2, 5.0)

BENCH_RE = re.compile(
    r"^(?P<name>[^\s]+)\s+time:\s+\[(?P<low>[\d.]+)\s+(?P<unit>[A-Za-zµ]+)\s+"
    r"(?P<median>[\d.]+)\s+(?P<unit2>[A-Za-zµ]+)\s+(?P<high>[\d.]+)\s+(?P<unit3>[A-Za-zµ]+)\]"
)
TIME_RE = re.compile(
    r"^time:\s+\[(?P<low>[\d.]+)\s+(?P<unit>[A-Za-zµ]+)\s+"
    r"(?P<median>[\d.]+)\s+(?P<unit2>[A-Za-zµ]+)\s+(?P<high>[\d.]+)\s+(?P<unit3>[A-Za-zµ]+)\]"
)
NS_PER_UNIT = {
    "ns": 1,
    "us": 1_000,
    "µs": 1_000,
    "μs": 1_000,
    "ms": 1_000_000,
    "s": 1_000_000_000,
}


def to_ns(value: float, unit: str) -> int:
    return int(value * NS_PER_UNIT[unit])


def safe_read_text(path: Path) -> str:
    """Read a CI-local file, refusing paths that escape the working directory.

    These scripts only ever read files from the workspace they are invoked in
    (bench outputs, baselines, exclude lists). Resolving against the current
    directory and rejecting escapes keeps every read inside that workspace, so
    a stray or crafted CLI argument cannot reach elsewhere on the runner
    (SonarCloud pythonsecurity:S8707).
    """
    resolved = path.resolve()
    base = Path.cwd().resolve()
    if not resolved.is_relative_to(base):
        raise ValueError(f"refusing to read outside the working directory: {path}")
    # `errors="replace"` (not "ignore") so any malformed UTF-8 byte in a bench output or
    # baseline is surfaced as the unicode replacement character (\uFFFD) in the parsed
    # output rather than silently dropped. A silently dropped byte could merge two bench
    # lines that the parser would otherwise see as distinct.
    return resolved.read_text(encoding="utf-8", errors="replace")


def parse_bench(path: Path, exclude: set[str] | None = None) -> dict[str, int]:
    out: dict[str, int] = {}
    pending_name: str | None = None
    for line in safe_read_text(path).splitlines():
        stripped = line.strip()
        m = BENCH_RE.match(stripped)
        if m:
            pending_name = m["name"]
        else:
            if re.fullmatch(r"[^\s]+", stripped):
                pending_name = stripped
            m = TIME_RE.match(stripped)
        if not m or not pending_name:
            continue
        if exclude and pending_name in exclude:
            pending_name = None
            continue
        ns = to_ns(float(m["median"]), m["unit2"])
        out[pending_name] = ns
        pending_name = None
    return out


def load_exclude_names(path: Path | None) -> set[str]:
    # If no exclude file was given, no exclusions. If a path was given but the
    # file does not exist, let safe_read_text raise — fail fast rather than
    # silently treating the gate as having no exclusions (which would let ns-
    # scale benches count toward the threshold and false-positive regress).
    if not path:
        return set()
    return {
        line.strip()
        for line in safe_read_text(path).splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def noise_budget_pct(base: dict, default_pct: float = DEFAULT_NOISE_BUDGET_PCT) -> float:
    """Relative noise budget (percent of the baseline) for one baseline entry.

    Uses the entry's measured `stddev_ns` when it has one (NOISE_SIGMA_K sigma
    of the benchmark's own run-to-run dispersion); otherwise falls back to the
    default relative noise budget. The default is a FLOOR in both cases - see
    the module comment for why a measured stddev can only widen the budget,
    never tighten it below the default.
    """
    base_ns = base.get("median_ns") or 0
    stddev_ns = base.get("stddev_ns") or 0
    budget = default_pct
    if base_ns > 0 and stddev_ns > 0:
        budget = max(default_pct, NOISE_SIGMA_K * (stddev_ns / base_ns) * 100.0)
    return max(MIN_REGRESS_PCT, min(MAX_NOISE_BUDGET_PCT, budget))


def common_mode_factor(ratios: list[float]) -> tuple[float, bool]:
    """Shared speed factor of this run relative to the baseline.

    Returns `(factor, applied)`. The factor is the median of the per-benchmark
    ratios: the part of a slowdown (or speed-up) that the whole suite shares,
    which is the runner rather than the code. Normalisation is skipped when
    too few benchmarks were compared to estimate it, or when the factor is so
    far outside a sane range that the run or the baseline is broken.
    """
    if len(ratios) < COMMON_MODE_MIN_BENCHES:
        return 1.0, False
    factor = statistics.median(ratios)
    low, high = COMMON_MODE_SANE_RANGE
    if not low <= factor <= high:
        return 1.0, False
    return factor, True


def evaluate(
    current: dict[str, int],
    baseline: dict,
    *,
    exclude: set[str] | None = None,
    default_budget_pct: float = DEFAULT_NOISE_BUDGET_PCT,
    max_regress: float | None = None,
) -> dict:
    """Compare one run's medians against the baseline and report regressions.

    `max_regress` overrides the noise budget with a single flat percentage
    applied to the RAW delta against the baseline - the legacy behaviour, with
    no common-mode normalisation. `None` means "use each benchmark's own noise
    budget, compared against the run's common-mode factor".
    """
    exclude = exclude or set()
    # Ignore any non-benchmark bookkeeping keys (for example a `_meta` block
    # recording the noise budget and where the measured stddev came from).
    entries = {
        name: entry
        for name, entry in baseline.items()
        if isinstance(entry, dict) and "median_ns" in entry
    }
    compared: list[tuple[str, dict, int, int]] = []
    skipped_excluded: list[str] = []
    skipped_no_baseline: list[str] = []
    for name, cur_ns in current.items():
        if name in exclude:
            skipped_excluded.append(name)
            continue
        base = entries.get(name)
        if not base:
            skipped_no_baseline.append(name)
            continue
        base_ns = base.get("median_ns") or 0
        if base_ns <= 0:
            continue
        compared.append((name, base, base_ns, cur_ns))

    ratios = [cur_ns / base_ns for _, _, base_ns, cur_ns in compared]
    if max_regress is None:
        runner_factor, common_mode_applied = common_mode_factor(ratios)
    else:
        runner_factor, common_mode_applied = 1.0, False

    regressions: list[dict] = []
    worst_deviation_pct = 0.0
    for name, base, base_ns, cur_ns in compared:
        ratio = cur_ns / base_ns
        delta_pct = (ratio - 1) * 100
        deviation_pct = (ratio / runner_factor - 1) * 100
        threshold_pct = (
            max_regress if max_regress is not None else noise_budget_pct(base, default_budget_pct)
        )
        worst_deviation_pct = max(worst_deviation_pct, deviation_pct)
        if deviation_pct > threshold_pct:
            regressions.append(
                {
                    "name": name,
                    "baseline_ns": base_ns,
                    "current_ns": cur_ns,
                    "delta_pct": round(delta_pct, 2),
                    "runner_normalized_delta_pct": round(deviation_pct, 2),
                    "threshold_pct": round(threshold_pct, 2),
                }
            )

    regressions.sort(key=lambda r: -r["runner_normalized_delta_pct"])
    return {
        "regressions": regressions,
        "ok": not regressions,
        "runner_factor": round(runner_factor, 4),
        "common_mode_applied": common_mode_applied,
        "worst_deviation_pct": round(worst_deviation_pct, 2),
        "checked": len(compared),
        "excluded": len(skipped_excluded),
        "no_baseline": len(skipped_no_baseline),
        "baseline_size": len(entries),
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("bench")
    p.add_argument("baseline")
    p.add_argument(
        "--noise-budget",
        type=float,
        default=DEFAULT_NOISE_BUDGET_PCT,
        help=(
            "Default relative noise budget (percent) for benchmarks whose baseline "
            "entry has no measured stddev_ns."
        ),
    )
    p.add_argument(
        "--max-regress",
        type=float,
        default=None,
        help="Override every benchmark's noise budget with this flat percentage.",
    )
    p.add_argument(
        "--exclude-file",
        type=Path,
        default=None,
        help="File with one bench name per line to exclude from the threshold gate.",
    )
    args = p.parse_args()

    exclude = load_exclude_names(args.exclude_file)
    current = parse_bench(Path(args.bench), exclude=exclude)
    baseline = json.loads(safe_read_text(Path(args.baseline)))

    report = evaluate(
        current,
        baseline,
        exclude=exclude,
        default_budget_pct=args.noise_budget,
        max_regress=args.max_regress,
    )
    report["noise_budget_pct"] = args.noise_budget
    if args.max_regress is not None:
        report["max_regress_pct"] = args.max_regress
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
