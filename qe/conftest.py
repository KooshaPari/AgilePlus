from __future__ import annotations

from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return Path(__file__).resolve().parents[1]


@pytest.fixture(scope="session")
def core_binary(repo_root: Path) -> Path:
    binary = repo_root / "target" / "debug" / "agileplus-grpc"
    if not binary.is_file():
        pytest.fail(
            "missing prebuilt core binary; run `cargo build --locked "
            "-p agileplus-grpc --bin agileplus-grpc -p agileplus-cli --bin agileplus`"
        )
    return binary
