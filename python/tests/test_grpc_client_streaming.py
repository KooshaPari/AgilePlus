"""Streaming behavior tests for the AgilePlus core gRPC client."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from agileplus_mcp.grpc_client import AgilePlusCoreClient


class _StreamingStub:
    """A small fake that models the generated server-streaming RPC shape."""

    def __init__(self) -> None:
        self.request = None

    def StreamAgentEvents(self, request):  # noqa: N802 - generated RPC naming
        self.request = request
        return self._responses()

    async def _responses(self):
        yield SimpleNamespace(
            event=SimpleNamespace(
                event_type="agent_started",
                feature_slug="coverage-gate",
                wp_sequence=14,
                agent_id="worker-7",
                payload="{\"state\":\"running\"}",
                timestamp="2026-08-12T00:00:00Z",
            )
        )


@pytest.mark.asyncio
async def test_stream_agent_events_unwraps_generated_response_event() -> None:
    """The generated response wraps the agent event in its ``event`` field."""
    client = AgilePlusCoreClient()
    stub = _StreamingStub()
    client._stub = stub

    received = [event async for event in client.stream_agent_events("coverage-gate")]

    assert received == [
        {
            "event_type": "agent_started",
            "feature_slug": "coverage-gate",
            "wp_sequence": 14,
            "agent_id": "worker-7",
            "payload": '{"state":"running"}',
            "timestamp": "2026-08-12T00:00:00Z",
        }
    ]
    assert stub.request.feature_slug == "coverage-gate"
