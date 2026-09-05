from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest


os.environ.setdefault("PYTHONDONTWRITEBYTECODE", "1")
sys.dont_write_bytecode = True

REPO_ROOT = Path(__file__).resolve().parents[1]
PYTHON_SRC = REPO_ROOT / "python" / "src"
if str(PYTHON_SRC) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC))


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return REPO_ROOT


@pytest.fixture(scope="session")
def core_binary(repo_root: Path) -> Path:
    binary = repo_root / "target" / "debug" / "agileplus-grpc"
    if not binary.is_file():
        pytest.fail(
            "missing prebuilt core binary; run `cargo build --locked "
            "-p agileplus-grpc --bin agileplus-grpc -p agileplus-cli --bin agileplus`"
        )
    return binary
