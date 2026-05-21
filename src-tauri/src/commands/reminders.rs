//! Reminders IPC. MVP scope: tasks only - notes can join in a future commit
//! once the editor has room for a "remind me" affordance.

use serde::Deserialize;
use tauri::{AppHandle, Emitter, State};
use time::OffsetDateTime;

use crate::db::repos;
use crate::error::HoverdoError;
use crate::models::{Reminder, ReminderTarget};
use crate::AppState;

/// Wrapper so the `due_at` parameter is deserialized as an RFC3339 string
/// (what the frontend sends via `Date.toISOString()`) rather than `time`'s
/// default array form.
#[derive(Debug, Deserialize)]
pub struct SetReminderInput {
    pub task_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub due_at: OffsetDateTime,
}

const EVENT_REMINDERS_CHANGED: &str = "reminders:changed";

fn emit(app: &AppHandle, payload: &str) {
    if let Err(e) = app.emit(EVENT_REMINDERS_CHANGED, payload.to_string()) {
        tracing::warn!(error = %e, "failed to emit reminders:changed");
    }
}

/// Upsert a single reminder for the given task: any existing pending
/// reminder pointing at this task is cancelled first, so each task has at
/// most one active reminder.
#[tauri::command]
pub async fn set_task_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SetReminderInput,
) -> Result<Reminder, HoverdoError> {
    let SetReminderInput { task_id, due_at } = input;
    let task = repos::tasks::get(&state.db.pool, &task_id)
        .await?
        .ok_or(HoverdoError::NotFound)?;

    // Cancel any prior pending reminder for the same task.
    let pending = repos::reminders::pending(&state.db.pool).await?;
    for r in pending {
        if r.target_type == "task" && r.target_id == task_id {
            let _ = repos::reminders::cancel(&state.db, &r.id).await;
        }
    }

    let reminder =
        repos::reminders::create(&state.db, ReminderTarget::Task, &task_id, due_at).await?;
    emit(&app, &task.list_id);
    Ok(reminder)
}

#[tauri::command]
pub async fn cancel_reminder(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), HoverdoError> {
    // Look up the reminder so the event payload still has the list_id,
    // even though we're soft-deleting the row.
    let target_list = match repos::reminders::get(&state.db.pool, &id).await? {
        Some(r) if r.target_type == "task" => repos::tasks::get(&state.db.pool, &r.target_id)
            .await?
            .map(|t| t.list_id),
        _ => None,
    };
    repos::reminders::cancel(&state.db, &id).await?;
    if let Some(list_id) = target_list {
        emit(&app, &list_id);
    }
    Ok(())
}

#[tauri::command]
pub async fn list_reminders_for_list(
    state: State<'_, AppState>,
    list_id: String,
) -> Result<Vec<Reminder>, HoverdoError> {
    repos::reminders::list_for_list_tasks(&state.db.pool, &list_id).await
}
