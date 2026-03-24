from __future__ import annotations

import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from phench.service import add_repo, init_target, lock_target, materialize_target, run_target


class RunContractTests(unittest.TestCase):
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

    def _make_git_repo(self, name: str, files: dict[str, str]) -> Path:
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
        for rel, content in files.items():
            path = repo / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        subprocess.run(["git", "add", "."], cwd=repo, check=True, capture_output=True)
        subprocess.run(["git", "commit", "-m", "init"], cwd=repo, check=True, capture_output=True)
        return repo

    def _materialize_repo(self, repo: Path, repo_id: str) -> None:
        init_target("alpha", mode="repo")
        add_repo("alpha", str(repo), "HEAD", repo_id=repo_id)
        lock_target("alpha")
        materialize_target("alpha")

    def test_run_target_rejects_runner_without_discovered_commands(self) -> None:
        repo = self._make_git_repo("plain", {"README.md": "# plain\n"})
        self._materialize_repo(repo, "plain")

        with self.assertRaisesRegex(ValueError, "runner has no discovered commands: make"):
            run_target("alpha", runner="make")

    def test_run_target_rejects_noninteractive_selection_without_tty(self) -> None:
        repo = self._make_git_repo("make-repo", {"Makefile": "build:\n\t@echo build\n"})
        self._materialize_repo(repo, "make-repo")

        with patch("sys.stdin.isatty", return_value=False), patch(
            "sys.stdout.isatty", return_value=False
        ):
            with self.assertRaisesRegex(ValueError, "requires a TTY"):
                run_target("alpha")


if __name__ == "__main__":
    unittest.main()
