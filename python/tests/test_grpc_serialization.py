"""Contract-preserving tests for gRPC response serialization."""

from __future__ import annotations

from types import SimpleNamespace

from agileplus_mcp.grpc_serialization import AgilePlusGrpcSerializationMixin


def test_feature_serialization_preserves_lifecycle_fields() -> None:
    feature = SimpleNamespace(
        id=42,
        slug="coverage-gate",
        friendly_name="Coverage gate",
        state="planned",
        target_branch="main",
        created_at="2026-08-01T00:00:00Z",
        updated_at="2026-08-12T00:00:00Z",
        wp_count=4,
        wp_done=2,
    )

    assert AgilePlusGrpcSerializationMixin._feature_to_dict(feature) == {
        "id": 42,
        "slug": "coverage-gate",
        "friendly_name": "Coverage gate",
        "state": "planned",
        "target_branch": "main",
        "created_at": "2026-08-01T00:00:00Z",
        "updated_at": "2026-08-12T00:00:00Z",
        "wp_count": 4,
        "wp_done": 2,
    }


def test_work_package_serialization_copies_repeated_fields() -> None:
    work_package = SimpleNamespace(
        id="WP14",
        title="Restore coverage",
        state="in_progress",
        sequence=14,
        agent_id="worker-7",
        pr_url="https://example.test/pr/953",
        pr_state="open",
        depends_on=["WP13"],
        file_scope=["python/src/agileplus_mcp"],
    )

    serialized = AgilePlusGrpcSerializationMixin._wp_to_dict(work_package)

    assert serialized["depends_on"] == ["WP13"]
    assert serialized["file_scope"] == ["python/src/agileplus_mcp"]
    assert serialized["depends_on"] is not work_package.depends_on
    assert serialized["file_scope"] is not work_package.file_scope


def test_audit_serialization_hex_encodes_hashes_and_copies_evidence() -> None:
    entry = SimpleNamespace(
        id=17,
        feature_slug="coverage-gate",
        wp_sequence=14,
        timestamp="2026-08-12T00:00:00Z",
        actor="codex",
        transition="planned->implemented",
        evidence_refs=["test://focused"],
        prev_hash=b"\x00\xff",
        hash=b"\x12\x34",
    )

    serialized = AgilePlusGrpcSerializationMixin._audit_entry_to_dict(entry)

    assert serialized["prev_hash"] == "00ff"
    assert serialized["hash"] == "1234"
    assert serialized["evidence_refs"] == ["test://focused"]
    assert serialized["evidence_refs"] is not entry.evidence_refs
