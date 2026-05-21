//! Tauri IPC commands. Each module here is a thin wrapper around a repo:
//! deserialize input, call the repo, return the result. Domain rules live
//! in the repo layer, not here. Commands are registered in `lib.rs`.

pub mod notes;
