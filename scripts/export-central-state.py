#!/usr/bin/env python3
"""Emit read-only provenance for an explicitly supplied central-state database."""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import sys
from pathlib import Path


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--db",
        type=Path,
        required=True,
        help="explicit SQLite database to inspect without write access",
    )
    return parser.parse_args()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def inspect_read_only_database(path: Path) -> int:
    connection = sqlite3.connect(f"{path.as_uri()}?mode=ro", uri=True)
    try:
        row = connection.execute("PRAGMA schema_version").fetchone()
    finally:
        connection.close()
    return int(row[0])


def main() -> int:
    arguments = parse_arguments()
    database_path = arguments.db.expanduser().resolve(strict=True)
    if not database_path.is_file():
        raise ValueError(f"database is not a regular file: {database_path}")

    export = {
        "source_db": str(database_path),
        "source_db_sha256": sha256_file(database_path),
        "sqlite_schema_version": inspect_read_only_database(database_path),
    }
    print(json.dumps(export, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, sqlite3.Error, ValueError) as error:
        print(f"export-central-state: {error}", file=sys.stderr)
        raise SystemExit(1) from error
