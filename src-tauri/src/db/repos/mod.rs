//! Repository functions grouped by domain. Repos are plain `async fn`s that
//! take `&Db` (or its pieces) and return `Result<...>`. They are the only
//! place SQL lives; Tauri commands in `commands/` are thin wrappers that
//! deserialize input, call a repo, and serialize the result.

pub mod lists;
pub mod notes;
pub mod reminders;
pub mod settings;
pub mod tags;
pub mod tasks;
pub mod widget_instances;
