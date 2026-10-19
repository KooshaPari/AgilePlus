#!/usr/bin/env python3
"""Unified per-cell classifier + cross-source reconciliation for the SFD
cockpit.

Sources
-------
1. PROGRESS_TREE_RAW.md  (recovered from orphaned git blob c46a4eed)
   Per-cell evidence harvest at identity ap-sfd-2026-10-17T22-14Z@d6b418a6.
   Every cell carries a verdict + citation.
2. PROGRESS_TREE.md      (rendered tree body)
   Per-cell cockpit at the current identity, hand-updated across cohorts.
3. PROGRESS_TREE.snapshot.json
   Aggregate counters only (dimension_breakdown + headline_counts).
   NO per-cell inventory -- so it can be checked against (1) and (2) but
   cannot be diffed row-by-row.

The question this answers: does the md tree body faithfully reflect the raw
evidence + documented cohort movements? If yes, md-derived counts are the
authoritative ones and the JSON counters are the artefact that drifted.
"""
import json
import re
import subprocess
import sys
from collections import Counter

DIMS = ["Implementation", "Usability", "Validation", "Assurance"]

VERDICT_WORDS = ("PASS", "FAIL", "NOT_RUN", "UNKNOWN", "STALE", "BLOCKED",
                 "NOT_APPLICABLE")
STRENGTHS = ("LOW", "MEDIUM", "HIGH")

# md tree token -> canonical verdict
MD_MAP = {
    "✓": "PASS", "~": "PARTIAL", "✗": "FAIL", "…": "NOT_RUN",
    "?": "UNKNOWN", "S": "STALE", "B": "BLOCKED", "N/A": "NOT_APPLICABLE",
    "_": "DIM_NOT_MEANINGFUL",
    "H": "PASS", "L": "DIM_NOT_MEANINGFUL", "M": "DIM_NOT_MEANINGFUL",
}

MD_TOKENS = ["N/A", "✓", "~", "✗", "…", "?", "S", "B", "_", "L", "H", "M"]
MD_TOKEN_RE = re.compile("|".join(re.escape(t) for t in MD_TOKENS))
MD_COMPOUND_RE = re.compile(
    r"(N/A|[✓~✗…?SB_HLM])\s*/\s*(N/A|[✓~✗…?SB_HLM])")


# --------------------------------------------------------------------------
# Raw-file classification
# --------------------------------------------------------------------------
def classify_raw(cell: str):
    """Classify a raw-file cell string.

    Raw cells look like:
        PASS
        PASS (cases specified)
        NOT_RUN
        LOW
        MEDIUM (CI) / LOW (prod)
        PASS in-CI / FAIL in-prod
        PARTIAL (Plane) / FAIL (rest)
        PARTIAL in-CI / STALE today
        n/a
    A compound resolves to its FIRST verdict (the row's own surface); a
    strength-vs-strength compound resolves to dim_not_meaningful.
    """
    c = (cell or "").strip()
    if not c:
        return "EMPTY"
    # "n/a" is the raw file's spelling for "dimension not meaningful".
    # It must be handled BEFORE the compound split, otherwise "n/a" is read
    # as the compound "n" / "a".
    if c.lower() in ("n/a", "na", "not applicable"):
        return "DIM_NOT_MEANINGFUL"
    # split on a compound boundary
    comp = re.match(r"^(.*?)\s*/\s*(.*)$", c)
    if comp:
        first = classify_raw(comp.group(1))
        second = classify_raw(comp.group(2))
        if first in ("LOW", "MEDIUM", "HIGH") and second in ("LOW", "MEDIUM", "HIGH"):
            return "DIM_NOT_MEANINGFUL"
        return first

    # strip parenthetical / trailing qualifiers
    base = re.split(r"\s*\(", c, maxsplit=1)[0].strip()
    up = base.upper()
    if up in VERDICT_WORDS:
        return up
    if up == "PARTIAL":
        return "PARTIAL"
    if up in STRENGTHS:
        # Assurance strength markers. The raw file's own G.1 tally resolves
        # LOW/MEDIUM as "dim not meaningful" and HIGH as a PASS verdict.
        # (Verified: raw G.1 Assurance = PASS 3 / nm 61, and exactly 3 rows
        # carry a HIGH marker: D.1, D.2, E.4.)
        return "PASS" if up == "HIGH" else "DIM_NOT_MEANINGFUL"
    # raw file writes "n/a" for "dimension not meaningful" (NOT the same as
    # NOT_APPLICABLE). Check this BEFORE the NOT_APPLICABLE branch because
    # "n/a".upper() == "N/A" which is NOT_APPLICABLE's spelling.
    if base.lower() in ("n/a", "not applicable", "na"):
        return "DIM_NOT_MEANINGFUL"
    if up == "NOT_APPLICABLE":
        return "NOT_APPLICABLE"
    # "PASS in-CI" style
    m = re.match(r"^(PASS|FAIL|NOT_RUN|UNKNOWN|STALE|BLOCKED|NOT_APPLICABLE|PARTIAL)\b", up)
    if m:
        return m.group(1)
    return "UNPARSED:" + c


def parse_raw(path="docs/sessions/20261017-state-forward-docket/PROGRESS_TREE_RAW.md"):
    rows = {}
    for ln in open(path):
        if not ln.startswith("|"):
            continue
        cells = [c.strip() for c in ln.strip().strip("|").split("|")]
        if len(cells) < 6:
            continue
        label = cells[0]
        if not re.match(r"^[A-E]\.\d+$", label):
            continue
        rows[label] = [classify_raw(c) for c in cells[2:6]]
    return rows


# --------------------------------------------------------------------------
# md tree-body classification
# --------------------------------------------------------------------------
def _scan4(text, start):
    pos = start
    toks = []
    while len(toks) < 4:
        cm = MD_COMPOUND_RE.match(text, pos)
        if cm:
            toks.append(cm.group(0))
            pos = cm.end()
        else:
            mm = MD_TOKEN_RE.match(text, pos)
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


def _find_run(rest):
    best = None
    for m in MD_TOKEN_RE.finditer(rest):
        toks, pos = _scan4(rest, m.start())
        if len(toks) != 4:
            continue
        if pos < len(rest) and not rest.startswith("  ", pos):
            continue
        best = toks
    return best


def parse_md(path="docs/sessions/20261017-state-forward-docket/PROGRESS_TREE.md"):
    lines = open(path).read().split("\n")
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

    occ = []
    for blk in blocks:
        for ln in blk:
            m = re.match(r"^([A-E](?:-std)?\.\d+)\s+(.*)$", ln)
            if not m:
                continue
            label, rest = m.group(1), m.group(2)
            toks = _find_run(rest)
            if toks is None:
                continue
            norm = []
            for t in toks:
                cm = MD_COMPOUND_RE.fullmatch(t)
                if cm:
                    a, b = cm.group(1), cm.group(2)
                    if a in ("L", "M", "H") and b in ("L", "M", "H"):
                        norm.append("_")
                    else:
                        norm.append(a)
                else:
                    norm.append(t)
            occ.append((label, [MD_MAP[t] for t in norm]))

    rows = {}
    for label, verdicts in occ:
        rows[label] = verdicts       # overlay (later occurrence) wins
    return rows, occ


# --------------------------------------------------------------------------
def tally(rows):
    counts = {d: Counter() for d in DIMS}
    for label, verdicts in rows.items():
        for d, v in zip(DIMS, verdicts):
            counts[d][v] += 1
    return counts


def show(name, counts):
    print(f"--- {name} ---")
    total = 0
    npass = 0
    for d in DIMS:
        c = counts[d]
        t = sum(c.values())
        total += t
        npass += c.get("PASS", 0)
        nm = c.get("DIM_NOT_MEANINGFUL", 0)
        print(f"  {d:15} n={t:3} PASS={c.get('PASS',0):2} PARTIAL={c.get('PARTIAL',0):2} "
              f"FAIL={c.get('FAIL',0):2} NOT_RUN={c.get('NOT_RUN',0):2} "
              f"UNKNOWN={c.get('UNKNOWN',0):2} STALE={c.get('STALE',0):2} "
              f"NA={c.get('NOT_APPLICABLE',0):2} nm={nm:2}")
    nm = sum(counts[d].get("DIM_NOT_MEANINGFUL", 0) for d in DIMS)
    na = sum(counts[d].get("NOT_APPLICABLE", 0) for d in DIMS)
    # "meaningful" = cells that carry a verdict for the dimension, i.e.
    # everything that is neither dim_not_meaningful nor NOT_APPLICABLE.
    # This must match sfd_validate_counts.py exactly.
    meaningful = total - nm - na
    print(f"  TOTAL={total}  PASS={npass}  meaningful={meaningful} "
          f"(= {total} - nm {nm} - NA {na})")
    return total, npass, meaningful


def main():
    raw = parse_raw()
    md, occ = parse_md()
    print(f"raw rows={len(raw)}  md rows={len(md)}  md occurrences={len(occ)}")
    print()

    raw_c = tally(raw)
    md_c = tally(md)
    show("RAW (blob c46a4eed, ap-sfd-2026-10-17T22-14Z)", raw_c)
    print()
    show("MD tree body (current worktree)", md_c)
    print()

    print("=== per-cell diff: raw -> md ===")
    changed = 0
    for label in sorted(set(raw) | set(md)):
        r = raw.get(label)
        m = md.get(label)
        if r is None:
            print(f"  {label}: MISSING in raw; md={m}")
            continue
        if m is None:
            print(f"  {label}: MISSING in md; raw={r}")
            continue
        for d, rv, mv in zip(DIMS, r, m):
            if rv != mv:
                print(f"  {label}.{d}: {rv} -> {mv}")
                changed += 1
    print(f"  changed cells = {changed}")

    js = json.load(open(
        "docs/sessions/20261017-state-forward-docket/PROGRESS_TREE.snapshot.json"))
    jb = js["dimension_breakdown"]
    print("\n=== json dimension_breakdown ===")
    for d in DIMS:
        print(f"  {d:15} {json.dumps(jb[d])}")
    hc = js["headline_counts"]
    print(f"  headline: {hc['cells_pass_any']} / {hc['cells_pass_meaningful']}")


if __name__ == "__main__":
    main()
