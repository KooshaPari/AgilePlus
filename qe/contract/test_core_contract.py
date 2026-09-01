from __future__ import annotations

from pathlib import Path

import pytest

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
