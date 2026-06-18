# SPDX-License-Identifier: MIT OR Apache-2.0
#!/usr/bin/env python3
"""Phase 3 docs consolidation: merge kitty-specs/ + specs/ into docs/ tree."""

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
KITTY = ROOT / "kitty-specs"
SPECS = ROOT / "specs"
DOCS = ROOT / "docs"
DOCS_SPECS = DOCS / "specs"
DOCS_JOURNEYS = DOCS / "journeys"
ARCHIVE_META = DOCS / "_archive" / "meta-json"
ARCHIVE_TRACES = DOCS / "_archive" / "traces-json"
TRACES = ROOT / "traces"
OPS_JOURNEYS = DOCS / "operations" / "journeys"

FRONTMATTER_RE = re.compile(r"^---\s*\n(.*?)\n---\s*\n", re.DOTALL)


def parse_frontmatter(text: str) -> tuple[dict[str, Any], str]:
    match = FRONTMATTER_RE.match(text)
    if not match:
        return {}, text
    body = text[match.end() :]
    fm: dict[str, Any] = {}
    for line in match.group(1).splitlines():
        if not line.strip() or line.strip().startswith("#"):
            continue
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        fm[key.strip()] = value.strip()
    return fm, body


def render_frontmatter(fm: dict[str, Any]) -> str:
    lines = ["---"]
    for key in sorted(fm.keys()):
        value = fm[key]
        if value is None:
            lines.append(f"{key}: null")
        elif isinstance(value, list):
            lines.append(f"{key}:")
            for item in value:
                lines.append(f"  - {item}")
        else:
            lines.append(f"{key}: {value}")
    lines.append("---")
    lines.append("")
    return "\n".join(lines)


def merge_meta_into_spec(spec_dir: Path, meta: dict[str, Any]) -> bool:
    spec_path = spec_dir / "spec.md"
    if not spec_path.is_file():
        return False
    text = spec_path.read_text(encoding="utf-8")
    fm, body = parse_frontmatter(text)
    for key, value in meta.items():
        if key == "_path":
            continue
        if key not in fm or fm[key] in ("", "-", None):
            fm[key] = value
    # Normalize canonical keys
    if "status" in meta and "state" not in fm:
        fm["state"] = str(meta["status"]).upper()
    if "created_at" in meta and "created" not in fm:
        fm["created"] = meta["created_at"]
    spec_path.write_text(render_frontmatter(fm) + body.lstrip("\n"), encoding="utf-8")
    return True


def fold_trace_into_journey(trace_path: Path) -> bool:
    data = json.loads(trace_path.read_text(encoding="utf-8"))
    fr_id = data.get("fr_id", trace_path.stem)
    journey_candidates = [
        OPS_JOURNEYS / f"{fr_id}.md",
        DOCS_JOURNEYS / f"{fr_id}.md",
    ]
    journey_path = next((p for p in journey_candidates if p.is_file()), None)
    if journey_path is None:
        journey_path = DOCS_JOURNEYS / f"{fr_id}.md"
        journey_path.parent.mkdir(parents=True, exist_ok=True)
        journey_path.write_text(
            f"# Journey: {fr_id}\n\n> Migrated from `{trace_path.relative_to(ROOT)}`.\n",
            encoding="utf-8",
        )
    text = journey_path.read_text(encoding="utf-8")
    fm, body = parse_frontmatter(text)
    for key in (
        "fr_id",
        "spec_slug",
        "spec_anchor",
        "docs_pages",
        "tests",
        "code_modules",
        "journeys",
        "status",
        "last_validated",
        "schema_version",
    ):
        if key in data:
            fm[key] = data[key]
    journey_path.write_text(
        render_frontmatter(fm) + body.lstrip("\n"), encoding="utf-8"
    )
    return True


def move_tree(src: Path, dst: Path) -> int:
    if not src.exists():
        return 0
    count = 0
    dst.parent.mkdir(parents=True, exist_ok=True)
    if src.is_dir() and not dst.exists():
        shutil.move(str(src), str(dst))
        return 1
    if src.is_dir():
        for child in sorted(src.iterdir()):
            count += move_tree(child, dst / child.name)
        if src.is_dir() and not any(src.iterdir()):
            src.rmdir()
        return count
    if src.is_file():
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(src), str(dst))
        return 1
    return count


def main() -> None:
    DOCS_SPECS.mkdir(parents=True, exist_ok=True)
    ARCHIVE_META.mkdir(parents=True, exist_ok=True)
    ARCHIVE_TRACES.mkdir(parents=True, exist_ok=True)

    meta_count = 0
    if KITTY.is_dir():
        for spec_dir in sorted(KITTY.iterdir()):
            if not spec_dir.is_dir():
                continue
            meta_path = spec_dir / "meta.json"
            if meta_path.is_file():
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
                merge_meta_into_spec(spec_dir, meta)
                archive_dest = ARCHIVE_META / spec_dir.name / "meta.json"
                archive_dest.parent.mkdir(parents=True, exist_ok=True)
                shutil.move(str(meta_path), str(archive_dest))
                meta_count += 1
        move_tree(KITTY, DOCS_SPECS / "eco")
        # Preserve INDEX if present
        index_src = DOCS_SPECS / "eco" / "INDEX.md"
        if index_src.is_file():
            index_src.rename(DOCS_SPECS / "INDEX-eco.md")

    if SPECS.is_dir():
        move_tree(SPECS, DOCS_SPECS / "crates")

    trace_count = 0
    if TRACES.is_dir():
        for trace_path in sorted(TRACES.glob("FR-*.json")):
            fold_trace_into_journey(trace_path)
            archive_dest = ARCHIVE_TRACES / trace_path.name
            shutil.move(str(trace_path), str(archive_dest))
            trace_count += 1
        # Move SCHEMA/MATRIX docs to docs/requirements
        req_dir = DOCS / "requirements" / "traceability"
        req_dir.mkdir(parents=True, exist_ok=True)
        for name in ("SCHEMA.md", "MATRIX.md"):
            src = TRACES / name
            if src.is_file():
                shutil.move(str(src), str(req_dir / name))

    if OPS_JOURNEYS.is_dir():
        for journey in sorted(OPS_JOURNEYS.glob("*.md")):
            dest = DOCS_JOURNEYS / journey.name
            if dest.exists():
                continue
            shutil.move(str(journey), str(dest))

    print(f"meta_json_merged={meta_count}")
    print(f"traces_folded={trace_count}")
    print(f"docs_specs_eco={DOCS_SPECS / 'eco'}")
    print(f"docs_specs_crates={DOCS_SPECS / 'crates'}")


if __name__ == "__main__":
    main()
