//! Widget window management.
//!
//! Each pinned widget is a row in `widget_instances` plus a Tauri window
//! whose label is `<kind>-<instance_id>`. This module owns spawn + restore +
//! cleanup; the IPC commands in `commands/widgets.rs` are thin wrappers.

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

use crate::db::repos;
use crate::error::{HoverdoError, Result};
use crate::models::{WidgetInstance, WidgetKind};
use crate::AppState;

pub const EVENT_WIDGETS_CHANGED: &str = "widgets:changed";

pub fn label_for(kind: &str, instance_id: &str) -> String {
    format!("{kind}-{instance_id}")
}

/// Notify listeners (Home Hub, future widget-list surfaces) that the set of
/// active widget instances changed. Best-effort: a missing listener isn't a
/// failure mode worth bubbling up.
pub fn emit_changed(app: &AppHandle) {
    if let Err(e) = app.emit(EVENT_WIDGETS_CHANGED, ()) {
        tracing::warn!(error = %e, "failed to emit widgets:changed");
    }
}

/// Spawn a Tauri window for the given widget instance. Idempotent: if a
/// window with the same label is already open, focus it instead.
pub fn spawn_window(app: &AppHandle, instance: &WidgetInstance) -> Result<()> {
    let label = label_for(&instance.kind, &instance.id);

    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.set_focus();
        return Ok(());
    }

    let url_path = format!("{}.html?widget_id={}", instance.kind, instance.id);

    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::App(url_path.into()))
        .title(window_title(&instance.kind))
        .inner_size(instance.win_w as f64, instance.win_h as f64)
        .min_inner_size(180.0, 140.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(instance.always_on_top)
        .resizable(true)
        .skip_taskbar(false);

    if let (Some(x), Some(y)) = (instance.win_x, instance.win_y) {
        builder = builder.position(x as f64, y as f64);
    }

    let window = builder
        .build()
        .map_err(|e| HoverdoError::internal(format!("spawn widget window: {e}")))?;

    crate::theme::apply_window_effects(&window);

    // When the user closes the widget, soft-delete its instance row so we
    // don't try to resurrect it on next startup. Note we don't preventClose
    // - close means unpin in the MVP UX.
    let app_handle = app.clone();
    let instance_id = instance.id.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            let app = app_handle.clone();
            let id = instance_id.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<AppState>();
                if let Err(e) = repos::widget_instances::unpin(&state.db, &id).await {
                    tracing::warn!(error = %e, widget = %id, "failed to soft-delete widget");
                }
                emit_changed(&app);
            });
        }
    });

    Ok(())
}

fn window_title(kind: &str) -> &'static str {
    match kind {
        "sticky" => "Sticky · Hoverdo",
        "todo" => "To-do · Hoverdo",
        _ => "Hoverdo widget",
    }
}

/// Re-open every active widget instance. Called once at startup so the
/// floating workspace looks the way the user left it.
pub async fn restore_active(app: &AppHandle) -> Result<usize> {
    let state = app.state::<AppState>();
    let widgets = repos::widget_instances::list_active(&state.db.pool).await?;
    let count = widgets.len();
    for w in widgets {
        if let Err(e) = spawn_window(app, &w) {
            tracing::warn!(error = %e, widget = %w.id, "failed to restore widget");
        }
    }
    Ok(count)
}

/// Convenience for the `pin_note` command path.
pub async fn pin_note_as_sticky(app: &AppHandle, note_id: &str) -> Result<WidgetInstance> {
    let state = app.state::<AppState>();
    let instance =
        repos::widget_instances::pin(&state.db, WidgetKind::Sticky, note_id).await?;
    spawn_window(app, &instance)?;
    emit_changed(app);
    Ok(instance)
}

/// Convenience for the `pin_list` command path.
pub async fn pin_list_as_todo(app: &AppHandle, list_id: &str) -> Result<WidgetInstance> {
    let state = app.state::<AppState>();
    let instance =
        repos::widget_instances::pin(&state.db, WidgetKind::Todo, list_id).await?;
    spawn_window(app, &instance)?;
    emit_changed(app);
    Ok(instance)
}
