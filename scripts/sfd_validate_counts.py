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
import re
import shutil
import subprocess
import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import sfd_headlines as headlines

DOC = "docs/sessions/20261017-state-forward-docket"
MD_PATH = f"{DOC}/PROGRESS_TREE.md"
JS_PATH = f"{DOC}/PROGRESS_TREE.snapshot.json"

DIMS = ["Implementation", "Usability", "Validation", "Assurance"]

VERDICTS = ["PASS", "PARTIAL", "FAIL", "NOT_RUN", "UNKNOWN", "STALE", "BLOCKED", "NOT_APPLICABLE"]
JSON_KEYS = [
    "PASS",
    "PARTIAL",
    "FAIL",
    "NOT_RUN",
    "UNKNOWN",
    "STALE",
    "BLOCKED",
    "NOT_APPLICABLE",
    "dim_not_meaningful",
    "total",
]

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
_TOK_ALT = "|".join(re.escape(t) for t in TOKENS)
COMPOUND_RE = re.compile(rf"({_TOK_ALT})\s*/\s*({_TOK_ALT})")

ROW_RE = re.compile(r"^([A-E](?:-std)?\.\d+)\s+(.*)$")

# The 64 intent rows the tree must contain: A.1-A.10, B.1-B.12, C.1-C.12,
# D.1-D.10, E.1-E.20. A mis-typed label (E.21 for E.19) keeps the row count at
# 64 while a gate row silently carries the wrong identity, so the labels are
# compared against this grid and not just counted.
EXPECTED_ROWS = (
    [f"A.{i}" for i in range(1, 11)]
    + [f"B.{i}" for i in range(1, 13)]
    + [f"C.{i}" for i in range(1, 13)]
    + [f"D.{i}" for i in range(1, 11)]
    + [f"E.{i}" for i in range(1, 21)]
)

# The md Counts table. `BLOCKED` is a verdict in its own right: when a cell
# carries it, the row must be shown, otherwise the column total includes a
# verdict the table never displays.
MD_COUNTS_TABLE_RE = re.compile(
    r"(\|\s*Verdict\s*\|\s*Impl\s*\|\s*Usab\s*\|\s*Val\s*\|\s*Assur\s*\|\s*Total\s*\|\n"
    r"\|[\s\-:|]*\|\n"
    r"(?:\|[^\n]*\|\n)+)"
)


def _normalise_label(label):
    """Map an overlay row label onto the canonical intent-row label.

    An overlay block may restate a row as `B-std.3`; that is the same intent
    row as `B.3` and must update it rather than create a new one.
    """
    m = re.match(r"^([A-E])(?:-std)?\.(\d+)$", label)
    return f"{m.group(1)}.{m.group(2)}" if m else label


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

    The cell run is the FIRST run of 4 verdict tokens that is followed by two
    or more spaces (the column separator before the citation column). The scan
    stops there: everything after that separator is the citation column, which
    may itself contain verdict-looking tokens (for example a quoted historical
    tuple) and must never be mistaken for the row's current verdicts.
    """
    for m in TOKEN_RE.finditer(rest):
        toks, pos = _scan4(rest, m.start())
        if len(toks) != 4:
            continue
        if pos < len(rest) and not rest.startswith("  ", pos):
            continue
        return toks
    return None


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
            label = _normalise_label(label)
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


def _table_rows(text):
    """Parse a markdown table into normalized rows.

    Prettier re-pads every cell and rewrites the separator row, so comparing
    raw text would make this validator and `prettier --check` fight each other.
    Comparing parsed cells instead keeps the two independent: any padding or
    `**bold**` markup is ignored, only the values have to agree.
    """
    out = []
    for ln in text.strip().splitlines():
        ln = ln.strip()
        if not ln.startswith("|"):
            continue
        cells = [c.strip().strip("*").strip() for c in ln.strip("|").split("|")]
        if cells and all(set(c) <= set("-: ") for c in cells):
            continue  # separator row
        out.append(cells)
    return out


def md_count_table(breakdown):
    """Render the md Counts table from a breakdown."""
    order = [
        ("PASS", "PASS"),
        ("PARTIAL", "PARTIAL"),
        ("FAIL", "FAIL"),
        ("NOT_RUN", "NOT_RUN"),
        ("UNKNOWN", "UNKNOWN"),
        ("STALE", "STALE"),
        ("BLOCKED", "BLOCKED"),
        ("NOT_APPLICABLE", "NOT_APPLICABLE"),
        ("dim_not_meaningful", "dim-not-meaningful"),
    ]
    out = ["| Verdict | Impl | Usab | Val | Assur | Total |", "|---|---|---|---|---|---|"]
    for key, label in order:
        vals = [breakdown[d][key] for d in DIMS]
        out.append(f"| {label} | " + " | ".join(str(v) for v in vals) + f" | **{sum(vals)}** |")
    cols = [breakdown[d]["total"] for d in DIMS]
    out.append(
        "| **Column total** | " + " | ".join(f"**{c}**" for c in cols) + f" | **{sum(cols)}** |"
    )
    return "\n".join(out)


def collect_drift(md_text, rows, d, js):
    """Return every inconsistency between the tree body, the md table and the JSON.

    The tree body is the source of truth; everything else is checked against
    what it derives.
    """
    drift = []
    jb = js.get("dimension_breakdown", {})
    for dim in DIMS:
        for k in JSON_KEYS:
            if jb.get(dim, {}).get(k) != d["dimension_breakdown"][dim][k]:
                drift.append(
                    f"json {dim}.{k}: {jb.get(dim, {}).get(k)} "
                    f"-> {d['dimension_breakdown'][dim][k]}"
                )
    hc = js.get("headline_counts", {})
    if hc.get("cells_pass_any") != f"{d['cells_pass_any']}/{d['total_cells']}":
        drift.append(
            f"json headline cells_pass_any: "
            f"{hc.get('cells_pass_any')} -> "
            f"{d['cells_pass_any']}/{d['total_cells']}"
        )
    want_mean = f"{d['cells_pass_any']}/{d['cells_pass_meaningful']}"
    if hc.get("cells_pass_meaningful") != want_mean:
        drift.append(
            f"json headline cells_pass_meaningful: {hc.get('cells_pass_meaningful')} -> {want_mean}"
        )

    # md counts table. Tolerant of prettier's padding and separator style.
    mm = MD_COUNTS_TABLE_RE.search(md_text)
    if not mm:
        drift.append("md: counts table not found")
    elif _table_rows(mm.group(0)) != _table_rows(md_count_table(d["dimension_breakdown"])):
        drift.append("md: counts table differs from derived breakdown")

    # derived headline counters (must be derivable from the tree body)
    ndr = headlines.ndr_items(rows)
    gates = headlines.clear_gates(rows)
    hc_raw = js.get("headline_counts", {})
    # Compare the stored headline NAMES, not just the count. A tree edit can
    # swap which items carry PASS without changing any count; then the stored
    # names go stale and the snapshot contradicts the tree body while the
    # headline still reads "3/10". Reporting only the count prefix would let
    # that pass validation and leave the docket self-contradicting.
    for key, total, derived, label in (
        ("ndr_items_with_target_witness", 10, ndr, "json NDR"),
        ("gates_pass", 20, gates, "json gates"),
    ):
        stored = hc_raw.get(key, "")
        m = headlines.HEADLINE_RE.match(stored)
        if not m:
            drift.append(f"{label}: unexpected headline shape in {key}: {stored[:60]!r}")
            continue
        if int(m.group("count")) != len(derived):
            drift.append(
                f"{label}: stored count {m.group('count')} != derived {len(derived)} "
                f"(derived from tree body)"
            )
        if int(m.group("total")) != total:
            drift.append(f"{label}: stored denominator {m.group('total')} != expected {total}")
        named = [x.strip() for x in m.group("items").split(",") if x.strip()]
        if named != derived:
            drift.append(f"{label}: stored names {named} != derived {derived}")
    for pat, want, label in (
        (r"\*\*(\d+) / 10\*\* NDR items", len(ndr), "md NDR"),
        (r"\*\*(\d+) / 20\*\* gates are clearly PASS", len(gates), "md gates"),
    ):
        m2 = re.search(pat, md_text)
        if not m2:
            drift.append(f"{label}: headline bullet not found")
        elif int(m2.group(1)) != want:
            drift.append(f"{label}: {m2.group(1)} != {want} (derived)")
    return drift


def main():
    """Validate the three cockpit artefacts; with --write, rewrite and re-check them."""
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--write", action="store_true", help="rewrite the JSON breakdown/headline and the md table"
    )
    args = ap.parse_args()

    md_text = Path(MD_PATH).read_text()
    rows, occ = parse_tree_body(md_text)
    if len(rows) != len(EXPECTED_ROWS):
        print(f"FAIL: {len(rows)} unique rows, expected {len(EXPECTED_ROWS)}")
        return 1
    if sorted(rows) != sorted(EXPECTED_ROWS):
        missing = sorted(set(EXPECTED_ROWS) - set(rows))
        extra = sorted(set(rows) - set(EXPECTED_ROWS))
        print("FAIL: row identities do not match the expected A-E intent grid")
        if missing:
            print("  missing:", ", ".join(missing))
        if extra:
            print("  unexpected:", ", ".join(extra))
        print(
            "  (a mis-typed label keeps the row count at "
            f"{len(EXPECTED_ROWS)} while a row silently carries the wrong identity)"
        )
        return 1
    d = derive(rows)

    print(f"rows={len(rows)}  occurrences={len(occ)}  cells={d['total_cells']}")
    for dim in DIMS:
        b = d["dimension_breakdown"][dim]
        print(f"  {dim:15} " + " ".join(f"{k}={b[k]}" for k in JSON_KEYS))
    print(f"  PASS any        = {d['cells_pass_any']}")
    print(f"  meaningful      = {d['cells_pass_meaningful']} (256 - nm - NOT_APPLICABLE)")
    print()

    js = json.loads(Path(JS_PATH).read_text())
    drift = collect_drift(md_text, rows, d, js)
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

    # The derived NDR / gate headlines carry narrative prose as well as a
    # count. Rewrite them from the tree body, or fail loudly: leaving a stale
    # headline behind and exiting 0 would report success while the docket
    # still contradicts itself.
    ndr = headlines.ndr_items(rows)
    gates = headlines.clear_gates(rows)
    try:
        js.setdefault("headline_counts", {})["ndr_items_with_target_witness"] = (
            headlines.rewrite_json_headline(
                js.get("headline_counts", {}).get("ndr_items_with_target_witness", ""), 10, ndr
            )
        )
        js["headline_counts"]["gates_pass"] = headlines.rewrite_json_headline(
            js.get("headline_counts", {}).get("gates_pass", ""), 20, gates
        )
        md_text = headlines.rewrite_md_bullets(md_text, ndr, gates)
    except ValueError as exc:
        print(f"FAIL: cannot rewrite the derived headlines: {exc}")
        print("       the stored narrative does not match the tree body; fix it by hand")
        return 1

    js["dimension_breakdown"] = d["dimension_breakdown"]
    js["total_cells"] = d["total_cells"]
    hc = dict(js["headline_counts"])
    hc["cells_pass_any"] = f"{d['cells_pass_any']}/{d['total_cells']}"
    hc["cells_pass_meaningful"] = f"{d['cells_pass_any']}/{d['cells_pass_meaningful']}"
    js["headline_counts"] = hc
    with open(JS_PATH, "w") as f:
        json.dump(js, f, indent=2, ensure_ascii=False)
        f.write("\n")
    print(f"wrote {JS_PATH}")

    new_table = md_count_table(d["dimension_breakdown"])
    md_text = MD_COUNTS_TABLE_RE.sub(new_table + "\n", md_text, count=1)
    with open(MD_PATH, "w") as f:
        f.write(md_text)
    print(f"wrote {MD_PATH}")

    # md_count_table renders unpadded cells; the repo formats markdown with
    # prettier, which re-pads every table cell. Run it here so a --write
    # followed by `prettier --check` is stable instead of always dirty.
    prettier = shutil.which("prettier")
    if prettier:
        rc = subprocess.run(
            [prettier, "--write", MD_PATH],
            capture_output=True,
            text=True,
            check=False,
        )
        if rc.returncode != 0:
            print(f"WARN: prettier --write {MD_PATH} failed: {rc.stderr.strip()[:200]}")
        else:
            print(f"prettier --write {MD_PATH}: OK")
    else:
        print(
            "NOTE: prettier not on PATH; run `prettier --write "
            f"{MD_PATH}` so the markdown matches the repo's format check."
        )

    # Re-check: --write must not report success while drift survives. The
    # artefacts are re-read from disk so the check covers what was written.
    md_after = Path(MD_PATH).read_text()
    rows_after, _ = parse_tree_body(md_after)
    js_after = json.loads(Path(JS_PATH).read_text())
    residual = collect_drift(md_after, rows_after, derive(rows_after), js_after)
    if residual:
        print("FAIL: drift remains after --write:")
        for x in residual:
            print("  -", x)
        return 1
    print("OK: md tree body, md counts table and json snapshot all agree after --write.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
