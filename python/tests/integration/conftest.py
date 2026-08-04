"""Shared fixtures and collection policy for integration tests."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

SKIP_REASON = "AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment"


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    """Skip only external-deployment tests unless a gRPC endpoint is configured."""
    if os.environ.get("AGILEPLUS_GRPC_URL"):
        return

    skip_marker = pytest.mark.skip(reason=SKIP_REASON)
    integration_directory = Path(__file__).parent
    for item in items:
        if integration_directory in item.path.parents:
            item.add_marker(skip_marker)


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c
