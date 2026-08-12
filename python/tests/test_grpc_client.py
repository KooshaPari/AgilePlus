"""Unit tests for AgilePlusCoreClient with mock gRPC stubs.

Traceability: WP14-T081
"""

from __future__ import annotations

from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock

import pytest

from agileplus_mcp.grpc_client import (
    AgilePlusCoreClient,
    GrpcCallError,
    GrpcConnectionError,
    connect_client,
)

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


def _make_mock_feature(slug: str = "test-feat") -> MagicMock:
    f = MagicMock()
    f.id = 1
    f.slug = slug
    f.friendly_name = "Test Feature"
    f.state = "created"
    f.target_branch = "main"
    f.created_at = "2026-01-01T00:00:00Z"
    f.updated_at = "2026-01-01T00:00:00Z"
    f.wp_count = 0
    f.wp_done = 0
    return f


def _make_mock_stub() -> MagicMock:
    stub = MagicMock()
    # Feature RPCs
    feature_response = MagicMock()
    feature_response.feature = _make_mock_feature()
    stub.GetFeature = AsyncMock(return_value=feature_response)

    list_response = MagicMock()
    list_response.features = [_make_mock_feature("feat-a"), _make_mock_feature("feat-b")]
    stub.ListFeatures = AsyncMock(return_value=list_response)

    state_response = MagicMock()
    state_response.feature_state = MagicMock(state="created", next_command="specify", blockers=[])
    stub.GetFeatureState = AsyncMock(return_value=state_response)

    # Command dispatch
    cmd_response = MagicMock()
    cmd_response.result = MagicMock(success=True, message="ok", outputs={})
    stub.DispatchCommand = AsyncMock(return_value=cmd_response)

    # Governance
    gate_response = MagicMock()
    gate_response.passed = False
    gate_response.violations = []
    stub.CheckGovernanceGate = AsyncMock(return_value=gate_response)

    # Audit
    verify_response = MagicMock()
    verify_response.valid = True
    verify_response.entries_verified = 3
    verify_response.first_invalid_id = ""
    verify_response.error_message = ""
    stub.VerifyAuditChain = AsyncMock(return_value=verify_response)

    work_package = SimpleNamespace(
        id=14,
        title="Restore coverage",
        state="in_progress",
        sequence=14,
        agent_id="worker-7",
        pr_url="https://example.test/pr/953",
        pr_state="open",
        depends_on=[13],
        file_scope=["python/src/agileplus_mcp/grpc_client.py"],
    )
    work_packages_response = SimpleNamespace(packages=[work_package])
    stub.ListWorkPackages = AsyncMock(return_value=work_packages_response)
    stub.GetWorkPackageStatus = AsyncMock(
        return_value=SimpleNamespace(work_package_status=work_package)
    )

    gate_response.violations = [
        SimpleNamespace(
            fr_id="FR-042",
            rule_id="review-required",
            message="Missing independent review",
            remediation="Request a review",
        )
    ]

    audit_entry = SimpleNamespace(
        id=17,
        feature_slug="test-feat",
        wp_sequence=14,
        timestamp="2026-08-12T00:00:00Z",
        actor="worker-7",
        transition="planned->implemented",
        evidence_refs=["test://focused"],
        prev_hash=b"\x00\xff",
        hash=b"\x12\x34",
    )

    def get_audit_trail(_request):
        return _stream(SimpleNamespace(audit_entry=audit_entry))

    stub.GetAuditTrail = get_audit_trail

    return stub


async def _stream(*responses):
    for response in responses:
        yield response


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------


@pytest.fixture
def client_with_stub():
    client = AgilePlusCoreClient("localhost:50051")
    client._stub = _make_mock_stub()
    return client


@pytest.mark.asyncio
async def test_get_feature_returns_dict(client_with_stub):
    result = await client_with_stub.get_feature("test-feat")
    assert result["slug"] == "test-feat"
    assert result["friendly_name"] == "Test Feature"
    assert result["state"] == "created"


@pytest.mark.asyncio
async def test_list_features_returns_list(client_with_stub):
    results = await client_with_stub.list_features()
    assert len(results) == 2
    slugs = {r["slug"] for r in results}
    assert "feat-a" in slugs
    assert "feat-b" in slugs


@pytest.mark.asyncio
async def test_list_features_with_state_filter(client_with_stub):
    # Just verify the request is built with the filter
    results = await client_with_stub.list_features(state="created")
    assert isinstance(results, list)


@pytest.mark.asyncio
async def test_get_feature_state(client_with_stub):
    result = await client_with_stub.get_feature_state("test-feat")
    assert result["state"] == "created"
    assert result["next_command"] == "specify"


@pytest.mark.asyncio
async def test_run_command_returns_dict(client_with_stub):
    result = await client_with_stub.run_command("specify", feature_slug="test-feat")
    assert result["success"] is True
    assert result["message"] == "ok"


@pytest.mark.asyncio
async def test_run_command_preserves_stringified_arguments(client_with_stub):
    result = await client_with_stub.run_command(
        "validate", feature_slug="test-feat", dry_run=True, retries=3
    )

    assert result["outputs"] == {}
    request = client_with_stub._stub.DispatchCommand.await_args.args[0]
    assert request.command.command == "validate"
    assert request.command.feature_slug == "test-feat"
    assert dict(request.command.args) == {"dry_run": "True", "retries": "3"}


@pytest.mark.asyncio
async def test_check_governance_gate_serializes_blocking_violations(client_with_stub):
    result = await client_with_stub.check_governance_gate("test-feat", "specified->planned")
    assert result["passed"] is False
    assert result["violations"] == [
        {
            "fr_id": "FR-042",
            "rule_id": "review-required",
            "message": "Missing independent review",
            "remediation": "Request a review",
        }
    ]


@pytest.mark.asyncio
async def test_list_work_packages_serializes_scope_and_state_filter(client_with_stub):
    packages = await client_with_stub.list_work_packages("test-feat", state="in_progress")

    assert packages == [
        {
            "id": 14,
            "title": "Restore coverage",
            "state": "in_progress",
            "sequence": 14,
            "agent_id": "worker-7",
            "pr_url": "https://example.test/pr/953",
            "pr_state": "open",
            "depends_on": [13],
            "file_scope": ["python/src/agileplus_mcp/grpc_client.py"],
        }
    ]
    request = client_with_stub._stub.ListWorkPackages.await_args.args[0]
    assert request.feature_slug == "test-feat"
    assert request.state_filter == "in_progress"


@pytest.mark.asyncio
async def test_get_work_package_status_uses_feature_and_sequence(client_with_stub):
    status = await client_with_stub.get_work_package_status("test-feat", 14)

    assert status["sequence"] == 14
    assert status["depends_on"] == [13]
    request = client_with_stub._stub.GetWorkPackageStatus.await_args.args[0]
    assert (request.feature_slug, request.wp_sequence) == ("test-feat", 14)


@pytest.mark.asyncio
async def test_get_audit_trail_streams_and_serializes_hash_chain_entries(client_with_stub):
    entries = await client_with_stub.get_audit_trail("test-feat", after_id=16)

    assert entries == [
        {
            "id": 17,
            "feature_slug": "test-feat",
            "wp_sequence": 14,
            "timestamp": "2026-08-12T00:00:00Z",
            "actor": "worker-7",
            "transition": "planned->implemented",
            "evidence_refs": ["test://focused"],
            "prev_hash": "00ff",
            "hash": "1234",
        }
    ]


@pytest.mark.asyncio
async def test_verify_audit_chain(client_with_stub):
    result = await client_with_stub.verify_audit_chain("test-feat")
    assert result["valid"] is True
    assert result["entries_verified"] == 3


@pytest.mark.asyncio
async def test_require_stub_raises_when_not_connected():
    client = AgilePlusCoreClient("localhost:50051")
    with pytest.raises(GrpcConnectionError, match="Not connected"):
        client._require_stub()


@pytest.mark.asyncio
async def test_close_resets_channel_and_stub_after_channel_close(client_with_stub):
    channel = MagicMock()
    channel.close = AsyncMock()
    client_with_stub._channel = channel

    await client_with_stub.close()

    channel.close.assert_awaited_once()
    assert client_with_stub._channel is None
    assert client_with_stub._stub is None


@pytest.mark.asyncio
async def test_connect_client_closes_client_after_context_exit(monkeypatch):
    created = MagicMock()
    created.connect = AsyncMock()
    created.close = AsyncMock()
    monkeypatch.setattr("agileplus_mcp.grpc_client.AgilePlusCoreClient", lambda address: created)

    async with connect_client("core.example:50051") as client:
        assert client is created

    created.connect.assert_awaited_once()
    created.close.assert_awaited_once()


@pytest.mark.asyncio
async def test_connect_builds_stub_after_the_channel_is_ready(monkeypatch):
    """A successful connection must not publish a stub before readiness."""
    import grpc

    from agileplus_proto.gen.agileplus.v1 import core_pb2_grpc

    client = AgilePlusCoreClient("core.example:50051")
    channel = MagicMock()
    channel.channel_ready = AsyncMock()
    stub = MagicMock()
    channel_factory = MagicMock(return_value=channel)
    stub_factory = MagicMock(return_value=stub)
    monkeypatch.setattr(grpc.aio, "insecure_channel", channel_factory)
    monkeypatch.setattr(core_pb2_grpc, "AgilePlusCoreServiceStub", stub_factory)

    await client.connect()

    channel_factory.assert_called_once_with("core.example:50051")
    channel.channel_ready.assert_awaited_once()
    stub_factory.assert_called_once_with(channel)
    assert client._channel is channel
    assert client._stub is stub


@pytest.mark.asyncio
async def test_connect_maps_channel_creation_failure_to_connection_error(monkeypatch):
    """Transport construction failures remain actionable public client errors."""
    import grpc

    client = AgilePlusCoreClient("core.example:50051")
    monkeypatch.setattr(
        grpc.aio,
        "insecure_channel",
        MagicMock(side_effect=RuntimeError("resolver unavailable")),
    )

    with pytest.raises(GrpcConnectionError, match=r"Failed to connect to core\.example:50051"):
        await client.connect()


@pytest.mark.asyncio
async def test_retry_on_unavailable(client_with_stub):
    """Verify that transient UNAVAILABLE errors are retried."""
    try:
        import grpc
    except ImportError:
        pytest.skip("grpcio not installed")

    call_count = 0
    original = client_with_stub._stub.GetFeature

    async def flaky(*args, **kwargs):
        nonlocal call_count
        call_count += 1
        if call_count < 2:
            exc = grpc.aio.AioRpcError(
                grpc.StatusCode.UNAVAILABLE,
                initial_metadata=grpc.aio.Metadata(),
                trailing_metadata=grpc.aio.Metadata(),
                details="try again",
            )
            raise exc
        return await original(*args, **kwargs)

    client_with_stub._stub.GetFeature = flaky
    client_with_stub._retry_delay = 0.01  # Speed up test
    result = await client_with_stub.get_feature("test-feat")
    assert result["slug"] == "test-feat"
    assert call_count == 2


@pytest.mark.asyncio
async def test_retry_maps_non_transient_rpc_error_to_call_error(client_with_stub):
    import grpc

    async def denied():
        raise grpc.aio.AioRpcError(
            grpc.StatusCode.PERMISSION_DENIED,
            initial_metadata=grpc.aio.Metadata(),
            trailing_metadata=grpc.aio.Metadata(),
            details="not authorized",
        )

    with pytest.raises(GrpcCallError, match="PERMISSION_DENIED") as error:
        await client_with_stub._call_with_retry(denied)

    assert error.value.code == grpc.StatusCode.PERMISSION_DENIED


@pytest.mark.asyncio
async def test_retry_exhaustion_raises_connection_error_without_silent_success(client_with_stub):
    import grpc

    attempts = 0

    async def unavailable():
        nonlocal attempts
        attempts += 1
        raise grpc.aio.AioRpcError(
            grpc.StatusCode.UNAVAILABLE,
            initial_metadata=grpc.aio.Metadata(),
            trailing_metadata=grpc.aio.Metadata(),
            details="core unavailable",
        )

    client_with_stub._retry_delay = 0
    with pytest.raises(GrpcConnectionError, match="after 3 retries"):
        await client_with_stub._call_with_retry(unavailable)

    assert attempts == 3
