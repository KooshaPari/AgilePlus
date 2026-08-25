"""Regression tests for the generated kitty-spec index."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class GovernanceIndexFormattingTest(unittest.TestCase):
    def test_regenerated_index_passes_repository_prettier_check(self) -> None:
        subprocess.run(
            [sys.executable, "tooling/governance_index.py"],
            cwd=ROOT,
            check=True,
        )
        result = subprocess.run(
            ["npx", "--no-install", "prettier", "--check", "kitty-specs/INDEX.md"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
