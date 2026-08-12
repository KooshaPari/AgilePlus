"""Tests for gRPC error boundaries exposed to MCP callers."""

from __future__ import annotations

from agileplus_mcp.grpc_errors import GrpcCallError, GrpcConnectionError


def test_grpc_call_error_retains_code_and_human_context() -> None:
    code = object()

    error = GrpcCallError(code, "permission denied")

    assert error.code is code
    assert str(error) == f"gRPC error {code}: permission denied"


def test_grpc_connection_error_is_an_exception_boundary() -> None:
    assert isinstance(GrpcConnectionError("unavailable"), Exception)
