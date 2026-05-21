//! Library-wide error type.
//!
//! Repo and DB code returns `HoverdoError`; Tauri commands convert to a
//! string-friendly representation via `Display` before crossing the IPC
//! boundary so the frontend never sees raw Rust error variants.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HoverdoError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("row not found")]
    NotFound,

    #[error("invalid HLC string: {0}")]
    InvalidHlc(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("system time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl HoverdoError {
    /// Convenience for ad-hoc internal errors.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

pub type Result<T, E = HoverdoError> = std::result::Result<T, E>;

/// Implement `serde::Serialize` so Tauri command results can carry the error
/// as a simple string payload to the frontend (avoiding enum variant
/// leakage).
impl serde::Serialize for HoverdoError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
