//! Integration tests for the sync subcommand.
//!
//! Covers: SyncConfig persistence, SyncReport construction, SyncArgs construction,
//! type serialization, conflict resolution enum.

#![cfg(feature = "sync")]

use agileplus_subcmds::{
    AutoSyncAction, ConflictResolution, SyncAutoArgs, SyncConfig, SyncConflict,
    SyncDirection, SyncItemOutcome, SyncPullArgs, SyncPushArgs, SyncReport, SyncReportEntry,
    SyncResolveArgs, SyncStatusArgs, SyncStatusRow, SyncSubcommand,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// SyncConfig: default
// ---------------------------------------------------------------------------

#[test]
fn sync_config_default_is_disabled() {
    let cfg = SyncConfig::default();
    assert!(!cfg.auto_sync_enabled);
}

// ---------------------------------------------------------------------------
// SyncConfig: save and load
// ---------------------------------------------------------------------------

#[test]
fn sync_config_save_and_load_round_trip() {
    let tmp = TempDir::new().unwrap();
    let cfg = SyncConfig {
        auto_sync_enabled: true,
    };
    cfg.save(tmp.path()).unwrap();

    let loaded = SyncConfig::load(tmp.path()).unwrap();
    assert!(loaded.auto_sync_enabled);
}

#[test]
fn sync_config_load_missing_returns_default() {
    let tmp = TempDir::new().unwrap();
    let cfg = SyncConfig::load(tmp.path()).unwrap();
    assert!(!cfg.auto_sync_enabled);
}

#[test]
fn sync_config_save_creates_agileplus_dir() {
    let tmp = TempDir::new().unwrap();
    let cfg = SyncConfig::default();
    cfg.save(tmp.path()).unwrap();

    assert!(tmp.path().join(".agileplus/sync-config.json").exists());
}

#[test]
fn sync_config_overwrite() {
    let tmp = TempDir::new().unwrap();

    SyncConfig {
        auto_sync_enabled: true,
    }
    .save(tmp.path())
    .unwrap();
    SyncConfig {
        auto_sync_enabled: false,
    }
    .save(tmp.path())
    .unwrap();

    let loaded = SyncConfig::load(tmp.path()).unwrap();
    assert!(!loaded.auto_sync_enabled);
}

// ---------------------------------------------------------------------------
// SyncConfig: serde round-trip
// ---------------------------------------------------------------------------

#[test]
fn sync_config_serde_round_trip() {
    let cfg = SyncConfig {
        auto_sync_enabled: true,
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let restored: SyncConfig = serde_json::from_str(&json).unwrap();
    assert!(restored.auto_sync_enabled);
}

// ---------------------------------------------------------------------------
// SyncReport: construction
// ---------------------------------------------------------------------------

#[test]
fn sync_report_new_starts_empty() {
    let r = SyncReport::new(SyncDirection::Push);
    assert_eq!(r.direction, SyncDirection::Push);
    assert!(r.entries.is_empty());
    assert_eq!(r.duration_ms, 0);
}

#[test]
fn sync_report_add_entry() {
    let mut r = SyncReport::new(SyncDirection::Pull);
    r.add(SyncReportEntry {
        entity_kind: "feature".into(),
        entity_name: "auth-flow".into(),
        outcome: SyncItemOutcome::Created,
        plane_id: Some("#1".into()),
        message: None,
    });
    assert_eq!(r.entries.len(), 1);
    assert_eq!(r.entries[0].entity_name, "auth-flow");
}

#[test]
fn sync_report_serde_round_trip() {
    let mut r = SyncReport::new(SyncDirection::Push);
    r.duration_ms = 123;
    r.add(SyncReportEntry {
        entity_kind: "wp".into(),
        entity_name: "db-schema".into(),
        outcome: SyncItemOutcome::Updated,
        plane_id: Some("#42".into()),
        message: Some("updated".into()),
    });

    let json = serde_json::to_string(&r).unwrap();
    let restored: SyncReport = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.direction, SyncDirection::Push);
    assert_eq!(restored.duration_ms, 123);
    assert_eq!(restored.entries.len(), 1);
    assert_eq!(restored.entries[0].outcome, SyncItemOutcome::Updated);
}

// ---------------------------------------------------------------------------
// SyncDirection serialization
// ---------------------------------------------------------------------------

#[test]
fn sync_direction_serde() {
    let push_json = serde_json::to_string(&SyncDirection::Push).unwrap();
    assert_eq!(push_json, "\"push\"");

    let pull_json = serde_json::to_string(&SyncDirection::Pull).unwrap();
    assert_eq!(pull_json, "\"pull\"");

    let restored: SyncDirection = serde_json::from_str("\"push\"").unwrap();
    assert_eq!(restored, SyncDirection::Push);
}

// ---------------------------------------------------------------------------
// SyncItemOutcome serialization
// ---------------------------------------------------------------------------

#[test]
fn sync_item_outcome_serde_all_variants() {
    let variants = [
        (SyncItemOutcome::Created, "\"created\""),
        (SyncItemOutcome::Updated, "\"updated\""),
        (SyncItemOutcome::Skipped, "\"skipped\""),
        (SyncItemOutcome::Conflict, "\"conflict\""),
        (SyncItemOutcome::Imported, "\"imported\""),
    ];

    for (variant, expected_json) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected_json, "Failed for {variant:?}");
        let restored: SyncItemOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, variant);
    }
}

// ---------------------------------------------------------------------------
// SyncStatusRow serialization
// ---------------------------------------------------------------------------

#[test]
fn sync_status_row_serde() {
    let row = SyncStatusRow {
        entity_kind: "Feature".into(),
        entity_name: "auth-flow".into(),
        local_state: "implementing".into(),
        remote_state: Some("in_progress".into()),
        last_synced: None,
        in_sync: true,
        conflict_count: 0,
    };
    let json = serde_json::to_string(&row).unwrap();
    let restored: SyncStatusRow = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.entity_name, "auth-flow");
    assert!(restored.in_sync);
    assert_eq!(restored.conflict_count, 0);
}

// ---------------------------------------------------------------------------
// SyncConflict serialization
// ---------------------------------------------------------------------------

#[test]
fn sync_conflict_serde() {
    let conflict = SyncConflict {
        entity_kind: "feature".into(),
        entity_id: "5".into(),
        entity_name: "api-design".into(),
        local_state: "researched".into(),
        local_description: "Initial API design".into(),
        remote_state: "unstarted".into(),
        remote_description: "API design".into(),
    };
    let json = serde_json::to_string(&conflict).unwrap();
    let restored: SyncConflict = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.entity_id, "5");
    assert_eq!(restored.local_state, "researched");
}

// ---------------------------------------------------------------------------
// ConflictResolution equality
// ---------------------------------------------------------------------------

#[test]
fn conflict_resolution_variants_are_distinct() {
    let resolutions = [
        ConflictResolution::KeepLocal,
        ConflictResolution::AcceptRemote,
        ConflictResolution::MergeManually,
        ConflictResolution::Cancel,
    ];
    for (i, a) in resolutions.iter().enumerate() {
        for (j, b) in resolutions.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SyncArgs construction
// ---------------------------------------------------------------------------

#[test]
fn sync_push_args_defaults() {
    let args = SyncPushArgs {
        feature: None,
        dry_run: false,
    };
    assert!(args.feature.is_none());
    assert!(!args.dry_run);
}

#[test]
fn sync_push_args_with_feature() {
    let args = SyncPushArgs {
        feature: Some("my-feature".into()),
        dry_run: true,
    };
    assert_eq!(args.feature.as_deref(), Some("my-feature"));
    assert!(args.dry_run);
}

#[test]
fn sync_pull_args_defaults() {
    let args = SyncPullArgs {
        feature: None,
        dry_run: false,
    };
    assert!(args.feature.is_none());
    assert!(!args.dry_run);
}

#[test]
fn sync_auto_args_default_action() {
    let args = SyncAutoArgs {
        action: AutoSyncAction::Status,
    };
    assert!(matches!(args.action, AutoSyncAction::Status));
}

#[test]
fn sync_status_args_default() {
    let args = SyncStatusArgs {
        output: "table".into(),
    };
    assert_eq!(args.output, "table");
}

#[test]
fn sync_resolve_args_construction() {
    let args = SyncResolveArgs {
        entity_type: "feature".into(),
        entity_id: "42".into(),
    };
    assert_eq!(args.entity_type, "feature");
    assert_eq!(args.entity_id, "42");
}

// ---------------------------------------------------------------------------
// SyncSubcommand dispatch variants
// ---------------------------------------------------------------------------

#[test]
fn sync_subcommand_push_variant() {
    let sub = SyncSubcommand::Push(SyncPushArgs {
        feature: Some("x".into()),
        dry_run: false,
    });
    assert!(matches!(sub, SyncSubcommand::Push(_)));
}

#[test]
fn sync_subcommand_pull_variant() {
    let sub = SyncSubcommand::Pull(SyncPullArgs {
        feature: None,
        dry_run: true,
    });
    assert!(matches!(sub, SyncSubcommand::Pull(_)));
}

#[test]
fn sync_subcommand_auto_variant() {
    let sub = SyncSubcommand::Auto(SyncAutoArgs {
        action: AutoSyncAction::On,
    });
    assert!(matches!(sub, SyncSubcommand::Auto(_)));
}

#[test]
fn sync_subcommand_status_variant() {
    let sub = SyncSubcommand::Status(SyncStatusArgs {
        output: "json".into(),
    });
    assert!(matches!(sub, SyncSubcommand::Status(_)));
}

#[test]
fn sync_subcommand_resolve_variant() {
    let sub = SyncSubcommand::Resolve(SyncResolveArgs {
        entity_type: "wp".into(),
        entity_id: "7".into(),
    });
    assert!(matches!(sub, SyncSubcommand::Resolve(_)));
}
