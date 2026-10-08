#!/usr/bin/env python3
"""Create and verify a transactionally consistent AgilePlus SQLite backup.

Uses sqlite3.Connection.backup() so an active WAL-mode database is copied
consistently. Does not stop or mutate the live database.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import os
from pathlib import Path
import sqlite3
import sys
import uuid


def backup_database(source: Path, output_dir: Path) -> Path:
    source = source.expanduser().resolve()
    if not source.is_file():
        raise FileNotFoundError(f"SQLite database not found: {source}")
    output_dir = output_dir.expanduser().resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    if not output_dir.is_dir():
        raise NotADirectoryError(output_dir)

    timestamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    unique_id = uuid.uuid4().hex[:8]
    filename = f"agileplus-{timestamp}-{unique_id}.sqlite3"
    temporary = output_dir / f".{filename}.incomplete"
    final = output_dir / filename

    try:
        with sqlite3.connect(f"{source.as_uri()}?mode=ro", uri=True, timeout=30) as live:
            with sqlite3.connect(temporary, timeout=30) as snapshot:
                live.backup(snapshot, pages=64, sleep=0.1)
                result = snapshot.execute("PRAGMA integrity_check").fetchone()
                if result is None or result[0] != "ok":
                    raise RuntimeError(f"backup integrity check failed: {result}")
        if os.name == "posix":
            temporary.chmod(0o600)
        with temporary.open("rb") as file:
            os.fsync(file.fileno())
        os.replace(temporary, final)
        if os.name == "posix":
            descriptor = os.open(output_dir, os.O_RDONLY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
        return final
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def verify_backup(path: Path) -> None:
    path = path.expanduser().resolve()
    if not path.is_file():
        raise FileNotFoundError(path)
    with sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True) as connection:
        result = connection.execute("PRAGMA integrity_check").fetchone()
        if result is None or result[0] != "ok":
            raise RuntimeError(f"invalid backup: {result}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    arguments = parser.parse_args()
    try:
        backup = backup_database(arguments.database, arguments.output_dir)
        verify_backup(backup)
    except (OSError, sqlite3.Error, RuntimeError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print(f"PASS: verified AgilePlus SQLite backup: {backup}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
