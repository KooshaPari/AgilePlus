// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tray menu builder — constructs dynamic menus from SQLite state.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Manager,
};

use crate::AppState;

/// Build the full tray menu from current DB state.
pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let conn_guard = state.db_connection()?;

    let menu = Menu::new(app)?;
    menu.append(&build_summary_submenu(app, &conn_guard)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&build_features_submenu(app, &conn_guard)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&build_quick_actions_submenu(app)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "show",
        "Show Window",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;

    Ok(menu)
}

/// Rebuild the tray menu and hot-swap it on the tray icon.
pub fn refresh_menu(app: &AppHandle) {
    if let Ok(menu) = build_tray_menu(app) {
        if let Some(tray) = app.tray_by_id("main-tray") {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Summary submenu showing feature and work-package counts.
fn build_summary_submenu(
    app: &AppHandle,
    conn: &std::sync::MutexGuard<'_, Option<rusqlite::Connection>>,
) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let submenu = Submenu::with_id(app, "summary", "AgilePlus", true)?;

    let (total_features, total_wps) = match conn.as_ref() {
        Some(c) => {
            let feat_count: i64 = c
                .query_row("SELECT COUNT(*) FROM features", [], |r| r.get(0))
                .unwrap_or(0);
            let wp_count: i64 = c
                .query_row("SELECT COUNT(*) FROM work_packages", [], |r| r.get(0))
                .unwrap_or(0);
            (feat_count, wp_count)
        }
        None => (0, 0),
    };

    submenu.append(&MenuItem::with_id(
        app,
        "summary-text",
        format!("Features: {total_features} | Work Packages: {total_wps}"),
        false,
        None::<&str>,
    )?)?;

    Ok(submenu)
}

/// Features submenu — one item per feature with state badge and advance action.
fn build_features_submenu(
    app: &AppHandle,
    conn: &std::sync::MutexGuard<'_, Option<rusqlite::Connection>>,
) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let submenu = Submenu::with_id(app, "features", "Features", true)?;

    let features = list_features_from_conn(conn)?;

    if features.is_empty() {
        submenu.append(&MenuItem::with_id(
            app,
            "features-empty",
            "(no features yet)",
            false,
            None::<&str>,
        )?)?;
        return Ok(submenu);
    }

    for (name, state) in &features {
        let slug = name_to_slug(name);
        let feature_sub = Submenu::with_id(
            app,
            format!("feature:{slug}"),
            format!("{name} [{state}]"),
            true,
        )?;

        if let Some(next) = next_state(state) {
            let next_label = capitalize(next);
            feature_sub.append(&MenuItem::with_id(
                app,
                format!("advance:{slug}"),
                format!("Advance to {next_label}"),
                true,
                None::<&str>,
            )?)?;
        } else {
            feature_sub.append(&MenuItem::with_id(
                app,
                format!("feature:{slug}:terminal"),
                "Terminal state",
                false,
                None::<&str>,
            )?)?;
        }

        submenu.append(&feature_sub)?;
    }

    Ok(submenu)
}

/// Quick-actions submenu — Refresh and Open Dashboard.
fn build_quick_actions_submenu(
    app: &AppHandle,
) -> Result<Submenu<tauri::Wry>, Box<dyn std::error::Error>> {
    let submenu = Submenu::with_id(app, "actions", "Quick Actions", true)?;

    submenu.append(&MenuItem::with_id(
        app,
        "refresh",
        "Refresh",
        true,
        None::<&str>,
    )?)?;

    submenu.append(&MenuItem::with_id(
        app,
        "open-dashboard",
        "Open Dashboard",
        true,
        None::<&str>,
    )?)?;

    Ok(submenu)
}

// ---------------------------------------------------------------------------
// Data helpers
// ---------------------------------------------------------------------------

/// Feature row: (name, state).
type FeatureRow = (String, String);

/// Read feature names and states directly from the connection.
fn list_features_from_conn(
    conn: &std::sync::MutexGuard<'_, Option<rusqlite::Connection>>,
) -> Result<Vec<FeatureRow>, Box<dyn std::error::Error>> {
    let c = conn.as_ref().ok_or("Database not initialized")?;
    let mut stmt = c.prepare("SELECT name, state FROM features ORDER BY created_at DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let features: Vec<FeatureRow> = rows.filter_map(|r| r.ok()).collect();
    Ok(features)
}

/// Map a feature name to a URL-friendly slug.
fn name_to_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Determine the next valid lifecycle state.
fn next_state(current: &str) -> Option<&'static str> {
    match current {
        "created" => Some("specified"),
        "specified" => Some("researched"),
        "researched" => Some("planned"),
        "planned" => Some("implementing"),
        "implementing" => Some("validated"),
        "validated" => Some("shipped"),
        "shipped" => Some("retrospected"),
        _ => None,
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
