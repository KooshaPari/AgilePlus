from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from phench.service import (
    add_repo,
    build_catalog,
    init_target,
    lock_target,
    materialize_target,
    run_target,
    target_timeline,
)
from phench.store import read_dual


class MaterializationTests(unittest.TestCase):
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

    def _make_git_repo(self, name: str) -> Path:
        repo = Path(self._tmp.name) / name
        repo.mkdir(parents=True)
        subprocess.run(["git", "init", "-b", "main"], cwd=repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Codex"], cwd=repo, check=True, capture_output=True)
        subprocess.run(
            ["git", "config", "user.email", "codex@example.com"],
            cwd=repo,
            check=True,
            capture_output=True,
        )
        (repo / "README.md").write_text(f"# {name}\n", encoding="utf-8")
        (repo / "Makefile").write_text("build:\n\t@echo build\nlint:\n\t@echo lint\n", encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "init"], cwd=repo, check=True, capture_output=True)
        return repo

    def test_materialize_target_creates_detached_checkout_and_env_snapshot(self) -> None:
        repo = self._make_git_repo("demo-repo")
        init_target("alpha", mode="repo")
        add_repo("alpha", str(repo), "HEAD", repo_id="demo")
        locked = lock_target("alpha")

        runtime = materialize_target("alpha")
        runtime_payload = read_dual("alpha", "runtime.json")
        env_payload = read_dual("alpha", "env.snapshot.json")
        checkout = Path(runtime.repo_materializations[0].checkout_path)

        self.assertEqual(locked.repos[0].resolved_sha, runtime.repo_materializations[0].resolved_sha)
        self.assertTrue(checkout.exists())
        self.assertTrue((checkout / "README.md").exists())
        self.assertIsNone(runtime.repo_materializations[0].head_branch)
        self.assertEqual(runtime_payload["target_name"], "alpha")
        self.assertEqual(runtime_payload["repo_materializations"][0]["repo_id"], "demo")
        self.assertEqual(env_payload["doctor_status"], "pass")
        self.assertIn(str(checkout / "Makefile"), env_payload["detected_files"])

    def test_build_catalog_requires_runtime_materialization(self) -> None:
        init_target("alpha", mode="repo")

        with self.assertRaisesRegex(ValueError, "run target materialize"):
            build_catalog("alpha")

    def test_run_target_requires_runtime_materialization(self) -> None:
        init_target("alpha", mode="repo")

        with self.assertRaisesRegex(ValueError, "run target materialize"):
            run_target("alpha")

    def test_target_timeline_reports_recent_history_for_selected_repo(self) -> None:
        repo = self._make_git_repo("timeline-repo")
        init_target("alpha", mode="repo")
        add_repo("alpha", str(repo), "HEAD", repo_id="timeline")
        lock_target("alpha")

        timeline = target_timeline("alpha", repo_id="timeline", limit=5)

        self.assertEqual(timeline["repo_id"], "timeline")
        self.assertEqual(timeline["selected_ref"], "HEAD")
        self.assertTrue(any("init" in line for line in timeline["recent"]))


if __name__ == "__main__":
    unittest.main()
