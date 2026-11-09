//! SQLite storage: one connection pool, versioned migrations (`src-tauri/migrations`),
//! foreign keys and WAL on every connection. Repositories hold the SQL; services never do.

pub mod repositories;

use crate::core::error::AppResult;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

#[derive(Clone, Debug)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Open (creating if needed) the database file and bring the schema up to date.
    pub async fn open(path: &Path) -> AppResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(|e| crate::core::error::AppError::Database(e.to_string()))?
            .create_if_missing(true);
        Self::connect(opts, 4).await
    }

    /// A private in-memory database (tests).
    pub async fn open_in_memory() -> AppResult<Self> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:").map_err(|e| crate::core::error::AppError::Database(e.to_string()))?;
        // One connection: every pooled connection would otherwise get its own empty database.
        Self::connect(opts, 1).await
    }

    async fn connect(opts: SqliteConnectOptions, max: u32) -> AppResult<Self> {
        let opts = opts
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new().max_connections(max).connect_with(opts).await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    /// Applied migration versions, newest last (diagnostics).
    pub async fn schema_version(&self) -> AppResult<i64> {
        let v: Option<i64> = sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1").fetch_one(&self.pool).await?;
        Ok(v.unwrap_or(0))
    }
}

/// Milliseconds since the Unix epoch, the timestamp format of every table.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrations_create_the_schema_with_foreign_keys() {
        let db = Database::open_in_memory().await.unwrap();
        assert_eq!(db.schema_version().await.unwrap(), 1);
        let tables: Vec<String> = sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE '\\_%' ESCAPE '\\' AND name NOT LIKE 'sqlite%' ORDER BY name")
            .fetch_all(&db.pool)
            .await
            .unwrap();
        for t in [
            "collections", "cookies", "environment_variables", "environments", "folders", "grpc_services", "request_history", "requests",
            "response_metadata", "scripts", "settings", "test_results", "test_runs", "websocket_sessions", "workspaces",
        ] {
            assert!(tables.iter().any(|x| x == t), "missing table {t}: {tables:?}");
        }
        // Foreign keys are on: a collection needs an existing workspace, and deleting the
        // workspace removes its collections.
        let bad = sqlx::query("INSERT INTO collections (id, workspace_id, name, created_at, updated_at) VALUES ('c', 'nope', 'C', 0, 0)").execute(&db.pool).await;
        assert!(bad.is_err());
        sqlx::query("INSERT INTO workspaces (id, name, created_at, updated_at) VALUES ('w', 'W', 0, 0)").execute(&db.pool).await.unwrap();
        sqlx::query("INSERT INTO collections (id, workspace_id, name, created_at, updated_at) VALUES ('c', 'w', 'C', 0, 0)").execute(&db.pool).await.unwrap();
        sqlx::query("DELETE FROM workspaces WHERE id = 'w'").execute(&db.pool).await.unwrap();
        let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM collections").fetch_one(&db.pool).await.unwrap();
        assert_eq!(left, 0);
    }

    #[tokio::test]
    async fn secret_variables_cannot_hold_a_plaintext_value() {
        let db = Database::open_in_memory().await.unwrap();
        let r = sqlx::query("INSERT INTO environment_variables (id, scope, name, value, is_secret, secret_ref, created_at, updated_at) VALUES ('v', 'global', 'token', 'abc', 1, 'k', 0, 0)")
            .execute(&db.pool)
            .await;
        assert!(r.is_err(), "a secret with a plaintext value must be rejected");
    }

    #[tokio::test]
    async fn reopening_a_file_keeps_data_and_does_not_rerun_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("enpost.db");
        {
            let db = Database::open(&path).await.unwrap();
            sqlx::query("INSERT INTO settings (key, value, updated_at) VALUES ('theme', '\"light\"', 1)").execute(&db.pool).await.unwrap();
            db.pool.close().await;
        }
        let db = Database::open(&path).await.unwrap();
        let v: String = sqlx::query_scalar("SELECT value FROM settings WHERE key = 'theme'").fetch_one(&db.pool).await.unwrap();
        assert_eq!(v, "\"light\"");
    }
}
