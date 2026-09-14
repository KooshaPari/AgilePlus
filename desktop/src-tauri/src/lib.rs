use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

mod adrs;
mod cli_bridge;
mod commands;
mod crashes;
mod db;
mod evidence;
mod traces;
mod tray;
mod work_packages;

pub use commands::*;
pub use db::{AppState, DatabaseState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set up panic handler for crash reporting
    std::panic::set_hook(Box::new(|info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unknown");

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Box<dyn Any>".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        let crash_report = format!(
            "=== CRASH REPORT ===\n\
             Thread: {thread_name}\n\
             Time: {}\n\
             Payload: {payload}\n\
             Location: {location}\n\
             ====================\n",
            chrono::Utc::now().to_rfc3339()
        );

        // Write to crash log file
        if let Some(data_dir) = dirs_next::data_dir() {
            let crash_dir = data_dir
                .join("com.phenotype.agileplus-desktop")
                .join("crashes");
            let _ = std::fs::create_dir_all(&crash_dir);
            let filename = format!(
                "crash-{}.log",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            );
            let _ = std::fs::write(crash_dir.join(filename), &crash_report);
        }

        eprintln!("{crash_report}");
    }));

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            // Build dynamic tray menu from DB state
            let menu = tray::menu::build_tray_menu(app.handle()).unwrap_or_else(|e| {
                log::warn!("Failed to build tray menu: {e}");
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

            // ── Auto-discover project database ──────────────────────────
            let state = app.state::<AppState>();

            // Try to find .agileplus/agileplus.db starting from CWD
            let cwd = std::env::current_dir().unwrap_or_default();
            if let Some(project_root) = db::find_project_root(&cwd) {
                match db::open_project_db(&project_root) {
                    Ok(conn) => {
                        *state.db.0.lock().unwrap() = Some(conn);
                        *state.repo_path.lock().unwrap() =
                            project_root.to_string_lossy().to_string();
                        log::info!(
                            "Connected to project: {}",
                            project_root.display()
                        );
                    }
                    Err(e) => {
                        log::warn!("Found project but failed to open DB: {e}");
                    }
                }
            } else {
                log::info!(
                    "No .agileplus project found in {}. User can open one via the UI.",
                    cwd.display()
                );
            }

            // Show the main window (was invisible by default)
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Feature commands
            commands::list_features,
            commands::get_feature,
            commands::get_feature_with_details,
            commands::get_dashboard_stats,
            commands::set_repo_path,
            commands::get_repo_path,
            commands::open_project,
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
            // Crash reporting
            crashes::list_crash_logs,
            crashes::clear_crash_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
