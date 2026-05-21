//! Notes IPC commands. Frontend calls these via `invoke('create_note', ...)`.

use tauri::{AppHandle, Emitter, State};

use crate::db::repos;
use crate::error::HoverdoError;
use crate::models::{NewNote, Note, NotePatch};
use crate::AppState;

const EVENT_NOTES_CHANGED: &str = "notes:changed";

#[tauri::command]
pub async fn create_note(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NewNote,
) -> Result<Note, HoverdoError> {
    let note = repos::notes::create(&state.db, input).await?;
    emit_changed(&app, &note.id);
    Ok(note)
}

#[tauri::command]
pub async fn list_notes(state: State<'_, AppState>) -> Result<Vec<Note>, HoverdoError> {
    repos::notes::list_active(&state.db.pool).await
}

#[tauri::command]
pub async fn get_note(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<Note>, HoverdoError> {
    repos::notes::get(&state.db.pool, &id).await
}

#[tauri::command]
pub async fn update_note(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    patch: NotePatch,
) -> Result<Note, HoverdoError> {
    let note = repos::notes::update(&state.db, &id, patch).await?;
    emit_changed(&app, &note.id);
    Ok(note)
}

#[tauri::command]
pub async fn delete_note(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), HoverdoError> {
    repos::notes::soft_delete(&state.db, &id).await?;
    emit_changed(&app, &id);
    Ok(())
}

/// Broadcast a "notes changed" event so every open window (main + widgets)
/// refreshes its view. Best-effort: a failure here just means a stale UI
/// until the next manual refresh, never a data loss.
fn emit_changed(app: &AppHandle, id: &str) {
    if let Err(e) = app.emit(EVENT_NOTES_CHANGED, id) {
        tracing::warn!(error = %e, "failed to emit notes:changed");
    }
}
