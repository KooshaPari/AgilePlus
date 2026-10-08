"""Executable zero-paid-hosting SQLite backup/restore smoke witnesses."""

from __future__ import annotations

from pathlib import Path
import runpy
import shutil
import sqlite3
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[2] / "scripts" / "backup-agileplus-sqlite.py"
FUNCTIONS = runpy.run_path(str(SCRIPT))
backup_database = FUNCTIONS["backup_database"]
verify_backup = FUNCTIONS["verify_backup"]


class BackupWitnesses(unittest.TestCase):
    def setUp(self) -> None:
        self.workspace = tempfile.TemporaryDirectory()
        self.addCleanup(self.workspace.cleanup)
        self.root = Path(self.workspace.name)
        self.database = self.root / "live.sqlite3"
        self.backups = self.root / "backups"

    def test_wal_live_backup_and_offline_restore(self) -> None:
        writer = sqlite3.connect(self.database)
        self.addCleanup(writer.close)
        writer.execute("PRAGMA journal_mode=WAL")
        writer.execute("CREATE TABLE receipts (id TEXT PRIMARY KEY, state TEXT NOT NULL)")
        writer.execute("INSERT INTO receipts VALUES (?, ?)", ("a", "validated"))
        writer.commit()

        backup = backup_database(self.database, self.backups)
        verify_backup(backup)
        self.assertTrue(backup.is_file())
        self.assertNotEqual(backup.resolve(), self.database.resolve())
        self.assertFalse(list(self.backups.glob("*.incomplete")))

        restored = self.root / "restored.sqlite3"
        shutil.copy2(backup, restored)
        with sqlite3.connect(restored) as reader:
            self.assertEqual(
                reader.execute("SELECT id, state FROM receipts").fetchall(),
                [("a", "validated")],
            )
        writer.execute("INSERT INTO receipts VALUES (?, ?)", ("b", "review"))
        writer.commit()
        with sqlite3.connect(restored) as reader:
            self.assertEqual(reader.execute("SELECT COUNT(*) FROM receipts").fetchone()[0], 1)

    def test_missing_source_fails_without_creating_fake_backup(self) -> None:
        with self.assertRaises(FileNotFoundError):
            backup_database(self.database, self.backups)
        self.assertFalse(self.backups.exists())

    def test_invalid_backup_file_is_rejected(self) -> None:
        corrupt = self.root / "corrupt.sqlite3"
        corrupt.write_bytes(b"not a SQLite database")
        with self.assertRaises(sqlite3.DatabaseError):
            verify_backup(corrupt)


if __name__ == "__main__":
    unittest.main()
