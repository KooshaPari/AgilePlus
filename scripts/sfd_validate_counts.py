#!/usr/bin/env python3
"""SFD cockpit count validator.

Single source of truth for the State & Forward Docket cockpit counts.

The cockpit has THREE artefacts that must agree:
  1. PROGRESS_TREE.md          -- the per-cell tree body (64 rows x 4 dims)
  2. PROGRESS_TREE.snapshot.json -- dimension_breakdown + headline_counts
  3. the Counts table inside PROGRESS_TREE.md

This validator derives the counts from the per-cell tree body (the only
artefact with per-cell granularity), and asserts the other two agree.

Verdict vocabulary (per State & Forward Docket v1.0):
  PASS FAIL NOT_RUN UNKNOWN STALE BLOCKED NOT_APPLICABLE
  plus:
  PARTIAL           -- a dimension is partially satisfied
  DIM_NOT_MEANINGFUL -- the dimension does not apply to that intent row

Token map used by the tree body:
  ✓            PASS
  ~            PARTIAL
  ✗            FAIL
  …            NOT_RUN
  ?            UNKNOWN
  S            STALE
  B            BLOCKED
  N/A          NOT_APPLICABLE
  _            dim-not-meaningful
  L M          dim-not-meaningful (Assurance strength: low / medium)
  H            PASS              (Assurance strength: high)
  compound A/B resolves to A, EXCEPT strength/strength -> dim-not-meaningful

Meaningful denominator rule (stated in PROGRESS_TREE.md Counts section):
  meaningful = 256 - dim_not_meaningful - NOT_APPLICABLE

Usage:
    python3 scripts/sfd_validate_counts.py            # report + exit 1 on drift
    python3 scripts/sfd_validate_counts.py --write    # rewrite JSON + md table
"""
import argparse
import json
import os
import re
import sys
from collections import Counter

DOC = "docs/sessions/20261017-state-forward-docket"
MD_PATH = f"{DOC}/PROGRESS_TREE.md"
JS_PATH = f"{DOC}/PROGRESS_TREE.snapshot.json"

DIMS = ["Implementation", "Usability", "Validation", "Assurance"]

VERDICTS = ["PASS", "PARTIAL", "FAIL", "NOT_RUN", "UNKNOWN", "STALE",
            "BLOCKED", "NOT_APPLICABLE"]
JSON_KEYS = ["PASS", "PARTIAL", "FAIL", "NOT_RUN", "UNKNOWN", "STALE",
             "BLOCKED", "NOT_APPLICABLE", "dim_not_meaningful", "total"]

TOKEN_MAP = {
    "✓": "PASS",
    "~": "PARTIAL",
    "✗": "FAIL",
    "…": "NOT_RUN",
    "?": "UNKNOWN",
    "S": "STALE",
    "B": "BLOCKED",
    "N/A": "NOT_APPLICABLE",
    "_": "DIM_NOT_MEANINGFUL",
    "L": "DIM_NOT_MEANINGFUL",
    "M": "DIM_NOT_MEANINGFUL",
    "H": "PASS",
}
TOKENS = list(TOKEN_MAP)
TOKEN_RE = re.compile("|".join(re.escape(t) for t in TOKENS))
COMPOUND_RE = re.compile(r"(%s)\s*/\s*(%s)" % (
    "|".join(re.escape(t) for t in TOKENS),
    "|".join(re.escape(t) for t in TOKENS),
))

ROW_RE = re.compile(r"^([A-E](?:-std)?\.\d+)\s+(.*)$")


def _scan4(text, start):
    """Read exactly 4 cell tokens starting at `start`.

    Compound tokens ('N/A N/A', '~/✗', 'M/L') are matched BEFORE single
    tokens so a compound is never split at its slash. The scan stops
    before the separator that ends the cell column.
    """
    pos = start
    toks = []
    while len(toks) < 4:
        cm = COMPOUND_RE.match(text, pos)
        if cm:
            toks.append(cm.group(0))
            pos = cm.end()
        else:
            mm = TOKEN_RE.match(text, pos)
            if not mm:
                break
            toks.append(mm.group(0))
            pos = mm.end()
        if len(toks) == 4:
            break
        if pos < len(text) and text[pos] == " ":
            pos += 1
        elif pos < len(text):
            break
    return toks, pos


def _find_cells(rest):
    """Locate the 4-token cell run in a row's text.

    The cell run is the one that is followed by two or more spaces
    (the column separator before the citation column).
    """
    found = None
    for m in TOKEN_RE.finditer(rest):
        toks, pos = _scan4(rest, m.start())
        if len(toks) != 4:
            continue
        if pos < len(rest) and not rest.startswith("  ", pos):
            continue
        found = toks
    return found


def _normalise(tok):
    cm = COMPOUND_RE.fullmatch(tok)
    if cm:
        a, b = cm.group(1), cm.group(2)
        if a in ("L", "M", "H") and b in ("L", "M", "H"):
            return "DIM_NOT_MEANINGFUL"
        return TOKEN_MAP[a]
    return TOKEN_MAP[tok]


def parse_tree_body(text):
    """Extract {row_label: [v_impl, v_usab, v_val, v_assur]} from the md.

    Overlay semantics: a row label may appear more than once (the
    Section B-std overlay restates B.3/B.4/B.6/B.7). The overlay block
    restates a row in full, so the LAST occurrence wins. Rows that are
    overlay-only (B-std) still map onto the same label space and must
    not create new intent rows.
    """
    lines = text.split("\n")
    in_tree = False
    blocks = []
    cur = None
    for ln in lines:
        if ln.startswith("## Tree"):
            in_tree = True
            continue
        if in_tree and ln.startswith("## Counts"):
            in_tree = False
        if not in_tree:
            continue
        if ln.strip().startswith("```"):
            if cur is None:
                cur = []
            else:
                blocks.append(cur)
                cur = None
            continue
        if cur is not None:
            cur.append(ln)

    occurrences = []
    for blk in blocks:
        for ln in blk:
            m = ROW_RE.match(ln)
            if not m:
                continue
            label, rest = m.group(1), m.group(2)
            toks = _find_cells(rest)
            if toks is None:
                raise SystemExit(f"unparsed tree row: {ln[:90]!r}")
            occurrences.append((label, [_normalise(t) for t in toks]))

    rows = {}
    for label, verdicts in occurrences:
        rows[label] = verdicts
    return rows, occurrences


def tally(rows):
    counts = {d: Counter() for d in DIMS}
    for verdicts in rows.values():
        for d, v in zip(DIMS, verdicts):
            counts[d][v] += 1
    return counts


def derive(rows):
    """Derive the breakdown + headline from the per-cell tree body."""
    counts = tally(rows)
    breakdown = {}
    for d in DIMS:
        c = counts[d]
        breakdown[d] = {k: c.get(k, 0) for k in VERDICTS}
        breakdown[d]["dim_not_meaningful"] = c.get("DIM_NOT_MEANINGFUL", 0)
        breakdown[d]["total"] = sum(breakdown[d].values())
    total_cells = sum(b["total"] for b in breakdown.values())
    npass = sum(b["PASS"] for b in breakdown.values())
    nm = sum(b["dim_not_meaningful"] for b in breakdown.values())
    na = sum(b["NOT_APPLICABLE"] for b in breakdown.values())
    meaningful = total_cells - nm - na
    return {
        "dimension_breakdown": breakdown,
        "total_cells": total_cells,
        "cells_pass_any": npass,
        "cells_pass_meaningful": meaningful,
    }


def md_count_table(breakdown):
    """Render the md Counts table from a breakdown."""
    order = [("PASS", "PASS"), ("PARTIAL", "PARTIAL"), ("FAIL", "FAIL"),
             ("NOT_RUN", "NOT_RUN"), ("UNKNOWN", "UNKNOWN"),
             ("STALE", "STALE"), ("NOT_APPLICABLE", "NOT_APPLICABLE"),
             ("dim_not_meaningful", "dim-not-meaningful")]
    out = ["| Verdict | Impl | Usab | Val | Assur | Total |",
           "|---|---|---|---|---|---|"]
    for key, label in order:
        vals = [breakdown[d][key] for d in DIMS]
        out.append(f"| {label} | " + " | ".join(str(v) for v in vals)
                   + f" | **{sum(vals)}** |")
    cols = [breakdown[d]["total"] for d in DIMS]
    out.append("| **Column total** | " + " | ".join(f"**{c}**" for c in cols)
               + f" | **{sum(cols)}** |")
    return "\n".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--write", action="store_true",
                    help="rewrite the JSON breakdown/headline and the md table")
    args = ap.parse_args()

    md_text = open(MD_PATH).read()
    rows, occ = parse_tree_body(md_text)
    if len(rows) != 64:
        print(f"FAIL: {len(rows)} unique rows, expected 64")
        return 1
    d = derive(rows)

    print(f"rows={len(rows)}  occurrences={len(occ)}  cells={d['total_cells']}")
    for dim in DIMS:
        b = d["dimension_breakdown"][dim]
        print(f"  {dim:15} " + " ".join(f"{k}={b[k]}" for k in JSON_KEYS))
    print(f"  PASS any        = {d['cells_pass_any']}")
    print(f"  meaningful      = {d['cells_pass_meaningful']} "
          f"(256 - nm - NOT_APPLICABLE)")
    print()

    drift = []
    js = json.load(open(JS_PATH))
    jb = js.get("dimension_breakdown", {})
    for dim in DIMS:
        for k in JSON_KEYS:
            if jb.get(dim, {}).get(k) != d["dimension_breakdown"][dim][k]:
                drift.append(f"json {dim}.{k}: {jb.get(dim,{}).get(k)} "
                             f"-> {d['dimension_breakdown'][dim][k]}")
    hc = js.get("headline_counts", {})
    if hc.get("cells_pass_any") != f"{d['cells_pass_any']}/{d['total_cells']}":
        drift.append(f"json headline cells_pass_any: "
                     f"{hc.get('cells_pass_any')} -> "
                     f"{d['cells_pass_any']}/{d['total_cells']}")
    want_mean = f"{d['cells_pass_any']}/{d['cells_pass_meaningful']}"
    if hc.get("cells_pass_meaningful") != want_mean:
        drift.append(f"json headline cells_pass_meaningful: "
                     f"{hc.get('cells_pass_meaningful')} -> {want_mean}")

    # md counts table
    table_re = re.compile(
        r"(\| Verdict \| Impl \| Usab \| Val \| Assur \| Total \|\n"
        r"\|---\|---\|---\|---\|---\|---\|\n"
        r"(?:\|[^\n]*\|\n)+)")
    mm = table_re.search(md_text)
    if not mm:
        drift.append("md: counts table not found")
    elif mm.group(0).strip() != md_count_table(d["dimension_breakdown"]).strip():
        drift.append("md: counts table differs from derived breakdown")

    # derived headline counters (must be derivable from the tree body)
    ndr_pass = [k for k in rows if k.startswith("D.")
                and "PASS" in rows[k]]
    gate_clear = []
    for k in rows:
        if not k.startswith("E."):
            continue
        vs = [v for v in rows[k] if v != "DIM_NOT_MEANINGFUL"]
        if vs and all(v == "PASS" for v in vs):
            gate_clear.append(k)
    hc_raw = js.get("headline_counts", {})
    if not hc_raw.get("ndr_items_with_target_witness", "").startswith(
            f"{len(ndr_pass)}/10"):
        drift.append(f"json ndr_items_with_target_witness: "
                     f"{hc_raw.get('ndr_items_with_target_witness', '')[:20]}... "
                     f"-> {len(ndr_pass)}/10 (derived from tree body)")
    if not hc_raw.get("gates_pass", "").startswith(f"{len(gate_clear)}/20"):
        drift.append(f"json gates_pass: "
                     f"{hc_raw.get('gates_pass', '')[:20]}... "
                     f"-> {len(gate_clear)}/20 (derived from tree body)")
    for pat, want, label in (
            (r"\*\*(\d+) / 10\*\* NDR items", len(ndr_pass), "md NDR"),
            (r"\*\*(\d+) / 20\*\* gates are clearly PASS", len(gate_clear),
             "md gates")):
        m2 = re.search(pat, md_text)
        if not m2:
            drift.append(f"{label}: headline bullet not found")
        elif int(m2.group(1)) != want:
            drift.append(f"{label}: {m2.group(1)} != {want} (derived)")

    if not drift:
        print("OK: md tree body, md counts table and json snapshot all agree.")
        return 0

    print("DRIFT:")
    for x in drift:
        print("  -", x)

    if not args.write:
        return 1

    for x in drift:
        print("FIXING:", x)

    js["dimension_breakdown"] = d["dimension_breakdown"]
    js["total_cells"] = d["total_cells"]
    hc = dict(hc)
    hc["cells_pass_any"] = f"{d['cells_pass_any']}/{d['total_cells']}"
    hc["cells_pass_meaningful"] = (
        f"{d['cells_pass_any']}/{d['cells_pass_meaningful']}")
    js["headline_counts"] = hc
    with open(JS_PATH, "w") as f:
        json.dump(js, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"wrote {JS_PATH}")

    new_table = md_count_table(d["dimension_breakdown"])
    md_text = table_re.sub(new_table + "\n", md_text, count=1)
    with open(MD_PATH, "w") as f:
        f.write(md_text)
    print(f"wrote {MD_PATH}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
