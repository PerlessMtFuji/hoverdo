//! Lists + tasks IPC. Tasks live "inside" a list so we expose them on the
//! same module; conceptually they always travel together in the UI.

use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};
use time::OffsetDateTime;

use crate::db::repos;
use crate::error::HoverdoError;
use crate::models::{List, NewList, NewTask, Task};
use crate::AppState;

const EVENT_LISTS_CHANGED: &str = "lists:changed";
const EVENT_TASKS_CHANGED: &str = "tasks:changed";

fn emit(app: &AppHandle, name: &str, payload: &str) {
    if let Err(e) = app.emit(name, payload.to_string()) {
        tracing::warn!(error = %e, event = name, "failed to emit");
    }
}

// ---- lists ---------------------------------------------------------------

#[tauri::command]
pub async fn create_list(
    app: AppHandle,
    state: State<'_, AppState>,
    input: NewList,
) -> Result<List, HoverdoError> {
    let list = repos::lists::create(&state.db, input).await?;
    emit(&app, EVENT_LISTS_CHANGED, &list.id);
    Ok(list)
}

#[tauri::command]
pub async fn list_lists(state: State<'_, AppState>) -> Result<Vec<List>, HoverdoError> {
    repos::lists::list_active(&state.db.pool).await
}

#[tauri::command]
pub async fn get_list(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<List>, HoverdoError> {
    repos::lists::get(&state.db.pool, &id).await
}

#[tauri::command]
pub async fn rename_list(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    title: String,
) -> Result<List, HoverdoError> {
    let list = repos::lists::rename(&state.db, &id, &title).await?;
    emit(&app, EVENT_LISTS_CHANGED, &id);
    Ok(list)
}

#[tauri::command]
pub async fn delete_list(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), HoverdoError> {
    repos::lists::soft_delete(&state.db, &id).await?;
    emit(&app, EVENT_LISTS_CHANGED, &id);
    Ok(())
}

// ---- tasks ---------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub list_id: String,
    pub title: String,
    pub due_at: Option<OffsetDateTime>,
}

#[tauri::command]
pub async fn create_task(
    app: AppHandle,
    state: State<'_, AppState>,
    input: CreateTaskInput,
) -> Result<Task, HoverdoError> {
    let task = repos::tasks::create(
        &state.db,
        NewTask {
            list_id: input.list_id.clone(),
            title: input.title,
            due_at: input.due_at,
            sort_key: None,
        },
    )
    .await?;
    emit(&app, EVENT_TASKS_CHANGED, &input.list_id);
    Ok(task)
}

#[tauri::command]
pub async fn list_tasks(
    state: State<'_, AppState>,
    list_id: String,
) -> Result<Vec<Task>, HoverdoError> {
    repos::tasks::list_for(&state.db.pool, &list_id).await
}

#[tauri::command]
pub async fn set_task_done(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    done: bool,
) -> Result<Task, HoverdoError> {
    let task = repos::tasks::set_done(&state.db, &id, done).await?;
    emit(&app, EVENT_TASKS_CHANGED, &task.list_id);
    Ok(task)
}

#[tauri::command]
pub async fn delete_task(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), HoverdoError> {
    // Look up the task before deletion so we can emit a useful payload.
    let list_id = repos::tasks::get(&state.db.pool, &id)
        .await?
        .map(|t| t.list_id)
        .unwrap_or_default();
    repos::tasks::soft_delete(&state.db, &id).await?;
    emit(&app, EVENT_TASKS_CHANGED, &list_id);
    Ok(())
}
