//! Tasks repo. The `sort_key` column drives in-list ordering; for MVP we use
//! a monotonic textual key (`a0`, `a1`, ...) appended after the highest
//! existing key. Real fractional indexing for drag-reorder lands with the
//! to-do widget commit.

use sqlx::{Row, SqlitePool};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{NewTask, Task};

const TABLE: &str = "tasks";

pub async fn create(db: &Db, input: NewTask) -> Result<Task> {
    if input.title.trim().is_empty() {
        return Err(HoverdoError::InvalidInput("task title is empty".into()));
    }

    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();
    let sort_key = match input.sort_key {
        Some(k) => k,
        None => next_sort_key(&db.pool, &input.list_id).await?,
    };

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO tasks (id, list_id, title, due_at, sort_key, created_at, updated_at, hlc_ts, origin_device_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8)",
    )
    .bind(&id)
    .bind(&input.list_id)
    .bind(&input.title)
    .bind(input.due_at)
    .bind(&sort_key)
    .bind(now)
    .bind(&hlc)
    .bind(&device)
    .execute(&mut *tx)
    .await?;
    log_change(&mut tx, TABLE, &id, "insert", None, &hlc).await?;
    tx.commit().await?;

    get(&db.pool, &id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Task>> {
    let row = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_for(pool: &SqlitePool, list_id: &str) -> Result<Vec<Task>> {
    let rows = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE list_id = ?1 AND deleted_at IS NULL ORDER BY sort_key",
    )
    .bind(list_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn set_done(db: &Db, id: &str, done: bool) -> Result<Task> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE tasks SET done = ?1, updated_at = ?2, hlc_ts = ?3 \
         WHERE id = ?4 AND deleted_at IS NULL",
    )
    .bind(done)
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
    get(&db.pool, id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn soft_delete(db: &Db, id: &str) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE tasks SET deleted_at = ?1, updated_at = ?1, hlc_ts = ?2 \
         WHERE id = ?3 AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(&hlc)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    if res.rows_affected() == 0 {
        return Err(HoverdoError::NotFound);
    }
    log_change(&mut tx, TABLE, id, "delete", None, &hlc).await?;
    tx.commit().await?;
    Ok(())
}

/// MVP-grade "append after the last task" key generator. Replaced by proper
/// fractional indexing once drag-reorder lands.
async fn next_sort_key(pool: &SqlitePool, list_id: &str) -> Result<String> {
    let row = sqlx::query(
        "SELECT MAX(sort_key) AS k FROM tasks WHERE list_id = ?1 AND deleted_at IS NULL",
    )
    .bind(list_id)
    .fetch_one(pool)
    .await?;
    let max: Option<String> = row.get("k");
    Ok(match max {
        Some(k) => increment_key(&k),
        None => "a0".to_string(),
    })
}

/// Trivial bump: parse trailing integer if any, increment, otherwise append "0".
fn increment_key(k: &str) -> String {
    if let Some(pos) = k.rfind(|c: char| !c.is_ascii_digit()) {
        let (prefix, num) = k.split_at(pos + 1);
        let n: u64 = num.parse().unwrap_or(0);
        format!("{prefix}{}", n + 1)
    } else {
        format!("{k}0")
    }
}
