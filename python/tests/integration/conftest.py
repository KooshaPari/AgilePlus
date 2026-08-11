"""Shared fixtures and collection policy for integration tests."""

from __future__ import annotations

import os
from pathlib import Path

import pytest

SKIP_REASON = "AGILEPLUS_GRPC_URL not set; skipped outside Docker Compose environment"
INTEGRATION_DIR = Path(__file__).parent.resolve()


def pytest_collection_modifyitems(items: list[pytest.Item]) -> None:
    """Mark integration-path tests and skip only those without a live server."""
    integration_items = [
        item for item in items if item.path.resolve().is_relative_to(INTEGRATION_DIR)
    ]
    for item in integration_items:
        item.add_marker(pytest.mark.integration)

    if os.environ.get("AGILEPLUS_GRPC_URL"):
        return

    skip_marker = pytest.mark.skip(reason=SKIP_REASON)
    for item in integration_items:
        item.add_marker(skip_marker)


@pytest.fixture
async def client():
    """Provide a connected AgilePlus gRPC client."""
    from agileplus_mcp.grpc_client import connect_client

    address = os.environ.get("AGILEPLUS_GRPC_URL", "localhost:50051")
    async with connect_client(address) as c:
        yield c
