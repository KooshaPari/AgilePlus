use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod adrs;
mod cli_bridge;
mod commands;
mod db;
mod evidence;
mod traces;
mod tray;
mod work_packages;

pub use commands::*;
pub use db::{AppState, DatabaseState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(|app| {
            // Build dynamic tray menu from DB state
            let menu = tray::menu::build_tray_menu(app.handle()).unwrap_or_else(|e| {
                log::warn!("Failed to build tray menu: {e}");
                // Fallback: minimal menu
                let show =
                    MenuItem::with_id(app, "show", "Show Window", true, None::<&str>).unwrap();
                let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
                Menu::with_items(app, &[&show, &quit]).unwrap()
            });

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("AgilePlus")
                .on_menu_event(move |app, event| {
                    let id = event.id().as_ref();
                    match id {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => std::process::exit(0),
                        "refresh" => tray::menu::refresh_menu(app),
                        "open-dashboard" => {
                            #[cfg(target_os = "macos")]
                            let _ = std::process::Command::new("open")
                                .arg("http://localhost:8080")
                                .spawn();
                            #[cfg(target_os = "linux")]
                            let _ = std::process::Command::new("xdg-open")
                                .arg("http://localhost:8080")
                                .spawn();
                            #[cfg(target_os = "windows")]
                            let _ = std::process::Command::new("cmd")
                                .args(["/C", "start", "http://localhost:8080"])
                                .spawn();
                        }
                        id if id.starts_with("advance:") => {
                            let slug = id["advance:".len()..].to_string();
                            let app_handle = app.clone();
                            std::thread::spawn(move || {
                                let result = tray::cli::run_cli_with_timeout(&[
                                    "specify",
                                    "--feature",
                                    &slug,
                                ]);
                                match result {
                                    Ok(_) => tray::notify::notify_user(
                                        &app_handle,
                                        "Success",
                                        &format!("Advanced {slug}"),
                                    ),
                                    Err(e) => {
                                        tray::notify::notify_user(&app_handle, "Error", &e);
                                    }
                                }
                                tray::menu::refresh_menu(&app_handle);
                            });
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Initialize database
            let state = app.state::<AppState>();
            let db_path = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("agileplus.db");

            // Ensure parent directory exists
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            // Initialize SQLite connection
            let conn = rusqlite::Connection::open(&db_path).expect("failed to open database");

            // Run migrations
            db::initialize_database(&conn).expect("failed to initialize database");

            *state.db.0.lock().unwrap() = Some(conn);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Feature commands
            commands::list_features,
            commands::get_feature,
            commands::get_feature_with_details,
            commands::create_feature,
            commands::update_feature_state,
            commands::get_dashboard_stats,
            commands::set_repo_path,
            commands::get_repo_path,
            // ADR filesystem commands
            adrs::list_adrs,
            adrs::read_adr,
            // Trace filesystem commands
            traces::list_traces,
            traces::read_trace,
            // Work package CRUD
            work_packages::list_work_packages,
            work_packages::create_work_package,
            work_packages::update_work_package_state,
            // Evidence CRUD
            evidence::list_evidence,
            evidence::create_evidence,
            // CLI bridge
            cli_bridge::run_cli_read,
            cli_bridge::run_cli_lifecycle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
