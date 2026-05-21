//! Domain model types.
//!
//! All "live" rows include the sync-ready columns described in
//! `db/migrations/0001_init.sql`. Repos return these structs; commands
//! serialize them straight to the frontend.
//!
//! Every `OffsetDateTime` goes through `time::serde::rfc3339` so the
//! frontend can do `new Date(iso)`. Without this, `time` defaults to
//! emitting a structured array which JS would parse as `NaN`.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

mod rfc3339 {
    pub use time::serde::rfc3339::*;
}

mod rfc3339_opt {
    pub use time::serde::rfc3339::option::*;
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: String,
    pub color: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct List {
    pub id: String,
    pub title: String,
    pub color: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub done: bool,
    #[serde(with = "rfc3339_opt")]
    pub due_at: Option<OffsetDateTime>,
    pub sort_key: String,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReminderTarget {
    Note,
    Task,
}

impl ReminderTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Task => "task",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Reminder {
    pub id: String,
    /// Stored as 'note'|'task' in SQLite; map manually when reading.
    pub target_type: String,
    pub target_id: String,
    #[serde(with = "rfc3339")]
    pub due_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub fired_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WidgetKind {
    Sticky,
    Todo,
}

impl WidgetKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sticky => "sticky",
            Self::Todo => "todo",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WidgetInstance {
    pub id: String,
    pub kind: String,
    pub target_id: String,
    pub win_x: Option<i64>,
    pub win_y: Option<i64>,
    pub win_w: i64,
    pub win_h: i64,
    pub opacity: f64,
    pub always_on_top: bool,
    pub theme_override: Option<String>,
    #[serde(with = "rfc3339_opt")]
    pub last_opened_at: Option<OffsetDateTime>,
    #[serde(with = "rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "rfc3339")]
    pub updated_at: OffsetDateTime,
    #[serde(with = "rfc3339_opt")]
    pub deleted_at: Option<OffsetDateTime>,
    pub hlc_ts: String,
    pub origin_device_id: String,
}

// -- input/patch types ------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewNote {
    pub title: String,
    pub body: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NotePatch {
    pub title: Option<String>,
    pub body: Option<String>,
    pub color: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewList {
    pub title: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewTask {
    pub list_id: String,
    pub title: String,
    #[serde(default, with = "rfc3339_opt")]
    pub due_at: Option<OffsetDateTime>,
    /// Optional explicit sort key; if `None`, repo appends after the last task.
    pub sort_key: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct NewTag {
    pub name: String,
    pub color: Option<String>,
}
