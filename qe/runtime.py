"""Owned process and environment primitives for the QE harness."""

from __future__ import annotations

import asyncio
import atexit
import hashlib
import os
import signal
import socket
import subprocess
import time
from collections.abc import Awaitable, Callable, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from threading import Lock, Thread
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from agileplus_mcp.grpc_client import AgilePlusCoreClient


class WaitTimeout(TimeoutError):
    """Raised when a managed process does not become ready in time."""


class ProcessOwnershipError(RuntimeError):
    """Raised when code attempts to control a process the harness did not start."""


@dataclass(frozen=True)
class ManagedProcess:
    """A child process and the files that capture its output."""

    name: str
    process: subprocess.Popen[bytes]
    stdout_path: Path
    stderr_path: Path
    _ownership_token: object | None = field(
        default=None, init=False, repr=False, compare=False
    )


@dataclass(frozen=True)
class _OwnedIdentity:
    process: subprocess.Popen[bytes]
    pid: int
    pgid: int
    session_id: int


_OWNED: dict[object, _OwnedIdentity] = {}
_OWNED_LOCK = Lock()


def _claim_process(process: ManagedProcess) -> None:
    pid = process.process.pid
    pgid = os.getpgid(pid)
    session_id = os.getsid(pid)
    if pgid != pid or session_id != pid:
        process.process.terminate()
        process.process.wait(timeout=5)
        raise ProcessOwnershipError(
            f"{process.name} did not start as its own process-group/session leader"
        )
    token = object()
    with _OWNED_LOCK:
        _OWNED[token] = _OwnedIdentity(process.process, pid, pgid, session_id)
    object.__setattr__(process, "_ownership_token", token)


def _owned_identity(process: ManagedProcess) -> _OwnedIdentity:
    token = process._ownership_token
    with _OWNED_LOCK:
        identity = _OWNED.get(token) if token is not None else None
    if identity is None or identity.process is not process.process:
        raise ProcessOwnershipError(f"process {process.name!r} is not owned by this harness")
    return identity



def reserve_loopback_port() -> int:
    """Reserve and release an ephemeral IPv4 loopback port."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def candidate_sha(repo: Path) -> str:
    """Return the exact Git candidate at ``repo``."""
    return subprocess.check_output(
        ["git", "-C", str(repo), "rev-parse", "HEAD"], text=True
    ).strip()


def binary_sha256(path: Path) -> str:
    """Hash a candidate binary without loading it all into memory."""
    digest = hashlib.sha256()
    with path.open("rb") as candidate:
        for chunk in iter(lambda: candidate.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _allowlisted_environment(
    parent: Mapping[str, str], keys: Sequence[str]
) -> dict[str, str]:
    return {key: parent[key] for key in keys if key in parent}


def build_core_environment(
    address: str,
    database: Path,
    *,
    parent: Mapping[str, str] | None = None,
) -> dict[str, str]:
    """Build a secret-free environment for the Rust core."""
    source = os.environ if parent is None else parent
    environment = _allowlisted_environment(source, ("PATH", "RUST_LOG"))
    environment.update(
        {
            "AGILEPLUS_CORE_DATABASE_PATH": str(database),
            "AGILEPLUS_GRPC_BIND": address,
        }
    )
    return environment


def build_mcp_environment(
    core_address: str,
    port: int,
    *,
    parent: Mapping[str, str] | None = None,
) -> dict[str, str]:
    """Build a secret-free environment for the Python MCP bridge."""
    source = os.environ if parent is None else parent
    environment = _allowlisted_environment(
        source, ("PATH", "PYTHONPATH", "RUST_LOG")
    )
    environment.update(
        {
            "AGILEPLUS_GRPC_ADDRESS": core_address,
            "AGILEPLUS_MCP_HOST": "127.0.0.1",
            "AGILEPLUS_MCP_PATH": "/mcp",
            "AGILEPLUS_MCP_PORT": str(port),
            "AGILEPLUS_MCP_TRANSPORT": "http",
        }
    )
    return environment


class CleanupRegistry:
    """Idempotently stop only processes explicitly registered with it."""

    def __init__(self) -> None:
        self._processes: list[ManagedProcess] = []
        self._lock = Lock()

    def register(self, process: ManagedProcess) -> ManagedProcess:
        _owned_identity(process)
        with self._lock:
            self._processes.append(process)
        return process

    def cleanup(self) -> None:
        with self._lock:
            processes = self._processes
            self._processes = []
        for process in reversed(processes):
            stop_process(process)

_CLEANUP = CleanupRegistry()
atexit.register(_CLEANUP.cleanup)


class _SignalRelay:
    """Relay process signals to cleanup outside the Python signal handler."""

    def __init__(self, registry: CleanupRegistry) -> None:
        self._registry = registry
        self._read_fd, self._write_fd = os.pipe()
        self._previous: dict[int, signal.Handlers] = {}
        self._worker = Thread(target=self._run, name="qe-signal-cleanup", daemon=True)
        self._worker.start()

    def install(self) -> None:
        for signum in (signal.SIGINT, signal.SIGTERM):
            previous = signal.getsignal(signum)
            if getattr(previous, "__self__", None) is self:
                continue
            self._previous[signum] = previous
            signal.signal(signum, self._handle)

    def _handle(self, signum: int, _frame: object) -> None:
        # Do not touch the registry here: this may interrupt code holding its lock.
        signal.signal(signum, self._previous[signum])
        os.write(self._write_fd, bytes((signum,)))

    def _run(self) -> None:
        while payload := os.read(self._read_fd, 1):
            signum = payload[0]
            self._registry.cleanup()
            if self._previous[signum] != signal.SIG_IGN:
                os.kill(os.getpid(), signum)


_SIGNAL_RELAY = _SignalRelay(_CLEANUP)
_SIGNAL_RELAY.install()


def start_process(
    name: str,
    argv: Sequence[str],
    env: Mapping[str, str],
    logs_dir: Path,
) -> ManagedProcess:
    """Start one harness-owned process group with file-backed output."""
    logs_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = logs_dir / f"{name}.stdout.log"
    stderr_path = logs_dir / f"{name}.stderr.log"
    stdout = stdout_path.open("wb")
    stderr = stderr_path.open("wb")
    try:
        child = subprocess.Popen(
            list(argv),
            env=dict(env),
            stdout=stdout,
            stderr=stderr,
            start_new_session=True,
        )
    except BaseException:
        stdout.close()
        stderr.close()
        raise
    stdout.close()
    stderr.close()
    managed = ManagedProcess(
        name=name,
        process=child,
        stdout_path=stdout_path,
        stderr_path=stderr_path,
    )
    _claim_process(managed)
    return _CLEANUP.register(managed)


def _tail(path: Path, lines: int = 20) -> str:
    try:
        content = path.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError as exc:
        return f"<{exc}>"
    return "\n".join(content[-lines:])


def _diagnostics(process: ManagedProcess) -> str:
    sections = []
    for path in (process.stdout_path, process.stderr_path):
        sections.append(f"--- {path.name} ---\n{_tail(path)}")
    return "\n".join(sections)


async def wait_until(
    name: str,
    probe: Callable[[], Awaitable[None]],
    process: ManagedProcess,
    timeout: float,
) -> None:
    """Poll an awaitable readiness probe until success or a bounded failure."""
    deadline = time.monotonic() + timeout
    last_error: Exception | None = None
    while True:
        return_code = process.process.poll()
        if return_code is not None:
            raise WaitTimeout(
                f"{name}: {process.name} exited with code {return_code}\n"
                f"{_diagnostics(process)}"
            )
        try:
            await probe()
            return
        except Exception as exc:  # readiness probes communicate retryable failure
            last_error = exc
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            detail = f"; last probe error: {last_error}" if last_error else ""
            raise WaitTimeout(
                f"{name} timed out after {timeout:.3f}s{detail}\n{_diagnostics(process)}"
            )
        await asyncio.sleep(min(0.05, remaining))


def stop_process(process: ManagedProcess, grace: float = 5.0) -> None:
    """TERM, wait, then KILL only the managed process group."""
    identity = _owned_identity(process)
    if process.process.poll() is not None:
        return
    try:
        current_pgid = os.getpgid(identity.pid)
        current_session = os.getsid(identity.pid)
    except ProcessLookupError:
        process.process.poll()
        return
    if (
        identity.pid != process.process.pid
        or current_pgid != identity.pgid
        or current_session != identity.session_id
        or current_pgid != identity.pid
        or current_session != identity.pid
    ):
        raise ProcessOwnershipError(
            f"process {process.name!r} no longer matches its owned process group/session"
        )
    try:
        os.killpg(identity.pgid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.process.wait(timeout=grace)
        return
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(identity.pgid, signal.SIGKILL)
    except ProcessLookupError:
        return
    process.process.wait()


class RuntimeHarness:
    """Own an ephemeral Rust core process and its readiness client.

    The harness deliberately accepts a prebuilt binary.  Compilation belongs to
    the caller (and to CI), so a test can identify the exact candidate binary
    that it exercised.  Every address and database path is instance-local.
    """

    def __init__(
        self,
        *,
        core_binary: Path,
        database: Path,
        logs_dir: Path,
        startup_timeout: float = 15.0,
        bind_attempts: int = 3,
    ) -> None:
        if startup_timeout <= 0:
            raise ValueError("startup_timeout must be positive")
        if bind_attempts < 1:
            raise ValueError("bind_attempts must be at least one")
        self.core_binary = Path(core_binary)
        self.database = Path(database)
        self.logs_dir = Path(logs_dir)
        self.startup_timeout = startup_timeout
        self.bind_attempts = bind_attempts
        self.core_process: ManagedProcess | None = None
        self.core_address: str | None = None
        self._closed = False

    def start_core(self) -> ManagedProcess:
        """Start the core immediately on a newly reserved loopback port."""
        if self._closed:
            raise RuntimeError("cannot start a closed runtime harness")
        if not self.core_binary.is_file():
            raise FileNotFoundError(f"core binary does not exist: {self.core_binary}")
        if self.core_process is not None and self.core_process.process.poll() is None:
            return self.core_process

        self.database.parent.mkdir(parents=True, exist_ok=True)
        port = reserve_loopback_port()
        address = f"127.0.0.1:{port}"
        environment = build_core_environment(address, self.database)
        process = start_process(
            "agileplus-grpc",
            [str(self.core_binary)],
            environment,
            self.logs_dir,
        )
        self.core_address = address
        self.core_process = process
        return process

    async def wait_for_core(self) -> AgilePlusCoreClient:
        """Connect and perform a real ``ListFeatures`` readiness probe.

        A listening socket is insufficient: readiness requires a successful
        gRPC channel and unary response.  The deadline covers channel setup,
        each RPC, and all retries.  A short-lived process is treated as a
        possible ephemeral-port bind race and retried a bounded number of
        times.
        """
        if self._closed:
            raise RuntimeError("cannot wait on a closed runtime harness")
        from agileplus_mcp.grpc_client import AgilePlusCoreClient

        deadline = time.monotonic() + self.startup_timeout
        last_error: Exception | None = None
        for attempt in range(self.bind_attempts):
            if self.core_process is None or self.core_process.process.poll() is not None:
                self.start_core()
            assert self.core_process is not None
            assert self.core_address is not None
            client = AgilePlusCoreClient(self.core_address)
            try:
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        break
                    try:
                        probe_timeout = min(remaining, 2.0)
                        await asyncio.wait_for(client.connect(), timeout=probe_timeout)
                        await asyncio.wait_for(client.list_features(), timeout=probe_timeout)
                        return client
                    except Exception as exc:
                        last_error = exc
                        if self.core_process.process.poll() is not None:
                            break
                        remaining = deadline - time.monotonic()
                        if remaining <= 0:
                            break
                        await asyncio.sleep(min(0.05, remaining))
            finally:
                if client._channel is not None and last_error is not None:
                    await client.close()

            stop_process(self.core_process)
            self.core_process = None
            if time.monotonic() >= deadline:
                break
            if attempt + 1 < self.bind_attempts:
                continue
        detail = f"; last probe error: {last_error}" if last_error else ""
        process = self.core_process
        diagnostics = f"\n{_diagnostics(process)}" if process is not None else ""
        raise WaitTimeout(
            f"core gRPC readiness timed out after {self.startup_timeout:.3f}s"
            f" after {self.bind_attempts} bind attempts{detail}{diagnostics}"
        )

    def cleanup(self) -> None:
        """Stop the owned core process; safe to call repeatedly."""
        if self._closed and self.core_process is None:
            return
        process = self.core_process
        self.core_process = None
        self._closed = True
        if process is not None:
            stop_process(process)
