//! Lists repo. CRUD shape mirrors `notes`; tasks reference lists via FK so
//! soft-deleting a list does NOT cascade-delete its tasks (FK cascade fires
//! only on hard delete, which we don't do in MVP).

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{List, NewList};

const TABLE: &str = "lists";

pub async fn create(db: &Db, input: NewList) -> Result<List> {
    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO lists (id, title, color, created_at, updated_at, hlc_ts, origin_device_id) \
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.color)
    .bind(now)
    .bind(&hlc)
    .bind(&device)
    .execute(&mut *tx)
    .await?;
    log_change(&mut tx, TABLE, &id, "insert", None, &hlc).await?;
    tx.commit().await?;

    get(&db.pool, &id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<List>> {
    let row = sqlx::query_as::<_, List>("SELECT * FROM lists WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_active(pool: &SqlitePool) -> Result<Vec<List>> {
    let rows = sqlx::query_as::<_, List>(
        "SELECT * FROM lists WHERE deleted_at IS NULL ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn rename(db: &Db, id: &str, title: &str) -> Result<List> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE lists SET title = ?1, updated_at = ?2, hlc_ts = ?3 \
         WHERE id = ?4 AND deleted_at IS NULL",
    )
    .bind(title)
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
        "UPDATE lists SET deleted_at = ?1, updated_at = ?1, hlc_ts = ?2 \
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
