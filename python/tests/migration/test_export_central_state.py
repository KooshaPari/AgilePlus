"""Tests for the read-only central-state export scaffold."""

from __future__ import annotations

import hashlib
import json
import sqlite3
import subprocess
import sys
from pathlib import Path

REPOSITORY_ROOT = Path(__file__).resolve().parents[3]
EXPORT_SCRIPT = REPOSITORY_ROOT / "scripts" / "export-central-state.py"


def test_export_records_source_database_sha256_without_modifying_source(tmp_path: Path) -> None:
    database_path = tmp_path / "core.db"
    connection = sqlite3.connect(database_path)
    connection.execute("CREATE TABLE features (slug TEXT PRIMARY KEY)")
    connection.execute("INSERT INTO features (slug) VALUES ('example-feature')")
    connection.commit()
    connection.close()

    expected_digest = hashlib.sha256(database_path.read_bytes()).hexdigest()
    result = subprocess.run(
        [sys.executable, str(EXPORT_SCRIPT), "--db", str(database_path)],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0, result.stderr
    export = json.loads(result.stdout)
    assert export["source_db_sha256"] == expected_digest
    assert export["source_db"] == str(database_path.resolve())
    assert export["sqlite_schema_version"] >= 0
    # The script hashes an immutable online-backup snapshot so WAL-only
    # writes do not desynchronise the digest and the schema_version read.
    assert export["snapshot_algorithm"] == "online-backup"
    assert len(export["snapshot_db_sha256"]) == 64
    assert hashlib.sha256(database_path.read_bytes()).hexdigest() == expected_digest
