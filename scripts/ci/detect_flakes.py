#!/usr/bin/env python3
"""Detect flaky tests by comparing first-run vs rerun output.

Usage:
    detect_flakes.py <first-run.log> <rerun.log>

Reads two `cargo test` outputs. A test is "flaky" if it failed in the
first run but passed in the rerun. The script prints a JSON document with
the list of flaky tests to stdout, suitable for upload as a workflow
artifact.

Traces to: FR-CI-01 (infrastructure), pillar L27 (Infrastructure CI).
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


def parse_failures(path: Path) -> set[str]:
    """Return the set of failing test names."""
    failures: set[str] = set()
    if not path.exists():
        return failures
    pattern = re.compile(r"^test (\S+) \.\.\. FAILED")
    for line in path.read_text(errors="ignore").splitlines():
        m = pattern.match(line.strip())
        if m:
            failures.add(m.group(1))
    return failures


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: detect_flakes.py <first-run.log> <rerun.log>", file=sys.stderr)
        return 2
    first = parse_failures(Path(sys.argv[1]))
    rerun = parse_failures(Path(sys.argv[2]))
    flaky = sorted(first - rerun)
    still_failing = sorted(first & rerun)
    report = {
        "flaky": flaky,
        "still_failing": still_failing,
        "flaky_count": len(flaky),
        "still_failing_count": len(still_failing),
    }
    json.dump(report, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())