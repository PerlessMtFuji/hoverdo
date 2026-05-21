//! Key-value settings, JSON-encoded values. Used for the device id, theme
//! preference, app-wide opacity default, etc. Reserved key prefix: `app.*`.

use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::error::Result;

pub async fn get_raw(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn get<T: DeserializeOwned>(pool: &SqlitePool, key: &str) -> Result<Option<T>> {
    match get_raw(pool, key).await? {
        Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
        None => Ok(None),
    }
}

pub async fn set<T: Serialize>(pool: &SqlitePool, key: &str, value: &T) -> Result<()> {
    let raw = serde_json::to_string(value)?;
    sqlx::query(
        "INSERT INTO settings(key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(&raw)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, key: &str) -> Result<()> {
    sqlx::query("DELETE FROM settings WHERE key = ?1")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}
