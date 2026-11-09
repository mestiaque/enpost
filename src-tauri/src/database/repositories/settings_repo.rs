//! Settings are stored one key per row (JSON value), so adding a setting never needs a migration
//! and one unreadable value only resets that key.

use crate::core::error::AppResult;
use crate::database::now_ms;
use serde_json::Value;
use sqlx::SqlitePool;
use std::collections::HashMap;

pub async fn load_all(pool: &SqlitePool) -> AppResult<HashMap<String, Value>> {
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM settings").fetch_all(pool).await?;
    Ok(rows.into_iter().filter_map(|(k, v)| serde_json::from_str(&v).ok().map(|v| (k, v))).collect())
}

/// Write several keys in one transaction (all or nothing).
pub async fn save(pool: &SqlitePool, values: &[(String, Value)]) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let now = now_ms();
    for (k, v) in values {
        sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at")
            .bind(k)
            .bind(serde_json::to_string(v)?)
            .bind(now)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn clear(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query("DELETE FROM settings").execute(pool).await?;
    Ok(())
}
