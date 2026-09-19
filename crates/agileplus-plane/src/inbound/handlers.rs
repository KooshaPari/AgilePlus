use agileplus_domain::domain::state_machine::FeatureState;
use anyhow::Result;

use crate::{
    content_hash::compute_content_hash, state_mapper::PlaneStateMapper, webhook::PlaneWebhookIssue,
};

use super::{InboundOutcome, LocalEntityStore};

pub(super) fn handle_create<S: LocalEntityStore>(
    mapper: &PlaneStateMapper,
    auto_import_enabled: bool,
    webhook_issue: PlaneWebhookIssue,
    store: &mut S,
) -> Result<InboundOutcome> {
    if store.get_content_hash(&webhook_issue.id).is_some() {
        return handle_update(mapper, webhook_issue, store);
    }

    if auto_import_enabled {
        let state = mapped_state(mapper, &webhook_issue);
        store.auto_import(&webhook_issue, state)?;
        tracing::info!(
            plane_issue_id = webhook_issue.id,
            title = webhook_issue.name,
            "auto-imported new Plane.so work item"
        );
        Ok(InboundOutcome::AutoImported {
            issue_id: webhook_issue.id,
            title: webhook_issue.name,
        })
    } else {
        tracing::debug!(
            plane_issue_id = webhook_issue.id,
            "work item not tracked; auto-import disabled"
        );
        Ok(InboundOutcome::NotTracked {
            issue_id: webhook_issue.id,
        })
    }
}

pub(super) fn handle_update<S: LocalEntityStore>(
    mapper: &PlaneStateMapper,
    webhook_issue: PlaneWebhookIssue,
    store: &mut S,
) -> Result<InboundOutcome> {
    let Some(existing_hash) = store.get_content_hash(&webhook_issue.id) else {
        return Ok(InboundOutcome::NotTracked {
            issue_id: webhook_issue.id,
        });
    };

    let new_state = mapped_state(mapper, &webhook_issue);
    let new_hash = compute_content_hash(
        &webhook_issue.name,
        webhook_issue.state.as_deref().unwrap_or(""),
        &new_state.to_string(),
        &webhook_issue.labels,
    );

    if new_hash == existing_hash {
        tracing::debug!(
            plane_issue_id = webhook_issue.id,
            "work item content hash unchanged; skipping"
        );
        return Ok(InboundOutcome::Unchanged {
            issue_id: webhook_issue.id,
        });
    }

    store.apply_update(&webhook_issue.id, new_state, new_hash.clone())?;
    tracing::info!(
        plane_issue_id = webhook_issue.id,
        new_state = ?new_state,
        "applied inbound work item update from Plane.so"
    );
    Ok(InboundOutcome::Updated {
        issue_id: webhook_issue.id,
        new_hash,
        new_state,
    })
}

pub(super) fn handle_delete<S: LocalEntityStore>(
    issue_id: String,
    store: &mut S,
) -> Result<InboundOutcome> {
    if store.get_content_hash(&issue_id).is_none() {
        return Ok(InboundOutcome::NotTracked {
            issue_id: issue_id.clone(),
        });
    }

    store.mark_archived(&issue_id)?;
    tracing::info!(
        plane_issue_id = issue_id,
        "archived deleted Plane.so work item"
    );
    Ok(InboundOutcome::Archived { issue_id })
}

fn mapped_state(mapper: &PlaneStateMapper, webhook_issue: &PlaneWebhookIssue) -> FeatureState {
    let state_group = webhook_issue.state.as_deref().unwrap_or("backlog");
    mapper.map_plane_state(state_group, state_group)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::content_hash::compute_content_hash;
    use crate::state_mapper::PlaneStateMapper;
    use crate::webhook::PlaneWebhookIssue;

    #[derive(Default)]
    struct MockStore {
        hashes: HashMap<String, String>,
        archived: Vec<String>,
        imported: Vec<String>,
        fail_apply: bool,
        fail_import: bool,
        fail_archive: bool,
    }

    impl MockStore {
        fn with_tracked(id: &str, hash: &str) -> Self {
            let mut s = Self::default();
            s.hashes.insert(id.to_string(), hash.to_string());
            s
        }
    }

    impl super::LocalEntityStore for MockStore {
        fn get_content_hash(&self, id: &str) -> Option<String> {
            self.hashes.get(id).cloned()
        }

        fn apply_update(
            &mut self,
            id: &str,
            _state: FeatureState,
            hash: String,
        ) -> anyhow::Result<()> {
            if self.fail_apply {
                anyhow::bail!("apply failed");
            }
            self.hashes.insert(id.to_string(), hash);
            Ok(())
        }

        fn mark_archived(&mut self, id: &str) -> anyhow::Result<()> {
            if self.fail_archive {
                anyhow::bail!("archive failed");
            }
            self.hashes.remove(id);
            self.archived.push(id.to_string());
            Ok(())
        }

        fn auto_import(
            &mut self,
            issue: &PlaneWebhookIssue,
            _state: FeatureState,
        ) -> anyhow::Result<()> {
            if self.fail_import {
                anyhow::bail!("import failed");
            }
            self.imported.push(issue.id.clone());
            Ok(())
        }
    }

    fn issue(id: &str, name: &str, state: Option<&str>, labels: &[&str]) -> PlaneWebhookIssue {
        PlaneWebhookIssue {
            id: id.to_string(),
            name: name.to_string(),
            description_html: None,
            state: state.map(str::to_string),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            project: None,
        }
    }

    fn mapper() -> PlaneStateMapper {
        PlaneStateMapper::new()
    }

    #[test]
    fn create_auto_imports_new_issue_when_enabled() {
        let mut store = MockStore::default();
        let outcome = handle_create(&mapper(), true, issue("1", "New", Some("backlog"), &[]), &mut store)
            .unwrap();
        assert_eq!(
            outcome,
            super::InboundOutcome::AutoImported {
                issue_id: "1".into(),
                title: "New".into()
            }
        );
        assert_eq!(store.imported, vec!["1".to_string()]);
    }

    #[test]
    fn create_skips_when_auto_import_disabled() {
        let mut store = MockStore::default();
        let outcome = handle_create(&mapper(), false, issue("2", "New", None, &[]), &mut store)
            .unwrap();
        assert_eq!(outcome, super::InboundOutcome::NotTracked { issue_id: "2".into() });
        assert!(store.imported.is_empty());
    }

    #[test]
    fn create_propagates_import_error() {
        let mut store = MockStore {
            fail_import: true,
            ..Default::default()
        };
        assert!(handle_create(&mapper(), true, issue("3", "New", None, &[]), &mut store).is_err());
    }

    #[test]
    fn create_on_tracked_issue_delegates_to_update() {
        let mut store = MockStore::with_tracked("4", "stale-hash");
        let outcome = handle_create(&mapper(), true, issue("4", "Changed", Some("started"), &[]), &mut store)
            .unwrap();
        assert!(matches!(outcome, super::InboundOutcome::Updated { .. }));
    }

    #[test]
    fn create_on_tracked_unchanged_content_is_unchanged() {
        let name = "Same";
        let state = Some("backlog");
        let labels: &[&str] = &[];
        let issue_val = issue("5", name, state, labels);
        let m = mapper();
        let hash = compute_content_hash(
            name,
            state.unwrap(),
            &m.map_plane_state("backlog", "backlog").to_string(),
            &[],
        );
        let mut store = MockStore::with_tracked("5", &hash);
        let outcome = handle_create(&m, true, issue_val, &mut store).unwrap();
        assert_eq!(outcome, super::InboundOutcome::Unchanged { issue_id: "5".into() });
    }

    #[test]
    fn update_untracked_issue_is_not_tracked() {
        let mut store = MockStore::default();
        let outcome = handle_update(&mapper(), issue("6", "X", None, &[]), &mut store).unwrap();
        assert_eq!(outcome, super::InboundOutcome::NotTracked { issue_id: "6".into() });
    }

    #[test]
    fn update_changed_content_returns_updated_with_state() {
        let mut store = MockStore::with_tracked("7", "old");
        let outcome = handle_update(&mapper(), issue("7", "New", Some("started"), &[]), &mut store)
            .unwrap();
        match outcome {
            super::InboundOutcome::Updated { issue_id, new_state, new_hash } => {
                assert_eq!(issue_id, "7");
                assert_eq!(new_state, FeatureState::Implementing);
                assert_ne!(new_hash, "old");
                assert_eq!(store.hashes.get("7"), Some(&new_hash));
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn update_unchanged_content_is_unchanged() {
        let m = mapper();
        let hash = compute_content_hash("Stable", "backlog", &FeatureState::Created.to_string(), &[]);
        let mut store = MockStore::with_tracked("8", &hash);
        let outcome = handle_update(&m, issue("8", "Stable", Some("backlog"), &[]), &mut store)
            .unwrap();
        assert_eq!(outcome, super::InboundOutcome::Unchanged { issue_id: "8".into() });
    }

    #[test]
    fn update_label_change_triggers_update() {
        let m = mapper();
        let hash = compute_content_hash("T", "started", &FeatureState::Implementing.to_string(), &[]);
        let mut store = MockStore::with_tracked("9", &hash);
        let outcome = handle_update(&m, issue("9", "T", Some("started"), &["bug"]), &mut store)
            .unwrap();
        assert!(matches!(outcome, super::InboundOutcome::Updated { .. }));
    }

    #[test]
    fn update_missing_state_defaults_to_backlog() {
        let mut store = MockStore::with_tracked("10", "old");
        let outcome = handle_update(&mapper(), issue("10", "X", None, &[]), &mut store).unwrap();
        match outcome {
            super::InboundOutcome::Updated { new_state, .. } => {
                assert_eq!(new_state, FeatureState::Created)
            }
            other => panic!("expected Updated, got {other:?}"),
        }
    }

    #[test]
    fn update_propagates_store_error() {
        let mut store = MockStore {
            fail_apply: true,
            ..Default::default()
        };
        store.hashes.insert("11".into(), "old".into());
        assert!(handle_update(&mapper(), issue("11", "X", None, &[]), &mut store).is_err());
    }

    #[test]
    fn delete_untracked_is_not_tracked() {
        let mut store = MockStore::default();
        let outcome = handle_delete("12".into(), &mut store).unwrap();
        assert_eq!(outcome, super::InboundOutcome::NotTracked { issue_id: "12".into() });
        assert!(store.archived.is_empty());
    }

    #[test]
    fn delete_tracked_archives() {
        let mut store = MockStore::with_tracked("13", "hash");
        let outcome = handle_delete("13".into(), &mut store).unwrap();
        assert_eq!(outcome, super::InboundOutcome::Archived { issue_id: "13".into() });
        assert_eq!(store.archived, vec!["13".to_string()]);
    }

    #[test]
    fn delete_is_idempotent_after_archival() {
        let mut store = MockStore::with_tracked("14", "hash");
        assert!(matches!(
            handle_delete("14".into(), &mut store).unwrap(),
            super::InboundOutcome::Archived { .. }
        ));
        assert!(matches!(
            handle_delete("14".into(), &mut store).unwrap(),
            super::InboundOutcome::NotTracked { .. }
        ));
    }

    #[test]
    fn delete_propagates_store_error_and_leaves_entity_tracked() {
        let mut store = MockStore {
            fail_archive: true,
            ..Default::default()
        };
        store.hashes.insert("17".into(), "hash".into());

        let err = handle_delete("17".into(), &mut store).unwrap_err();

        assert!(err.to_string().contains("archive failed"), "{err}");
        assert!(store.archived.is_empty());
        assert_eq!(
            store.hashes.get("17").map(String::as_str),
            Some("hash"),
            "a failed archive must not report the entity as gone"
        );
    }

    #[test]
    fn mapped_state_unknown_group_defaults_to_created() {
        let state = mapped_state(&mapper(), &issue("15", "X", Some("mystery"), &[]));
        assert_eq!(state, FeatureState::Created);
    }

    #[test]
    fn mapped_state_missing_group_defaults_to_backlog() {
        let state = mapped_state(&mapper(), &issue("16", "X", None, &[]));
        assert_eq!(state, FeatureState::Created);
    }
}
