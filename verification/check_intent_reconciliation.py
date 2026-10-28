"""Validate the supplemental source crosswalk; requires Python jsonschema.

Checks documentation consistency only, not runtime product acceptance.
"""

import hashlib
import json
import re
import subprocess
from pathlib import Path
from urllib.parse import urlparse

import jsonschema

root = Path(__file__).resolve().parents[1]
folder = root / "spec/product/recovery-2026-10-14"


def unique_pairs(pairs):
    d = {}
    for k, v in pairs:
        assert k not in d, f"duplicate JSON key: {k}"
        d[k] = v
    return d


def read(p):
    return json.loads(p.read_text(), object_pairs_hook=unique_pairs)


cross = read(folder / "requirement-crosswalk.v1.json")
schema = read(folder / "requirement-crosswalk.schema.json")
jsonschema.Draft202012Validator.check_schema(schema)
jsonschema.validate(cross, schema)
d = {}
for name, ref in cross["canonical_inputs"].items():
    p = root / ref["path"]
    assert hashlib.sha256(p.read_bytes()).hexdigest() == ref["sha256"], p
    assert (
        subprocess.check_output(
            ["git", "show", cross["base_revision"] + ":" + ref["path"]], cwd=root
        )
        == p.read_bytes()
    ), p
    d[name] = read(p)
r = d["requirements"]["requirements"]
j = d["journeys"]["journeys"]
c = d["oracles"]["cases"]
rows = cross["mappings"]
for records in [r, j, c]:
    assert len({x["id"] for x in records}) == len(records)
assert len({x["requirement_id"] for x in rows}) == len(rows)
assert {x["requirement_id"] for x in rows} == {x["id"] for x in r}
assert cross["derived_counts"] == {
    "requirements": len(r),
    "journeys": len(j),
    "oracles": len(c),
}
m = d["manifest"]
assert m["canonical_spine"]["derived_requirement_count"] == len(r)
assert m["journeys"]["count"] == len(j)
assert m["verification"]["case_count"] == len(c)
assert all(x["status"] == "specified_not_executed" for x in c)
by_id = {x["id"]: x for x in r}
journeys = {x["id"] for x in j}
for case in c:
    assert case["journey_id"] in journeys
    if "requirement_id" in case:
        assert case["requirement_id"] in by_id
sources = set(re.findall(r"^## (S\d+) ", (folder / "OWNER_EXCERPTS.md").read_text(), re.MULTILINE))
assert len(sources) == 9
for row in rows:
    req = by_id[row["requirement_id"]]
    assert row["title"] == req["title"]
    assert row["journey_ids"] == req["journeys"]
    assert set(row["source_refs"]) <= sources
    assert set(req["journeys"]) <= journeys
    assert row["direct_oracle_ids"] == [x["id"] for x in c if x.get("requirement_id") == req["id"]]
    assert row["journey_context_oracle_ids"] == [
        x["id"] for x in c if not x.get("requirement_id") and x["journey_id"] in req["journeys"]
    ]
files = [folder / p for p in ["README.md", "OWNER_EXCERPTS.md", "FUTURE_WORK.md"]] + [
    root / "docs/adr/proposed-product-work-simulation-boundary.md",
    root / "spec/product/AUTHORITY_INDEX.md",
    root / "spec/product/NON_CODE_FINALITY_LEDGER.md",
]
links = 0
for p in files:
    for requirement_ref in re.findall(r"AP-R-[A-Z0-9-]+", p.read_text()):
        assert requirement_ref in by_id, (p, requirement_ref)
    for target in re.findall(r"\]\(([^)]+)\)", p.read_text()):
        if target.startswith("https://"):
            u = urlparse(target)
            assert u.netloc == "chatgpt.com" and u.path.startswith("/c/")
            links += 1
        elif not target.startswith("#"):
            assert (p.parent / target.split("#")[0]).resolve().exists(), (p, target)
            links += 1
text = "\n".join(p.read_text() for p in files) + json.dumps(cross)
for pattern in [
    r"(?i)employment",
    r"(?i)raw transcript",
    r"libfile_",
    r"Sentinel_",
    r"(?i)completion_token",
    r"(?i)download_url",
    r"(?i)GH_TOKEN",
]:
    assert not re.search(pattern, text), pattern
print(
    "PASS: JSON schema and duplicate-key checks; exact canonical hashes/base bytes; unique requirement/journey/oracle IDs"
)
print(
    f"PASS: {len(rows)} requirement mappings; {len(j)} journeys; {len(c)} specified cases; exact direct and journey-context references"
)
print(
    f"PASS: {len(sources)} source anchors; {links} local/source link targets structurally valid; source URLs not remotely read back"
)
print(
    "PASS: privacy exclusion scan; canonical runtime statuses unchanged. Quote fidelity and diff privacy require human-style review."
)
