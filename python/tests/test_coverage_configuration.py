"""Coverage configuration contract tests."""

from __future__ import annotations

import tomllib
from pathlib import Path


def test_generated_proto_is_the_only_coverage_omission() -> None:
    """Generated protobuf code is omitted without masking handwritten coverage."""
    pyproject = Path(__file__).parents[1] / "pyproject.toml"
    configuration = tomllib.loads(pyproject.read_text())

    assert configuration["tool"]["coverage"]["run"]["omit"] == [
        "src/agileplus_proto/gen/**"
    ]
