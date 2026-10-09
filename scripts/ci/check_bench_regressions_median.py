#!/usr/bin/env python3
"""Compare the MEDIAN of N cargo bench runs against a stored baseline.

Usage:
    check_bench_regressions_median.py <bench1.txt> <bench2.txt> [<bench3.txt> ...] <baseline.json> [--max-regress N]

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

Fails (exit 1) if any benchmark regressed by more than `--max-regress`
percent (default 50, the floor of catastrophic-regression signal).

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
from check_bench_regressions import parse_bench


def load_exclude_names(path: Path | None) -> set[str]:
    if not path or not path.exists():
        return set()
    return {
        line.strip()
        for line in path.read_text(errors="ignore").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument(
        "runs",
        nargs="+",
        help="One or more bench.txt files (one per run). The LAST positional is the baseline JSON.",
    )
    p.add_argument("--max-regress", type=float, default=50.0)
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
    for name in all_names:
        samples = [d[name] for d in per_run if name in d]
        if not samples:
            continue
        median_per_bench[name] = int(statistics.median(samples))

    baseline = json.loads(Path(baseline_path).read_text())

    regressions: list[dict] = []
    skipped_excluded: list[str] = []
    skipped_no_baseline: list[str] = []
    for name, cur_ns in median_per_bench.items():
        if name in exclude:
            skipped_excluded.append(name)
            continue
        base = baseline.get(name)
        if not base:
            skipped_no_baseline.append(name)
            continue
        base_ns = base["median_ns"]
        if base_ns <= 0:
            continue
        delta_pct = (cur_ns - base_ns) / base_ns * 100
        if delta_pct > args.max_regress:
            regressions.append(
                {
                    "name": name,
                    "baseline_ns": base_ns,
                    "current_ns": cur_ns,
                    "delta_pct": round(delta_pct, 2),
                }
            )

    report = {
        "max_regress_pct": args.max_regress,
        "runs": [str(p) for p in run_paths],
        "run_count": len(run_paths),
        "regressions": regressions,
        "ok": not regressions,
        "checked": len(median_per_bench)
        - len(skipped_excluded)
        - len(skipped_no_baseline),
        "excluded": len(skipped_excluded),
        "no_baseline": len(skipped_no_baseline),
        "baseline_size": len(baseline),
    }
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if not regressions else 1


if __name__ == "__main__":
    raise SystemExit(main())
