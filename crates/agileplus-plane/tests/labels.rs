//! Integration tests for labels module.
//!
//! Covers: PlaneLabel serialization, CreateLabelRequest, LabelSync construction.

use agileplus_plane::labels::{CreateLabelRequest, LabelSync, PlaneLabel};
use agileplus_plane::PlaneClient;

// ── PlaneLabel ──────────────────────────────────────────────

#[test]
fn plane_label_serializes_with_color() {
    let label = PlaneLabel {
        id: "abc".into(),
        name: "bug".into(),
        color: Some("#ff0000".into()),
    };
    let json = serde_json::to_string(&label).unwrap();
    assert!(json.contains("abc"));
    assert!(json.contains("bug"));
    assert!(json.contains("#ff0000"));
}

#[test]
fn plane_label_serializes_without_color() {
    let label = PlaneLabel {
        id: "xyz".into(),
        name: "feature".into(),
        color: None,
    };
    let json = serde_json::to_string(&label).unwrap();
    assert!(json.contains("xyz"));
    assert!(json.contains("feature"));
}

#[test]
fn plane_label_deserializes_with_color() {
    let json = r#"{"id":"abc","name":"bug","color":"red"}"#;
    let label: PlaneLabel = serde_json::from_str(json).unwrap();
    assert_eq!(label.id, "abc");
    assert_eq!(label.name, "bug");
    assert_eq!(label.color, Some("red".into()));
}

#[test]
fn plane_label_deserializes_without_color() {
    let json = r#"{"id":"xyz","name":"feature"}"#;
    let label: PlaneLabel = serde_json::from_str(json).unwrap();
    assert_eq!(label.id, "xyz");
    assert!(label.color.is_none());
}

#[test]
fn plane_label_clone() {
    let label = PlaneLabel {
        id: "1".into(),
        name: "test".into(),
        color: Some("#000".into()),
    };
    let cloned = label.clone();
    assert_eq!(cloned.id, label.id);
    assert_eq!(cloned.name, label.name);
    assert_eq!(cloned.color, label.color);
}

#[test]
fn plane_label_debug_format() {
    let label = PlaneLabel {
        id: "1".into(),
        name: "test".into(),
        color: None,
    };
    let debug = format!("{:?}", label);
    assert!(debug.contains("PlaneLabel"));
    assert!(debug.contains("test"));
}

#[test]
fn plane_label_field_comparison() {
    let label = PlaneLabel {
        id: "1".into(),
        name: "bug".into(),
        color: None,
    };
    assert_eq!(label.id, "1");
    assert_eq!(label.name, "bug");
    assert!(label.color.is_none());
}

// ── CreateLabelRequest ──────────────────────────────────────

#[test]
fn create_label_request_with_color() {
    let req = CreateLabelRequest {
        name: "urgent".into(),
        color: Some("#ff0000".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("urgent"));
    assert!(json.contains("#ff0000"));
}

#[test]
fn create_label_request_without_color_omits_field() {
    let req = CreateLabelRequest {
        name: "enhancement".into(),
        color: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("enhancement"));
    // skip_serializing_if = "Option::is_none" should omit the field
    assert!(!json.contains("color"));
}

#[test]
fn create_label_request_debug_format() {
    let req = CreateLabelRequest {
        name: "bug".into(),
        color: None,
    };
    let debug = format!("{:?}", req);
    assert!(debug.contains("CreateLabelRequest"));
}

#[test]
fn create_label_request_clone() {
    let req = CreateLabelRequest {
        name: "test".into(),
        color: Some("#abc".into()),
    };
    let cloned = req.clone();
    assert_eq!(cloned.name, "test");
    assert_eq!(cloned.color, Some("#abc".into()));
}

// ── LabelSync ───────────────────────────────────────────────

#[test]
fn label_sync_construction() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "key".into(),
        "slug".into(),
        "project".into(),
    );
    let sync = LabelSync::new(client);
    let debug = format!("{:?}", sync);
    assert!(debug.contains("LabelSync"));
}

#[test]
fn label_sync_debug_format() {
    let client = PlaneClient::new(
        "http://localhost".into(),
        "key".into(),
        "slug".into(),
        "project".into(),
    );
    let sync = LabelSync::new(client);
    let debug = format!("{:?}", sync);
    assert!(debug.contains("LabelSync"));
}
