//! Reminders repo. The actual scheduling/firing loop lives in the reminders
//! module that lands with the notifications commit; this repo just persists
//! the bookings.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{Reminder, ReminderTarget};

const TABLE: &str = "reminders";

pub async fn create(
    db: &Db,
    target: ReminderTarget,
    target_id: &str,
    due_at: OffsetDateTime,
) -> Result<Reminder> {
    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO reminders (id, target_type, target_id, due_at, created_at, updated_at, hlc_ts, origin_device_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
    )
    .bind(&id)
    .bind(target.as_str())
    .bind(target_id)
    .bind(due_at)
    .bind(now)
    .bind(&hlc)
    .bind(&device)
    .execute(&mut *tx)
    .await?;
    log_change(&mut tx, TABLE, &id, "insert", None, &hlc).await?;
    tx.commit().await?;

    get(&db.pool, &id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Reminder>> {
    let row = sqlx::query_as::<_, Reminder>("SELECT * FROM reminders WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// All reminders that haven't fired yet, ordered by due time. The future
/// scheduler polls this list.
pub async fn pending(pool: &SqlitePool) -> Result<Vec<Reminder>> {
    let rows = sqlx::query_as::<_, Reminder>(
        "SELECT * FROM reminders \
         WHERE deleted_at IS NULL AND fired_at IS NULL \
         ORDER BY due_at ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn mark_fired(db: &Db, id: &str) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE reminders SET fired_at = ?1, updated_at = ?1, hlc_ts = ?2 \
         WHERE id = ?3 AND fired_at IS NULL",
    )
    .bind(now)
    .bind(&hlc)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(HoverdoError::NotFound);
    }
    log_change(&mut tx, TABLE, id, "update", None, &hlc).await?;
    tx.commit().await?;
    Ok(())
}
