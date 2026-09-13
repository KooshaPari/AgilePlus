// SPDX-License-Identifier: MIT OR Apache-2.0
//! Notification helpers — tray tooltip and desktop notification fallback.

use tauri::AppHandle;

/// Update the tray icon tooltip with a summary line.
pub fn update_tooltip(app: &AppHandle, summary: &str) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(summary));
    }
}

/// Notify the user via desktop notification or tray tooltip.
///
/// If `tauri-plugin-notification` is available it will be used in the future;
/// for now, the tooltip is updated as a lightweight fallback.
pub fn notify_user(app: &AppHandle, title: &str, body: &str) {
    // Future: use tauri-plugin-notification for OS-level toasts.
    // Fallback: update tray tooltip with a combined message.
    update_tooltip(app, &format!("{title}: {body}"));
}
