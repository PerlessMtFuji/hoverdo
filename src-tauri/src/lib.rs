//! Hoverdo core library.
//!
//! The Tauri shell is intentionally thin: it owns window/tray lifecycle and
//! exposes IPC commands. Domain logic lives in dedicated modules so it can be
//! unit-tested without the desktop runtime.

pub mod clock;
pub mod commands;
pub mod db;
pub mod error;
pub mod models;
pub mod sync;
mod theme;
pub mod widgets;

use std::sync::Arc;

use tauri::Manager;
use tracing_subscriber::EnvFilter;

use crate::db::Db;

/// Shared application state - one DB handle owned by all commands.
pub struct AppState {
    pub db: Arc<Db>,
}

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

            // Open the on-disk database under the OS-standard app data dir.
            // `block_on` is acceptable here because setup runs once on the
            // main thread before any window starts driving.
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("resolve app_data_dir");
            let db_path = app_data_dir.join("hoverdo.db");
            let db = tauri::async_runtime::block_on(Db::open(&db_path))
                .expect("open hoverdo database");
            tracing::info!(path = ?db_path, device_id = %db.device_id, "database ready");

            app.manage(AppState { db: Arc::new(db) });

            // Restore any widgets that were open when the app last quit. We
            // run this async so setup() doesn't block on window creation.
            let restore_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match widgets::restore_active(&restore_handle).await {
                    Ok(n) if n > 0 => tracing::info!(count = n, "restored widgets"),
                    Ok(_) => {}
                    Err(e) => tracing::warn!(error = %e, "widget restore failed"),
                }
            });

            tracing::info!("Hoverdo started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ping,
            health,
            commands::notes::create_note,
            commands::notes::list_notes,
            commands::notes::get_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::widgets::pin_note,
            commands::widgets::unpin_widget,
            commands::widgets::get_widget,
            commands::widgets::list_widgets,
            commands::widgets::save_widget_geometry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn ping() -> &'static str {
    "pong"
}

/// Diagnostic command exposing DB connectivity + device id - lets the
/// frontend confirm the backend is fully booted before doing real work.
#[tauri::command]
async fn health(state: tauri::State<'_, AppState>) -> Result<Health, error::HoverdoError> {
    let active_widgets = db::repos::widget_instances::list_active(&state.db.pool)
        .await?
        .len();
    Ok(Health {
        device_id: state.db.device_id_str(),
        active_widgets,
    })
}

#[derive(serde::Serialize)]
pub struct Health {
    pub device_id: String,
    pub active_widgets: usize,
}
