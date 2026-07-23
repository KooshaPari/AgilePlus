from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path

from phench.store import dual_write, read_dual, sync_dual


class StoreTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self._old_root = os.environ.get("THGENT_PHENOTYPE_ROOT")
        self._old_home = os.environ.get("THGENT_PHENCH_HOME_ROOT")
        root = Path(self._tmp.name)
        os.environ["THGENT_PHENOTYPE_ROOT"] = str(root / "phenotype")
        os.environ["THGENT_PHENCH_HOME_ROOT"] = str(root / "home")
        self.addCleanup(self._restore_env)

    def _restore_env(self) -> None:
        if self._old_root is None:
            os.environ.pop("THGENT_PHENOTYPE_ROOT", None)
        else:
            os.environ["THGENT_PHENOTYPE_ROOT"] = self._old_root
        if self._old_home is None:
            os.environ.pop("THGENT_PHENCH_HOME_ROOT", None)
        else:
            os.environ["THGENT_PHENCH_HOME_ROOT"] = self._old_home

    def test_dual_write_and_read_dual_round_trip_payload(self) -> None:
        result = dual_write("alpha", "state.json", {"answer": 42})

        self.assertIn("project_path", result)
        self.assertIn("mirror_path", result)
        self.assertEqual(read_dual("alpha", "state.json"), {"answer": 42})

    def test_read_dual_falls_back_to_home_mirror_when_project_copy_missing(self) -> None:
        dual_write("alpha", "state.json", {"answer": 42})
        project_file = (
            Path(os.environ["THGENT_PHENOTYPE_ROOT"]) / "projects" / "alpha" / ".phench" / "state.json"
        )
        project_file.unlink()

        self.assertEqual(read_dual("alpha", "state.json"), {"answer": 42})

    def test_sync_dual_prefers_project_copy_when_requested(self) -> None:
        dual_write("alpha", "state.json", {"answer": 42})
        project_file = (
            Path(os.environ["THGENT_PHENOTYPE_ROOT"]) / "projects" / "alpha" / ".phench" / "state.json"
        )
        mirror_file = (
            Path(os.environ["THGENT_PHENCH_HOME_ROOT"]) / "alpha" / ".phench" / "state.json"
        )
        project_file.write_text(
            '{"content_hash":"x","payload":{"source":"projects"},"sync_id":"1"}\n',
            encoding="utf-8",
        )
        mirror_file.write_text(
            '{"content_hash":"y","payload":{"source":"home"},"sync_id":"2"}\n',
            encoding="utf-8",
        )

        result = sync_dual("alpha", "state.json", prefer="projects")

        self.assertEqual(result["status"], "repaired")
        self.assertEqual(read_dual("alpha", "state.json"), {"source": "projects"})


if __name__ == "__main__":
    unittest.main()
