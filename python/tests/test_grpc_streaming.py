"""Unit tests for finite agent-event streaming."""

from __future__ import annotations

from types import SimpleNamespace

import pytest

from agileplus_mcp.grpc_streaming import AgilePlusGrpcStreamingMixin


class StreamingClient(AgilePlusGrpcStreamingMixin):
    def __init__(self, stub: object) -> None:
        self.stub = stub

    def _require_stub(self) -> object:
        return self.stub


@pytest.mark.asyncio
async def test_stream_agent_events_maps_a_clean_finite_stream() -> None:
    seen_requests: list[object] = []

    async def events():
        yield SimpleNamespace(
            event_type="started",
            feature_slug="coverage-repair",
            wp_sequence=1,
            agent_id="agent-1",
            payload="working",
            timestamp="2026-08-29T00:00:00Z",
        )

    class Stub:
        def StreamAgentEvents(self, request: object):  # noqa: N802
            seen_requests.append(request)
            return events()

    client = StreamingClient(Stub())
    received = [event async for event in client.stream_agent_events("coverage-repair")]

    assert received == [
        {
            "event_type": "started",
            "feature_slug": "coverage-repair",
            "wp_sequence": 1,
            "agent_id": "agent-1",
            "payload": "working",
            "timestamp": "2026-08-29T00:00:00Z",
        }
    ]
    assert seen_requests[0].feature_slug == "coverage-repair"
