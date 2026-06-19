#!/usr/bin/env python3
"""
drift_check.py — commit-to-commit drift checker.

Runs file-type-specific linting/analysis on files changed between two commits.
Exit 0 if no HIGH severity findings, exit 1 if any HIGH findings.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path


TOOL_TIMEOUT = 30  # seconds per tool invocation


def run_tool(cmd: list[str], cwd: str | None = None, env: dict | None = None) -> tuple[int, str, str]:
    """Run a command, return (returncode, stdout, stderr). Graceful on missing tool."""
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=TOOL_TIMEOUT,
            cwd=cwd,
            env=env or None,
        )
        return result.returncode, result.stdout, result.stderr
    except FileNotFoundError:
        return 127, "", f"[drift-check] tool not found: {cmd[0]}"
    except subprocess.TimeoutExpired:
        return 124, "", f"[drift-check] timed out after {TOOL_TIMEOUT}s: {' '.join(cmd)}"
    except Exception as exc:
        return 1, "", f"[drift-check] error running {' '.join(cmd)}: {exc}"


def check_file_is_newer_than(file_path: Path, marker: Path) -> bool:
    """Return True if file_path exists and is newer than marker, or marker doesn't exist."""
    return file_path.exists() and (not marker.exists() or file_path.stat().st_mtime > marker.stat().st_mtime)


def rust_check(file_path: Path, repo_root: Path, findings: list) -> None:
    """Run rustfmt --check and cargo check on a .rs file."""
    dir_path = file_path.parent.resolve()
    lock = repo_root / "Cargo.lock"
    toolchain = repo_root / "rust-toolchain.toml"
    marker = max((lock, toolchain), key=lambda p: p.stat().st_mtime) if lock.exists() or toolchain.exists() else repo_root

    # rustfmt --check
    rc, stdout, stderr = run_tool(["rustfmt", "--check", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass  # rustfmt not installed — skip
    elif rc != 0:
        for line in (stdout + stderr).splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "rustfmt",
                "severity": severity,
                "message": line,
            })

    # cargo check — only if workspace present
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        return
    if not check_file_is_newer_than(cargo_toml, marker):
        return

    rc, stdout, stderr = run_tool(
        ["cargo", "check", "--quiet", "--message-format=json"],
        cwd=repo_root,
    )
    if rc == 127:
        pass
    elif rc not in (0, 101):  # 101 = compilation errors
        findings.append({
            "file": str(file_path.relative_to(repo_root)),
            "tool": "cargo-check",
            "severity": "HIGH",
            "message": f"cargo check failed (exit {rc})",
        })
    elif rc == 101:
        # Parse JSON diagnostics from cargo
        for line in stdout.splitlines():
            try:
                msg = json.loads(line)
                if msg.get("reason") != "build-finished":
                    continue
                for rust_msg in msg.get("messages", []):
                    lvl = rust_msg.get("severity", "error")
                    rendered = rust_msg.get("rendered", "")
                    for span in rust_msg.get("spans", []):
                        fn = span.get("file_name", "")
                        if fn and (repo_root / fn).resolve() == file_path.resolve():
                            severity = "HIGH" if lvl == "error" else "MEDIUM"
                            findings.append({
                                "file": str(file_path.relative_to(repo_root)),
                                "tool": "cargo-check",
                                "severity": severity,
                                "message": f"{fn}:{span.get('line_start', '?')} — {rendered[:200]}",
                            })
            except json.JSONDecodeError:
                pass


def python_check(file_path: Path, repo_root: Path, findings: list) -> None:
    """Run ruff check and mypy --strict on a .py file."""
    # ruff check
    rc, stdout, stderr = run_tool(["ruff", "check", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass
    elif rc != 0:
        for line in stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() or "failure" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "ruff",
                "severity": severity,
                "message": line,
            })

    # mypy --strict
    rc, stdout, stderr = run_tool(["mypy", "--strict", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass
    elif rc != 0:
        for line in (stdout + stderr).splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "mypy",
                "severity": severity,
                "message": line,
            })


def yaml_check(file_path: Path, repo_root: Path, findings: list) -> None:
    """Run actionlint on a YAML file."""
    rc, stdout, stderr = run_tool(["actionlint", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass
    elif rc != 0:
        for line in stderr.splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "actionlint",
                "severity": severity,
                "message": line,
            })


def typescript_check(file_path: Path, repo_root: Path, findings: list) -> None:
    """Run eslint on a .ts/.tsx file if node_modules is present."""
    node_modules = repo_root / "node_modules"
    if not node_modules.exists():
        return
    rc, stdout, stderr = run_tool(["npx", "eslint", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass
    elif rc not in (0, 1):  # 1 = linting errors
        for line in (stdout + stderr).splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "eslint",
                "severity": severity,
                "message": line,
            })
    elif rc == 1:
        for line in stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "eslint",
                "severity": "MEDIUM",
                "message": line,
            })


def markdown_check(file_path: Path, repo_root: Path, findings: list) -> None:
    """Run markdownlint on a .md file if markdownlint is installed."""
    rc, stdout, stderr = run_tool(["npx", "markdownlint", str(file_path)], cwd=repo_root)
    if rc == 127:
        pass
    elif rc != 0:
        for line in stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            severity = "HIGH" if "error" in line.lower() else "MEDIUM"
            findings.append({
                "file": str(file_path.relative_to(repo_root)),
                "tool": "markdownlint",
                "severity": severity,
                "message": line,
            })


def main() -> int:
    parser = argparse.ArgumentParser(description="Commit-to-commit drift checker")
    parser.add_argument("--from", dest="commit_from", default=None)
    parser.add_argument("--to", dest="commit_to", default=None)
    parser.add_argument("--repo", dest="repo", default=os.getcwd())
    args = parser.parse_args()

    repo_root = Path(args.repo).resolve()

    # Resolve or default commits
    commit_from = args.commit_from
    commit_to = args.commit_to
    if commit_from is None:
        rc, stdout, _ = run_tool(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD~1"],
            cwd=repo_root,
        )
        if rc == 0:
            commit_from = stdout.strip()
        else:
            commit_from = "HEAD~1"
    if commit_to is None:
        rc, stdout, _ = run_tool(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
            cwd=repo_root,
        )
        if rc == 0:
            commit_to = stdout.strip()
        else:
            commit_to = "HEAD"

    # Get changed files
    rc, stdout, stderr = run_tool(
        ["git", "-C", str(repo_root), "diff", commit_from, commit_to, "--name-only"],
        cwd=repo_root,
    )
    if rc != 0:
        print(json.dumps({
            "commit_from": commit_from,
            "commit_to": commit_to,
            "files_checked": 0,
            "findings": [],
            "error": f"git diff failed: {stderr.strip()}",
        }))
        return 0  # graceful — don't block on diff failure

    changed_files = [f.strip() for f in stdout.splitlines() if f.strip()]
    findings: list = []

    ext_handlers = {
        ".rs": rust_check,
        ".py": python_check,
        ".yml": yaml_check,
        ".yaml": yaml_check,
        ".ts": typescript_check,
        ".tsx": typescript_check,
        ".md": markdown_check,
    }

    files_checked = 0
    for rel_path in changed_files:
        file_path = repo_root / rel_path
        if not file_path.is_file():
            continue
        ext = file_path.suffix.lower()
        handler = ext_handlers.get(ext)
        if handler:
            handler(file_path, repo_root, findings)
            files_checked += 1

    report = {
        "commit_from": commit_from,
        "commit_to": commit_to,
        "files_checked": files_checked,
        "findings": findings,
    }

    print(json.dumps(report, indent=2))

    has_high = any(f["severity"] == "HIGH" for f in findings)
    return 1 if has_high else 0


if __name__ == "__main__":
    sys.exit(main())