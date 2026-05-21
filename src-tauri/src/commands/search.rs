//! Global search backed by the FTS5 virtual tables wired up in 0001_init.sql.
//!
//! Each kind (notes / lists / tasks) is queried separately and the union is
//! ranked client-side by FTS rank so results stay heterogeneous. Snippets
//! come straight from FTS5's `snippet()` function.

use serde::Serialize;
use sqlx::Row;
use tauri::State;

use crate::error::HoverdoError;
use crate::AppState;

const MAX_PER_KIND: u32 = 15;

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub kind: &'static str,
    pub id: String,
    pub title: String,
    pub snippet: String,
    /// For tasks: the parent list_id so the caller can jump straight there.
    pub list_id: Option<String>,
    /// FTS5 rank (lower is better) - used for client-side sort.
    pub rank: f64,
}

#[tauri::command]
pub async fn search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchHit>, HoverdoError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    // Wrap in quotes to force phrase matching and side-step FTS5's
    // special-character syntax (so the user can type "foo: bar" without
    // tripping a parser error). Append a `*` so single-token queries do a
    // prefix match instead of exact-only.
    let fts_query = format!("\"{}\"*", trimmed.replace('"', "\"\""));

    let mut hits = Vec::new();
    hits.extend(search_notes(&state, &fts_query).await?);
    hits.extend(search_lists(&state, &fts_query).await?);
    hits.extend(search_tasks(&state, &fts_query).await?);
    hits.sort_by(|a, b| a.rank.partial_cmp(&b.rank).unwrap_or(std::cmp::Ordering::Equal));
    Ok(hits)
}

async fn search_notes(state: &State<'_, AppState>, fts: &str) -> Result<Vec<SearchHit>, HoverdoError> {
    let rows = sqlx::query(
        "SELECT n.id AS id, \
                n.title AS title, \
                snippet(notes_fts, 1, char(1), char(2), '…', 16) AS snippet, \
                rank AS rank \
         FROM notes_fts \
         JOIN notes n ON n.rowid = notes_fts.rowid \
         WHERE notes_fts MATCH ?1 AND n.deleted_at IS NULL \
         ORDER BY rank \
         LIMIT ?2",
    )
    .bind(fts)
    .bind(MAX_PER_KIND)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SearchHit {
            kind: "note",
            id: r.get("id"),
            title: r.get("title"),
            snippet: r.get("snippet"),
            list_id: None,
            rank: r.get::<f64, _>("rank"),
        })
        .collect())
}

async fn search_lists(state: &State<'_, AppState>, fts: &str) -> Result<Vec<SearchHit>, HoverdoError> {
    let rows = sqlx::query(
        "SELECT l.id AS id, \
                l.title AS title, \
                snippet(lists_fts, 0, char(1), char(2), '…', 16) AS snippet, \
                rank AS rank \
         FROM lists_fts \
         JOIN lists l ON l.rowid = lists_fts.rowid \
         WHERE lists_fts MATCH ?1 AND l.deleted_at IS NULL \
         ORDER BY rank \
         LIMIT ?2",
    )
    .bind(fts)
    .bind(MAX_PER_KIND)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SearchHit {
            kind: "list",
            id: r.get("id"),
            title: r.get("title"),
            snippet: r.get("snippet"),
            list_id: None,
            rank: r.get::<f64, _>("rank"),
        })
        .collect())
}

async fn search_tasks(state: &State<'_, AppState>, fts: &str) -> Result<Vec<SearchHit>, HoverdoError> {
    let rows = sqlx::query(
        "SELECT t.id AS id, \
                t.title AS title, \
                t.list_id AS list_id, \
                snippet(tasks_fts, 0, char(1), char(2), '…', 16) AS snippet, \
                rank AS rank \
         FROM tasks_fts \
         JOIN tasks t ON t.rowid = tasks_fts.rowid \
         WHERE tasks_fts MATCH ?1 AND t.deleted_at IS NULL \
         ORDER BY rank \
         LIMIT ?2",
    )
    .bind(fts)
    .bind(MAX_PER_KIND)
    .fetch_all(&state.db.pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SearchHit {
            kind: "task",
            id: r.get("id"),
            title: r.get("title"),
            snippet: r.get("snippet"),
            list_id: Some(r.get("list_id")),
            rank: r.get::<f64, _>("rank"),
        })
        .collect())
}
