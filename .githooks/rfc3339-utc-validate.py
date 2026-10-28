#!/usr/bin/env python3
"""Strict RFC3339 UTC timestamp validator for pre-commit updated_at audit.

Reads `id|updated_at` lines on stdin (sqlite3 separator output) and
prints only the rows whose updated_at is NOT a genuine UTC RFC3339
timestamp. Exit 0 if all pass; exit 1 if any fail; exit 0 with no
output if input is empty.

A genuine UTC RFC3339 timestamp:
  - Date:  YYYY-MM-DD with month 01-12 and day 01-31 (Feb 30 rejected;
           leap-year rules: divisible by 4 except centuries unless 400)
  - 'T' separator
  - Time:  HH:MM:SS with hour 00-23, minute 00-59, second 00-59
  - Optional fractional seconds: 1+ digits
  - Timezone: literal 'Z' (Zulu) OR '+00:00' (chrono's default offset form)
  - No trailing whitespace or other characters

This matches the agileplus producer emission. Every
`updated_at`/`created_at`/`timestamp` in
crates/agileplus-api/src/responses.rs and crates/agileplus-api/src/routes/
passes DateTime<Utc> through .to_rfc3339() before serialization, so the
emission is guaranteed UTC. chrono's default form is
`YYYY-MM-DDTHH:MM:SS.fffffffff+00:00` (nanoseconds + zero offset);
opt-in `use_z=True` form is `YYYY-MM-DDTHH:MM:SSZ`. Both pass here.

This validator does NOT rely on the host's Python fromisoformat
implementation (which differs between Python 3.9 / 3.11+); calendar
correctness is verified by an in-line leap-year and days-per-month
table for portability.
"""

import re
import sys

# Catches structural conformance with anchored end-to-end matching.
# Range constraints here are necessary; calendar correctness (Feb 30,
# Apr 31, etc.) is verified post-regex via _calendar_ok().
_STRUCT_RE = re.compile(
    r"\A"
    r"(?P<year>\d{4})-(?P<month>0[1-9]|1[0-2])-(?P<day>0[1-9]|[12]\d|3[01])"
    r"T(?P<hour>[01]\d|2[0-3]):(?P<minute>[0-5]\d):(?P<second>[0-5]\d)"
    r"(?:\.\d+)?"
    r"(?P<tz>Z|\+00:00)"
    r"\Z"
)

_DAYS_IN_MONTH_NONLEAP = (31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31)


def _is_leap(year: int) -> bool:
    return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0)


def _calendar_ok(year: int, month: int, day: int) -> bool:
    if month == 2:
        return day <= 29 if _is_leap(year) else day <= 28
    return 1 <= day <= _DAYS_IN_MONTH_NONLEAP[month - 1]


def is_utc_rfc3339(s) -> bool:
    if not isinstance(s, str):
        return False
    s = s.rstrip("\r\n")
    m = _STRUCT_RE.match(s)
    if not m:
        return False
    # Timezone is already constrained to Z or +00:00 by the regex;
    # no offset-arithmetic required. The zbus/glib emissions also produce
    # Z or +00:00 only; non-UTC offsets like +02:00 or -05:00 are rejected
    # at the regex level.
    return _calendar_ok(int(m["year"]), int(m["month"]), int(m["day"]))


def main() -> int:
    rc = 0
    for raw in sys.stdin:
        raw = raw.rstrip("\n")
        if not raw:
            continue
        if "|" not in raw:
            print(f"NO_SEPARATOR|{raw}", flush=True)
            rc = 1
            continue
        _id, _, value = raw.partition("|")
        if not is_utc_rfc3339(value):
            print(f"{_id}|{value}", flush=True)
            rc = 1
    return rc


if __name__ == "__main__":
    sys.exit(main())
