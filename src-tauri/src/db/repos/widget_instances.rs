//! Widget instances repo. One row per pinned floating widget; stores window
//! geometry, opacity, always-on-top and an optional theme override.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{WidgetInstance, WidgetKind};

const TABLE: &str = "widget_instances";

pub async fn pin(db: &Db, kind: WidgetKind, target_id: &str) -> Result<WidgetInstance> {
    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO widget_instances \
            (id, kind, target_id, created_at, updated_at, hlc_ts, origin_device_id, last_opened_at) \
         VALUES (?1, ?2, ?3, ?4, ?4, ?5, ?6, ?4)",
    )
    .bind(&id)
    .bind(kind.as_str())
    .bind(target_id)
    .bind(now)
    .bind(&hlc)
    .bind(&device)
    .execute(&mut *tx)
    .await?;
    log_change(&mut tx, TABLE, &id, "insert", None, &hlc).await?;
    tx.commit().await?;

    get(&db.pool, &id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<WidgetInstance>> {
    let row = sqlx::query_as::<_, WidgetInstance>(
        "SELECT * FROM widget_instances WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// All currently-pinned widgets, used on startup to restore the workspace.
pub async fn list_active(pool: &SqlitePool) -> Result<Vec<WidgetInstance>> {
    let rows = sqlx::query_as::<_, WidgetInstance>(
        "SELECT * FROM widget_instances WHERE deleted_at IS NULL ORDER BY last_opened_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Persist window geometry. Called from a debounced listener on Tauri
/// `tauri://move` / `tauri://resize` events.
pub async fn update_geometry(
    db: &Db,
    id: &str,
    x: Option<i64>,
    y: Option<i64>,
    w: i64,
    h: i64,
) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE widget_instances \
         SET win_x = ?1, win_y = ?2, win_w = ?3, win_h = ?4, updated_at = ?5, hlc_ts = ?6 \
         WHERE id = ?7 AND deleted_at IS NULL",
    )
    .bind(x)
    .bind(y)
    .bind(w)
    .bind(h)
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

pub async fn set_opacity(db: &Db, id: &str, opacity: f64) -> Result<()> {
    let opacity = opacity.clamp(0.1, 1.0);
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE widget_instances SET opacity = ?1, updated_at = ?2, hlc_ts = ?3 \
         WHERE id = ?4 AND deleted_at IS NULL",
    )
    .bind(opacity)
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

pub async fn set_always_on_top(db: &Db, id: &str, on_top: bool) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE widget_instances SET always_on_top = ?1, updated_at = ?2, hlc_ts = ?3 \
         WHERE id = ?4 AND deleted_at IS NULL",
    )
    .bind(on_top)
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

pub async fn unpin(db: &Db, id: &str) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();
    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE widget_instances SET deleted_at = ?1, updated_at = ?1, hlc_ts = ?2 \
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
