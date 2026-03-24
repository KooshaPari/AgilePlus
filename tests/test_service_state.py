from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path

from phench.service import init_target, list_targets, load_target_lock, sync_target, target_status
from phench.store import dual_write, read_dual


class ServiceStateTests(unittest.TestCase):
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

    def test_init_target_creates_lock_and_lists_target(self) -> None:
        lock = init_target("alpha", mode="repo")
        loaded = load_target_lock("alpha")

        self.assertEqual(lock.target_name, "alpha")
        self.assertEqual(loaded.target_name, "alpha")
        self.assertEqual(loaded.mode, "repo")
        self.assertEqual(list_targets(), ["alpha"])

    def test_target_status_returns_lock_without_runtime_files(self) -> None:
        init_target("alpha", mode="stack")

        status = target_status("alpha")

        self.assertEqual(status["target"], "alpha")
        self.assertEqual(status["mode"], "stack")
        self.assertEqual(status["runtime"], None)
        self.assertEqual(status["env"], None)
        self.assertEqual(status["repos"], [])

    def test_sync_target_repairs_missing_mirror_copy(self) -> None:
        init_target("alpha", mode="repo")
        mirror_file = (
            Path(os.environ["THGENT_PHENCH_HOME_ROOT"]) / "alpha" / ".phench" / "target.lock.json"
        )
        mirror_file.unlink()

        result = sync_target("alpha")
        payload = read_dual("alpha", "target.lock.json")

        self.assertEqual(result["target.lock.json"]["status"], "repaired")
        self.assertEqual(payload["target_name"], "alpha")


if __name__ == "__main__":
    unittest.main()
