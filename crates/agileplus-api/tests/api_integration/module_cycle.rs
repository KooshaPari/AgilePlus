use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::module::{Module, ModuleFeatureTag};
use agileplus_domain::domain::state_machine::FeatureState;
use axum::http::StatusCode;
use chrono::NaiveDate;

use crate::support::{
    MockStorage, TEST_API_KEY, setup_test_server, setup_test_server_with_storage,
};

fn seeded_mutation_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();
    let mut modules = storage.modules.lock().expect("modules lock poisoned");
    modules.push(Module {
        id: 1,
        slug: "platform".to_string(),
        friendly_name: "Platform".to_string(),
        description: Some("Original module".to_string()),
        parent_module_id: None,
        created_at: now,
        updated_at: now,
    });
    drop(modules);

    let mut cycles = storage.cycles.lock().expect("cycles lock poisoned");
    cycles.push(Cycle {
        id: 1,
        name: "Q1".to_string(),
        description: Some("Original cycle".to_string()),
        state: CycleState::Draft,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        end_date: NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid date"),
        module_scope_id: None,
        created_at: now,
        updated_at: now,
    });
    drop(cycles);
    storage
}

async fn setup_mutation_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_mutation_storage()).await
}

fn seeded_shipping_gate_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(Feature {
            id: 1,
            slug: "blocking-feature".to_string(),
            friendly_name: "Blocking Feature".to_string(),
            state: FeatureState::Implementing,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .push(Cycle {
            id: 1,
            name: "Q2".to_string(),
            description: Some("Shipping gate".to_string()),
            state: CycleState::Review,
            start_date: NaiveDate::from_ymd_opt(2026, 4, 1).expect("valid date"),
            end_date: NaiveDate::from_ymd_opt(2026, 6, 30).expect("valid date"),
            module_scope_id: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycle_features
        .lock()
        .expect("cycle_features lock poisoned")
        .push(CycleFeature::new(1, 1));

    storage
}

async fn setup_shipping_gate_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_shipping_gate_storage()).await
}

#[tokio::test]
async fn module_routes_require_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/api/modules").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_modules_with_valid_key_returns_empty_array() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/modules")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("modules response should be an array");
    assert!(arr.is_empty());
}

#[tokio::test]
async fn get_module_not_found_returns_404() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cycle_routes_require_auth() {
    let server = setup_test_server().await;
    let resp = server.get("/api/cycles").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_cycles_with_valid_key_returns_empty_array() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body.as_array().expect("cycles response should be an array");
    assert!(arr.is_empty());
}

#[tokio::test]
async fn get_cycle_not_found_returns_404() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_cycles_invalid_state_returns_400() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/cycles?state=NotAState")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_module_persists_mutations() {
    let server = setup_mutation_server().await;
    let resp = server
        .patch("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "friendly_name": "Platform Core",
            "description": "Renamed module"
        }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["friendly_name"], "Platform Core");
    assert_eq!(body["slug"], "platform-core");
    assert_eq!(body["description"], "Renamed module");
}

#[tokio::test]
async fn delete_module_removes_record() {
    let server = setup_mutation_server().await;
    let resp = server
        .delete("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NO_CONTENT);

    let follow_up = server
        .get("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    follow_up.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn transition_cycle_persists_mutation() {
    let server = setup_mutation_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "state": "Active"
        }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "Active");

    let follow_up = server
        .get("/api/cycles/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    follow_up.assert_status_ok();
    let cycle_body: serde_json::Value = follow_up.json();
    assert_eq!(cycle_body["cycle"]["state"], "Active");
}

#[tokio::test]
async fn transition_cycle_to_shipped_blocks_unvalidated_features() {
    let server = setup_shipping_gate_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "state": "Shipped"
        }))
        .await;
    resp.assert_status(StatusCode::CONFLICT);
}

// ── Cycle transitions: error paths and the shipping gate's success side ──────

#[tokio::test]
async fn transition_cycle_unknown_id_is_404() {
    let server = setup_mutation_server().await;
    let resp = server
        .post("/api/cycles/404/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "state": "Active" }))
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn transition_cycle_unknown_state_name_is_400() {
    let server = setup_mutation_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "state": "Nonsense" }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn transition_cycle_illegal_edge_is_400() {
    let server = setup_mutation_server().await;
    // Cycle 1 is Draft; Draft -> Archived is not a permitted edge, and the
    // target is not Shipped, so the failure maps to 400 (not the 409 used by
    // the shipping gate).
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "state": "Archived" }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn transition_cycle_to_same_state_is_400() {
    let server = setup_mutation_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "state": "Draft" }))
        .await;
    resp.assert_status(StatusCode::BAD_REQUEST);
}

/// The shipping gate is only a gate: once every member feature is Validated the
/// same `Review -> Shipped` transition succeeds and is persisted.
#[tokio::test]
async fn transition_cycle_to_shipped_succeeds_when_all_features_validated() {
    let server = setup_shippable_server().await;
    let resp = server
        .post("/api/cycles/1/transition")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "state": "Shipped" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["state"], "Shipped");
    assert_eq!(body["id"], 1);

    let reread = server
        .get("/api/cycles/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    assert_eq!(
        reread.json::<serde_json::Value>()["cycle"]["state"],
        "Shipped"
    );
}

// ── Cycle HTML pages: real data, not just the empty board ────────────────────

#[tokio::test]
async fn cycle_kanban_page_groups_cycles_into_state_columns() {
    let server = setup_cycle_board_server().await;
    let resp = server.get("/cycles").await;
    resp.assert_status_ok();
    let body = resp.text();

    // One cycle per state, so every column reports exactly one entry.
    for (column, name) in [
        ("Draft", "Cycle Draft"),
        ("Active", "Cycle Active"),
        ("Review", "Cycle Review"),
        ("Shipped", "Cycle Shipped"),
        ("Archived", "Cycle Archived"),
    ] {
        assert!(
            body.contains(&format!("{column} (1)")),
            "kanban should place one cycle in the {column} column, got: {body}"
        );
        assert!(
            body.contains(name),
            "kanban should link the {column} cycle by name, got: {body}"
        );
    }

    // The Draft cycle has one member feature: the card must report the count.
    assert!(
        body.contains("1 features"),
        "kanban cards should surface the member-feature count, got: {body}"
    );
}

#[tokio::test]
async fn cycle_detail_page_renders_feature_and_module_scope() {
    let server = setup_cycle_board_server().await;
    let resp = server.get("/cycles/1").await;
    resp.assert_status_ok();
    let body = resp.text();

    assert!(body.contains("Cycle Draft"), "got: {body}");
    assert!(
        body.contains("Draft"),
        "cycle state should render, got: {body}"
    );
    assert!(
        body.contains("Scope: Platform"),
        "a scoped cycle should resolve its module name, got: {body}"
    );
    assert!(
        body.contains("scoped-feature"),
        "burndown table should list member features, got: {body}"
    );
    assert!(
        body.contains("days remaining"),
        "detail page should render the days-remaining line, got: {body}"
    );
}

// ── Module tree: nesting, depth, and counts ──────────────────────────────────

#[tokio::test]
async fn module_tree_api_returns_nested_nodes_with_depth() {
    let server = setup_module_tree_server().await;
    let resp = server
        .get("/api/modules/1/tree")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let nodes: Vec<serde_json::Value> = resp.json();

    let ids: Vec<i64> = nodes
        .iter()
        .map(|n| n["module"]["id"].as_i64().expect("module id"))
        .collect();
    assert_eq!(
        ids,
        vec![1, 2, 3],
        "tree should flatten depth-first: {nodes:?}"
    );

    let depths: Vec<u64> = nodes
        .iter()
        .map(|n| n["depth"].as_u64().expect("depth"))
        .collect();
    assert_eq!(depths, vec![0, 1, 2], "depth should increment per level");

    // Module 2 owns feature 1; module 1 only carries a many-to-many tag for it.
    let owned: Vec<u64> = nodes
        .iter()
        .map(|n| n["owned_count"].as_u64().expect("owned_count"))
        .collect();
    let tagged: Vec<u64> = nodes
        .iter()
        .map(|n| n["tagged_count"].as_u64().expect("tagged_count"))
        .collect();
    assert_eq!(
        owned,
        vec![0, 1, 0],
        "owned features belong to the owning module"
    );
    assert_eq!(
        tagged,
        vec![1, 0, 0],
        "tags are counted separately from ownership"
    );
}

#[tokio::test]
async fn module_page_renders_nested_tree_with_counts() {
    let server = setup_module_tree_server().await;
    let resp = server.get("/modules").await;
    resp.assert_status_ok();
    let body = resp.text();

    for name in ["Platform", "Platform Core", "Platform Deep"] {
        assert!(
            body.contains(name),
            "sidebar should list {name}, got: {body}"
        );
    }
    // Indentation is derived from node depth.
    assert!(
        body.contains("padding-left: 16px"),
        "depth 1 indent missing: {body}"
    );
    assert!(
        body.contains("padding-left: 32px"),
        "depth 2 indent missing: {body}"
    );
    assert!(
        body.contains("(1 owned, 0 tagged)"),
        "owning module should report its owned feature, got: {body}"
    );
}

#[tokio::test]
async fn patch_module_unknown_is_404() {
    let server = setup_mutation_server().await;
    let resp = server
        .patch("/api/modules/404")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "friendly_name": "Ghost" }))
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_module_unknown_is_404() {
    let server = setup_mutation_server().await;
    let resp = server
        .delete("/api/modules/404")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_module_without_description_clears_it() {
    let server = setup_mutation_server().await;
    let resp = server
        .patch("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({ "friendly_name": "Platform" }))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["friendly_name"], "Platform");
    assert!(
        body.get("description").is_none(),
        "an omitted description clears the field (and is skipped when None), got: {body}"
    );

    let reread = server
        .get("/api/modules/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    assert!(
        reread.json::<serde_json::Value>()["module"]
            .get("description")
            .is_none()
    );
}

// ── Seeds for the tests above ────────────────────────────────────────────────

/// Review-state cycle whose single member feature is Validated — the shipping
/// gate's success case.
fn seeded_shippable_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(Feature {
            id: 1,
            slug: "ready-feature".to_string(),
            friendly_name: "Ready Feature".to_string(),
            state: FeatureState::Validated,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: None,
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycles
        .lock()
        .expect("cycles lock poisoned")
        .push(Cycle {
            id: 1,
            name: "Ready Cycle".to_string(),
            description: None,
            state: CycleState::Review,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            end_date: NaiveDate::from_ymd_opt(2026, 2, 1).expect("valid date"),
            module_scope_id: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .cycle_features
        .lock()
        .expect("cycle_features lock poisoned")
        .push(CycleFeature::new(1, 1));

    storage
}

async fn setup_shippable_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_shippable_storage()).await
}

/// One cycle per kanban state; the Draft cycle scopes a module and carries one
/// member feature so both the board and the detail page render real data.
fn seeded_cycle_board_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    storage
        .modules
        .lock()
        .expect("modules lock poisoned")
        .push(Module {
            id: 1,
            slug: "platform".to_string(),
            friendly_name: "Platform".to_string(),
            description: None,
            parent_module_id: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(Feature {
            id: 1,
            slug: "scoped-feature".to_string(),
            friendly_name: "Scoped Feature".to_string(),
            state: FeatureState::Planned,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: Some(1),
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });

    let mut cycles = storage.cycles.lock().expect("cycles lock poisoned");
    for (id, name, state) in [
        (1, "Cycle Draft", CycleState::Draft),
        (2, "Cycle Active", CycleState::Active),
        (3, "Cycle Review", CycleState::Review),
        (4, "Cycle Shipped", CycleState::Shipped),
        (5, "Cycle Archived", CycleState::Archived),
    ] {
        cycles.push(Cycle {
            id,
            name: name.to_string(),
            description: None,
            state,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            end_date: NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid date"),
            module_scope_id: (id == 1).then_some(1),
            created_at: now,
            updated_at: now,
        });
    }
    drop(cycles);

    storage
        .cycle_features
        .lock()
        .expect("cycle_features lock poisoned")
        .push(CycleFeature::new(1, 1));

    storage
}

async fn setup_cycle_board_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_cycle_board_storage()).await
}

/// Three-level module tree: module 2 owns the feature, module 1 carries a tag.
fn seeded_module_tree_storage() -> MockStorage {
    let storage = MockStorage::default();
    let now = chrono::Utc::now();

    let mut modules = storage.modules.lock().expect("modules lock poisoned");
    for (id, name, parent) in [
        (1, "Platform", None),
        (2, "Platform Core", Some(1)),
        (3, "Platform Deep", Some(2)),
    ] {
        modules.push(Module {
            id,
            slug: Module::slug_from_name(name),
            friendly_name: name.to_string(),
            description: None,
            parent_module_id: parent,
            created_at: now,
            updated_at: now,
        });
    }
    drop(modules);

    storage
        .features
        .lock()
        .expect("features lock poisoned")
        .push(Feature {
            id: 1,
            slug: "owned-feature".to_string(),
            friendly_name: "Owned Feature".to_string(),
            state: FeatureState::Planned,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            plane_issue_id: None,
            plane_state_id: None,
            labels: vec![],
            module_id: Some(2),
            project_id: None,
            created_at_commit: None,
            last_modified_commit: None,
            created_at: now,
            updated_at: now,
        });

    storage
        .module_tags
        .lock()
        .expect("module_tags lock poisoned")
        .push(ModuleFeatureTag::new(1, 1));

    storage
}

async fn setup_module_tree_server() -> axum_test::TestServer {
    setup_test_server_with_storage(seeded_module_tree_storage()).await
}
