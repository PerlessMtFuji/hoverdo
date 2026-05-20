//! Hoverdo core library.
//!
//! The Tauri shell is intentionally thin: it owns window/tray lifecycle and
//! exposes IPC commands. Domain logic lives in dedicated modules so it can be
//! unit-tested without the desktop runtime.

mod theme;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("HOVERDO_LOG")
                .unwrap_or_else(|_| EnvFilter::new("hoverdo_lib=info,tauri=warn")),
        )
        .with_target(false)
        .compact()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Apply Mica/Acrylic to the main window. On non-Windows targets this
            // is a no-op so the app still runs on dev machines (Linux/macOS).
            if let Some(window) = app.get_webview_window("main") {
                theme::apply_window_effects(&window);
            }
            tracing::info!("Hoverdo started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Smoke-test IPC command; will be replaced by real domain commands in commit 2.
#[tauri::command]
fn ping() -> &'static str {
    "pong"
}
