// SPDX-License-Identifier: MIT OR Apache-2.0
//! Seed runner: drives Epic + Story upserts for the 4 FR/NFR catalogs.
//!
//! Each initiative gets one Epic (upserted by `requirement_id = "EPIC-<slug>"`).
//! Each FR/NFR entry under that initiative gets one Story (upserted by
//! `requirement_id = <fr-id>`).
//!
//! Status mapping:
//!   CatalogStatus::Shipped → StoryStatus::Done
//!   CatalogStatus::Planned → StoryStatus::Todo

use rusqlite::Connection;

use agileplus_domain::{
    domain::{
        epic::Epic,
        story::{Story, StoryStatus},
    },
    error::DomainError,
};

use crate::{
    repository::{epics, projects, stories},
    seed::catalog::{CatalogEntry, CatalogStatus, parse_catalog},
};

/// Describes one initiative to seed.
pub struct Initiative<'a> {
    /// Short slug used for project/epic lookup, e.g. "agileplus".
    pub slug: &'a str,
    /// Human-readable epic title, e.g. "AgilePlus".
    pub title: &'a str,
    /// The raw markdown content of the FR/NFR catalog for this initiative.
    pub catalog_markdown: &'a str,
}

/// Summary of what was seeded.
#[derive(Debug, Default)]
pub struct SeedReport {
    pub epics_upserted: usize,
    pub stories_upserted: usize,
    pub initiatives: Vec<InitiativeReport>,
}

/// Per-initiative seed result.
#[derive(Debug)]
pub struct InitiativeReport {
    pub initiative_slug: String,
    pub epic_id: i64,
    pub epic_requirement_id: String,
    pub stories: Vec<StoryReport>,
}

/// Per-story seed result.
#[derive(Debug)]
pub struct StoryReport {
    pub requirement_id: String,
    pub story_id: i64,
    pub status: StoryStatus,
}

/// Seed all four FR/NFR catalogs into the database.
///
/// For each initiative:
/// 1. Ensure a `Project` row exists (created if absent; upsert by slug).
/// 2. Upsert one `Epic` with `requirement_id = "EPIC-<slug>"`.
/// 3. For each FR/NFR entry, upsert one `Story` cross-referencing the requirement ID.
///
/// The connection must have migration 021 applied (requirement_id columns exist).
pub fn seed_requirements(
    conn: &Connection,
    initiatives: &[Initiative<'_>],
) -> Result<SeedReport, DomainError> {
    let mut report = SeedReport::default();

    for initiative in initiatives {
        let project_id = ensure_project(conn, initiative.slug, initiative.title)?;
        let epic_req_id = format!("EPIC-{}", initiative.slug);

        // Build epic
        let mut epic = Epic::new(project_id, initiative.title)?;
        epic.requirement_id = Some(epic_req_id.clone());
        epic.description = Some(format!(
            "Initiative epic for {} FR/NFR catalog",
            initiative.title
        ));

        let epic_id = epics::upsert_epic_by_requirement_id(conn, &epic)?;
        report.epics_upserted += 1;

        let entries = parse_catalog(initiative.catalog_markdown);
        let mut story_reports = Vec::new();

        for entry in &entries {
            let story_status = map_status(entry.status);
            let story = build_story(epic_id, project_id, entry, story_status)?;
            let story_id = stories::upsert_story_by_requirement_id(conn, &story)?;
            report.stories_upserted += 1;
            story_reports.push(StoryReport {
                requirement_id: entry.id.clone(),
                story_id,
                status: story_status,
            });
        }

        report.initiatives.push(InitiativeReport {
            initiative_slug: initiative.slug.to_string(),
            epic_id,
            epic_requirement_id: epic_req_id,
            stories: story_reports,
        });
    }

    Ok(report)
}

fn map_status(catalog_status: CatalogStatus) -> StoryStatus {
    match catalog_status {
        CatalogStatus::Shipped => StoryStatus::Done,
        CatalogStatus::Planned => StoryStatus::Todo,
    }
}

fn build_story(
    epic_id: i64,
    project_id: i64,
    entry: &CatalogEntry,
    status: StoryStatus,
) -> Result<Story, DomainError> {
    let title = if entry.title.is_empty() {
        entry.id.clone()
    } else {
        format!("[{}] {}", entry.id, entry.title)
    };

    let mut story = Story::new(epic_id, project_id, &title, None)?;
    story.requirement_id = Some(entry.id.clone());
    story.description = Some(format!("Tracera requirement cross-reference: {}", entry.id));
    // Override the default Todo status that Story::new() sets.
    story.status = status;
    Ok(story)
}

/// Ensure a project row exists for the initiative and return its ID.
/// Creates one if absent; does NOT update if already present (idempotent).
fn ensure_project(conn: &Connection, slug: &str, title: &str) -> Result<i64, DomainError> {
    // Try to find existing
    if let Some(p) = projects::get_project_by_slug(conn, slug)? {
        return Ok(p.id);
    }
    use agileplus_domain::domain::project::Project;
    let project = Project::new(title, slug)
        .map_err(|e| DomainError::Validation(format!("project creation failed: {e}")))?;
    projects::create_project(conn, &project)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SqliteStorageAdapter;
    use agileplus_domain::domain::story::StoryStatus;

    fn in_memory_conn() -> rusqlite::Connection {
        let adapter = SqliteStorageAdapter::in_memory().expect("in-memory adapter");
        // Extract the underlying connection for synchronous seed calls.
        // We use conn_for_bench which returns a MutexGuard; we need to re-open
        // a fresh in-memory db for the sync tests.
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let runner = crate::migrations::MigrationRunner::new(&conn);
        runner.run_all().unwrap();
        // drop adapter to avoid unused warning
        drop(adapter);
        conn
    }

    const MINI_CATALOG: &str = r#"
# Mini Test Catalog

## Functional Requirements

### FR-MINI-001 — Shipped feature

| Field | Value |
|---|---|
| **Status** | SHIPPED |

---

### FR-MINI-002 — Planned feature

| Field | Value |
|---|---|
| **Status** | PLANNED |

---

### NFR-MINI-001 — No status field (defaults Shipped)

Some description without a status row.
"#;

    #[test]
    fn seed_creates_epic_and_stories() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini",
            title: "Mini Initiative",
            catalog_markdown: MINI_CATALOG,
        }];
        let report = seed_requirements(&conn, &initiatives).unwrap();
        assert_eq!(report.epics_upserted, 1);
        // FR-MINI-001, FR-MINI-002, NFR-MINI-001
        assert_eq!(report.stories_upserted, 3);
    }

    #[test]
    fn seed_status_mapping_shipped_to_done() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini2",
            title: "Mini2",
            catalog_markdown: MINI_CATALOG,
        }];
        let report = seed_requirements(&conn, &initiatives).unwrap();
        let initiative = &report.initiatives[0];
        let shipped = initiative
            .stories
            .iter()
            .find(|s| s.requirement_id == "FR-MINI-001")
            .unwrap();
        assert_eq!(shipped.status, StoryStatus::Done);
    }

    #[test]
    fn seed_status_mapping_planned_to_todo() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini3",
            title: "Mini3",
            catalog_markdown: MINI_CATALOG,
        }];
        let report = seed_requirements(&conn, &initiatives).unwrap();
        let initiative = &report.initiatives[0];
        let planned = initiative
            .stories
            .iter()
            .find(|s| s.requirement_id == "FR-MINI-002")
            .unwrap();
        assert_eq!(planned.status, StoryStatus::Todo);
    }

    #[test]
    fn seed_is_idempotent() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini4",
            title: "Mini4",
            catalog_markdown: MINI_CATALOG,
        }];
        let r1 = seed_requirements(&conn, &initiatives).unwrap();
        let r2 = seed_requirements(&conn, &initiatives).unwrap();
        // Same counts — no duplicates
        assert_eq!(r1.epics_upserted, r2.epics_upserted);
        assert_eq!(r1.stories_upserted, r2.stories_upserted);
        // Verify epic by requirement_id still unique
        let epic = epics::get_epic_by_requirement_id(&conn, "EPIC-mini4")
            .unwrap()
            .unwrap();
        assert_eq!(epic.title, "Mini4");
    }

    #[test]
    fn story_requirement_id_set_correctly() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini5",
            title: "Mini5",
            catalog_markdown: MINI_CATALOG,
        }];
        seed_requirements(&conn, &initiatives).unwrap();
        let story = stories::get_story_by_requirement_id(&conn, "FR-MINI-001")
            .unwrap()
            .unwrap();
        assert_eq!(story.requirement_id.as_deref(), Some("FR-MINI-001"));
    }

    #[test]
    fn epic_requirement_id_set_correctly() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini6",
            title: "Mini6",
            catalog_markdown: MINI_CATALOG,
        }];
        seed_requirements(&conn, &initiatives).unwrap();
        let epic = epics::get_epic_by_requirement_id(&conn, "EPIC-mini6")
            .unwrap()
            .unwrap();
        assert_eq!(epic.requirement_id.as_deref(), Some("EPIC-mini6"));
    }

    #[test]
    fn invariants_preserved_nonempty_titles() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "mini7",
            title: "Mini7",
            catalog_markdown: MINI_CATALOG,
        }];
        seed_requirements(&conn, &initiatives).unwrap();
        // All stories must have non-empty titles
        let all_stories = stories::list_stories_by_project(
            &conn,
            // project id will be 1 (first created)
            projects::get_project_by_slug(&conn, "mini7")
                .unwrap()
                .unwrap()
                .id,
        )
        .unwrap();
        for s in &all_stories {
            assert!(!s.title.is_empty(), "story has empty title: {s:?}");
        }
    }

    const TITLELESS_CATALOG: &str = r#"
# Titleless Catalog

## Functional Requirements

### FR-TITLE-900
"#;

    #[test]
    fn heading_without_title_uses_requirement_id_as_story_title() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "titleless",
            title: "Titleless",
            catalog_markdown: TITLELESS_CATALOG,
        }];
        seed_requirements(&conn, &initiatives).unwrap();

        let story = stories::get_story_by_requirement_id(&conn, "FR-TITLE-900")
            .unwrap()
            .expect("story seeded from titleless heading");
        assert_eq!(story.title, "FR-TITLE-900");
        assert_eq!(story.requirement_id.as_deref(), Some("FR-TITLE-900"));
    }

    #[test]
    fn seed_rejects_invalid_project_slug() {
        let conn = in_memory_conn();
        let initiatives = vec![Initiative {
            slug: "Not A Slug",
            title: "Invalid",
            catalog_markdown: MINI_CATALOG,
        }];

        let err = seed_requirements(&conn, &initiatives).unwrap_err();
        assert!(
            matches!(err, DomainError::Validation(_)),
            "expected Validation error, got {err:?}"
        );
        assert!(
            projects::get_project_by_slug(&conn, "Not A Slug")
                .unwrap()
                .is_none(),
            "a rejected initiative must not create a project row"
        );
    }

    #[test]
    fn existing_project_row_is_reused_and_not_renamed() {
        use agileplus_domain::domain::project::Project;

        let conn = in_memory_conn();
        let existing =
            projects::create_project(&conn, &Project::new("Original Name", "mini8").unwrap())
                .unwrap();
        let initiatives = vec![Initiative {
            slug: "mini8",
            title: "Renamed Initiative",
            catalog_markdown: MINI_CATALOG,
        }];

        let report = seed_requirements(&conn, &initiatives).unwrap();

        let project = projects::get_project_by_slug(&conn, "mini8")
            .unwrap()
            .expect("project exists");
        assert_eq!(project.id, existing);
        assert_eq!(project.name, "Original Name");
        assert_eq!(projects::list_all_projects(&conn).unwrap().len(), 1);
        let epic = epics::get_epic_by_requirement_id(&conn, "EPIC-mini8")
            .unwrap()
            .expect("epic seeded");
        assert_eq!(epic.project_id, existing);
        assert_eq!(report.initiatives[0].epic_id, epic.id);
    }

    const AGILEPLUS_CATALOG: &str =
        include_str!("../../../../docs/requirements/agileplus-frnfr.md");
    const AUTHVAULT_CATALOG: &str =
        include_str!("../../../../docs/requirements/authvault-frnfr.md");
    const TRACERA_CATALOG: &str = include_str!("../../../../docs/requirements/tracera-frnfr.md");
    const VOXEL_CATALOG: &str =
        include_str!("../../../../docs/requirements/phenotype-voxel-frnfr.md");

    /// End-to-end seed of the shipped catalogs: every initiative must yield an
    /// epic plus at least one requirement-keyed story, and a second run must not
    /// duplicate anything.
    #[test]
    fn seeds_shipped_catalogs_without_duplicates() {
        use std::collections::HashSet;

        let conn = in_memory_conn();
        let initiatives = vec![
            Initiative {
                slug: "agileplus",
                title: "AgilePlus",
                catalog_markdown: AGILEPLUS_CATALOG,
            },
            Initiative {
                slug: "authvault",
                title: "Authvault",
                catalog_markdown: AUTHVAULT_CATALOG,
            },
            Initiative {
                slug: "tracera",
                title: "Tracera",
                catalog_markdown: TRACERA_CATALOG,
            },
            Initiative {
                slug: "phenotype-voxel",
                title: "phenotype-voxel",
                catalog_markdown: VOXEL_CATALOG,
            },
        ];

        let first = seed_requirements(&conn, &initiatives).unwrap();
        assert_eq!(first.epics_upserted, initiatives.len());
        assert!(first.stories_upserted > 0, "catalogs must yield stories");
        assert_eq!(first.initiatives.len(), initiatives.len());

        let mut seen: HashSet<String> = HashSet::new();
        for initiative in &first.initiatives {
            assert!(
                !initiative.stories.is_empty(),
                "initiative {} produced no stories",
                initiative.initiative_slug
            );
            assert_eq!(
                initiative.epic_requirement_id,
                format!("EPIC-{}", initiative.initiative_slug)
            );
            for story in &initiative.stories {
                assert!(
                    story.requirement_id.starts_with("FR-")
                        || story.requirement_id.starts_with("NFR-"),
                    "unexpected requirement id {}",
                    story.requirement_id
                );
                assert!(
                    seen.insert(story.requirement_id.clone()),
                    "requirement id {} was seeded twice",
                    story.requirement_id
                );
            }
        }

        // Re-running is a no-op: the same counts and no extra rows.
        let second = seed_requirements(&conn, &initiatives).unwrap();
        assert_eq!(second.epics_upserted, first.epics_upserted);
        assert_eq!(second.stories_upserted, first.stories_upserted);
        assert_eq!(
            projects::list_all_projects(&conn).unwrap().len(),
            initiatives.len()
        );
        for initiative in &first.initiatives {
            let stories = stories::list_stories_by_project(&conn, {
                projects::get_project_by_slug(&conn, &initiative.initiative_slug)
                    .unwrap()
                    .unwrap()
                    .id
            })
            .unwrap();
            assert_eq!(
                stories.len(),
                initiative.stories.len(),
                "re-seeding {} duplicated stories",
                initiative.initiative_slug
            );
        }
    }
}
