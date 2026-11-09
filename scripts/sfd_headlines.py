#!/usr/bin/env python3
"""Derived-headline helpers for the SFD cockpit validator.

`scripts/sfd_validate_counts.py` owns the parsing and counting side of the
cockpit. This module owns the *narrative* side: deriving which NDR items and
which gates actually carry a PASS, and rewriting the stored headline strings
so they cannot drift away from the per-cell tree body.

Every rewrite here is deliberately conservative. A headline is only rewritten
when its stored narrative names exactly the items the tree body derives.
When it does not, the caller is told to fail loudly rather than leave a stale
number (or a stale item list) behind.
"""

import re

# The stored headline shape: `<count>/<total> (<item list>. <narrative>)`.
# The narrative runs to the end of the string, so the closing parenthesis is
# part of `rest` and is preserved verbatim by the rewrite.
HEADLINE_RE = re.compile(
    r"^(?P<count>\d+)/(?P<total>\d+) \((?P<items>[^)]*?)\.\s*(?P<rest>.*)$", re.DOTALL
)

# The md headline bullets. The NDR bullet lists bare item names; the gates
# bullet lists per-gate descriptions, so only its count is rewritten.
#
# The pattern matches the bullet's opening parenthesis only; the *content* is
# extracted with `_balanced_group` below. A flat `[^)]*` would stop at the
# first `)`, so a description that itself contains brackets (for example
# "G9 = GitHub pull implemented (GH PR)") would truncate the captured text,
# silently drop every gate name that follows it, and make the rewrite report
# a false mismatch instead of repairing the docket.
MD_NDR_RE = re.compile(r"\*\*(?P<count>\d+) / 10\*\* NDR items \(")
MD_GATES_RE = re.compile(r"\*\*(?P<count>\d+) / 20\*\* gates are clearly PASS \(")


def _balanced_group(text, open_idx):
    """Return `(inner, end)` for the parenthesised group starting at `open_idx`.

    `open_idx` must index the opening `(`. Nested parentheses are matched, so
    a description containing brackets is captured whole rather than cut at the
    first `)`. Raises ValueError when the group is never closed.
    """
    depth = 0
    for i in range(open_idx, len(text)):
        ch = text[i]
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return text[open_idx + 1 : i], i + 1
    raise ValueError("unbalanced parenthesis in headline bullet")


def _bullet(text, regex):
    """Return `(count, inner, span)` for a headline bullet matched by `regex`.

    `span` is the `(start, end)` slice covering the whole bullet -- the
    leading `**N / M**` label through the closing parenthesis -- so a rewrite
    can replace it exactly.
    """
    m = regex.search(text)
    if not m:
        raise ValueError("headline bullet not found")
    inner, end = _balanced_group(text, m.end() - 1)
    return int(m.group("count")), inner, (m.start(), end)


def row_index(label):
    """Numeric index of a row label: `A.7` -> 7."""
    return int(label.split(".", 1)[1])


def ndr_items(rows):
    """Ordered NDR item names for the rows that carry a PASS cell."""
    return [
        f"NDR-{row_index(k)}"
        for k in sorted((k for k in rows if k.startswith("D.")), key=row_index)
        if "PASS" in rows[k]
    ]


def gate_clear(verdicts):
    """True when every meaningful cell of a gate row is PASS.

    NOT_APPLICABLE and dim-not-meaningful cells are not meaningful cells
    (see the denominator rule in PROGRESS_TREE.md), so they are excluded
    from the test instead of counting as a non-PASS cell.
    """
    vs = [v for v in verdicts if v not in ("DIM_NOT_MEANINGFUL", "NOT_APPLICABLE")]
    return bool(vs) and all(v == "PASS" for v in vs)


def clear_gates(rows):
    """Ordered gate names for the rows whose every meaningful cell is PASS."""
    return [
        f"G{row_index(k)}"
        for k in sorted((k for k in rows if k.startswith("E.")), key=row_index)
        if gate_clear(rows[k])
    ]


def rewrite_json_headline(value, total, items):
    """Rewrite a stored JSON headline so its count and item list match `items`.

    Returns the rewritten string. Raises ValueError when the stored narrative
    cannot be rewritten safely -- the caller must fail loudly rather than
    leave a stale headline behind.
    """
    m = HEADLINE_RE.match(value)
    if not m:
        raise ValueError(f"unexpected headline shape: {value[:80]!r}")
    if int(m.group("total")) != total:
        raise ValueError(f"headline denominator {m.group('total')!r} != expected {total}")
    named = [x.strip() for x in m.group("items").split(",") if x.strip()]
    if named != items:
        raise ValueError(f"headline names {named} but the tree body derives {items}")
    return f"{len(items)}/{total} ({', '.join(items)}. {m.group('rest')}"


def rewrite_md_bullets(md_text, ndr, gates):
    """Rewrite the md headline bullets' counts to match the derived numbers.

    Raises ValueError when a bullet's stored item list does not match the
    tree body, so a stale narrative is reported instead of silently kept.
    """
    _, inner, (start, end) = _bullet(md_text, MD_NDR_RE)
    named = [x.strip() for x in inner.split(",") if x.strip()]
    if named != ndr:
        raise ValueError(f"md NDR bullet names {named} but the tree body derives {ndr}")
    md_text = md_text[:start] + f"**{len(ndr)} / 10** NDR items ({', '.join(ndr)})" + md_text[end:]

    _, descriptions, (start, end) = _bullet(md_text, MD_GATES_RE)
    named = sorted(
        {"G" + g for g in re.findall(r"\bG(\d+)\b", descriptions)}, key=lambda g: int(g[1:])
    )
    if named != gates:
        raise ValueError(f"md gates bullet names {named} but the tree body derives {gates}")
    md_text = (
        md_text[:start]
        + f"**{len(gates)} / 20** gates are clearly PASS ({descriptions})"
        + md_text[end:]
    )
    return md_text
