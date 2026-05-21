//! Sync layer - stubs only in MVP.
//!
//! Hoverdo is offline-only today, but the schema already carries every column
//! a future sync transport needs (HLC, soft deletes, change_log). The trait
//! below pins the contract any future provider will implement, so adding a
//! relay/CRDT backend later is additive rather than disruptive.

use async_trait::async_trait;

use crate::clock::Hlc;
use crate::error::Result;

/// One row of the append-only `change_log`. Sync providers push these in
/// HLC order and merge incoming entries on the receiving device.
#[derive(Debug, Clone)]
pub struct ChangeEntry {
    pub seq: i64,
    pub table: String,
    pub row_id: String,
    pub op: String,
    pub payload: Option<String>,
    pub hlc: Hlc,
}

/// Transport-agnostic sync interface. Implementations may target a
/// self-hosted relay, an iCloud-style file drop, a Yjs document, etc.
#[async_trait]
pub trait SyncProvider: Send + Sync {
    async fn push(&self, changes: &[ChangeEntry]) -> Result<()>;
    async fn pull(&self, since: Hlc) -> Result<Vec<ChangeEntry>>;
}
