#!/usr/bin/env python3
"""Compare the MEDIAN of N cargo bench runs against a stored baseline.

Usage:
    check_bench_regressions_median.py <bench1.txt> <bench2.txt> [<bench3.txt> ...] <baseline.json> [--noise-budget PCT] [--max-regress N]

This is the noise-tolerant variant of `check_bench_regressions.py`. It takes the
median of median_ns values per benchmark across N runs and compares that
against the JSON baseline. Single-run Criterion noise is ±15-25%, so even
N=3 (the default bench-thresholds workflow run count) cuts variance to ~5%.

Reads Criterion's human-readable `cargo bench` output (including its two-line
benchmark-name format) and compares the median ns/iter for each benchmark
against the JSON baseline:

    {
      "name::bench_name": {"median_ns": 12345, "stddev_ns": 678},
      ...
    }

Fails (exit 1) if any benchmark regressed by more than its own noise budget:
a RELATIVE budget (percent of the baseline) derived from the baseline entry's
measured `stddev_ns` when it has one, and from the documented default relative
noise budget when it does not. The comparison is made against the run's
common-mode speed factor so that a runner slower than the machine the
baseline came from is not reported as a regression. See
`check_bench_regressions.py` for the full rationale and the known limitation.

Traces to: FR-CI-01 (infrastructure), pillar L27 (Infrastructure CI).
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

# Reuse the parsing logic from the single-run script. Keeps the two scripts
# in lockstep on regex / unit handling.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_bench_regressions import (
    DEFAULT_NOISE_BUDGET_PCT,
    evaluate,
    parse_bench,
    safe_read_text,
)


def load_exclude_names(path: Path | None) -> set[str]:
    if not path or not path.exists():
        return set()
    return {
        line.strip()
        for line in safe_read_text(path).splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument(
        "runs",
        nargs="+",
        help="One or more bench.txt files (one per run). The LAST positional is the baseline JSON.",
    )
    p.add_argument(
        "--noise-budget",
        type=float,
        default=DEFAULT_NOISE_BUDGET_PCT,
        help=(
            "Default relative noise budget (percent) for benchmarks whose baseline "
            "entry carries no measured stddev_ns."
        ),
    )
    p.add_argument(
        "--max-regress",
        type=float,
        default=None,
        help=(
            "Override every benchmark's noise budget with this flat percentage. "
            "Normally left unset so each benchmark uses its own noise budget."
        ),
    )
    p.add_argument(
        "--exclude-file",
        type=Path,
        default=None,
        help="File with one bench name per line to exclude from the threshold gate.",
    )
    args = p.parse_args()

    if len(args.runs) < 2:
        print(
            "error: need at least 2 positional args (>=1 run + baseline)",
            file=sys.stderr,
        )
        return 2

    *run_paths, baseline_path = args.runs
    if len(run_paths) < 2:
        print(
            f"warning: only {len(run_paths)} run(s) provided; median is trivially that single value",
            file=sys.stderr,
        )

    exclude = load_exclude_names(args.exclude_file)

    # Parse each run, then take the median per benchmark across runs.
    per_run: list[dict[str, int]] = [parse_bench(Path(rp)) for rp in run_paths]
    all_names: set[str] = set().union(*(d.keys() for d in per_run))

    median_per_bench: dict[str, int] = {}
    missing_samples: list[tuple[str, int]] = []
    for name in all_names:
        samples = [d[name] for d in per_run if name in d]
        if not samples:
            missing_samples.append((name, 0))
            continue
        if len(samples) < len(per_run):
            missing_samples.append((name, len(samples)))
        median_per_bench[name] = int(statistics.median(samples))

    if missing_samples:
        print(
            f"WARNING: {len(missing_samples)} benchmark(s) had missing samples "
            f"(out of {len(per_run)} runs each):",
            file=sys.stderr,
        )
        for name, have in missing_samples:
            print(f"  - {name}: {have}/{len(per_run)} runs", file=sys.stderr)

    baseline = json.loads(safe_read_text(Path(baseline_path)))

    report = evaluate(
        median_per_bench,
        baseline,
        exclude=exclude,
        default_budget_pct=args.noise_budget,
        max_regress=args.max_regress,
    )
    report["runs"] = [str(p) for p in run_paths]
    report["run_count"] = len(run_paths)
    report["noise_budget_pct"] = args.noise_budget
    if args.max_regress is not None:
        report["max_regress_pct"] = args.max_regress

    # A runner that is materially faster or slower than the machine the
    # baseline came from is normalised out of the per-benchmark comparison.
    # Say so loudly: it is the difference between "the code regressed" and
    # "this box is different".
    factor = report["runner_factor"]
    if report["common_mode_applied"] and abs(factor - 1.0) >= 0.10:
        print(
            f"NOTE: this run's common-mode speed factor is {factor:.3f}x the baseline "
            f"machine; per-benchmark deltas are normalised by it before the gate "
            f"compares them.",
            file=sys.stderr,
        )

    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if report["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
