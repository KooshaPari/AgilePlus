"""Contract tests for the remaining core-client RPC adapters."""

from __future__ import annotations

from collections.abc import AsyncIterator
from unittest.mock import AsyncMock, MagicMock

import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient
from agileplus_proto.gen.agileplus.v1 import common_pb2, core_pb2


def _client_with_stub() -> tuple[AgilePlusCoreClient, MagicMock]:
    client = AgilePlusCoreClient()
    stub = MagicMock()
    client._stub = stub
    return client, stub


def _work_package() -> common_pb2.WorkPackageStatus:
    return common_pb2.WorkPackageStatus(
        id=14,
        title="Coverage contract",
        state="doing",
        sequence=2,
        agent_id="codex",
        pr_url="https://example.test/pr/14",
        pr_state="open",
        depends_on=[1],
        file_scope=["python/src/agileplus_mcp/grpc_client.py"],
    )


@pytest.mark.asyncio
async def test_work_package_adapters_preserve_request_filters_and_fields() -> None:
    client, stub = _client_with_stub()
    package = _work_package()
    stub.ListWorkPackages = AsyncMock(
        return_value=core_pb2.ListWorkPackagesResponse(packages=[package])
    )
    stub.GetWorkPackageStatus = AsyncMock(
        return_value=core_pb2.GetWorkPackageStatusResponse(work_package_status=package)
    )

    listed = await client.list_work_packages("engine", state="doing")
    status = await client.get_work_package_status("engine", 2)

    assert listed == [status]
    assert status["depends_on"] == [1]
    assert status["file_scope"] == ["python/src/agileplus_mcp/grpc_client.py"]
    list_request = stub.ListWorkPackages.call_args.args[0]
    status_request = stub.GetWorkPackageStatus.call_args.args[0]
    assert (list_request.feature_slug, list_request.state_filter) == ("engine", "doing")
    assert (status_request.feature_slug, status_request.wp_sequence) == ("engine", 2)


async def _audit_responses() -> AsyncIterator[core_pb2.GetAuditTrailResponse]:
    yield core_pb2.GetAuditTrailResponse(
        audit_entry=common_pb2.AuditEntry(
            id=8,
            feature_slug="engine",
            wp_sequence=2,
            timestamp="2026-08-29T00:00:00Z",
            actor="codex",
            transition="planned->doing",
            evidence_refs=["cargo test"],
            prev_hash=b"previous",
            hash=b"current",
        )
    )


@pytest.mark.asyncio
async def test_audit_trail_adapter_streams_canonical_entries() -> None:
    client, stub = _client_with_stub()
    stub.GetAuditTrail.side_effect = lambda _request: _audit_responses()

    entries = await client.get_audit_trail("engine", after_id=7, limit=10)

    assert entries == [
        {
            "id": 8,
            "feature_slug": "engine",
            "wp_sequence": 2,
            "timestamp": "2026-08-29T00:00:00Z",
            "actor": "codex",
            "transition": "planned->doing",
            "evidence_refs": ["cargo test"],
            "prev_hash": "70726576696f7573",
            "hash": "63757272656e74",
        }
    ]
    request = stub.GetAuditTrail.call_args.args[0]
    assert (request.feature_slug, request.after_id, request.limit) == ("engine", 7, 10)


@pytest.mark.asyncio
async def test_governance_adapter_preserves_failed_violation_details() -> None:
    client, stub = _client_with_stub()
    stub.CheckGovernanceGate = AsyncMock(
        return_value=core_pb2.CheckGovernanceGateResponse(
            passed=False,
            violations=[
                common_pb2.GateViolation(
                    fr_id="FR-14",
                    rule_id="evidence-required",
                    message="missing audit evidence",
                    remediation="attach command output",
                )
            ],
        )
    )

    result = await client.check_governance_gate("engine", "doing->review")

    assert result == {
        "passed": False,
        "violations": [
            {
                "fr_id": "FR-14",
                "rule_id": "evidence-required",
                "message": "missing audit evidence",
                "remediation": "attach command output",
            }
        ],
    }
