"""Shared fixtures and collection policy for integration tests."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

SKIP_REASON = "AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment"
INTEGRATION_ROOT = Path(__file__).parent.resolve()


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    """Mark integration tests and skip them only when their endpoint is absent."""
    for item in items:
        if not item.path.resolve().is_relative_to(INTEGRATION_ROOT):
            continue
        item.add_marker(pytest.mark.integration)
        if not os.environ.get("AGILEPLUS_GRPC_URL"):
            item.add_marker(pytest.mark.skip(reason=SKIP_REASON))


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c
