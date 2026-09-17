//! Integration tests: project and user identity repositories.

use agileplus_domain::domain::{
    project::Project,
    user::{User, UserRole, UserStatus},
};
use agileplus_sqlite::{
    repository::{projects, users},
    SqliteStorageAdapter,
};

fn adapter() -> SqliteStorageAdapter {
    SqliteStorageAdapter::in_memory().unwrap()
}

fn user(name: &str, email: &str, role: UserRole) -> User {
    User::new(name, email, role).unwrap()
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

#[test]
fn project_create_returns_id_and_get_by_slug() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let p = Project::new("Alpha", "alpha").unwrap();
    let id = projects::create_project(&conn, &p).unwrap();
    assert!(id > 0);

    let got = projects::get_project_by_slug(&conn, "alpha").unwrap().unwrap();
    assert_eq!(got.id, id);
    assert_eq!(got.name, "Alpha");
    assert_eq!(got.slug, "alpha");
    assert!(got.description.is_none(), "empty description maps to None");
}

#[test]
fn project_get_by_id_roundtrips_description() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut p = Project::new("Beta", "beta").unwrap();
    p.description = Some("the beta project".into());
    let id = projects::create_project(&conn, &p).unwrap();
    let got = projects::get_project_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.description.as_deref(), Some("the beta project"));
}

#[test]
fn project_get_by_slug_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(projects::get_project_by_slug(&conn, "ghost").unwrap().is_none());
}

#[test]
fn project_get_by_id_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(projects::get_project_by_id(&conn, 4321).unwrap().is_none());
}

#[test]
fn project_list_all_orders_by_creation() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    projects::create_project(&conn, &Project::new("One", "one").unwrap()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    projects::create_project(&conn, &Project::new("Two", "two").unwrap()).unwrap();

    let all = projects::list_all_projects(&conn).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].slug, "one");
    assert_eq!(all[1].slug, "two");
}

#[test]
fn project_list_all_empty() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(projects::list_all_projects(&conn).unwrap().is_empty());
}

#[test]
fn project_slug_is_unique() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    projects::create_project(&conn, &Project::new("Dup", "dup").unwrap()).unwrap();
    assert!(projects::create_project(&conn, &Project::new("Dup 2", "dup").unwrap()).is_err());
}

#[test]
fn project_delete_then_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = projects::create_project(&conn, &Project::new("Gone", "gone").unwrap()).unwrap();
    projects::delete_project(&conn, id).unwrap();
    assert!(projects::get_project_by_id(&conn, id).unwrap().is_none());
    let err = projects::delete_project(&conn, id).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

#[test]
fn user_create_and_get_by_id() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let u = user("Ada", "ada@example.com", UserRole::Admin);
    let id = users::create_user(&conn, &u).unwrap();
    assert!(id > 0);

    let got = users::get_user_by_id(&conn, id).unwrap().unwrap();
    assert_eq!(got.display_name, "Ada");
    assert_eq!(got.email, "ada@example.com");
    assert_eq!(got.role, UserRole::Admin);
    assert_eq!(got.status, UserStatus::Active);
}

#[test]
fn user_get_by_email_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let mut u = user("Grace", "grace@example.com", UserRole::Member);
    u.github_login = Some("graceh".into());
    users::create_user(&conn, &u).unwrap();

    let got = users::get_user_by_email(&conn, "grace@example.com")
        .unwrap()
        .unwrap();
    assert_eq!(got.display_name, "Grace");
    assert_eq!(got.github_login.as_deref(), Some("graceh"));
}

#[test]
fn user_get_by_id_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(users::get_user_by_id(&conn, 777).unwrap().is_none());
}

#[test]
fn user_get_by_email_nonexistent() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(users::get_user_by_email(&conn, "nobody@example.com")
        .unwrap()
        .is_none());
}

#[test]
fn user_email_is_unique() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    users::create_user(&conn, &user("A", "dup@example.com", UserRole::Member)).unwrap();
    assert!(users::create_user(&conn, &user("B", "dup@example.com", UserRole::Member)).is_err());
}

#[test]
fn user_update_status_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = users::create_user(&conn, &user("A", "a@example.com", UserRole::Member)).unwrap();
    users::update_user_status(&conn, id, UserStatus::Suspended).unwrap();
    assert_eq!(
        users::get_user_by_id(&conn, id).unwrap().unwrap().status,
        UserStatus::Suspended
    );
}

#[test]
fn user_update_status_unknown_id_is_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = users::update_user_status(&conn, 999, UserStatus::Inactive).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

#[test]
fn user_update_role_persists() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = users::create_user(&conn, &user("A", "a@example.com", UserRole::Viewer)).unwrap();
    users::update_user_role(&conn, id, UserRole::Admin).unwrap();
    assert_eq!(
        users::get_user_by_id(&conn, id).unwrap().unwrap().role,
        UserRole::Admin
    );
}

#[test]
fn user_update_role_unknown_id_is_not_found() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let err = users::update_user_role(&conn, 1000, UserRole::Admin).unwrap_err();
    assert!(matches!(err, agileplus_domain::error::DomainError::NotFound(_)));
}

#[test]
fn user_all_roles_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for (i, role) in [UserRole::Admin, UserRole::Member, UserRole::Viewer]
        .into_iter()
        .enumerate()
    {
        let id = users::create_user(
            &conn,
            &user(&format!("u{i}"), &format!("u{i}@example.com"), role),
        )
        .unwrap();
        assert_eq!(users::get_user_by_id(&conn, id).unwrap().unwrap().role, role);
    }
}

#[test]
fn user_all_statuses_roundtrip() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    for (i, status) in [UserStatus::Active, UserStatus::Inactive, UserStatus::Suspended]
        .into_iter()
        .enumerate()
    {
        let id = users::create_user(
            &conn,
            &user(&format!("s{i}"), &format!("s{i}@example.com"), UserRole::Member),
        )
        .unwrap();
        users::update_user_status(&conn, id, status).unwrap();
        assert_eq!(
            users::get_user_by_id(&conn, id).unwrap().unwrap().status,
            status
        );
    }
}

#[test]
fn user_list_all_orders_by_created_at() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    users::create_user(&conn, &user("First", "first@example.com", UserRole::Member)).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    users::create_user(&conn, &user("Second", "second@example.com", UserRole::Member)).unwrap();

    let all = users::list_all_users(&conn).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].display_name, "First");
    assert_eq!(all[1].display_name, "Second");
}

#[test]
fn user_list_all_empty() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    assert!(users::list_all_users(&conn).unwrap().is_empty());
}

#[test]
fn user_delete_then_missing() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = users::create_user(&conn, &user("Temp", "temp@example.com", UserRole::Member)).unwrap();
    users::delete_user(&conn, id).unwrap();
    assert!(users::get_user_by_id(&conn, id).unwrap().is_none());
    assert!(users::delete_user(&conn, id).is_err());
}

#[test]
fn user_avatar_url_none_roundtrips() {
    let a = adapter();
    let conn = a.conn_for_bench().unwrap();
    let id = users::create_user(&conn, &user("NoA", "noa@example.com", UserRole::Member)).unwrap();
    let got = users::get_user_by_id(&conn, id).unwrap().unwrap();
    assert!(got.avatar_url.is_none());
    assert!(got.github_login.is_none());
}
