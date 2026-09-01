"""Unit tests for gRPC helper value conversions."""

from __future__ import annotations

from types import SimpleNamespace

from agileplus_mcp.grpc_errors import GrpcCallError, GrpcConnectionError
from agileplus_mcp.grpc_serialization import AgilePlusGrpcSerializationMixin


def test_grpc_errors_retain_machine_readable_code_and_context() -> None:
    code = SimpleNamespace(name="NOT_FOUND")
    error = GrpcCallError(code, "feature is absent")

    assert error.code is code
    assert str(error) == "gRPC error namespace(name='NOT_FOUND'): feature is absent"
    assert str(GrpcConnectionError("offline")) == "offline"


def test_serialization_helpers_preserve_feature_work_package_and_audit_fields() -> None:
    feature = SimpleNamespace(
        id=1,
        slug="coverage-repair",
        friendly_name="Coverage Repair",
        state="doing",
        target_branch="main",
        created_at="2026-08-29T00:00:00Z",
        updated_at="2026-08-29T00:01:00Z",
        wp_count=2,
        wp_done=1,
    )
    work_package = SimpleNamespace(
        id=2,
        title="Repair coverage",
        state="doing",
        sequence=1,
        agent_id="agent-1",
        pr_url="https://example.invalid/pr/1",
        pr_state="open",
        depends_on=[0],
        file_scope=["python/src/agileplus_mcp/server.py"],
    )
    audit = SimpleNamespace(
        id=3,
        feature_slug="coverage-repair",
        wp_sequence=1,
        timestamp="2026-08-29T00:02:00Z",
        actor="agent-1",
        transition="planned->doing",
        evidence_refs=["test://coverage"],
        prev_hash=b"\x01\x02",
        hash=b"\x03\x04",
    )

    assert AgilePlusGrpcSerializationMixin._feature_to_dict(feature)["slug"] == "coverage-repair"
    assert AgilePlusGrpcSerializationMixin._wp_to_dict(work_package)["depends_on"] == [0]
    assert AgilePlusGrpcSerializationMixin._audit_entry_to_dict(audit)["hash"] == "0304"
