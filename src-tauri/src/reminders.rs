//! Background reminder scheduler.
//!
//! Polls the `reminders` table every `POLL_INTERVAL` for entries whose
//! due_at has passed; for each, fires an OS notification, marks the
//! reminder fired, and broadcasts a `reminders:changed` event so any open
//! window can update its UI.
//!
//! Single-process, single-pool: no distributed locking needed.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::time::interval;

use crate::db::{repos, Db};
use crate::error::Result;
use crate::AppState;

const POLL_INTERVAL: Duration = Duration::from_secs(20);
const EVENT_REMINDERS_CHANGED: &str = "reminders:changed";

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut tick = interval(POLL_INTERVAL);
        // Skip the first immediate tick to avoid firing reminders during
        // app boot before windows are even ready.
        tick.tick().await;
        loop {
            tick.tick().await;
            let db = match app.try_state::<AppState>() {
                Some(state) => state.db.clone(),
                None => continue,
            };
            if let Err(e) = check_once(&app, &db).await {
                tracing::warn!(error = %e, "reminder check failed");
            }
        }
    });
}

async fn check_once(app: &AppHandle, db: &Arc<Db>) -> Result<()> {
    let pending = repos::reminders::pending(&db.pool).await?;
    if pending.is_empty() {
        return Ok(());
    }
    let now = time::OffsetDateTime::now_utc();

    for reminder in pending {
        if reminder.due_at > now {
            // Pending list is sorted by due_at ASC; everything else is in
            // the future.
            break;
        }
        let (title, body, list_id) = compose_notification(db, &reminder).await;
        if let Err(e) = app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
        {
            tracing::warn!(error = %e, "failed to show OS notification");
        }
        if let Err(e) = repos::reminders::mark_fired(db, &reminder.id).await {
            tracing::warn!(error = %e, "failed to mark reminder fired");
        }
        if let Some(list_id) = list_id {
            let _ = app.emit(EVENT_REMINDERS_CHANGED, list_id);
        }
    }
    Ok(())
}

async fn compose_notification(
    db: &Db,
    reminder: &crate::models::Reminder,
) -> (String, String, Option<String>) {
    match reminder.target_type.as_str() {
        "task" => {
            let task = repos::tasks::get(&db.pool, &reminder.target_id).await.ok().flatten();
            let title = "Hoverdo reminder".to_string();
            let body = task
                .as_ref()
                .map(|t| t.title.clone())
                .unwrap_or_else(|| "Task".to_string());
            (title, body, task.map(|t| t.list_id))
        }
        "note" => {
            let note = repos::notes::get(&db.pool, &reminder.target_id).await.ok().flatten();
            let title = "Hoverdo reminder".to_string();
            let body = note
                .map(|n| {
                    if !n.title.trim().is_empty() {
                        n.title
                    } else {
                        n.body
                            .lines()
                            .next()
                            .unwrap_or("Note")
                            .chars()
                            .take(80)
                            .collect()
                    }
                })
                .unwrap_or_else(|| "Note".to_string());
            (title, body, None)
        }
        _ => ("Hoverdo reminder".into(), "".into(), None),
    }
}
