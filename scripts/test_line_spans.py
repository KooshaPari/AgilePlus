#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Count the lines that live inside `#[cfg(test)]` regions of a Rust tree.

Why this exists
---------------
A hand-run coverage binary reports *everything* it holds, including test code:
integration test files under `tests/`, and inline `#[cfg(test)] mod ...` blocks
inside ordinary source files. Test code executes whenever the suite runs, so
counting it as covered production code inflates any coverage figure badly -
measured here at 2.3x for agileplus-cli up to 14x for agileplus-events.

cargo-llvm-cov's own rows already exclude that code, which is why its per-file
line counts are much smaller than a hand-run report's for the same file. To put
both views on one basis, subtract the test spans.

Usage:
    test_line_spans.py <crate-root> [...]

Prints TSV: <absolute path>\t<test line count>, one row per file that has any.

KNOWN LIMITATION - do not subtract these from an llvm-cov report
---------------------------------------------------------------
This counts RAW source lines inside test regions (including blank lines,
comments and anything a formatter wrapped). llvm-cov counts only instrumented
lines. Measured on agileplus-cli: this script reports 8,681 raw test lines while
the difference between the hand-run union and the tool's production-only rows
implies ~9,190 instrumented test lines - i.e. the two quantities are not
comparable, and subtracting one from the other produces nonsense (more test
lines than the file has code). Use these spans for ranking and for spotting
which files carry large test regions; do not use them as a correction term.

The reliable way to get production-only coverage for every crate is to make
cargo-llvm-cov resolve the crates it currently drops (24 of 37 declared members
report zero rows, the same absolute-vs-relative path symptom that hides
agileplus-cli), rather than to post-process a hand-run union, because the union
is inherently test-inclusive and mixes path forms for the same file.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

CFG_TEST = re.compile(r"#\[cfg\(\s*test\s*\)\]")
MOD_DECL = re.compile(r"^\s*(?:pub\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")


def test_span_end(lines: list[str], start: int, brace_hint: int) -> int:
    """Return the last line index (inclusive) of the block opened at `start`.

    `brace_hint` is the first line at or after `start` that should contain the
    opening brace. Lines are scanned with a crude brace counter, which is sound
    enough for unformatted-by-generators Rust: string literals containing braces
    would skew it, and that risk is accepted and reported rather than hidden.
    """
    depth = 0
    opened = False
    for index in range(brace_hint, len(lines)):
        line = lines[index]
        # Strip line comments so braces in prose do not count.
        code = line.split("//", 1)[0]
        for char in code:
            if char == "{":
                depth += 1
                opened = True
            elif char == "}":
                depth -= 1
                if opened and depth == 0:
                    return index
    return len(lines) - 1


def spans_for_file(path: Path) -> int:
    """Count lines inside `#[cfg(test)]` regions of one file."""
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()

    # Files that are themselves test modules: `#[cfg(test)] mod tests;` points
    # at a sibling file such as tests.rs. Those are entirely test code and are
    # handled by the caller via path, not here.
    marked: set[int] = set()
    for index, line in enumerate(lines):
        if not CFG_TEST.search(line):
            continue
        # An inline module: find its opening brace, then its extent.
        hint = index
        while hint < len(lines) and "{" not in lines[hint] and not MOD_DECL.match(lines[hint]):
            hint += 1
        if hint >= len(lines):
            marked.add(index)
            continue
        if MOD_DECL.match(lines[hint]):
            # `#[cfg(test)] mod tests;` - the target file is test code; the
            # declaration line itself is all that lives here.
            marked.add(hint)
            continue
        end = test_span_end(lines, index, hint)
        marked.update(range(index, end + 1))
    return len(marked)


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2

    for root in argv[1:]:
        root_path = Path(root).resolve()
        for path in sorted(root_path.rglob("*.rs")):
            count = spans_for_file(path)
            if count:
                print(f"{path}\t{count}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
