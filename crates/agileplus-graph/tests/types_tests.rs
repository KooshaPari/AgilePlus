// SPDX-License-Identifier: MIT OR Apache-2.0
//! Comprehensive unit tests for agileplus-graph types.

use agileplus_graph::{Node, NodeType, RelType, Relationship};
use serde_json::json;

// ── NodeType ────────────────────────────────────────────────────────────────

#[test]
fn node_type_as_str_all_variants() {
    assert_eq!(NodeType::Feature.as_str(), "Feature");
    assert_eq!(NodeType::WorkPackage.as_str(), "WorkPackage");
    assert_eq!(NodeType::Agent.as_str(), "Agent");
    assert_eq!(NodeType::Label.as_str(), "Label");
    assert_eq!(NodeType::Project.as_str(), "Project");
}

#[test]
fn node_type_eq_and_hash() {
    let a = NodeType::Feature;
    let b = NodeType::Feature;
    assert_eq!(a, b);
    assert_eq!(hash_of(a), hash_of(b));

    let c = NodeType::WorkPackage;
    assert_ne!(a, c);
}

#[test]
fn node_type_debug_format() {
    assert_eq!(format!("{:?}", NodeType::Feature), "Feature");
    assert_eq!(format!("{:?}", NodeType::Project), "Project");
}

#[test]
fn node_type_copy_semantics() {
    let original = NodeType::Agent;
    let copied = original;
    assert_eq!(original, copied);
}

#[test]
fn node_type_clone_semantics() {
    let original = NodeType::Label;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ── RelType ─────────────────────────────────────────────────────────────────

#[test]
fn rel_type_as_str_all_variants() {
    assert_eq!(RelType::Owns.as_str(), "OWNS");
    assert_eq!(RelType::AssignedTo.as_str(), "ASSIGNED_TO");
    assert_eq!(RelType::DependsOn.as_str(), "DEPENDS_ON");
    assert_eq!(RelType::Blocks.as_str(), "BLOCKS");
    assert_eq!(RelType::Tagged.as_str(), "TAGGED");
    assert_eq!(RelType::InProject.as_str(), "IN_PROJECT");
}

#[test]
fn rel_type_eq_and_hash() {
    let a = RelType::DependsOn;
    let b = RelType::DependsOn;
    assert_eq!(a, b);
    assert_eq!(hash_of(a), hash_of(b));

    let c = RelType::Blocks;
    assert_ne!(a, c);
}

#[test]
fn rel_type_debug_format() {
    assert_eq!(format!("{:?}", RelType::Blocks), "Blocks");
    assert_eq!(format!("{:?}", RelType::InProject), "InProject");
}

#[test]
fn rel_type_copy_semantics() {
    let original = RelType::Owns;
    let copied = original;
    assert_eq!(original, copied);
}

#[test]
fn rel_type_clone_semantics() {
    let original = RelType::AssignedTo;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

// ── Node ────────────────────────────────────────────────────────────────────

#[test]
fn node_new_generates_unique_id() {
    let a = Node::new(NodeType::Feature, json!({}));
    let b = Node::new(NodeType::Feature, json!({}));
    assert_ne!(a.id, b.id);
}

#[test]
fn node_new_preserves_type_and_properties() {
    let props = json!({"title": "Auth", "points": 5});
    let node = Node::new(NodeType::Feature, props.clone());
    assert_eq!(node.node_type, NodeType::Feature);
    assert_eq!(node.properties, props);
}

#[test]
fn node_with_id_uses_supplied_id() {
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::WorkPackage, json!({"title": "task"}));
    assert_eq!(node.id, id);
}

#[test]
fn node_with_id_preserves_type_and_properties() {
    let id = uuid::Uuid::new_v4();
    let props = json!({"name": "agent-1", "model": "gpt-4"});
    let node = Node::with_id(id, NodeType::Agent, props.clone());
    assert_eq!(node.node_type, NodeType::Agent);
    assert_eq!(node.properties, props);
}

#[test]
fn node_serialization_roundtrip() {
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::Project, json!({"name": "Acme"}));
    let serialized = serde_json::to_string(&node).unwrap();
    let deserialized: Node = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, id);
    assert_eq!(deserialized.node_type, NodeType::Project);
    assert_eq!(deserialized.properties["name"], "Acme");
}

#[test]
fn node_clone_produces_independent_copy() {
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::Label, json!({"color": "red"}));
    let cloned = node.clone();
    assert_eq!(node.id, cloned.id);
    assert_eq!(node.node_type, cloned.node_type);
    assert_eq!(node.properties, cloned.properties);
}

#[test]
fn node_debug_format_includes_fields() {
    let node = Node::new(NodeType::Feature, json!({"k": "v"}));
    let dbg = format!("{:?}", node);
    assert!(dbg.contains("Node"));
    assert!(dbg.contains("Feature"));
}

#[test]
fn node_properties_empty_object() {
    let node = Node::new(NodeType::Feature, json!({}));
    assert!(node.properties.is_object());
    assert!(node.properties.as_object().unwrap().is_empty());
}

#[test]
fn node_properties_complex_nested() {
    let props = json!({
        "tags": ["backend", "urgent"],
        "meta": {"priority": 1, "assignees": ["alice", "bob"]}
    });
    let node = Node::new(NodeType::WorkPackage, props.clone());
    assert_eq!(node.properties, props);
    assert_eq!(node.properties["meta"]["priority"], 1);
}

// ── Relationship ────────────────────────────────────────────────────────────

#[test]
fn relationship_new_generates_unique_id() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let a = Relationship::new(from, to, RelType::DependsOn);
    let b = Relationship::new(from, to, RelType::DependsOn);
    assert_ne!(a.id, b.id);
}

#[test]
fn relationship_new_default_properties_empty() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::new(from, to, RelType::Owns);
    assert!(rel.properties.is_object());
    assert!(rel.properties.as_object().unwrap().is_empty());
}

#[test]
fn relationship_with_id_uses_supplied_id() {
    let id = uuid::Uuid::new_v4();
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::with_id(id, from, to, RelType::Blocks);
    assert_eq!(rel.id, id);
}

#[test]
fn relationship_with_id_preserves_endpoints_and_type() {
    let id = uuid::Uuid::new_v4();
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::with_id(id, from, to, RelType::AssignedTo);
    assert_eq!(rel.from_node_id, from);
    assert_eq!(rel.to_node_id, to);
    assert_eq!(rel.rel_type, RelType::AssignedTo);
}

#[test]
fn relationship_serialization_roundtrip() {
    let id = uuid::Uuid::new_v4();
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::with_id(id, from, to, RelType::Tagged);
    let serialized = serde_json::to_string(&rel).unwrap();
    let deserialized: Relationship = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.id, id);
    assert_eq!(deserialized.from_node_id, from);
    assert_eq!(deserialized.to_node_id, to);
    assert_eq!(deserialized.rel_type, RelType::Tagged);
}

#[test]
fn relationship_clone_produces_independent_copy() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::new(from, to, RelType::InProject);
    let cloned = rel.clone();
    assert_eq!(rel.id, cloned.id);
    assert_eq!(rel.from_node_id, cloned.from_node_id);
    assert_eq!(rel.to_node_id, cloned.to_node_id);
    assert_eq!(rel.rel_type, cloned.rel_type);
}

#[test]
fn relationship_debug_format_includes_fields() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let rel = Relationship::new(from, to, RelType::DependsOn);
    let dbg = format!("{:?}", rel);
    assert!(dbg.contains("Relationship"));
    assert!(dbg.contains("DependsOn"));
}

#[test]
fn relationship_endpoints_can_be_same_node() {
    let id = uuid::Uuid::new_v4();
    let node = uuid::Uuid::new_v4();
    let rel = Relationship::with_id(id, node, node, RelType::Owns);
    assert_eq!(rel.from_node_id, rel.to_node_id);
}

// ── NodeType serialization ──────────────────────────────────────────────────

#[test]
fn node_type_serialization_roundtrip() {
    let variants = vec![
        NodeType::Feature,
        NodeType::WorkPackage,
        NodeType::Agent,
        NodeType::Label,
        NodeType::Project,
    ];
    for variant in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        let deserialized: NodeType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(variant, deserialized);
    }
}

// ── RelType serialization ───────────────────────────────────────────────────

#[test]
fn rel_type_serialization_roundtrip() {
    let variants = vec![
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ];
    for variant in variants {
        let serialized = serde_json::to_string(&variant).unwrap();
        let deserialized: RelType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(variant, deserialized);
    }
}

// ── NodeType as_str count ───────────────────────────────────────────────────

#[test]
fn node_type_has_five_variants() {
    assert_eq!(
        std::mem::size_of::<NodeType>(),
        1,
        "NodeType should be 1 byte (Niche enum)"
    );
}

#[test]
fn rel_type_has_six_variants() {
    // RelType has 6 variants; ensure enum size is compact
    assert!(
        std::mem::size_of::<RelType>() <= 2,
        "RelType should be compact"
    );
}

// ── NodeType in hash sets ───────────────────────────────────────────────────

#[test]
fn node_type_in_hash_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NodeType::Feature);
    set.insert(NodeType::Feature); // duplicate
    set.insert(NodeType::WorkPackage);
    assert_eq!(set.len(), 2);
    assert!(set.contains(&NodeType::Feature));
    assert!(set.contains(&NodeType::WorkPackage));
}

#[test]
fn rel_type_in_hash_map() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert(RelType::DependsOn, "dep");
    map.insert(RelType::Blocks, "blk");
    map.insert(RelType::DependsOn, "dep2");
    assert_eq!(map.len(), 2);
    assert_eq!(map[&RelType::DependsOn], "dep2");
}

// ── Node / Relationship combined scenarios ──────────────────────────────────

#[test]
fn build_diamond_dependency_graph() {
    // a -> b, a -> c, b -> d, c -> d
    let a = Node::with_id(
        uuid::Uuid::new_v4(),
        NodeType::Feature,
        json!({"name": "a"}),
    );
    let b = Node::with_id(
        uuid::Uuid::new_v4(),
        NodeType::Feature,
        json!({"name": "b"}),
    );
    let c = Node::with_id(
        uuid::Uuid::new_v4(),
        NodeType::Feature,
        json!({"name": "c"}),
    );
    let d = Node::with_id(
        uuid::Uuid::new_v4(),
        NodeType::Feature,
        json!({"name": "d"}),
    );

    let r_ab = Relationship::new(a.id, b.id, RelType::DependsOn);
    let r_ac = Relationship::new(a.id, c.id, RelType::DependsOn);
    let r_bd = Relationship::new(b.id, d.id, RelType::DependsOn);
    let r_cd = Relationship::new(c.id, d.id, RelType::DependsOn);

    // Verify all relationships are distinct
    assert_ne!(r_ab.id, r_ac.id);
    assert_ne!(r_bd.id, r_cd.id);
    assert_ne!(r_ab.id, r_bd.id);

    // All connect back to their endpoints
    assert_eq!(r_ab.from_node_id, a.id);
    assert_eq!(r_ab.to_node_id, b.id);
    assert_eq!(r_cd.from_node_id, c.id);
    assert_eq!(r_cd.to_node_id, d.id);
}

#[test]
fn multiple_rel_types_on_same_pair() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let r1 = Relationship::new(from, to, RelType::DependsOn);
    let r2 = Relationship::new(from, to, RelType::Blocks);
    let r3 = Relationship::new(from, to, RelType::Tagged);

    assert_ne!(r1.id, r2.id);
    assert_ne!(r2.id, r3.id);
    assert_eq!(r1.from_node_id, r2.from_node_id);
    assert_eq!(r1.to_node_id, r3.to_node_id);
}

#[test]
fn node_with_large_properties() {
    let mut properties = serde_json::Map::new();
    for i in 0..100 {
        properties.insert(format!("key_{i}"), json!(format!("value_{}", i * 42)));
    }
    let node = Node::new(
        NodeType::WorkPackage,
        serde_json::Value::Object(properties.clone()),
    );
    assert_eq!(node.properties.as_object().unwrap().len(), 100);
    assert_eq!(node.properties["key_0"], "value_0");
    assert_eq!(node.properties["key_99"], "value_4158");
}

#[test]
fn relationship_properties_can_be_set_directly() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let mut rel = Relationship::new(from, to, RelType::DependsOn);
    // Relationship::new sets empty properties; this confirms that
    assert!(rel.properties.as_object().unwrap().is_empty());
    // We can modify it
    rel.properties = json!({"weight": 10, "label": "critical"});
    assert_eq!(rel.properties["weight"], 10);
    assert_eq!(rel.properties["label"], "critical");
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn hash_of<T: std::hash::Hash>(v: T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish()
}

// ════════════════════════════════════════════════════════════════════════════
// Deepened coverage: string contracts, deep clone, serde edge cases
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn node_type_as_str_values_are_pascal_case() {
    for t in [
        NodeType::Feature,
        NodeType::WorkPackage,
        NodeType::Agent,
        NodeType::Label,
        NodeType::Project,
    ] {
        let s = t.as_str();
        let mut chars = s.chars();
        let first = chars.next().unwrap();
        assert!(first.is_ascii_uppercase(), "{s} should start uppercase");
        assert!(
            s.chars().all(|c| c.is_ascii_alphanumeric()),
            "{s} should be alphanumeric"
        );
    }
}

#[test]
fn rel_type_as_str_values_are_upper_snake() {
    for t in [
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ] {
        let s = t.as_str();
        assert_eq!(s, s.to_uppercase(), "{s} should be uppercase");
        assert!(s.chars().all(|c| c.is_ascii_uppercase() || c == '_'));
    }
}

#[test]
fn rel_type_as_str_values_are_unique() {
    let values = [
        RelType::Owns.as_str(),
        RelType::AssignedTo.as_str(),
        RelType::DependsOn.as_str(),
        RelType::Blocks.as_str(),
        RelType::Tagged.as_str(),
        RelType::InProject.as_str(),
    ];
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 6, "relationship names must be unique");
}

#[test]
fn node_type_as_str_values_are_unique() {
    let values = [
        NodeType::Feature.as_str(),
        NodeType::WorkPackage.as_str(),
        NodeType::Agent.as_str(),
        NodeType::Label.as_str(),
        NodeType::Project.as_str(),
    ];
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 5, "node type names must be unique");
}

#[test]
fn node_properties_null_is_allowed() {
    let node = Node::new(NodeType::Label, serde_json::Value::Null);
    let json = serde_json::to_string(&node).unwrap();
    let back: Node = serde_json::from_str(&json).unwrap();
    assert!(back.properties.is_null());
}

#[test]
fn node_properties_array_is_supported() {
    let node = Node::new(NodeType::Label, json!(["a", "b", "c"]));
    assert_eq!(node.properties.as_array().unwrap().len(), 3);
}

#[test]
fn relationship_with_id_properties_are_empty_object() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    for rel_type in [
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ] {
        let rel = Relationship::with_id(uuid::Uuid::new_v4(), from, to, rel_type);
        assert_eq!(rel.properties, json!({}));
        assert_eq!(rel.rel_type, rel_type);
    }
}

#[test]
fn node_clone_properties_are_deep_copied() {
    let mut node = Node::new(NodeType::Feature, json!({"tags": ["a"]}));
    let clone = node.clone();

    node.properties["tags"] = json!(["b"]);

    assert_eq!(clone.properties["tags"], json!(["a"]));
}

#[test]
fn relationship_clone_properties_are_deep_copied() {
    let mut rel = Relationship::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4(), RelType::Owns);
    rel.properties = json!({"weight": 1});
    let clone = rel.clone();

    rel.properties = json!({"weight": 2});

    assert_eq!(clone.properties["weight"], 1);
    assert_eq!(clone.from_node_id, rel.from_node_id);
}

#[test]
fn node_serde_preserves_uuid_string_form() {
    let id = uuid::Uuid::new_v4();
    let node = Node::with_id(id, NodeType::Agent, json!({"name": "agent"}));
    let value = serde_json::to_value(&node).unwrap();

    assert_eq!(value["id"], json!(id.to_string()));
    assert_eq!(value["node_type"], json!("Agent"));
}

#[test]
fn all_node_types_roundtrip_through_json() {
    for t in [
        NodeType::Feature,
        NodeType::WorkPackage,
        NodeType::Agent,
        NodeType::Label,
        NodeType::Project,
    ] {
        let node = Node::new(t, json!({}));
        let json = serde_json::to_string(&node).unwrap();
        let back: Node = serde_json::from_str(&json).unwrap();
        assert_eq!(back.node_type, t);
    }
}

#[test]
fn all_rel_types_roundtrip_through_json() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    for t in [
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ] {
        let rel = Relationship::new(from, to, t);
        let json = serde_json::to_string(&rel).unwrap();
        let back: Relationship = serde_json::from_str(&json).unwrap();
        assert_eq!(back.rel_type, t);
    }
}

#[test]
fn rel_type_hash_values_are_distinct() {
    let types = [
        RelType::Owns,
        RelType::AssignedTo,
        RelType::DependsOn,
        RelType::Blocks,
        RelType::Tagged,
        RelType::InProject,
    ];
    let hashes: Vec<u64> = types.iter().map(|t| hash_of(*t)).collect();
    let mut unique = hashes.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), types.len(), "hash collision across variants");
}
