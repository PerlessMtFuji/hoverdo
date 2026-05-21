//! Database facade.
//!
//! `Db` bundles the SQLx pool, the HLC clock and this device's id, so that
//! repo functions can be written as plain `async fn(db: &Db, ...) -> Result`
//! without ten parameters each. Tauri stores one `Arc<Db>` in `tauri::State`.

use std::path::Path;
use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::clock::HlcClock;
use crate::error::{HoverdoError, Result};

pub mod repos;

/// Compile-time-embedded migrations directory.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./src/db/migrations");

const SETTING_DEVICE_ID: &str = "app.device_id";

/// Open or create the on-disk database, run migrations, ensure this device
/// has a stable identity, and return a ready-to-use `Db`.
pub struct Db {
    pub pool: SqlitePool,
    pub clock: HlcClock,
    pub device_id: Uuid,
}

impl Db {
    pub async fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(HoverdoError::Sqlx)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Self::finish(pool).await
    }

    /// In-memory database for tests. One connection (since `:memory:` is
    /// per-connection); WAL is disabled because it's incompatible with
    /// in-memory storage.
    pub async fn open_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(HoverdoError::Sqlx)?
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        Self::finish(pool).await
    }

    async fn finish(pool: SqlitePool) -> Result<Self> {
        MIGRATOR.run(&pool).await?;
        let device_id = ensure_device_id(&pool).await?;
        let clock = HlcClock::new(short_node_id(&device_id));
        Ok(Self {
            pool,
            clock,
            device_id,
        })
    }

    pub fn device_id_str(&self) -> String {
        self.device_id.to_string()
    }
}

async fn ensure_device_id(pool: &SqlitePool) -> Result<Uuid> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = ?1")
            .bind(SETTING_DEVICE_ID)
            .fetch_optional(pool)
            .await?;

    if let Some(raw) = existing {
        // Stored as JSON-encoded string: "uuid".
        let s: String =
            serde_json::from_str(&raw).map_err(|e| HoverdoError::internal(e.to_string()))?;
        Uuid::parse_str(&s).map_err(|e| HoverdoError::internal(e.to_string()))
    } else {
        let id = Uuid::now_v7();
        let value = serde_json::to_string(&id.to_string())?;
        sqlx::query("INSERT INTO settings(key, value) VALUES (?1, ?2)")
            .bind(SETTING_DEVICE_ID)
            .bind(&value)
            .execute(pool)
            .await?;
        Ok(id)
    }
}

/// Compress a full UUID into the 8-byte node id slot used by the HLC.
/// We take the first 8 bytes of the UUID's big-endian encoding - good
/// enough as a stable per-device tie-breaker.
fn short_node_id(uuid: &Uuid) -> [u8; 8] {
    let bytes = uuid.as_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&bytes[..8]);
    out
}

/// Internal helper used by repo modules: record a change-log entry on the
/// caller's connection (typically inside the same transaction as the
/// data mutation, so the two either both commit or both roll back).
pub(crate) async fn log_change(
    conn: &mut sqlx::SqliteConnection,
    table: &str,
    row_id: &str,
    op: &str,
    payload: Option<&str>,
    hlc_ts: &str,
) -> Result<()> {
    let now = time::OffsetDateTime::now_utc();
    sqlx::query(
        "INSERT INTO change_log(table_name, row_id, op, payload, hlc_ts, applied_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(table)
    .bind(row_id)
    .bind(op)
    .bind(payload)
    .bind(hlc_ts)
    .bind(now)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Number of rows currently in `change_log`. Useful for tests that want to
/// assert "exactly one mutation ran".
#[cfg(test)]
pub(crate) async fn change_log_len(pool: &SqlitePool) -> Result<i64> {
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM change_log")
        .fetch_one(pool)
        .await?;
    Ok(n)
}
