//! Notes repo. Reference implementation - other repos follow the same shape.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::{log_change, Db};
use crate::error::{HoverdoError, Result};
use crate::models::{NewNote, Note, NotePatch};

const TABLE: &str = "notes";

pub async fn create(db: &Db, input: NewNote) -> Result<Note> {
    let now = OffsetDateTime::now_utc();
    let id = Uuid::now_v7().to_string();
    let hlc = db.clock.now().to_string();
    let device = db.device_id_str();

    let mut tx = db.pool.begin().await?;

    sqlx::query(
        "INSERT INTO notes (id, title, body, color, created_at, updated_at, hlc_ts, origin_device_id) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
    )
    .bind(&id)
    .bind(&input.title)
    .bind(&input.body)
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

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Note>> {
    let row = sqlx::query_as::<_, Note>("SELECT * FROM notes WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// All non-deleted notes, newest-updated first.
pub async fn list_active(pool: &SqlitePool) -> Result<Vec<Note>> {
    let rows = sqlx::query_as::<_, Note>(
        "SELECT * FROM notes WHERE deleted_at IS NULL ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update(db: &Db, id: &str, patch: NotePatch) -> Result<Note> {
    let existing = get(&db.pool, id).await?.ok_or(HoverdoError::NotFound)?;
    if existing.deleted_at.is_some() {
        return Err(HoverdoError::InvalidInput(
            "cannot update soft-deleted note".into(),
        ));
    }

    let title = patch.title.unwrap_or(existing.title);
    let body = patch.body.unwrap_or(existing.body);
    let color = match patch.color {
        Some(v) => v,
        None => existing.color,
    };

    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();

    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "UPDATE notes SET title = ?1, body = ?2, color = ?3, updated_at = ?4, hlc_ts = ?5 \
         WHERE id = ?6",
    )
    .bind(&title)
    .bind(&body)
    .bind(&color)
    .bind(now)
    .bind(&hlc)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    log_change(&mut tx, TABLE, id, "update", None, &hlc).await?;
    tx.commit().await?;

    get(&db.pool, id).await?.ok_or(HoverdoError::NotFound)
}

pub async fn soft_delete(db: &Db, id: &str) -> Result<()> {
    let now = OffsetDateTime::now_utc();
    let hlc = db.clock.now().to_string();

    let mut tx = db.pool.begin().await?;
    let res = sqlx::query(
        "UPDATE notes SET deleted_at = ?1, updated_at = ?1, hlc_ts = ?2 WHERE id = ?3 \
         AND deleted_at IS NULL",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    async fn fresh_db() -> Db {
        Db::open_in_memory().await.expect("open in-memory db")
    }

    #[tokio::test]
    async fn create_then_get() {
        let db = fresh_db().await;
        let note = create(
            &db,
            NewNote {
                title: "first".into(),
                body: "body".into(),
                color: Some("#fff".into()),
            },
        )
        .await
        .unwrap();
        assert_eq!(note.title, "first");
        assert_eq!(note.origin_device_id, db.device_id_str());
        let fetched = get(&db.pool, &note.id).await.unwrap().unwrap();
        assert_eq!(fetched.id, note.id);
    }

    #[tokio::test]
    async fn list_excludes_soft_deleted() {
        let db = fresh_db().await;
        let a = create(&db, NewNote { title: "a".into(), ..Default::default() })
            .await
            .unwrap();
        let _b = create(&db, NewNote { title: "b".into(), ..Default::default() })
            .await
            .unwrap();
        soft_delete(&db, &a.id).await.unwrap();
        let live = list_active(&db.pool).await.unwrap();
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].title, "b");
    }

    #[tokio::test]
    async fn update_changes_hlc_and_bumps_timestamp() {
        let db = fresh_db().await;
        let note = create(&db, NewNote { title: "x".into(), ..Default::default() })
            .await
            .unwrap();
        let original_hlc = note.hlc_ts.clone();
        let updated = update(
            &db,
            &note.id,
            NotePatch {
                title: Some("renamed".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.title, "renamed");
        assert!(
            updated.hlc_ts > original_hlc,
            "hlc must advance on update: {original_hlc} -> {}",
            updated.hlc_ts
        );
        assert!(updated.updated_at >= note.updated_at);
    }

    #[tokio::test]
    async fn cannot_update_soft_deleted() {
        let db = fresh_db().await;
        let note = create(&db, NewNote { title: "x".into(), ..Default::default() })
            .await
            .unwrap();
        soft_delete(&db, &note.id).await.unwrap();
        let err = update(
            &db,
            &note.id,
            NotePatch {
                title: Some("y".into()),
                ..Default::default()
            },
        )
        .await
        .expect_err("must reject");
        assert!(matches!(err, HoverdoError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn change_log_records_every_mutation() {
        let db = fresh_db().await;
        let n = create(&db, NewNote { title: "x".into(), ..Default::default() })
            .await
            .unwrap();
        update(
            &db,
            &n.id,
            NotePatch {
                title: Some("y".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        soft_delete(&db, &n.id).await.unwrap();
        let count = crate::db::change_log_len(&db.pool).await.unwrap();
        assert_eq!(count, 3);
    }
}
