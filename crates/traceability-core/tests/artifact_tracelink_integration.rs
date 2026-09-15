//! Integration tests for Artifact, ArtifactRef, TraceLink, and Neo4j schema.
//!
//! These tests exercise cross-type serialization, error handling,
//! and Neo4j schema generation.

use uuid::Uuid;

use traceability_core::{
    Artifact, ArtifactKind, ArtifactRef, LinkKind, Neo4jSchema, TraceLink, TraceLinkError,
    TraceLinkType, CORE_TRACE_LINK_TYPES, NEO4J_NODE_LABELS, NEO4J_RELATIONSHIP_TYPES,
    ids::{NfrId, RequirementId},
};

// ---------------------------------------------------------------------------
// Artifact construction and serialization
// ---------------------------------------------------------------------------

#[test]
fn artifact_full_construction_and_metadata() {
    let project = Uuid::new_v4();
    let mut a = Artifact::new(project, ArtifactKind::Requirement, "FR-1 must validate input");
    a.description = Some("Detailed validation requirement".into());
    a.external_id = Some("FR-1".into());
    a.metadata.insert(
        "priority".into(),
        serde_json::Value::String("P0".into()),
    );
    a.metadata.insert(
        "owner".into(),
        serde_json::Value::String("team-platform".into()),
    );

    let json = serde_json::to_string(&a).unwrap();
    let back: Artifact = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, a.id);
    assert_eq!(back.project_id, project);
    assert_eq!(back.kind, ArtifactKind::Requirement);
    assert_eq!(back.title, "FR-1 must validate input");
    assert_eq!(back.description, Some("Detailed validation requirement".into()));
    assert_eq!(back.external_id, Some("FR-1".into()));
    assert_eq!(back.metadata.len(), 2);
    assert_eq!(
        back.metadata.get("priority"),
        Some(&serde_json::Value::String("P0".into()))
    );
}

// ---------------------------------------------------------------------------
// ArtifactRef all variants
// ---------------------------------------------------------------------------

#[test]
fn artifact_ref_all_variants_kind_str() {
    let cases: Vec<(ArtifactRef, &str)> = vec![
        (
            ArtifactRef::Requirement {
                id: RequirementId::from_string("FR-1"),
            },
            "requirement",
        ),
        (
            ArtifactRef::NonFunctionalRequirement {
                id: NfrId::from_string("NFR-PERF-1"),
            },
            "nfr",
        ),
        (
            ArtifactRef::Test {
                id: "T-1".into(),
            },
            "test",
        ),
        (
            ArtifactRef::CodeEntity {
                id: "lib::main".into(),
                lang: "rust".into(),
            },
            "code",
        ),
        (
            ArtifactRef::Journey {
                id: "J-1".into(),
            },
            "journey",
        ),
        (
            ArtifactRef::AgentRun {
                id: "AR-1".into(),
            },
            "agent",
        ),
        (
            ArtifactRef::Evidence {
                id: "ev-1".into(),
                sha256: "a".repeat(64),
            },
            "evidence",
        ),
        (
            ArtifactRef::Document {
                id: "doc-1".into(),
                range: Some("1-10".into()),
            },
            "document",
        ),
    ];
    for (r, expected) in cases {
        assert_eq!(r.kind_str(), expected);
    }
}

#[test]
fn artifact_ref_serde_roundtrip_all_variants() {
    let refs = vec![
        ArtifactRef::Requirement {
            id: RequirementId::from_string("FR-42"),
        },
        ArtifactRef::NonFunctionalRequirement {
            id: NfrId::from_string("NFR-SEC-1"),
        },
        ArtifactRef::Test {
            id: "T-99".into(),
        },
        ArtifactRef::CodeEntity {
            id: "mod::fn".into(),
            lang: "python".into(),
        },
        ArtifactRef::Journey {
            id: "J-10".into(),
        },
        ArtifactRef::AgentRun {
            id: "AR-5".into(),
        },
        ArtifactRef::Evidence {
            id: "ev-2".into(),
            sha256: "b".repeat(64),
        },
        ArtifactRef::Document {
            id: "doc-7".into(),
            range: None,
        },
    ];
    for r in &refs {
        let json = serde_json::to_string(r).unwrap();
        let back: ArtifactRef = serde_json::from_str(&json).unwrap();
        assert_eq!(&back, r);
    }
}

// ---------------------------------------------------------------------------
// TraceLink creation and validation
// ---------------------------------------------------------------------------

#[test]
fn tracelink_self_loop_rejected() {
    let id = Uuid::new_v4();
    let result = TraceLink::new(Uuid::new_v4(), id, id, TraceLinkType::Satisfies);
    assert!(matches!(result, Err(TraceLinkError::SelfLoop)));
}

#[test]
fn tracelink_valid_creation() {
    let project = Uuid::new_v4();
    let src = Uuid::new_v4();
    let tgt = Uuid::new_v4();
    let link = TraceLink::new(project, src, tgt, TraceLinkType::Verifies).unwrap();

    assert_eq!(link.project_id, project);
    assert_eq!(link.source_artifact_id, src);
    assert_eq!(link.target_artifact_id, tgt);
    assert_eq!(link.link_type, TraceLinkType::Verifies);
    assert_eq!(link.confidence, 1.0);
    assert!(link.created_at.is_some());
    assert!(link.updated_at.is_some());
    assert!(link.metadata.is_empty());
    assert!(link.rationale.is_none());
    assert!(link.is_core());
}

#[test]
fn tracelink_confidence_validation() {
    let make = || {
        TraceLink::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            TraceLinkType::Satisfies,
        )
        .unwrap()
    };

    assert!(make().with_confidence(0.0).is_ok());
    assert!(make().with_confidence(0.5).is_ok());
    assert!(make().with_confidence(1.0).is_ok());
    assert!(make().with_confidence(-0.1).is_err());
    assert!(make().with_confidence(1.1).is_err());
}

#[test]
fn tracelink_is_core_for_each_type() {
    for &ty in CORE_TRACE_LINK_TYPES {
        let link = TraceLink::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ty,
        )
        .unwrap();
        assert!(link.is_core());
    }

    for ty in [
        TraceLinkType::Refines,
        TraceLinkType::ConflictsWith,
        TraceLinkType::Duplicates,
    ] {
        let link = TraceLink::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ty,
        )
        .unwrap();
        assert!(!link.is_core());
    }
}

#[test]
fn tracelink_serde_roundtrip_with_typed_refs() {
    let mut link = TraceLink::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        TraceLinkType::Verifies,
    )
    .unwrap();
    link.from = ArtifactRef::Requirement {
        id: RequirementId::from_string("FR-77"),
    };
    link.to = ArtifactRef::Test {
        id: "checkout/test-verifies-receipt".into(),
    };
    link.confidence = 0.85;
    link.rationale = Some("verified by integration test".into());
    link.metadata.insert(
        "env".into(),
        serde_json::Value::String("staging".into()),
    );

    let json = serde_json::to_string(&link).unwrap();
    let back: TraceLink = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, link.id);
    assert_eq!(back.from, link.from);
    assert_eq!(back.to, link.to);
    assert_eq!(back.confidence, 0.85);
    assert_eq!(back.rationale, Some("verified by integration test".into()));
    assert_eq!(
        back.metadata.get("env"),
        Some(&serde_json::Value::String("staging".into()))
    );
}

// ---------------------------------------------------------------------------
// TraceLinkType db strings
// ---------------------------------------------------------------------------

#[test]
fn tracelink_type_all_db_strings() {
    assert_eq!(TraceLinkType::Satisfies.as_db_str(), "SATISFIES");
    assert_eq!(TraceLinkType::Verifies.as_db_str(), "VERIFIES");
    assert_eq!(TraceLinkType::Implements.as_db_str(), "IMPLEMENTS");
    assert_eq!(TraceLinkType::DerivesFrom.as_db_str(), "DERIVES_FROM");
    assert_eq!(TraceLinkType::Refines.as_db_str(), "REFINES");
    assert_eq!(TraceLinkType::ConflictsWith.as_db_str(), "CONFLICTS_WITH");
    assert_eq!(TraceLinkType::Duplicates.as_db_str(), "DUPLICATES");
}

#[test]
fn tracelink_type_serde_roundtrip() {
    let types = [
        TraceLinkType::Satisfies,
        TraceLinkType::Verifies,
        TraceLinkType::Implements,
        TraceLinkType::DerivesFrom,
        TraceLinkType::Refines,
        TraceLinkType::ConflictsWith,
        TraceLinkType::Duplicates,
    ];
    for ty in types {
        let json = serde_json::to_string(&ty).unwrap();
        let back: TraceLinkType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ty);
    }
}

// ---------------------------------------------------------------------------
// Neo4j schema
// ---------------------------------------------------------------------------

#[test]
fn neo4j_schema_all_statements_are_idempotent() {
    let stmts = Neo4jSchema::all_statements();
    assert!(stmts.len() >= 7);
    for s in &stmts {
        assert!(s.contains("IF NOT EXISTS"));
    }
}

#[test]
fn neo4j_node_labels_count() {
    assert_eq!(NEO4J_NODE_LABELS.len(), 9);
}

#[test]
fn neo4j_relationship_types_count() {
    assert_eq!(NEO4J_RELATIONSHIP_TYPES.len(), 7);
}

#[test]
fn neo4j_schema_node_label_for_all_kinds() {
    let cases = [
        (ArtifactKind::Requirement, "Requirement"),
        (ArtifactKind::Design, "Design"),
        (ArtifactKind::Code, "Code"),
        (ArtifactKind::Test, "Test"),
        (ArtifactKind::Evidence, "Evidence"),
        (ArtifactKind::Risk, "Risk"),
        (ArtifactKind::Rationale, "Rationale"),
    ];
    for (kind, expected_label) in cases {
        assert_eq!(Neo4jSchema::node_label_for(kind), expected_label);
    }
}

#[test]
fn neo4j_schema_relationship_label_for_all_types() {
    let types = [
        TraceLinkType::Satisfies,
        TraceLinkType::Verifies,
        TraceLinkType::Implements,
        TraceLinkType::DerivesFrom,
        TraceLinkType::Refines,
        TraceLinkType::ConflictsWith,
        TraceLinkType::Duplicates,
    ];
    for ty in types {
        assert_eq!(Neo4jSchema::relationship_label_for(ty), ty.as_db_str());
    }
}

// ---------------------------------------------------------------------------
// TraceLinkError display
// ---------------------------------------------------------------------------

#[test]
fn tracelink_error_all_variants_display() {
    let e = TraceLinkError::SelfLoop;
    assert!(e.to_string().contains("must differ"));

    let e = TraceLinkError::WrongArtifactKind {
        expected: ArtifactKind::Requirement,
        got: ArtifactKind::Code,
    };
    let msg = e.to_string();
    assert!(msg.contains("REQUIREMENT"));
    assert!(msg.contains("Code"));

    let e = TraceLinkError::BadConfidence(1.5);
    assert!(e.to_string().contains("1.5"));
}

// ---------------------------------------------------------------------------
// LinkKind alias
// ---------------------------------------------------------------------------

#[test]
fn link_kind_is_trace_link_type() {
    fn _assert_link_kind_is_trace_link_type() {
        let _: LinkKind = TraceLinkType::Verifies;
    }
    _assert_link_kind_is_trace_link_type();
}

// ---------------------------------------------------------------------------
// ArtifactKind serde roundtrip
// ---------------------------------------------------------------------------

#[test]
fn artifact_kind_serde_uses_lowercase() {
    for kind in [
        ArtifactKind::Requirement,
        ArtifactKind::Design,
        ArtifactKind::Code,
        ArtifactKind::Test,
        ArtifactKind::Evidence,
        ArtifactKind::Risk,
        ArtifactKind::Rationale,
    ] {
        let json = serde_json::to_string(&kind).unwrap();
        assert!(json.starts_with('"'));
        assert!(json.ends_with('"'));
        let back: ArtifactKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kind);
    }
}

// ---------------------------------------------------------------------------
// Requirement construction
// ---------------------------------------------------------------------------

#[test]
fn requirement_new_validates_kind() {
    let artifact = Artifact::new(Uuid::new_v4(), ArtifactKind::Requirement, "FR-1");
    let req = traceability_core::Requirement::new(artifact).unwrap();
    assert_eq!(req.status, traceability_core::RequirementStatus::Draft);
    assert!(req.acceptance_criteria.is_empty());
    assert!(req.verification_method.is_none());
}

#[test]
fn requirement_rejects_wrong_kind() {
    let artifact = Artifact::new(Uuid::new_v4(), ArtifactKind::Code, "not a requirement");
    let result = traceability_core::Requirement::new(artifact);
    assert!(matches!(result, Err(TraceLinkError::WrongArtifactKind { .. })));
}

#[test]
fn requirement_new_unchecked_skips_validation() {
    let artifact = Artifact::new(Uuid::new_v4(), ArtifactKind::Test, "test artifact");
    let req = traceability_core::Requirement::new_unchecked(artifact);
    assert_eq!(req.artifact.kind, ArtifactKind::Test);
}

#[test]
fn requirement_status_terminal_and_in_progress() {
    assert!(traceability_core::RequirementStatus::Verified.is_terminal());
    assert!(traceability_core::RequirementStatus::Deprecated.is_terminal());
    assert!(traceability_core::RequirementStatus::Rejected.is_terminal());
    assert!(!traceability_core::RequirementStatus::Draft.is_terminal());
    assert!(!traceability_core::RequirementStatus::Proposed.is_terminal());
    assert!(!traceability_core::RequirementStatus::Approved.is_terminal());
    assert!(!traceability_core::RequirementStatus::Implemented.is_terminal());

    assert!(traceability_core::RequirementStatus::Draft.is_in_progress());
    assert!(!traceability_core::RequirementStatus::Verified.is_in_progress());
}

#[test]
fn verification_method_db_strings() {
    assert_eq!(traceability_core::VerificationMethod::Test.as_db_str(), "test");
    assert_eq!(traceability_core::VerificationMethod::Analysis.as_db_str(), "analysis");
    assert_eq!(traceability_core::VerificationMethod::Inspection.as_db_str(), "inspection");
    assert_eq!(
        traceability_core::VerificationMethod::Demonstration.as_db_str(),
        "demonstration"
    );
    assert_eq!(traceability_core::VerificationMethod::Review.as_db_str(), "review");
}

#[test]
fn requirement_full_serde_roundtrip() {
    let artifact = Artifact::new(Uuid::new_v4(), ArtifactKind::Requirement, "FR-42");
    let mut req = traceability_core::Requirement::new(artifact).unwrap();
    req.status = traceability_core::RequirementStatus::Approved;
    req.priority = Some(1);
    req.rationale = Some("business critical".into());
    req.push_acceptance_criterion("must return 200");
    req.push_acceptance_criterion("must validate input");
    req.verification_method = Some(traceability_core::VerificationMethod::Test);

    let json = serde_json::to_string(&req).unwrap();
    let back: traceability_core::Requirement = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, traceability_core::RequirementStatus::Approved);
    assert_eq!(back.priority, Some(1));
    assert_eq!(back.acceptance_criteria.len(), 2);
    assert_eq!(
        back.verification_method,
        Some(traceability_core::VerificationMethod::Test)
    );
}
