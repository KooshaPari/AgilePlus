// SPDX-License-Identifier: MIT OR Apache-2.0
//! Contract tests for generated `agileplus.v1` types (see `mod.rs`).

use crate::agileplus::v1::*;
use prost::Message;

// ── work_items.proto ─────────────────────────────────────────────────────────

roundtrip!(
    rt_list_projects_request,
    ListProjectsRequest,
    ListProjectsRequest {}
);

roundtrip!(
    rt_project_proto,
    ProjectProto,
    ProjectProto {
        id: 1,
        slug: "agileplus".to_string(),
        name: "AgilePlus".to_string(),
        description: "Spec-driven development".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_list_projects_response,
    ListProjectsResponse,
    ListProjectsResponse {
        projects: vec![ProjectProto {
            id: 1,
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_list_epics_request,
    ListEpicsRequest,
    ListEpicsRequest { project_id: 1 }
);

roundtrip!(
    rt_epic_proto,
    EpicProto,
    EpicProto {
        id: 2,
        project_id: 1,
        title: "Epic".to_string(),
        description: "desc".to_string(),
        status: "Open".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_list_epics_response,
    ListEpicsResponse,
    ListEpicsResponse {
        epics: vec![EpicProto {
            id: 2,
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_list_stories_request,
    ListStoriesRequest,
    ListStoriesRequest { epic_id: 2 }
);

roundtrip!(
    rt_story_proto,
    StoryProto,
    StoryProto {
        id: 3,
        epic_id: 2,
        project_id: 1,
        title: "Story".to_string(),
        description: "as a user".to_string(),
        status: "InProgress".to_string(),
        points: 5,
        requirement_id: "FR-1".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-02T00:00:00Z".to_string(),
    }
);

roundtrip!(
    rt_list_stories_response,
    ListStoriesResponse,
    ListStoriesResponse {
        stories: vec![StoryProto {
            id: 3,
            ..Default::default()
        }],
    }
);

roundtrip!(
    rt_sync_repository_request,
    SyncRepositoryRequest,
    SyncRepositoryRequest {
        repo: "acme/backend".to_string(),
        project_id: 1,
        epic_id: 2,
    }
);

roundtrip!(
    rt_sync_repository_response,
    SyncRepositoryResponse,
    SyncRepositoryResponse {
        stories_synced: 10,
        stories_skipped: 2,
        errors: vec!["issue #3 unmapped".to_string()],
    }
);

