//! Collections service: workspaces, collections, nested folders and saved requests.
//! Rules live here (names, parents that match, no folder inside itself); SQL lives in
//! `collections_repo`.

pub mod dto;

use crate::core::error::{AppError, AppResult};
use crate::database::repositories::collections_repo as repo;
use crate::database::Database;
use dto::*;

pub const DEFAULT_WORKSPACE: &str = "My Workspace";

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Trimmed, non-empty, at most 200 characters.
pub fn clean_name(name: &str) -> AppResult<String> {
    let n = name.trim();
    if n.is_empty() {
        return Err(AppError::Validation("Name cannot be empty".into()));
    }
    if n.chars().count() > 200 {
        return Err(AppError::Validation("Name is too long (200 characters at most)".into()));
    }
    Ok(n.to_string())
}

fn clean_method(m: &str) -> AppResult<String> {
    let m = m.trim().to_ascii_uppercase();
    if m.is_empty() || m.len() > 20 || !m.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err(AppError::Validation(format!("\"{m}\" is not a method")));
    }
    Ok(m)
}

pub struct CollectionsService {
    db: Database,
}

impl CollectionsService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn pool(&self) -> &sqlx::SqlitePool {
        &self.db.pool
    }

    /// Workspaces; creates "My Workspace" on the first run.
    pub async fn workspaces(&self) -> AppResult<Vec<Workspace>> {
        let list = repo::list_workspaces(self.pool()).await?;
        if !list.is_empty() {
            return Ok(list);
        }
        repo::insert_workspace(self.pool(), &new_id(), DEFAULT_WORKSPACE).await?;
        repo::list_workspaces(self.pool()).await
    }

    pub async fn create_workspace(&self, name: &str) -> AppResult<Workspace> {
        let id = new_id();
        repo::insert_workspace(self.pool(), &id, &clean_name(name)?).await?;
        Ok(repo::list_workspaces(self.pool()).await?.into_iter().find(|w| w.id == id).expect("just inserted"))
    }

    pub async fn rename_workspace(&self, id: &str, name: &str) -> AppResult<()> {
        repo::rename_workspace(self.pool(), id, &clean_name(name)?).await
    }

    /// Deleting the last workspace is refused (the app always has one).
    pub async fn delete_workspace(&self, id: &str) -> AppResult<()> {
        if repo::list_workspaces(self.pool()).await?.len() <= 1 {
            return Err(AppError::Validation("The last workspace cannot be deleted".into()));
        }
        repo::delete_workspace(self.pool(), id).await
    }

    pub async fn tree(&self, workspace_id: &str) -> AppResult<WorkspaceTree> {
        if !repo::workspace_exists(self.pool(), workspace_id).await? {
            return Err(AppError::NotFound("workspace".into()));
        }
        repo::tree(self.pool(), workspace_id).await
    }

    pub async fn create_collection(&self, workspace_id: &str, name: &str) -> AppResult<String> {
        if !repo::workspace_exists(self.pool(), workspace_id).await? {
            return Err(AppError::NotFound("workspace".into()));
        }
        let id = new_id();
        repo::insert_collection(self.pool(), &id, workspace_id, &clean_name(name)?).await?;
        Ok(id)
    }

    /// `folder_id` given: it must belong to `collection_id`.
    async fn check_parent(&self, collection_id: &str, folder_id: Option<&str>) -> AppResult<()> {
        if !repo::collection_exists(self.pool(), collection_id).await? {
            return Err(AppError::NotFound("collection".into()));
        }
        if let Some(f) = folder_id {
            if repo::folder_collection(self.pool(), f).await? != collection_id {
                return Err(AppError::Validation("That folder is in another collection".into()));
            }
        }
        Ok(())
    }

    pub async fn create_folder(&self, collection_id: &str, parent_id: Option<&str>, name: &str) -> AppResult<String> {
        self.check_parent(collection_id, parent_id).await?;
        let id = new_id();
        repo::insert_folder(self.pool(), &id, collection_id, parent_id, &clean_name(name)?).await?;
        Ok(id)
    }

    pub async fn create_request(&self, collection_id: &str, folder_id: Option<&str>, name: &str, method: &str, url: &str, definition: &RequestDefinition) -> AppResult<SavedRequest> {
        self.check_parent(collection_id, folder_id).await?;
        let id = new_id();
        repo::insert_request(self.pool(), &id, collection_id, folder_id, &clean_name(name)?, &clean_method(method)?, url, &serde_json::to_string(definition)?, None).await?;
        self.request(&id).await
    }

    pub async fn request(&self, id: &str) -> AppResult<SavedRequest> {
        let r = repo::get_request(self.pool(), id).await?;
        Ok(SavedRequest {
            id: r.id,
            collection_id: r.collection_id,
            folder_id: r.folder_id,
            name: r.name,
            description: r.description,
            method: r.method,
            url: r.url,
            // A definition written by an older version (or damaged) opens as an empty one.
            definition: serde_json::from_str(&r.definition).unwrap_or_default(),
            favorite: r.favorite,
            updated_at: r.updated_at,
        })
    }

    pub async fn save_request(&self, id: &str, patch: &RequestPatch) -> AppResult<SavedRequest> {
        let cur = self.request(id).await?;
        let name = match &patch.name {
            Some(n) => clean_name(n)?,
            None => cur.name,
        };
        let method = match &patch.method {
            Some(m) => clean_method(m)?,
            None => cur.method,
        };
        let definition = patch.definition.as_ref().unwrap_or(&cur.definition);
        repo::update_request(
            self.pool(),
            id,
            &name,
            patch.description.as_deref().unwrap_or(&cur.description),
            &method,
            patch.url.as_deref().unwrap_or(&cur.url),
            &serde_json::to_string(definition)?,
        )
        .await?;
        self.request(id).await
    }

    pub async fn duplicate_request(&self, id: &str) -> AppResult<SavedRequest> {
        let row = repo::get_request(self.pool(), id).await?;
        let copy = new_id();
        let name: String = format!("{} copy", row.name).chars().take(200).collect();
        repo::insert_request(self.pool(), &copy, &row.collection_id, row.folder_id.as_deref(), &name, &row.method, &row.url, &row.definition, Some(row.sort_order)).await?;
        self.request(&copy).await
    }

    pub async fn rename(&self, kind: ItemKind, id: &str, name: &str) -> AppResult<()> {
        repo::rename(self.pool(), kind, id, &clean_name(name)?).await
    }

    pub async fn delete(&self, kind: ItemKind, id: &str) -> AppResult<()> {
        repo::delete(self.pool(), kind, id).await
    }

    pub async fn set_favorite(&self, id: &str, favorite: bool) -> AppResult<()> {
        repo::set_favorite(self.pool(), id, favorite).await
    }

    /// Drag and drop: reorder, or move into another folder or collection.
    pub async fn move_item(&self, kind: ItemKind, id: &str, target: &MoveTarget) -> AppResult<()> {
        match kind {
            ItemKind::Collection => {
                let ws = repo::collection_workspace(self.pool(), id).await?;
                repo::move_item(self.pool(), kind, id, Some(&ws), None, None, target.index, &[]).await
            }
            ItemKind::Folder | ItemKind::Request => {
                // Dropping on a folder implies its collection.
                let collection = match (&target.folder_id, &target.collection_id) {
                    (Some(f), _) => repo::folder_collection(self.pool(), f).await?,
                    (None, Some(c)) => c.clone(),
                    (None, None) => return Err(AppError::Validation("Drop it into a collection or folder".into())),
                };
                self.check_parent(&collection, target.folder_id.as_deref()).await?;
                let subtree = if kind == ItemKind::Folder {
                    let sub = repo::folder_subtree(self.pool(), id).await?;
                    if target.folder_id.as_ref().is_some_and(|f| sub.contains(f)) {
                        return Err(AppError::Validation("A folder cannot be moved into itself".into()));
                    }
                    sub
                } else {
                    vec![]
                };
                repo::move_item(self.pool(), kind, id, None, Some(&collection), target.folder_id.as_deref(), target.index, &subtree).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn svc() -> (CollectionsService, String) {
        let db = Database::open_in_memory().await.unwrap();
        let s = CollectionsService::new(db);
        let ws = s.workspaces().await.unwrap()[0].id.clone();
        (s, ws)
    }

    fn def() -> RequestDefinition {
        RequestDefinition { body_type: "json".into(), body: "{}".into(), headers: vec![KeyValueRow { id: "h".into(), key: "Accept".into(), value: "x".into(), enabled: true, description: None }], ..Default::default() }
    }

    #[tokio::test]
    async fn first_run_creates_a_workspace_and_the_last_one_stays() {
        let (s, ws) = svc().await;
        let list = s.workspaces().await.unwrap();
        assert_eq!((list.len(), list[0].name.as_str()), (1, DEFAULT_WORKSPACE));
        assert!(s.delete_workspace(&ws).await.is_err());
        let w2 = s.create_workspace("  Team  ").await.unwrap();
        assert_eq!(w2.name, "Team");
        s.delete_workspace(&w2.id).await.unwrap();
        assert!(s.create_workspace("   ").await.is_err());
    }

    #[tokio::test]
    async fn builds_a_nested_tree_and_saves_requests() {
        let (s, ws) = svc().await;
        let c = s.create_collection(&ws, "Users API").await.unwrap();
        let f = s.create_folder(&c, None, "Admin").await.unwrap();
        let sub = s.create_folder(&c, Some(&f), "Roles").await.unwrap();
        let r = s.create_request(&c, Some(&sub), "List roles", "get", "{{base}}/roles", &def()).await.unwrap();
        assert_eq!((r.method.as_str(), r.definition.body_type.as_str()), ("GET", "json"));

        let t = s.tree(&ws).await.unwrap();
        assert_eq!(t.collections.len(), 1);
        assert_eq!(t.folders.iter().find(|x| x.id == sub).unwrap().parent_id.as_deref(), Some(f.as_str()));
        assert_eq!(t.requests[0].folder_id.as_deref(), Some(sub.as_str()));

        let saved = s.save_request(&r.id, &RequestPatch { url: Some("https://x/roles".into()), name: Some("Roles".into()), ..Default::default() }).await.unwrap();
        assert_eq!((saved.url.as_str(), saved.name.as_str(), saved.definition.body.as_str()), ("https://x/roles", "Roles", "{}"));
        assert!(s.save_request(&r.id, &RequestPatch { method: Some("G E T".into()), ..Default::default() }).await.is_err());

        // A folder from another collection is refused as a parent.
        let other = s.create_collection(&ws, "Other").await.unwrap();
        assert!(s.create_request(&other, Some(&f), "x", "GET", "", &def()).await.is_err());

        // Deleting the top folder removes everything inside.
        s.delete(ItemKind::Folder, &f).await.unwrap();
        let t = s.tree(&ws).await.unwrap();
        assert!(t.folders.is_empty() && t.requests.is_empty());
    }

    #[tokio::test]
    async fn duplicate_rename_favorite() {
        let (s, ws) = svc().await;
        let c = s.create_collection(&ws, "C").await.unwrap();
        let a = s.create_request(&c, None, "A", "GET", "/a", &def()).await.unwrap();
        let b = s.create_request(&c, None, "B", "GET", "/b", &def()).await.unwrap();
        let copy = s.duplicate_request(&a.id).await.unwrap();
        assert_eq!(copy.name, "A copy");
        assert_eq!(copy.definition, a.definition);
        let names: Vec<String> = s.tree(&ws).await.unwrap().requests.into_iter().map(|r| r.name).collect();
        assert_eq!(names, ["A", "A copy", "B"], "the copy goes right after the original");
        s.rename(ItemKind::Request, &b.id, "Bee").await.unwrap();
        assert!(s.rename(ItemKind::Request, &b.id, "").await.is_err());
        s.set_favorite(&b.id, true).await.unwrap();
        assert!(s.request(&b.id).await.unwrap().favorite);
    }

    #[tokio::test]
    async fn moves_and_reorders() {
        let (s, ws) = svc().await;
        let c1 = s.create_collection(&ws, "One").await.unwrap();
        let c2 = s.create_collection(&ws, "Two").await.unwrap();
        let f = s.create_folder(&c1, None, "F").await.unwrap();
        let inner = s.create_folder(&c1, Some(&f), "Inner").await.unwrap();
        let r1 = s.create_request(&c1, None, "r1", "GET", "", &def()).await.unwrap();
        let r2 = s.create_request(&c1, None, "r2", "GET", "", &def()).await.unwrap();
        let deep = s.create_request(&c1, Some(&inner), "deep", "GET", "", &def()).await.unwrap();

        // Reorder within the collection root.
        s.move_item(ItemKind::Request, &r2.id, &MoveTarget { collection_id: Some(c1.clone()), folder_id: None, index: 0 }).await.unwrap();
        let root: Vec<String> = s.tree(&ws).await.unwrap().requests.into_iter().filter(|r| r.folder_id.is_none()).map(|r| r.name).collect();
        assert_eq!(root, ["r2", "r1"]);

        // Into a folder (collection taken from the folder).
        s.move_item(ItemKind::Request, &r1.id, &MoveTarget { collection_id: None, folder_id: Some(f.clone()), index: 5 }).await.unwrap();
        assert_eq!(s.request(&r1.id).await.unwrap().folder_id.as_deref(), Some(f.as_str()));

        // A folder cannot go into itself or below itself.
        assert!(s.move_item(ItemKind::Folder, &f, &MoveTarget { collection_id: None, folder_id: Some(inner.clone()), index: 0 }).await.is_err());

        // Moving a folder to another collection takes its subfolders and requests along.
        s.move_item(ItemKind::Folder, &f, &MoveTarget { collection_id: Some(c2.clone()), folder_id: None, index: 0 }).await.unwrap();
        let t = s.tree(&ws).await.unwrap();
        assert!(t.folders.iter().all(|x| x.collection_id == c2));
        assert_eq!(s.request(&deep.id).await.unwrap().collection_id, c2);
        assert_eq!(s.request(&r1.id).await.unwrap().collection_id, c2);

        // Collections reorder within the workspace.
        s.move_item(ItemKind::Collection, &c2, &MoveTarget { collection_id: None, folder_id: None, index: 0 }).await.unwrap();
        assert_eq!(s.tree(&ws).await.unwrap().collections[0].id, c2);
    }
}
