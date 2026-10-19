#!/usr/bin/env python3
"""Derive the per-section headline counters from the SFD tree body.

The cockpit's headline bullets (NDR items with a target witness, gates
that are clearly PASS, ...) must be derivable from the per-cell tree
body, not hand-maintained prose. This script derives them so the md
bullets and the JSON agree.

Usage:
    python3 scripts/sfd_derive_headlines.py
"""
import json
import re
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from sfd_validate_counts import (  # noqa: E402
    DIMS, MD_PATH, JS_PATH, parse_tree_body,
)

# A "target-level PASS witness" for an NDR item = the row has at least one
# PASS cell. A gate is "clearly PASS" only when EVERY one of its meaningful
# cells is PASS -- a single PARTIAL / NOT_RUN / UNKNOWN / FAIL cell in the
# gate means the gate is not clear. This is the rule the cockpit prose uses
# and it is what the JSON headline_counts.gates_pass encodes.
#
# Note the looser ">=1 PASS and no FAIL" rule yields 7/20 (E.1, E.4, E.8,
# E.15, E.16, E.18, E.19) because E.8/E.15/E.16/E.18 each carry a PARTIAL
# and/or NOT_RUN cell. Those gates are NOT clearly PASS. Use the strict rule.
SECTION_TITLES = {
    "A": "operator-directive sections",
    "B": "PRD epics",
    "C": "journeys",
    "D": "NDR items",
    "E": "gates",
}


def main():
    md_text = open(MD_PATH).read()
    rows, _ = parse_tree_body(md_text)

    print("=== per-section row counts ===")
    for sec in "ABCDE":
        n = sum(1 for k in rows if k.startswith(sec + "."))
        print(f"  {sec}: {n} rows")

    print("\n=== NDR items (section D) ===")
    ndr_pass = []
    for k in sorted(rows, key=lambda x: (x[0], int(x.split(".")[1]))):
        if not k.startswith("D."):
            continue
        v = rows[k]
        has_pass = "PASS" in v
        has_fail = "FAIL" in v
        mark = "PASS" if has_pass else ("FAIL" if has_fail else "-")
        print(f"  {k:5} {v}  {mark}")
        if has_pass:
            ndr_pass.append(k)
    print(f"  -> NDR items with >=1 PASS cell: {len(ndr_pass)}/10 {ndr_pass}")

    print("\n=== gates (section E) ===")
    gate_clear = []
    for k in sorted(rows, key=lambda x: (x[0], int(x.split(".")[1]))):
        if not k.startswith("E."):
            continue
        v = rows[k]
        meaningful = [c for c in v if c not in ("DIM_NOT_MEANINGFUL", "NOT_APPLICABLE")]
        has_pass = "PASS" in v
        has_fail = "FAIL" in v
        all_pass = bool(meaningful) and all(c == "PASS" for c in meaningful)
        clear = all_pass
        mark = "CLEAR-PASS" if clear else ("HAS-FAIL" if has_fail else "-")
        if has_pass or has_fail:
            print(f"  {k:5} {v}  {mark}")
        if clear:
            gate_clear.append(k)
    print(f"  -> gates clearly PASS (EVERY meaningful cell is PASS): "
          f"{len(gate_clear)}/20 {gate_clear}")

    print("\n=== journey rows with any PASS (C) ===")
    cpass = [k for k in rows if k.startswith("C.") and "PASS" in rows[k]]
    print(f"  {len(cpass)}/12 {cpass}")

    print("\n=== integration / tracera surfaces (not derivable from tree) ===")
    print("  1/11 integration surfaces (SQLite) -- from INTEGRATION.md §3")
    print("  1/5 Tracera-relevant surfaces -- from E11 row")

    print("\n=== JSON headline_counts (for comparison) ===")
    js = json.load(open(JS_PATH))
    for k, v in js["headline_counts"].items():
        if k.startswith("cells_"):
            continue
        if isinstance(v, str):
            print(f"  {k}: {v[:120]}")
        else:
            print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
