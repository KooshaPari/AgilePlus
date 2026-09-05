from __future__ import annotations

import hashlib
import os
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

import pytest

from qe.runtime import (
    CleanupRegistry,
    ManagedProcess,
    ProcessOwnershipError,
    WaitTimeout,
    binary_sha256,
    build_core_environment,
    build_mcp_environment,
    candidate_sha,
    reserve_loopback_port,
    start_process,
    stop_process,
    wait_until,
)


def _wait_until_not_running(pid: int, timeout: float = 5.0) -> None:
    """Accept a reaped process or a transient zombie awaiting its parent."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        status = subprocess.run(
            ["ps", "-o", "stat=", "-p", str(pid)],
            check=False,
            capture_output=True,
            text=True,
        )
        state = status.stdout.strip()
        if status.returncode != 0 or not state or state.startswith("Z"):
            return
        time.sleep(0.02)
    pytest.fail(f"process {pid} remained live after {timeout}s")


def test_reserve_loopback_port_returns_a_reusable_local_port() -> None:
    port = reserve_loopback_port()

    assert 0 < port < 65536
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", port))


def test_candidate_sha_returns_the_repository_head(tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q", tmp_path], check=True)
    subprocess.run(
        ["git", "-C", str(tmp_path), "config", "user.email", "qe@example.invalid"],
        check=True,
    )
    subprocess.run(
        ["git", "-C", str(tmp_path), "config", "user.name", "QE"], check=True
    )
    (tmp_path / "candidate.txt").write_text("candidate\n")
    subprocess.run(["git", "-C", str(tmp_path), "add", "candidate.txt"], check=True)
    subprocess.run(
        ["git", "-C", str(tmp_path), "commit", "-qm", "test candidate"], check=True
    )
    expected = subprocess.check_output(
        ["git", "-C", str(tmp_path), "rev-parse", "HEAD"], text=True
    ).strip()

    assert candidate_sha(tmp_path) == expected


def test_binary_sha256_hashes_file_bytes(tmp_path: Path) -> None:
    binary = tmp_path / "agileplus-grpc"
    binary.write_bytes(b"agileplus-runtime\x00")

    assert binary_sha256(binary) == hashlib.sha256(binary.read_bytes()).hexdigest()


def test_core_environment_contains_only_allowlisted_parent_values(
    tmp_path: Path,
) -> None:
    parent = {
        "PATH": "/test/bin",
        "RUST_LOG": "warn",
        "HOME": "/must-not-leak",
        "AWS_SECRET_ACCESS_KEY": "must-not-leak",
    }

    result = build_core_environment(
        address="127.0.0.1:54321", database=tmp_path / "core.db", parent=parent
    )

    assert result == {
        "AGILEPLUS_CORE_DATABASE_PATH": str(tmp_path / "core.db"),
        "AGILEPLUS_GRPC_BIND": "127.0.0.1:54321",
        "PATH": "/test/bin",
        "RUST_LOG": "warn",
    }


def test_mcp_environment_contains_only_allowlisted_parent_values() -> None:
    parent = {
        "PATH": "/test/bin",
        "PYTHONPATH": "/test/python",
        "RUST_LOG": "debug",
        "HOME": "/must-not-leak",
        "TOKEN": "must-not-leak",
    }

    result = build_mcp_environment(
        core_address="127.0.0.1:50051", port=8765, parent=parent
    )

    assert result == {
        "AGILEPLUS_GRPC_ADDRESS": "127.0.0.1:50051",
        "AGILEPLUS_MCP_HOST": "127.0.0.1",
        "AGILEPLUS_MCP_PATH": "/mcp",
        "AGILEPLUS_MCP_PORT": "8765",
        "AGILEPLUS_MCP_TRANSPORT": "http",
        "PATH": "/test/bin",
        "PYTHONPATH": "/test/python",
        "RUST_LOG": "debug",
    }


def test_managed_process_uses_own_session_and_captures_logs(tmp_path: Path) -> None:
    process = start_process(
        "logger",
        [
            sys.executable,
            "-c",
            (
                "import os, sys, time; print(os.getpid(), flush=True); "
                + "print('err', file=sys.stderr, flush=True); time.sleep(0.2)"
            ),
        ],
        env={"PATH": os.environ["PATH"]},
        logs_dir=tmp_path,
    )

    assert os.getpgid(process.process.pid) == process.process.pid
    assert process.process.wait(timeout=5) == 0
    assert process.stdout_path.read_text().strip() == str(process.process.pid)
    assert process.stderr_path.read_text().strip() == "err"
    stop_process(process)


def test_managed_process_stop_terminates_its_owned_process_group(
    tmp_path: Path,
) -> None:
    child_pid_path = tmp_path / "child.pid"
    script = (
        "import pathlib, subprocess, sys, time; "
        "p=subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)']); "
        f"pathlib.Path({str(child_pid_path)!r}).write_text(str(p.pid)); "
        "time.sleep(60)"
    )
    process = start_process(
        "tree",
        [sys.executable, "-c", script],
        env={"PATH": os.environ["PATH"]},
        logs_dir=tmp_path,
    )
    deadline = time.monotonic() + 5
    while not child_pid_path.exists() and time.monotonic() < deadline:
        time.sleep(0.01)
    assert child_pid_path.exists()
    child_pid = int(child_pid_path.read_text())

    stop_process(process, grace=2)

    assert process.process.poll() is not None
    _wait_until_not_running(child_pid)


def test_stop_process_rejects_forged_unowned_handle(tmp_path: Path) -> None:
    unowned = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
    forged = ManagedProcess(
        name="forged",
        process=unowned,
        stdout_path=tmp_path / "forged.stdout.log",
        stderr_path=tmp_path / "forged.stderr.log",
    )

    try:
        with pytest.raises(ProcessOwnershipError, match="not owned"):
            stop_process(forged)
        assert unowned.poll() is None
    finally:
        unowned.terminate()
        unowned.wait(timeout=5)


@pytest.mark.asyncio
async def test_wait_until_returns_after_awaitable_probe_succeeds(
    tmp_path: Path,
) -> None:
    process = start_process(
        "ready",
        [sys.executable, "-c", "import time; time.sleep(60)"],
        env={"PATH": os.environ["PATH"]},
        logs_dir=tmp_path,
    )
    attempts = 0

    async def probe() -> None:
        nonlocal attempts
        attempts += 1
        if attempts < 3:
            raise ConnectionError("not ready")

    try:
        await wait_until("runtime readiness", probe, process, timeout=1)
        assert attempts == 3
    finally:
        stop_process(process)


@pytest.mark.asyncio
async def test_wait_until_timeout_includes_name_and_log_tails(tmp_path: Path) -> None:
    process = start_process(
        "core",
        [
            sys.executable,
            "-c",
            "import sys, time; print('core failure', file=sys.stderr, flush=True); time.sleep(60)",
        ],
        env={"PATH": os.environ["PATH"]},
        logs_dir=tmp_path,
    )

    # Synchronize on the child output before exercising the timeout path.  The
    # assertion is about diagnostic retention, so a scheduler-dependent race
    # must not decide whether the child has flushed its deliberately emitted
    # failure line.
    deadline = time.monotonic() + 5
    while (
        "core failure"
        not in process.stderr_path.read_text(encoding="utf-8", errors="replace")
        and time.monotonic() < deadline
    ):
        time.sleep(0.01)
    assert "core failure" in process.stderr_path.read_text(
        encoding="utf-8", errors="replace"
    )

    async def unavailable() -> None:
        raise ConnectionError("not ready")

    try:
        with pytest.raises(WaitTimeout) as caught:
            await wait_until("core readiness", unavailable, process, timeout=0.1)
    finally:
        stop_process(process)

    message = str(caught.value)
    assert "core readiness" in message
    assert "core.stderr.log" in message and "core failure" in message


def test_cleanup_registry_is_idempotent_and_only_stops_registered_processes(
    tmp_path: Path,
) -> None:
    registered = start_process(
        "registered",
        [sys.executable, "-c", "import time; time.sleep(60)"],
        env={"PATH": os.environ["PATH"]},
        logs_dir=tmp_path,
    )
    unowned = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"])
    registry = CleanupRegistry()
    registry.register(registered)

    try:
        registry.cleanup()
        registry.cleanup()

        assert registered.process.poll() is not None
        assert unowned.poll() is None
    finally:
        if unowned.poll() is None:
            unowned.terminate()
            unowned.wait(timeout=5)


@pytest.mark.parametrize("signum", [signal.SIGINT, signal.SIGTERM])
def test_installed_signal_handler_cleans_child_and_preserves_signal_exit(
    tmp_path: Path, signum: signal.Signals
) -> None:
    script = """
import pathlib
import sys
import time
from qe.runtime import start_process

managed = start_process(
    "signal-child",
    [sys.executable, "-c", "import time; time.sleep(60)"],
    {"PATH": __import__("os").environ["PATH"]},
    pathlib.Path(sys.argv[1]),
)
print(managed.process.pid, flush=True)
while True:
    time.sleep(1)
"""
    harness = subprocess.Popen(
        [sys.executable, "-c", script, str(tmp_path)],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    assert harness.stdout is not None
    child_pid = int(harness.stdout.readline().strip())

    os.kill(harness.pid, signum)
    _, stderr = harness.communicate(timeout=5)

    assert harness.returncode == -signum, stderr
    _wait_until_not_running(child_pid)


def test_installed_signal_handler_does_not_lock_registry_in_handler() -> None:
    script = """
import os
import signal
import time
from qe import runtime

runtime._CLEANUP._lock.acquire()
os.kill(os.getpid(), signal.SIGTERM)
print("handler returned", flush=True)
runtime._CLEANUP._lock.release()
while True:
    time.sleep(1)
"""
    harness = subprocess.Popen(
        [sys.executable, "-c", script],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    stdout, stderr = harness.communicate(timeout=5)

    assert "handler returned" in stdout
    assert harness.returncode == -signal.SIGTERM, stderr
