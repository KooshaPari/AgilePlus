#!/usr/bin/env python3
"""Compare cargo bench output against a stored baseline.

Usage:
    check_bench_regressions.py <bench.txt> <baseline.json> [--max-regress N]

Reads cargo bench's `--output-format bencher` output and compares the
median ns/iter for each benchmark against the JSON baseline:

    {
      "name::bench_name": {"median_ns": 12345, "stddev_ns": 678},
      ...
    }

Fails (exit 1) if any benchmark regressed by more than `--max-regress`
percent (default 15).

Traces to: FR-CI-01 (infrastructure), pillar L27 (Infrastructure CI).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path


BENCH_RE = re.compile(
    r"^(?P<name>[^\s]+)\s+time:\s+\[(?P<low>[\d.]+)\s+(?P<unit>\w+)\s+(?P<high>[\d.]+)\s+(?P<unit2>\w+)\]"
)
NS_PER_UNIT = {
    "ns": 1,
    "us": 1_000,
    "ms": 1_000_000,
    "s": 1_000_000_000,
}


def to_ns(value: float, unit: str) -> int:
    return int(value * NS_PER_UNIT[unit])


def parse_bench(path: Path) -> dict[str, int]:
    out: dict[str, int] = {}
    for line in path.read_text(errors="ignore").splitlines():
        m = BENCH_RE.match(line.strip())
        if not m:
            continue
        ns = to_ns((float(m["low"]) + float(m["high"])) / 2, m["unit"])
        out[m["name"]] = ns
    return out


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("bench")
    p.add_argument("baseline")
    p.add_argument("--max-regress", type=float, default=15.0)
    args = p.parse_args()

    current = parse_bench(Path(args.bench))
    baseline = json.loads(Path(args.baseline).read_text())

    regressions: list[dict] = []
    for name, cur_ns in current.items():
        base = baseline.get(name)
        if not base:
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
        "regressions": regressions,
        "ok": not regressions,
        "checked": len(current),
        "baseline_size": len(baseline),
    }
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if not regressions else 1


if __name__ == "__main__":
    raise SystemExit(main())