use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod commands;
mod db;

pub use commands::*;
pub use db::DatabaseState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .manage(DatabaseState::default())
        .setup(|app| {
            // Create tray menu
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            // Build tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("AgilePlus")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
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
            let state = app.state::<DatabaseState>();
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

            *state.0.lock().unwrap() = Some(conn);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_features,
            commands::get_feature,
            commands::create_feature,
            commands::update_feature_state,
            commands::get_dashboard_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
