from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from phench.git_ops import sanitize_repo_id
from phench.models import RunnerCatalog, RunnerCommand
from phench.runner import _makefile_targets, _task_targets, pick_command_interactive


class RuntimeHelperTests(unittest.TestCase):
    def test_sanitize_repo_id_normalizes_to_safe_token(self) -> None:
        self.assertEqual(sanitize_repo_id(" repo / weird name "), "repo-weird-name")
        self.assertEqual(sanitize_repo_id("..."), "...")
        self.assertEqual(sanitize_repo_id(""), "repo")

    def test_makefile_targets_ignore_comments_and_hidden_targets(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "Makefile"
            path.write_text(
                "build:\n\t@echo ok\n# comment\n.deploy:\nserve: ## help\n",
                encoding="utf-8",
            )
            self.assertEqual(_makefile_targets(path), ["build", "serve"])

    def test_task_targets_detect_top_level_tasks_only(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "Taskfile.yml"
            path.write_text(
                "version: '3'\ntasks:\n  build:\n    cmds:\n      - echo ok\nlint:\n  cmds:\n    - echo lint\n",
                encoding="utf-8",
            )
            self.assertEqual(_task_targets(path), ["lint"])

    def test_pick_command_interactive_requires_tty(self) -> None:
        catalog = RunnerCatalog(
            target_name="alpha",
            commands=[RunnerCommand("make", "build", "make build", "Makefile")],
        )

        with patch("sys.stdin.isatty", return_value=False), patch(
            "sys.stdout.isatty", return_value=False
        ):
            with self.assertRaisesRegex(ValueError, "requires a TTY"):
                pick_command_interactive(catalog)


if __name__ == "__main__":
    unittest.main()
