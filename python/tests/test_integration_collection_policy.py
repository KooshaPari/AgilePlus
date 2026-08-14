"""Regression tests for integration-test collection policy."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import MagicMock

from tests.integration.conftest import pytest_collection_modifyitems


def test_collection_policy_skips_only_integration_tests(monkeypatch) -> None:
    """A missing gRPC endpoint must not skip regular unit tests."""
    monkeypatch.delenv("AGILEPLUS_GRPC_URL", raising=False)
    integration_item = MagicMock(path=Path("tests/integration/test_mcp_workflow.py"))
    unit_item = MagicMock(path=Path("tests/test_tools.py"))

    pytest_collection_modifyitems([integration_item, unit_item])

    integration_item.add_marker.assert_called_once()
    unit_item.add_marker.assert_not_called()
