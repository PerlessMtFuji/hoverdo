//! Widget IPC commands.

use tauri::{AppHandle, Manager, State};

use crate::db::repos;
use crate::error::HoverdoError;
use crate::models::WidgetInstance;
use crate::{widgets, AppState};

/// Pin the given note as a sticky-note widget: create the instance row and
/// spawn its window. If the note is already pinned we just focus the
/// existing window and return its instance.
#[tauri::command]
pub async fn pin_note(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
) -> Result<WidgetInstance, HoverdoError> {
    // Reuse an existing pin for this note instead of stacking duplicates -
    // the user almost always wants "show me my sticky for this note".
    let existing = repos::widget_instances::list_active(&state.db.pool).await?;
    if let Some(instance) = existing
        .into_iter()
        .find(|w| w.kind == "sticky" && w.target_id == note_id)
    {
        let label = widgets::label_for(&instance.kind, &instance.id);
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.set_focus();
        } else {
            widgets::spawn_window(&app, &instance)?;
        }
        return Ok(instance);
    }

    widgets::pin_note_as_sticky(&app, &note_id).await
}

#[tauri::command]
pub async fn pin_list(
    app: AppHandle,
    state: State<'_, AppState>,
    list_id: String,
) -> Result<WidgetInstance, HoverdoError> {
    let existing = repos::widget_instances::list_active(&state.db.pool).await?;
    if let Some(instance) = existing
        .into_iter()
        .find(|w| w.kind == "todo" && w.target_id == list_id)
    {
        let label = widgets::label_for(&instance.kind, &instance.id);
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.set_focus();
        } else {
            widgets::spawn_window(&app, &instance)?;
        }
        return Ok(instance);
    }

    widgets::pin_list_as_todo(&app, &list_id).await
}

/// Hide & remove the widget. Closing the window also unpins via the
/// Destroyed handler in `widgets.rs`; this command is for explicit
/// "unpin from main window" actions where the window may not exist.
#[tauri::command]
pub async fn unpin_widget(
    app: AppHandle,
    state: State<'_, AppState>,
    widget_id: String,
) -> Result<(), HoverdoError> {
    let Some(instance) = repos::widget_instances::get(&state.db.pool, &widget_id).await? else {
        return Ok(());
    };
    let label = widgets::label_for(&instance.kind, &instance.id);
    if let Some(win) = app.get_webview_window(&label) {
        // Closing dispatches Destroyed → soft-delete + widgets:changed event
        // in the handler; we don't double-emit.
        let _ = win.close();
    } else {
        repos::widget_instances::unpin(&state.db, &widget_id).await?;
        widgets::emit_changed(&app);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_widget(
    state: State<'_, AppState>,
    widget_id: String,
) -> Result<Option<WidgetInstance>, HoverdoError> {
    repos::widget_instances::get(&state.db.pool, &widget_id).await
}

#[tauri::command]
pub async fn list_widgets(
    state: State<'_, AppState>,
) -> Result<Vec<WidgetInstance>, HoverdoError> {
    repos::widget_instances::list_active(&state.db.pool).await
}

/// Save the widget's window geometry. Called debounced from the frontend
/// when the user moves/resizes the window.
#[tauri::command]
pub async fn save_widget_geometry(
    state: State<'_, AppState>,
    widget_id: String,
    x: Option<i64>,
    y: Option<i64>,
    w: i64,
    h: i64,
) -> Result<(), HoverdoError> {
    repos::widget_instances::update_geometry(&state.db, &widget_id, x, y, w, h).await
}

/// Persist the opacity preference. Visual opacity is applied frontend-side
/// (CSS), so this is purely persistence so it survives restarts.
#[tauri::command]
pub async fn set_widget_opacity(
    state: State<'_, AppState>,
    widget_id: String,
    opacity: f64,
) -> Result<(), HoverdoError> {
    repos::widget_instances::set_opacity(&state.db, &widget_id, opacity).await
}

/// Persist + apply always-on-top. We also flip the native window state so
/// the change is immediate; the DB value is what we restore on next start.
#[tauri::command]
pub async fn set_widget_always_on_top(
    app: AppHandle,
    state: State<'_, AppState>,
    widget_id: String,
    on_top: bool,
) -> Result<(), HoverdoError> {
    repos::widget_instances::set_always_on_top(&state.db, &widget_id, on_top).await?;
    if let Some(instance) = repos::widget_instances::get(&state.db.pool, &widget_id).await? {
        let label = widgets::label_for(&instance.kind, &instance.id);
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.set_always_on_top(on_top);
        }
    }
    Ok(())
}
