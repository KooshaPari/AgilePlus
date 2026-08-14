"""Shared fixtures and collection policy for integration tests."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

SKIP_REASON = "AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment"
INTEGRATION_TESTS_DIR = Path(__file__).parent.resolve()


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    """Skip integration tests unless the gRPC server is available."""
    if os.environ.get("AGILEPLUS_GRPC_URL"):
        return

    for item in items:
        if Path(item.path).resolve().is_relative_to(INTEGRATION_TESTS_DIR):
            item.add_marker(pytest.mark.skip(reason=SKIP_REASON))


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c
