//! SQL for workspaces, collections, folders and saved requests.

use crate::core::error::{AppError, AppResult};
use crate::database::now_ms;
use crate::modules::collections::dto::*;
use sqlx::{Sqlite, SqlitePool, Transaction};

// ---------- workspaces ----------

pub async fn list_workspaces(pool: &SqlitePool) -> AppResult<Vec<Workspace>> {
    Ok(sqlx::query_as("SELECT id, name, description FROM workspaces ORDER BY sort_order, created_at").fetch_all(pool).await?)
}

pub async fn insert_workspace(pool: &SqlitePool, id: &str, name: &str) -> AppResult<()> {
    let now = now_ms();
    sqlx::query("INSERT INTO workspaces (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, (SELECT COALESCE(MAX(sort_order), -1) + 1 FROM workspaces), ?3, ?3)")
        .bind(id)
        .bind(name)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(())
}

// ---------- tree ----------

pub async fn tree(pool: &SqlitePool, workspace_id: &str) -> AppResult<WorkspaceTree> {
    let collections = sqlx::query_as("SELECT id, name, description FROM collections WHERE workspace_id = ?1 ORDER BY sort_order, created_at")
        .bind(workspace_id)
        .fetch_all(pool)
        .await?;
    let folders = sqlx::query_as(
        "SELECT f.id, f.collection_id, f.parent_id, f.name FROM folders f JOIN collections c ON c.id = f.collection_id
         WHERE c.workspace_id = ?1 ORDER BY f.sort_order, f.created_at",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;
    let requests = sqlx::query_as(
        "SELECT r.id, r.collection_id, r.folder_id, r.name, r.method, r.url, r.favorite FROM requests r JOIN collections c ON c.id = r.collection_id
         WHERE c.workspace_id = ?1 ORDER BY r.sort_order, r.created_at",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;
    Ok(WorkspaceTree { workspace_id: workspace_id.to_string(), collections, folders, requests })
}

// ---------- create ----------

pub async fn insert_collection(pool: &SqlitePool, id: &str, workspace_id: &str, name: &str) -> AppResult<()> {
    let now = now_ms();
    sqlx::query(
        "INSERT INTO collections (id, workspace_id, name, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(sort_order), -1) + 1 FROM collections WHERE workspace_id = ?2), ?4, ?4)",
    )
    .bind(id)
    .bind(workspace_id)
    .bind(name)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_folder(pool: &SqlitePool, id: &str, collection_id: &str, parent_id: Option<&str>, name: &str) -> AppResult<()> {
    let now = now_ms();
    sqlx::query(
        "INSERT INTO folders (id, collection_id, parent_id, name, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, (SELECT COALESCE(MAX(sort_order), -1) + 1 FROM folders WHERE collection_id = ?2 AND parent_id IS ?3), ?5, ?5)",
    )
    .bind(id)
    .bind(collection_id)
    .bind(parent_id)
    .bind(name)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_request(pool: &SqlitePool, id: &str, collection_id: &str, folder_id: Option<&str>, name: &str, method: &str, url: &str, definition: &str, after: Option<i64>) -> AppResult<()> {
    let now = now_ms();
    let mut tx = pool.begin().await?;
    let order: i64 = match after {
        // Right after a sibling (duplicate): shift the ones behind it.
        Some(pos) => {
            sqlx::query("UPDATE requests SET sort_order = sort_order + 1 WHERE collection_id = ?1 AND folder_id IS ?2 AND sort_order > ?3")
                .bind(collection_id)
                .bind(folder_id)
                .bind(pos)
                .execute(&mut *tx)
                .await?;
            pos + 1
        }
        None => sqlx::query_scalar("SELECT COALESCE(MAX(sort_order), -1) + 1 FROM requests WHERE collection_id = ?1 AND folder_id IS ?2")
            .bind(collection_id)
            .bind(folder_id)
            .fetch_one(&mut *tx)
            .await?,
    };
    sqlx::query(
        "INSERT INTO requests (id, collection_id, folder_id, name, method, url, definition, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
    )
    .bind(id)
    .bind(collection_id)
    .bind(folder_id)
    .bind(name)
    .bind(method)
    .bind(url)
    .bind(definition)
    .bind(order)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

// ---------- read ----------

#[derive(sqlx::FromRow)]
pub struct RequestRow {
    pub id: String,
    pub collection_id: String,
    pub folder_id: Option<String>,
    pub name: String,
    pub description: String,
    pub method: String,
    pub url: String,
    pub definition: String,
    pub favorite: bool,
    pub sort_order: i64,
    pub updated_at: i64,
}

pub async fn get_request(pool: &SqlitePool, id: &str) -> AppResult<RequestRow> {
    sqlx::query_as("SELECT id, collection_id, folder_id, name, description, method, url, definition, favorite, sort_order, updated_at FROM requests WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("request".into()))
}

/// The collection a folder belongs to.
pub async fn folder_collection(pool: &SqlitePool, folder_id: &str) -> AppResult<String> {
    sqlx::query_scalar("SELECT collection_id FROM folders WHERE id = ?1").bind(folder_id).fetch_optional(pool).await?.ok_or_else(|| AppError::NotFound("folder".into()))
}

pub async fn collection_exists(pool: &SqlitePool, id: &str) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM collections WHERE id = ?1").bind(id).fetch_one(pool).await? > 0)
}

pub async fn workspace_exists(pool: &SqlitePool, id: &str) -> AppResult<bool> {
    Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM workspaces WHERE id = ?1").bind(id).fetch_one(pool).await? > 0)
}

/// Ids of a folder and every folder below it.
pub async fn folder_subtree(pool: &SqlitePool, folder_id: &str) -> AppResult<Vec<String>> {
    Ok(sqlx::query_scalar(
        "WITH RECURSIVE sub(id) AS (SELECT ?1 UNION ALL SELECT f.id FROM folders f JOIN sub ON f.parent_id = sub.id) SELECT id FROM sub",
    )
    .bind(folder_id)
    .fetch_all(pool)
    .await?)
}

// ---------- update ----------

pub async fn update_request(pool: &SqlitePool, id: &str, name: &str, description: &str, method: &str, url: &str, definition: &str) -> AppResult<()> {
    let n = sqlx::query("UPDATE requests SET name = ?2, description = ?3, method = ?4, url = ?5, definition = ?6, updated_at = ?7 WHERE id = ?1")
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(method)
        .bind(url)
        .bind(definition)
        .bind(now_ms())
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("request".into()));
    }
    Ok(())
}

pub async fn rename(pool: &SqlitePool, kind: ItemKind, id: &str, name: &str) -> AppResult<()> {
    let sql = match kind {
        ItemKind::Collection => "UPDATE collections SET name = ?2, updated_at = ?3 WHERE id = ?1",
        ItemKind::Folder => "UPDATE folders SET name = ?2, updated_at = ?3 WHERE id = ?1",
        ItemKind::Request => "UPDATE requests SET name = ?2, updated_at = ?3 WHERE id = ?1",
    };
    let n = sqlx::query(sql).bind(id).bind(name).bind(now_ms()).execute(pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("item".into()));
    }
    Ok(())
}

pub async fn rename_workspace(pool: &SqlitePool, id: &str, name: &str) -> AppResult<()> {
    let n = sqlx::query("UPDATE workspaces SET name = ?2, updated_at = ?3 WHERE id = ?1").bind(id).bind(name).bind(now_ms()).execute(pool).await?.rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("workspace".into()));
    }
    Ok(())
}

pub async fn set_favorite(pool: &SqlitePool, id: &str, favorite: bool) -> AppResult<()> {
    sqlx::query("UPDATE requests SET favorite = ?2 WHERE id = ?1").bind(id).bind(favorite).execute(pool).await?;
    Ok(())
}

// ---------- delete ----------

pub async fn delete(pool: &SqlitePool, kind: ItemKind, id: &str) -> AppResult<()> {
    let sql = match kind {
        ItemKind::Collection => "DELETE FROM collections WHERE id = ?1",
        ItemKind::Folder => "DELETE FROM folders WHERE id = ?1",
        ItemKind::Request => "DELETE FROM requests WHERE id = ?1",
    };
    sqlx::query(sql).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete_workspace(pool: &SqlitePool, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM workspaces WHERE id = ?1").bind(id).execute(pool).await?;
    Ok(())
}

// ---------- move / reorder ----------

/// Ordered ids of the siblings an item is placed among (same kind, same parent).
async fn siblings(tx: &mut Transaction<'_, Sqlite>, kind: ItemKind, workspace_id: Option<&str>, collection_id: Option<&str>, folder_id: Option<&str>) -> AppResult<Vec<String>> {
    let ids = match kind {
        ItemKind::Collection => sqlx::query_scalar("SELECT id FROM collections WHERE workspace_id = ?1 ORDER BY sort_order, created_at").bind(workspace_id).fetch_all(&mut **tx).await?,
        ItemKind::Folder => sqlx::query_scalar("SELECT id FROM folders WHERE collection_id = ?1 AND parent_id IS ?2 ORDER BY sort_order, created_at")
            .bind(collection_id)
            .bind(folder_id)
            .fetch_all(&mut **tx)
            .await?,
        ItemKind::Request => sqlx::query_scalar("SELECT id FROM requests WHERE collection_id = ?1 AND folder_id IS ?2 ORDER BY sort_order, created_at")
            .bind(collection_id)
            .bind(folder_id)
            .fetch_all(&mut **tx)
            .await?,
    };
    Ok(ids)
}

async fn write_order(tx: &mut Transaction<'_, Sqlite>, kind: ItemKind, ids: &[String]) -> AppResult<()> {
    let sql = match kind {
        ItemKind::Collection => "UPDATE collections SET sort_order = ?2 WHERE id = ?1",
        ItemKind::Folder => "UPDATE folders SET sort_order = ?2 WHERE id = ?1",
        ItemKind::Request => "UPDATE requests SET sort_order = ?2 WHERE id = ?1",
    };
    for (i, id) in ids.iter().enumerate() {
        sqlx::query(sql).bind(id).bind(i as i64).execute(&mut **tx).await?;
    }
    Ok(())
}

/// Move an item to a parent (already validated by the service) and position, in one transaction.
/// `subtree` lists folders moving along with a folder that changes collection.
pub async fn move_item(pool: &SqlitePool, kind: ItemKind, id: &str, workspace_id: Option<&str>, collection_id: Option<&str>, folder_id: Option<&str>, index: u32, subtree: &[String]) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    match kind {
        ItemKind::Collection => {}
        ItemKind::Folder => {
            sqlx::query("UPDATE folders SET collection_id = ?2, parent_id = ?3, updated_at = ?4 WHERE id = ?1")
                .bind(id)
                .bind(collection_id)
                .bind(folder_id)
                .bind(now_ms())
                .execute(&mut *tx)
                .await?;
            // Everything inside follows the folder into its new collection.
            for f in subtree.iter().filter(|f| f.as_str() != id) {
                sqlx::query("UPDATE folders SET collection_id = ?2 WHERE id = ?1").bind(f).bind(collection_id).execute(&mut *tx).await?;
            }
            for f in subtree {
                sqlx::query("UPDATE requests SET collection_id = ?2 WHERE folder_id = ?1").bind(f).bind(collection_id).execute(&mut *tx).await?;
            }
        }
        ItemKind::Request => {
            sqlx::query("UPDATE requests SET collection_id = ?2, folder_id = ?3, updated_at = ?4 WHERE id = ?1")
                .bind(id)
                .bind(collection_id)
                .bind(folder_id)
                .bind(now_ms())
                .execute(&mut *tx)
                .await?;
        }
    }
    let mut ids: Vec<String> = siblings(&mut tx, kind, workspace_id, collection_id, folder_id).await?.into_iter().filter(|x| x != id).collect();
    let at = (index as usize).min(ids.len());
    ids.insert(at, id.to_string());
    write_order(&mut tx, kind, &ids).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn collection_workspace(pool: &SqlitePool, id: &str) -> AppResult<String> {
    sqlx::query_scalar("SELECT workspace_id FROM collections WHERE id = ?1").bind(id).fetch_optional(pool).await?.ok_or_else(|| AppError::NotFound("collection".into()))
}
