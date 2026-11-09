//! Sent requests and their response metadata. Rows arrive here already redacted.

use crate::core::error::AppResult;
use serde::Serialize;
use sqlx::SqlitePool;
use ts_rs::TS;

/// One row of the History list.
#[derive(Debug, Clone, PartialEq, Serialize, TS, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HistoryEntry {
    pub id: String,
    pub method: String,
    pub url: String,
    #[ts(type = "number")]
    pub sent_at: i64,
    #[ts(type = "number | null")]
    pub status: Option<i64>,
    #[ts(type = "number | null")]
    pub duration_ms: Option<i64>,
    #[ts(type = "number | null")]
    pub size_bytes: Option<i64>,
    pub error: Option<String>,
    /// JSON snapshot (redacted): headers and body type, to reopen the request.
    pub snapshot: String,
}

pub struct NewHistory<'a> {
    pub id: &'a str,
    pub method: &'a str,
    pub url: &'a str,
    pub snapshot: &'a str,
    pub sent_at: i64,
    pub status: Option<i64>,
    pub status_text: Option<&'a str>,
    pub duration_ms: Option<i64>,
    pub size_bytes: Option<i64>,
    pub headers: &'a str,
    pub body_path: Option<&'a str>,
    pub error: Option<&'a str>,
}

pub async fn insert(pool: &SqlitePool, h: &NewHistory<'_>) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO request_history (id, method, url, snapshot, sent_at) VALUES (?1, ?2, ?3, ?4, ?5)")
        .bind(h.id)
        .bind(h.method)
        .bind(h.url)
        .bind(h.snapshot)
        .bind(h.sent_at)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO response_metadata (id, history_id, status, status_text, duration_ms, size_bytes, headers, body_path, error, received_at)
         VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .bind(h.id)
    .bind(h.status)
    .bind(h.status_text)
    .bind(h.duration_ms)
    .bind(h.size_bytes)
    .bind(h.headers)
    .bind(h.body_path)
    .bind(h.error)
    .bind(h.sent_at + h.duration_ms.unwrap_or(0))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list(pool: &SqlitePool, limit: i64) -> AppResult<Vec<HistoryEntry>> {
    Ok(sqlx::query_as(
        "SELECT h.id, h.method, h.url, h.sent_at, m.status, m.duration_ms, m.size_bytes, m.error, h.snapshot
         FROM request_history h LEFT JOIN response_metadata m ON m.history_id = h.id
         ORDER BY h.sent_at DESC LIMIT ?1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Body files of the rows `delete_*` is about to remove.
async fn body_paths(pool: &SqlitePool, where_sql: &str, arg: i64) -> AppResult<Vec<String>> {
    let sql = format!("SELECT m.body_path FROM response_metadata m JOIN request_history h ON h.id = m.history_id WHERE m.body_path IS NOT NULL AND {where_sql}");
    Ok(sqlx::query_scalar(&sql).bind(arg).fetch_all(pool).await?)
}

/// Delete entries sent before `cutoff_ms`; returns the body files to remove.
pub async fn delete_older_than(pool: &SqlitePool, cutoff_ms: i64) -> AppResult<Vec<String>> {
    let files = body_paths(pool, "h.sent_at < ?1", cutoff_ms).await?;
    sqlx::query("DELETE FROM request_history WHERE sent_at < ?1").bind(cutoff_ms).execute(pool).await?;
    Ok(files)
}

pub async fn delete_all(pool: &SqlitePool) -> AppResult<Vec<String>> {
    let files = body_paths(pool, "?1 = ?1", 0).await?;
    sqlx::query("DELETE FROM request_history").execute(pool).await?;
    Ok(files)
}

pub async fn body_path(pool: &SqlitePool, id: &str) -> AppResult<Option<String>> {
    Ok(sqlx::query_scalar("SELECT body_path FROM response_metadata WHERE history_id = ?1").bind(id).fetch_optional(pool).await?.flatten())
}
