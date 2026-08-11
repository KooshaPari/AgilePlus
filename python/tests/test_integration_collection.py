"""Regression tests for the integration collection policy."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

import pytest

from tests.integration.conftest import pytest_collection_modifyitems


@dataclass
class _CollectedItem:
    """Minimal pytest-item stand-in that records collection markers."""

    path: Path
    marker_names: set[str] = field(default_factory=set)

    def add_marker(self, marker: pytest.MarkDecorator) -> None:
        self.marker_names.add(marker.mark.name)


def test_collection_marks_and_skips_only_integration_path_without_endpoint(monkeypatch):
    """A missing gRPC endpoint must not suppress unit or contract tests."""
    monkeypatch.delenv("AGILEPLUS_GRPC_URL", raising=False)
    integration_item = _CollectedItem(Path(__file__).parent / "integration" / "test_live.py")
    unit_item = _CollectedItem(Path(__file__).parent / "test_tools.py")

    pytest_collection_modifyitems([integration_item, unit_item])

    assert integration_item.marker_names == {"integration", "skip"}
    assert unit_item.marker_names == set()


def test_collection_marks_but_does_not_skip_integration_path_with_endpoint(monkeypatch):
    """A configured gRPC endpoint keeps integration tests selected and runnable."""
    monkeypatch.setenv("AGILEPLUS_GRPC_URL", "localhost:50051")
    integration_item = _CollectedItem(Path(__file__).parent / "integration" / "test_live.py")
    unit_item = _CollectedItem(Path(__file__).parent / "test_tools.py")

    pytest_collection_modifyitems([integration_item, unit_item])

    assert integration_item.marker_names == {"integration"}
    assert unit_item.marker_names == set()
