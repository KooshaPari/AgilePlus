#!/usr/bin/env python3
"""Compare cargo bench output against a stored baseline.

Usage:
    check_bench_regressions.py <bench.txt> <baseline.json> [--max-regress N]

Reads Criterion's human-readable `cargo bench` output (including its
two-line benchmark-name format) and compares the median ns/iter for each
benchmark against the JSON baseline:

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
    return resolved.read_text(encoding="utf-8", errors="ignore")


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
    if not path or not path.exists():
        return set()
    return {
        line.strip()
        for line in safe_read_text(path).splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("bench")
    p.add_argument("baseline")
    p.add_argument("--max-regress", type=float, default=15.0)
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
        "excluded": len(exclude),
        "baseline_size": len(baseline),
    }
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")

    return 0 if not regressions else 1


if __name__ == "__main__":
    raise SystemExit(main())
