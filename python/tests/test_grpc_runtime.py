"""Hermetic gRPC runtime tests for the AgilePlus MCP client."""

# ruff: noqa: N802

from __future__ import annotations

from collections.abc import AsyncIterator

import grpc
import pytest
import pytest_asyncio

from agileplus_mcp.grpc_client import AgilePlusCoreClient
from agileplus_proto.gen.agileplus.v1 import common_pb2, core_pb2, core_pb2_grpc


def _feature(slug: str = "runtime-feature") -> common_pb2.Feature:
    return common_pb2.Feature(
        id=7,
        slug=slug,
        friendly_name="Runtime Feature",
        state="planned",
        target_branch="main",
        created_at="2026-08-04T00:00:00Z",
        updated_at="2026-08-04T00:00:00Z",
        wp_count=1,
        wp_done=0,
    )


class _LoopbackCoreService(core_pb2_grpc.AgilePlusCoreServiceServicer):
    """Minimal in-process core service exercising the generated wire contract."""

    async def GetFeature(self, request, context):
        return core_pb2.GetFeatureResponse(feature=_feature(request.slug))

    async def ListFeatures(self, request, context):
        return core_pb2.ListFeaturesResponse(features=[_feature("alpha"), _feature("beta")])

    async def GetFeatureState(self, request, context):
        return core_pb2.GetFeatureStateResponse(
            feature_state=common_pb2.FeatureState(
                state="planned", next_command="implement", blockers=["approval"]
            )
        )

    async def ListWorkPackages(self, request, context):
        return core_pb2.ListWorkPackagesResponse(
            packages=[
                common_pb2.WorkPackageStatus(
                    id=11,
                    title="Implement runtime coverage",
                    state="doing",
                    sequence=2,
                    agent_id="agent-1",
                    pr_url="https://example.test/pr/1",
                    pr_state="open",
                    depends_on=[1],
                    file_scope=["python/tests/test_grpc_runtime.py"],
                )
            ]
        )

    async def GetWorkPackageStatus(self, request, context):
        return core_pb2.GetWorkPackageStatusResponse(
            work_package_status=common_pb2.WorkPackageStatus(
                id=11,
                title="Implement runtime coverage",
                state="doing",
                sequence=request.wp_sequence,
            )
        )

    async def CheckGovernanceGate(self, request, context):
        return core_pb2.CheckGovernanceGateResponse(
            passed=False,
            violations=[
                common_pb2.GateViolation(
                    fr_id="FR-1",
                    rule_id="evidence",
                    message="missing proof",
                    remediation="add proof",
                )
            ],
        )

    async def GetAuditTrail(
        self, request, context
    ) -> AsyncIterator[core_pb2.GetAuditTrailResponse]:
        yield core_pb2.GetAuditTrailResponse(
            audit_entry=common_pb2.AuditEntry(
                id=3,
                feature_slug=request.feature_slug,
                wp_sequence=2,
                timestamp="2026-08-04T00:00:00Z",
                actor="runtime-test",
                transition="planned->implementing",
                evidence_refs=["FR-1"],
                prev_hash=b"a" * 32,
                hash=b"b" * 32,
            )
        )

    async def VerifyAuditChain(self, request, context):
        return core_pb2.VerifyAuditChainResponse(valid=True, entries_verified=1)

    async def DispatchCommand(self, request, context):
        return core_pb2.DispatchCommandResponse(
            result=common_pb2.CommandResponse(
                success=True,
                message=f"ran {request.command.command}",
                outputs={"feature_slug": request.command.feature_slug},
            )
        )

    async def StreamAgentEvents(
        self, request, context
    ) -> AsyncIterator[core_pb2.StreamAgentEventsResponse]:
        yield core_pb2.StreamAgentEventsResponse(
            event=common_pb2.AgentEvent(
                event_type="work-package-updated",
                feature_slug=request.feature_slug,
                wp_sequence=2,
                agent_id="agent-1",
                payload="ready",
                timestamp="2026-08-04T00:00:00Z",
            )
        )


@pytest_asyncio.fixture
async def loopback_grpc_address() -> AsyncIterator[str]:
    server = grpc.aio.server()
    core_pb2_grpc.add_AgilePlusCoreServiceServicer_to_server(_LoopbackCoreService(), server)
    port = server.add_insecure_port("127.0.0.1:0")
    await server.start()
    try:
        yield f"127.0.0.1:{port}"
    finally:
        await server.stop(0)


@pytest.mark.asyncio
async def test_client_exercises_real_loopback_service(loopback_grpc_address: str) -> None:
    """The client uses generated stubs against an ephemeral local service."""
    client = AgilePlusCoreClient(loopback_grpc_address)
    await client.connect()
    try:
        assert (await client.get_feature("runtime-feature"))["slug"] == "runtime-feature"
        assert [feature["slug"] for feature in await client.list_features("planned")] == [
            "alpha",
            "beta",
        ]
        assert (await client.get_feature_state("runtime-feature"))["next_command"] == "implement"
        assert (await client.list_work_packages("runtime-feature", "doing"))[0]["sequence"] == 2
        assert (await client.get_work_package_status("runtime-feature", 2))[
            "title"
        ] == "Implement runtime coverage"
        gate = await client.check_governance_gate("runtime-feature", "planned->implementing")
        assert gate == {
            "passed": False,
            "violations": [
                {
                    "fr_id": "FR-1",
                    "rule_id": "evidence",
                    "message": "missing proof",
                    "remediation": "add proof",
                }
            ],
        }
        assert (await client.get_audit_trail("runtime-feature"))[0]["hash"] == (b"b" * 32).hex()
        assert (await client.verify_audit_chain("runtime-feature"))["entries_verified"] == 1
        assert (await client.run_command("plan", "runtime-feature"))["message"] == "ran plan"
        events = [event async for event in client.stream_agent_events("runtime-feature")]
        assert events == [
            {
                "event_type": "work-package-updated",
                "feature_slug": "runtime-feature",
                "wp_sequence": 2,
                "agent_id": "agent-1",
                "payload": "ready",
                "timestamp": "2026-08-04T00:00:00Z",
            }
        ]
    finally:
        await client.close()
