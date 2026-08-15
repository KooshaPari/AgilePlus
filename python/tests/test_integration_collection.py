from pathlib import Path

from tests.integration import conftest


class Item:
    def __init__(self, path: Path) -> None:
        self.path = path
        self.markers: list[object] = []

    def add_marker(self, marker: object) -> None:
        self.markers.append(marker)


def test_collection_marks_and_skips_only_integration_paths(monkeypatch) -> None:
    monkeypatch.delenv("AGILEPLUS_GRPC_URL", raising=False)
    integration = Item(Path(conftest.__file__).parent / "test_mcp_feature_listing.py")
    unit = Item(Path(__file__).with_name("test_mcp_tools.py"))

    conftest.pytest_collection_modifyitems([integration, unit])

    assert len(integration.markers) == 2
    assert unit.markers == []


def test_collection_marks_but_does_not_skip_integration_with_endpoint(monkeypatch) -> None:
    monkeypatch.setenv("AGILEPLUS_GRPC_URL", "127.0.0.1:50051")
    integration = Item(Path(conftest.__file__).parent / "test_mcp_feature_listing.py")

    conftest.pytest_collection_modifyitems([integration])

    assert len(integration.markers) == 1
