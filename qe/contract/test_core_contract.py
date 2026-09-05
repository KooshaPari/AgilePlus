from __future__ import annotations

from pathlib import Path

import grpc
import pytest

from agileplus_mcp.grpc_client import GrpcCallError
from qe.runtime import RuntimeHarness


@pytest.mark.asyncio
async def test_real_core_lists_no_features_from_a_fresh_database(
    core_binary: Path, tmp_path: Path
) -> None:
    harness = RuntimeHarness(
        core_binary=core_binary,
        database=tmp_path / "state" / "core.db",
        logs_dir=tmp_path / "logs",
    )
    client = None

    try:
        harness.start_core()
        client = await harness.wait_for_core()

        assert await client.list_features() == []
    finally:
        if client is not None:
            await client.close()
        harness.cleanup()


@pytest.mark.asyncio
async def test_real_core_backlog_create_and_feature_filtered_list(
    core_binary: Path, tmp_path: Path
) -> None:
    harness = RuntimeHarness(
        core_binary=core_binary,
        database=tmp_path / "state" / "core.db",
        logs_dir=tmp_path / "logs",
    )
    client = None

    try:
        harness.start_core()
        client = await harness.wait_for_core()
        created = await client.create_backlog_item(
            item_type="task",
            title="Exercise real backlog",
            body="Persist through the Rust core",
            feature_id="qe-feature",
        )

        listed = await client.list_backlog(feature_slug="qe-feature")

        assert len(listed) == 1
        assert listed[0]["id"] == created["id"]
        assert listed[0]["title"] == "Exercise real backlog"
        assert listed[0]["body"] == "Persist through the Rust core"
    finally:
        if client is not None:
            await client.close()
        harness.cleanup()


@pytest.mark.asyncio
async def test_real_core_rejects_unsupported_work_package_association(
    core_binary: Path, tmp_path: Path
) -> None:
    harness = RuntimeHarness(
        core_binary=core_binary,
        database=tmp_path / "state" / "core.db",
        logs_dir=tmp_path / "logs",
    )
    client = None

    try:
        harness.start_core()
        client = await harness.wait_for_core()

        with pytest.raises(GrpcCallError) as caught:
            await client.create_backlog_item(
                item_type="task",
                title="Reject unsupported WP",
                feature_id="qe-feature",
                wp_id="WP01",
            )

        assert caught.value.code == grpc.StatusCode.INVALID_ARGUMENT
        assert "wp_id" in str(caught.value).lower()
    finally:
        if client is not None:
            await client.close()
        harness.cleanup()
