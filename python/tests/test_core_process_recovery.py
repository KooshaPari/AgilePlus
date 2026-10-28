"""Real Rust core/SQLite/stdio MCP journey; explicit cached binaries required.

Run with AGILEPLUS_TEST_CORE_BINARY and AGILEPLUS_TEST_CLI_BINARY set.
These artifacts are hashed independently from the Python candidate.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import os
import socket
import subprocess
import sys
import time
from pathlib import Path

import pytest
from fastmcp import Client
from fastmcp.client.transports import StdioTransport

SPEC = """# Specification: Recovery Journey

## Problem
An MCP session must recover after its core starts or restarts.

## Functional Requirements
- FR-001: Keep the same durable feature ID after core restart.

## Acceptance Criteria
- Query the feature and dashboard before and after core restart.
"""


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


async def wait_ready(mcp: Client, within: float = 12) -> dict:
    deadline = time.monotonic() + within
    while time.monotonic() < deadline:
        health = (await mcp.call_tool("health_check")).data
        if health["ready"]:
            return health
        await asyncio.sleep(0.1)
    raise AssertionError("core readiness did not recover within deadline")


def stop_owned(process: subprocess.Popen) -> None:
    process.terminate()
    try:
        process.wait(timeout=2)
    except subprocess.TimeoutExpired:
        # The Rust server drains open gRPC channels. MCP intentionally retains
        # its channel here, so terminate ONLY this owned child after grace.
        process.kill()
        process.wait(timeout=2)


@pytest.mark.asyncio
async def test_real_core_late_start_restart_and_durable_feature(tmp_path: Path) -> None:
    configured = [os.getenv(f"AGILEPLUS_TEST_{kind}_BINARY") for kind in ("CORE", "CLI")]
    if not all(configured):
        pytest.skip("explicit cached Rust core and CLI binaries are required")
    core, cli = [Path(value).resolve(strict=True) for value in configured]
    root = (tmp_path / "subject").resolve()
    root.mkdir()
    subprocess.run(["git", "init", "-q", "-b", "main", str(root)], check=True)
    spec = tmp_path / "spec.md"
    spec.write_text(SPEC)
    env = {key: value for key, value in os.environ.items() if not key.startswith("AGILEPLUS_")}
    seeded = subprocess.run(
        [
            str(cli),
            "--repo",
            str(root),
            "specify",
            "--feature",
            "recovery-journey",
            "--from-file",
            str(spec),
        ],
        env=env,
        capture_output=True,
        text=True,
        timeout=20,
        check=True,
    )
    with socket.socket() as reserved:
        reserved.bind(("127.0.0.1", 0))
        port = reserved.getsockname()[1]
    address = f"127.0.0.1:{port}"
    env.update(AGILEPLUS_GRPC_ADDRESS=address)
    transport = StdioTransport(sys.executable, ["-m", "agileplus_mcp"], env=env)
    core_env = dict(env, AGILEPLUS_GRPC_BIND=address, AGILEPLUS_PROJECT_ROOT=str(root))
    observations = []
    process = None
    try:
        async with Client(transport, roots=[root.as_uri()], timeout=20) as mcp:
            initial = (await mcp.call_tool("health_check")).data
            assert initial["live"] is True and initial["ready"] is False
            observations.append({"stage": "mcp_before_core", "health": initial})
            process = subprocess.Popen(
                [str(core)], env=core_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )
            health = await wait_ready(mcp)
            feature = (await mcp.call_tool("get_feature", {"slug": "recovery-journey"})).data
            dashboard = (await mcp.call_tool("get_dashboard")).data
            assert feature["id"] > 0 and feature["slug"] == "recovery-journey"
            assert sum(dashboard["feature_counts"].values()) == 1
            observations.append(
                {"stage": "late_core", "health": health, "feature": feature, "dashboard": dashboard}
            )
            stop_owned(process)
            process = None
            unavailable = (await mcp.call_tool("health_check")).data
            assert unavailable["live"] is True and unavailable["ready"] is False
            observations.append({"stage": "core_stopped", "health": unavailable})
            process = subprocess.Popen(
                [str(core)], env=core_env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )
            recovered = await wait_ready(mcp)
            after = (await mcp.call_tool("get_feature", {"slug": "recovery-journey"})).data
            after_dashboard = (await mcp.call_tool("get_dashboard")).data
            assert after == feature
            assert after_dashboard == dashboard
            observations.append(
                {
                    "stage": "core_restarted_same_mcp_session",
                    "health": recovered,
                    "feature": after,
                    "dashboard": after_dashboard,
                }
            )
    finally:
        if process is not None:
            stop_owned(process)
    receipt = os.getenv("AGILEPLUS_TEST_RECEIPT")
    if receipt:
        Path(receipt).write_text(
            json.dumps(
                {
                    "schema": "agileplus-recovery-execution-v1",
                    "core_binary": str(core),
                    "core_sha256": digest(core),
                    "core_source_sha": None,
                    "core_source_sha_reason": "cached artifact build provenance is UNKNOWN",
                    "cli_binary": str(cli),
                    "cli_sha256": digest(cli),
                    "input_spec_sha256": digest(spec),
                    "seed_stdout": seeded.stdout,
                    "subject_root": str(root),
                    "grpc_address": address,
                    "observations": observations,
                    "user_validation": "NOT_RUN",
                    "scope": "local cached Rust artifacts plus new canonical Python candidate",
                },
                indent=2,
            )
            + "\n"
        )
