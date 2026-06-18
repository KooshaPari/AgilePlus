#!/usr/bin/env python3
"""Backfill AgilePlus SQLite state from git-tracked specs.

This keeps the DB ledger closer to the repo reality without inventing runtime data by hand.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPECS_DIR = ROOT / ".agileplus" / "specs"
DB_PATH = ROOT / ".agileplus" / "agileplus.db"


FEATURE_MODULE_MAP = {
    "001-spec-driven-development-engine": (
        "core-foundation",
        "Core Foundation",
        "Foundational spec-driven workflow, domain model, CLI, and orchestration core.",
    ),
    "002-org-wide-release-governance-dx-automation": (
        "release-governance",
        "Release Governance",
        "Cross-repo release automation, DX standards, and governance gates.",
    ),
    "003-agileplus-platform-completion": (
        "platform-services",
        "Platform Services",
        "Platform infrastructure, sync, dashboard, and observability surface.",
    ),
    "004-modules-and-cycles": (
        "planning-structure",
        "Planning Structure",
        "Native modules and cycles planning model.",
    ),
    "005-heliosapp-completion": (
        "repo-completions",
        "Repo Completions",
        "Repo-specific completion and stabilization lanes.",
    ),
    "006-helioscli-completion": (
        "repo-completions",
        "Repo Completions",
        "Repo-specific completion and stabilization lanes.",
    ),
    "007-thegent-completion": (
        "repo-completions",
        "Repo Completions",
        "Repo-specific completion and stabilization lanes.",
    ),
}


FEATURE_STATUS_MAP = {
    "draft": "specified",
    "specified": "specified",
    "in_progress": "implementing",
    "implementing": "implementing",
    "planned": "planned",
    "validated": "validated",
    "shipped": "shipped",
    "retrospected": "retrospected",
}


WP_HEADING_PATTERNS = [
    re.compile(r"^## Work Package (WP\d+): (.+?)(?: \(Priority: .+\))?$"),
    re.compile(r"^### (WP\d+) [—-] (.+)$"),
    re.compile(r"^### (WP\d+): (.+)$"),
]


@dataclass
class WorkPackageSeed:
    code: str
    title: str
    state: str
    sequence: int
    acceptance_criteria: str


@dataclass
class FeatureSeed:
    slug: str
    title: str
    state: str
    module_slug: str
    module_name: str
    module_description: str
    spec_hash: bytes
    work_packages: list[WorkPackageSeed]


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def feature_state_from_spec(spec_text: str, task_done: int, task_total: int) -> str:
    match = re.search(r"^\*\*Status:\*\*\s+(.+)$", spec_text, re.MULTILINE)
    if match:
        raw = match.group(1).strip().lower().replace(" ", "_")
        if raw in FEATURE_STATUS_MAP:
            mapped = FEATURE_STATUS_MAP[raw]
            if mapped == "specified" and task_total and task_done > 0:
                return "implementing"
            return mapped
    if task_total == 0:
        return "specified"
    ratio = task_done / task_total
    if ratio == 0:
        return "planned"
    if ratio >= 0.95:
        return "validated"
    return "implementing"


def wp_state(done: int, total: int) -> str:
    if total == 0:
        return "planned"
    if done == 0:
        return "planned"
    if done == total:
        return "done"
    return "doing"


def normalize_title(raw: str) -> str:
    return re.sub(r"\s+", " ", raw.strip())


def extract_acceptance(text: str) -> str:
    lines: list[str] = []
    capture = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("**Goal**:"):
            return stripped.replace("**Goal**:", "", 1).strip()
        if stripped.startswith("**Independent Test**:"):
            return stripped.replace("**Independent Test**:", "", 1).strip()
        if stripped == "### Acceptance Scenarios":
            capture = True
            continue
        if capture:
            if stripped.startswith("### ") or stripped.startswith("## "):
                break
            if stripped:
                lines.append(stripped)
    return " ".join(lines[:3]).strip()


def parse_large_tasks(task_text: str) -> list[WorkPackageSeed]:
    headings: list[tuple[str, str, int, int]] = []
    lines = task_text.splitlines()
    for idx, line in enumerate(lines):
        for pattern in WP_HEADING_PATTERNS:
            match = pattern.match(line)
            if match:
                headings.append((match.group(1), normalize_title(match.group(2)), idx, -1))
                break
    if not headings:
        return []
    enriched: list[WorkPackageSeed] = []
    for i, (code, title, start, _) in enumerate(headings):
        end = headings[i + 1][2] if i + 1 < len(headings) else len(lines)
        section = "\n".join(lines[start:end])
        done = len(re.findall(r"^- \[x\]", section, re.MULTILINE))
        total = len(re.findall(r"^- \[(?: |x)\]", section, re.MULTILINE))
        seq_match = re.search(r"WP(\d+)", code)
        sequence = int(seq_match.group(1)) if seq_match else i + 1
        enriched.append(
            WorkPackageSeed(
                code=code,
                title=f"{code}: {title}",
                state=wp_state(done, total),
                sequence=sequence,
                acceptance_criteria=extract_acceptance(section),
            )
        )
    return enriched


def parse_small_tasks(task_text: str) -> list[WorkPackageSeed]:
    seeds: list[WorkPackageSeed] = []
    for idx, line in enumerate(task_text.splitlines(), start=1):
        match = re.match(r"^- \[(?: |x)\]\s+(.+)$", line)
        if not match:
            continue
        title = normalize_title(match.group(1))
        state = "done" if line.startswith("- [x]") else "planned"
        seeds.append(
            WorkPackageSeed(
                code=f"WP{idx:02d}",
                title=title,
                state=state,
                sequence=idx,
                acceptance_criteria="",
            )
        )
    return seeds


def load_feature_seed(spec_dir: Path) -> FeatureSeed:
    meta = json.loads((spec_dir / "meta.json").read_text())
    spec_text = (spec_dir / "spec.md").read_text()
    tasks_text = (spec_dir / "tasks.md").read_text()
    spec_hash = hashlib.sha256(spec_text.encode("utf-8")).digest()

    task_done = len(re.findall(r"^- \[x\]", tasks_text, re.MULTILINE))
    task_total = len(re.findall(r"^- \[(?: |x)\]", tasks_text, re.MULTILINE))
    state = feature_state_from_spec(spec_text, task_done, task_total)

    wp_seeds = parse_large_tasks(tasks_text)
    if not wp_seeds:
        wp_seeds = parse_small_tasks(tasks_text)

    module_slug, module_name, module_desc = FEATURE_MODULE_MAP[meta["id"]]
    return FeatureSeed(
        slug=meta["id"],
        title=meta["title"],
        state=state,
        module_slug=module_slug,
        module_name=module_name,
        module_description=module_desc,
        spec_hash=spec_hash,
        work_packages=wp_seeds,
    )


def ensure_project(conn: sqlite3.Connection) -> None:
    now = utc_now()
    conn.execute(
        """
        INSERT INTO projects (slug, name, description, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(slug) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            updated_at = excluded.updated_at
        """,
        (
            "agileplus",
            "AgilePlus",
            "Spec-driven development platform repo baseline",
            now,
            now,
        ),
    )


def ensure_module(conn: sqlite3.Connection, slug: str, name: str, description: str) -> int:
    now = utc_now()
    row = conn.execute("SELECT id FROM modules WHERE slug = ? AND parent_module_id IS NULL", (slug,)).fetchone()
    if row:
        conn.execute(
            "UPDATE modules SET friendly_name = ?, description = ?, updated_at = ? WHERE id = ?",
            (name, description, now, row[0]),
        )
        return row[0]
    cur = conn.execute(
        """
        INSERT INTO modules (slug, friendly_name, description, parent_module_id, created_at, updated_at)
        VALUES (?, ?, ?, NULL, ?, ?)
        """,
        (slug, name, description, now, now),
    )
    return int(cur.lastrowid)


def upsert_feature(conn: sqlite3.Connection, seed: FeatureSeed, module_id: int) -> int:
    now = utc_now()
    row = conn.execute("SELECT id, state FROM features WHERE slug = ?", (seed.slug,)).fetchone()
    if row:
        feature_id, existing_state = int(row[0]), row[1]
        state = existing_state if existing_state in {"shipped", "retrospected"} else seed.state
        conn.execute(
            """
            UPDATE features
            SET friendly_name = ?, state = ?, spec_hash = ?, target_branch = 'main',
                updated_at = ?, module_id = ?
            WHERE id = ?
            """,
            (seed.title, state, seed.spec_hash, now, module_id, feature_id),
        )
        return feature_id
    cur = conn.execute(
        """
        INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at, module_id)
        VALUES (?, ?, ?, ?, 'main', ?, ?, ?)
        """,
        (seed.slug, seed.title, seed.state, seed.spec_hash, now, now, module_id),
    )
    return int(cur.lastrowid)


def replace_work_packages(conn: sqlite3.Connection, feature_id: int, work_packages: list[WorkPackageSeed]) -> None:
    now = utc_now()
    conn.execute("DELETE FROM work_packages WHERE feature_id = ?", (feature_id,))
    for wp in work_packages:
        conn.execute(
            """
            INSERT INTO work_packages
                (feature_id, title, state, sequence, file_scope, acceptance_criteria, created_at, updated_at)
            VALUES (?, ?, ?, ?, '[]', ?, ?, ?)
            """,
            (feature_id, wp.title, wp.state, wp.sequence, wp.acceptance_criteria, now, now),
        )


def collect_seeds() -> list[FeatureSeed]:
    seeds = []
    for spec_dir in sorted(SPECS_DIR.iterdir()):
        if not spec_dir.is_dir():
            continue
        if spec_dir.name not in FEATURE_MODULE_MAP:
            continue
        seeds.append(load_feature_seed(spec_dir))
    return seeds


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", type=Path, default=DB_PATH)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    seeds = collect_seeds()
    if args.dry_run:
        for seed in seeds:
            print(seed.slug, seed.state, seed.module_slug, len(seed.work_packages))
        return

    conn = sqlite3.connect(args.db)
    try:
        conn.execute("PRAGMA foreign_keys = ON")
        ensure_project(conn)
        for seed in seeds:
            module_id = ensure_module(conn, seed.module_slug, seed.module_name, seed.module_description)
            feature_id = upsert_feature(conn, seed, module_id)
            replace_work_packages(conn, feature_id, seed.work_packages)
        conn.commit()
    finally:
        conn.close()


if __name__ == "__main__":
    main()
