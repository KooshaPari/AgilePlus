// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::work_items_service_server::{
    WorkItemsService, WorkItemsServiceServer,
};
use crate::agileplus::v1::*;
use prost::Message;

// ── proto3 semantics ─────────────────────────────────────────────────────────

#[test]
fn default_messages_are_empty() {
    assert_eq!(Feature::default(), Feature::default());
    assert!(FeatureState::default().blockers.is_empty());
    assert!(FeatureState::default().governance.is_none());
    assert!(GetFeatureResponse::default().feature.is_none());
    assert_eq!(CommandRequest::default().args.len(), 0);
    assert_eq!(ListProjectsRequest::default(), ListProjectsRequest {});
}

#[test]
fn default_values_roundtrip_to_empty_buffer() {
    let mut buf = Vec::new();
    Feature::default().encode(&mut buf).expect("encode");
    assert!(buf.is_empty(), "proto3 defaults must be omitted");
    assert_eq!(Feature::default().encoded_len(), 0);
    let decoded = Feature::decode(&buf[..]).expect("decode");
    assert_eq!(decoded, Feature::default());
}

#[test]
fn option_none_is_omitted_and_some_is_preserved() {
    let none = GetFeatureResponse { feature: None };
    assert_eq!(none.encoded_len(), 0);
    assert!(none.feature.is_none());

    let some = GetFeatureResponse {
        feature: Some(Feature {
            id: 1,
            slug: "x".to_string(),
            ..Default::default()
        }),
    };
    assert!(some.encoded_len() > 0);

    let mut buf = Vec::new();
    some.encode(&mut buf).expect("encode");
    let decoded = GetFeatureResponse::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.feature.expect("present").slug, "x");
}

#[test]
fn empty_repeated_vectors_roundtrip() {
    let msg = SyncRepositoryResponse {
        stories_synced: 0,
        stories_skipped: 0,
        errors: Vec::new(),
    };
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encode");
    assert_eq!(buf.len(), 0);
    assert_eq!(
        SyncRepositoryResponse::decode(&buf[..]).expect("decode"),
        msg
    );
}

#[test]
fn bytes_fields_preserve_non_utf8_content() {
    let entry = AuditEntry {
        prev_hash: vec![0xff, 0x00, 0x7f, 0x80],
        hash: vec![0x01, 0x02],
        ..Default::default()
    };
    let mut buf = Vec::new();
    entry.encode(&mut buf).expect("encode");
    let decoded = AuditEntry::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.prev_hash, vec![0xff, 0x00, 0x7f, 0x80]);
    assert_eq!(decoded.hash, vec![0x01, 0x02]);
}

#[test]
fn unicode_strings_roundtrip() {
    let msg = Feature {
        slug: "crème-brûlée-🚀".to_string(),
        friendly_name: "日本語テスト".to_string(),
        ..Default::default()
    };
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encode");
    let decoded = Feature::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.slug, "crème-brûlée-🚀");
    assert_eq!(decoded.friendly_name, "日本語テスト");
}

#[test]
fn numeric_extremes_roundtrip() {
    let msg = GetAuditTrailRequest {
        feature_slug: "f".to_string(),
        after_id: i64::MAX,
        project_scope: None,
    };
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encode");
    assert_eq!(
        GetAuditTrailRequest::decode(&buf[..])
            .expect("decode")
            .after_id,
        i64::MAX
    );

    let neg = GetWorkPackageStatusRequest {
        feature_slug: "f".to_string(),
        wp_sequence: i32::MIN,
        project_scope: None,
    };
    let mut buf = Vec::new();
    neg.encode(&mut buf).expect("encode");
    assert_eq!(
        GetWorkPackageStatusRequest::decode(&buf[..])
            .expect("decode")
            .wp_sequence,
        i32::MIN
    );
}

#[test]
fn map_fields_roundtrip_all_entries() {
    let mut args = std::collections::HashMap::new();
    args.insert("a".to_string(), "1".to_string());
    args.insert("b".to_string(), "2".to_string());
    args.insert("c".to_string(), "3".to_string());

    let msg = CommandResponse {
        success: true,
        message: "ok".to_string(),
        outputs: args,
    };
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encode");
    let decoded = CommandResponse::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.outputs.len(), 3);
    assert_eq!(decoded.outputs.get("a"), Some(&"1".to_string()));
    assert_eq!(decoded.outputs.get("c"), Some(&"3".to_string()));
}

#[test]
fn merge_combines_concatenated_messages() {
    let first = FeatureState {
        state: "Planned".to_string(),
        ..Default::default()
    };
    let second = FeatureState {
        blockers: vec!["b1".to_string()],
        ..Default::default()
    };
    assert_eq!(second.state, "", "second message leaves state at default");

    let mut buf = Vec::new();
    first.encode(&mut buf).expect("encode first");
    second.encode(&mut buf).expect("encode second");
    assert_eq!(first.encoded_len() + second.encoded_len(), buf.len());

    let mut merged = FeatureState::default();
    merged.merge(&buf[..]).expect("merge");
    assert_eq!(merged.state, "Planned");
    assert_eq!(merged.blockers, vec!["b1".to_string()]);
}

#[test]
fn merge_overwrites_scalar_with_last_value() {
    let mut buf = Vec::new();
    Feature {
        slug: "first".to_string(),
        ..Default::default()
    }
    .encode(&mut buf)
    .expect("encode first");
    Feature {
        slug: "second".to_string(),
        ..Default::default()
    }
    .encode(&mut buf)
    .expect("encode second");

    let decoded = Feature::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.slug, "second");
}

// ── decode validation ────────────────────────────────────────────────────────

#[test]
fn decoding_empty_buffer_yields_default() {
    let decoded = FeatureState::decode(&[][..]).expect("empty decode");
    assert_eq!(decoded, FeatureState::default());
}

#[test]
fn decoding_truncated_buffer_errors() {
    let mut buf = Vec::new();
    Feature {
        slug: "a-long-slug-value".to_string(),
        ..Default::default()
    }
    .encode(&mut buf)
    .expect("encode");
    let truncated = &buf[..buf.len() - 1];
    assert!(
        Feature::decode(truncated).is_err(),
        "truncated length-delimited field must fail to decode"
    );
}

#[test]
fn decoding_rejects_invalid_wire_type_payload() {
    // Field 1, varint wire type, but truncated varint value.
    let bad = [0x08_u8, 0x80];
    assert!(Feature::decode(&bad[..]).is_err());
}

#[test]
fn unknown_fields_are_skipped() {
    // tag for field 99, varint wire type -> 99 << 3 | 0 = 792 -> 0x98 0x06
    let bytes = [0x98_u8, 0x06, 0x01];
    let decoded = Feature::decode(&bytes[..]).expect("unknown fields ignored");
    assert_eq!(decoded, Feature::default());
}

#[test]
fn decode_uses_defaults_for_missing_fields() {
    let mut buf = Vec::new();
    Feature {
        id: 5,
        slug: "s".to_string(),
        ..Default::default()
    }
    .encode(&mut buf)
    .expect("encode");

    let decoded = Feature::decode(&buf[..]).expect("decode");
    assert_eq!(decoded.id, 5);
    assert_eq!(decoded.wp_count, 0);
    assert_eq!(decoded.friendly_name, "");
}

// ── equality / clone semantics ───────────────────────────────────────────────

#[test]
fn equality_reflects_field_changes() {
    let base = WorkPackageStatus {
        id: 1,
        title: "WP".to_string(),
        ..Default::default()
    };
    let mut changed = base.clone();
    changed.pr_state = "merged".to_string();
    assert_ne!(base, changed);
    changed.pr_state.clear();
    assert_eq!(base, changed);
}

#[test]
fn clone_produces_independent_vectors_and_maps() {
    let mut original = CommandRequest {
        command: "c".to_string(),
        feature_slug: "f".to_string(),
        args: [("k".to_string(), "v".to_string())]
            .into_iter()
            .collect(),
    };
    let cloned = original.clone();
    original.args.insert("k2".to_string(), "v2".to_string());
    original.command.push_str("-mutated");

    assert_eq!(cloned.args.len(), 1);
    assert_eq!(cloned.command, "c");
    assert_eq!(original.args.len(), 2);
}

#[test]
fn hashable_messages_work_as_map_keys() {
    use std::collections::HashMap;
    let mut map: HashMap<ProjectScope, u32> = HashMap::new();
    map.insert(
        ProjectScope {
            canonical_repo_root: "/a".to_string(),
        },
        1,
    );
    map.insert(
        ProjectScope {
            canonical_repo_root: "/b".to_string(),
        },
        2,
    );
    assert_eq!(
        map.get(&ProjectScope {
            canonical_repo_root: "/a".to_string()
        }),
        Some(&1)
    );
}

#[test]
fn debug_formatting_contains_field_values() {
    let msg = Feature {
        id: 3,
        slug: "dbg".to_string(),
        ..Default::default()
    };
    let rendered = format!("{msg:?}");
    assert!(rendered.contains("dbg"), "got {rendered}");
    assert!(rendered.contains("Feature"), "got {rendered}");
}

// ── tonic conversions ────────────────────────────────────────────────────────

#[test]
fn tonic_request_into_inner_preserves_payload() {
    let request = tonic::Request::new(GetFeatureRequest {
        slug: "conv".to_string(),
        project_scope: None,
    });
    let inner = request.into_inner();
    assert_eq!(inner.slug, "conv");
}

#[test]
fn tonic_response_get_ref_and_into_inner() {
    let response = tonic::Response::new(GetFeatureResponse {
        feature: Some(Feature {
            id: 9,
            slug: "resp".to_string(),
            ..Default::default()
        }),
    });
    assert_eq!(response.get_ref().feature.as_ref().expect("feature").id, 9);
    let inner = response.into_inner();
    assert_eq!(inner.feature.expect("feature").slug, "resp");
}

#[test]
fn tonic_request_metadata_is_mutable() {
    let mut request = tonic::Request::new(VerifyAuditChainRequest {
        feature_slug: "f".to_string(),
        project_scope: None,
    });
    request
        .metadata_mut()
        .insert("x-agileplus-agent", "jcode".parse().expect("valid header"));
    assert_eq!(
        request.metadata().get("x-agileplus-agent").unwrap(),
        "jcode"
    );
}

#[test]
fn tonic_request_with_extensions_roundtrips_payload() {
    let mut request = tonic::Request::new(DispatchCommandRequest {
        command: Some(CommandRequest {
            command: "validate".to_string(),
            ..Default::default()
        }),
        project_scope: None,
    });
    request.extensions_mut().insert(7_u32);
    assert_eq!(request.extensions().get::<u32>(), Some(&7));
    assert_eq!(
        request.into_inner().command.expect("command").command,
        "validate"
    );
}

// ── service wiring ───────────────────────────────────────────────────────────

/// Minimal [`WorkItemsService`] implementation used to validate the generated
/// server wrapper and `NamedService` wiring without a running gRPC transport.
struct DummyWorkItemsService;

#[tonic::async_trait]
impl WorkItemsService for DummyWorkItemsService {
    async fn list_projects(
        &self,
        _request: tonic::Request<ListProjectsRequest>,
    ) -> Result<tonic::Response<ListProjectsResponse>, tonic::Status> {
        Ok(tonic::Response::new(ListProjectsResponse::default()))
    }

    async fn list_epics(
        &self,
        _request: tonic::Request<ListEpicsRequest>,
    ) -> Result<tonic::Response<ListEpicsResponse>, tonic::Status> {
        Ok(tonic::Response::new(ListEpicsResponse::default()))
    }

    async fn list_stories(
        &self,
        _request: tonic::Request<ListStoriesRequest>,
    ) -> Result<tonic::Response<ListStoriesResponse>, tonic::Status> {
        Ok(tonic::Response::new(ListStoriesResponse::default()))
    }

    async fn sync_repository(
        &self,
        _request: tonic::Request<SyncRepositoryRequest>,
    ) -> Result<tonic::Response<SyncRepositoryResponse>, tonic::Status> {
        Ok(tonic::Response::new(SyncRepositoryResponse {
            stories_synced: 1,
            ..Default::default()
        }))
    }
}

#[test]
fn work_items_service_server_exposes_expected_name() {
    use tonic::server::NamedService;
    let _server = WorkItemsServiceServer::new(DummyWorkItemsService);
    assert_eq!(
        WorkItemsServiceServer::<DummyWorkItemsService>::NAME,
        "agileplus.v1.WorkItemsService"
    );
}
