//! Tags repo. `tags.name` is UNIQUE, so `upsert_by_name` is the canonical
//! entry point - duplicates are merged rather than rejected.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{NewTag, Tag};

const TABLE: &str = "tags";

pub async fn upsert_by_name(db: &Db, input: NewTag) -> Result<Tag> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(HoverdoError::InvalidInput("tag name is empty".into()));
    }
    if let Some(existing) = get_by_name(&db.pool, &name).await? {
        return Ok(existing);
    }

    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO tags (id, name, color, created_at, updated_at, hlc_ts, origin_device_id) \
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6)",
    )
    .bind(&id)
    .bind(&name)
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

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Tag>> {
    let row = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn get_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Tag>> {
    let row = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE name = ?1 AND deleted_at IS NULL",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Tag>> {
    let rows = sqlx::query_as::<_, Tag>(
        "SELECT * FROM tags WHERE deleted_at IS NULL ORDER BY name COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn attach_to_note(pool: &SqlitePool, note_id: &str, tag_id: &str) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
    )
    .bind(note_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn detach_from_note(pool: &SqlitePool, note_id: &str, tag_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM note_tags WHERE note_id = ?1 AND tag_id = ?2")
        .bind(note_id)
        .bind(tag_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn attach_to_list(pool: &SqlitePool, list_id: &str, tag_id: &str) -> Result<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO list_tags (list_id, tag_id) VALUES (?1, ?2)",
    )
    .bind(list_id)
    .bind(tag_id)
    .execute(pool)
    .await?;
    Ok(())
}
