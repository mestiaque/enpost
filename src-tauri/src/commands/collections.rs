use crate::core::error::AppResult;
use crate::core::state::AppState;
use crate::modules::collections::dto::*;
use tauri::State;

#[tauri::command]
pub async fn list_workspaces(state: State<'_, AppState>) -> AppResult<Vec<Workspace>> {
    state.collections.workspaces().await
}

#[tauri::command]
pub async fn create_workspace(state: State<'_, AppState>, name: String) -> AppResult<Workspace> {
    state.collections.create_workspace(&name).await
}

#[tauri::command]
pub async fn rename_workspace(state: State<'_, AppState>, id: String, name: String) -> AppResult<()> {
    state.collections.rename_workspace(&id, &name).await
}

#[tauri::command]
pub async fn delete_workspace(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.collections.delete_workspace(&id).await
}

#[tauri::command]
pub async fn workspace_tree(state: State<'_, AppState>, workspace_id: String) -> AppResult<WorkspaceTree> {
    state.collections.tree(&workspace_id).await
}

#[tauri::command]
pub async fn create_collection(state: State<'_, AppState>, workspace_id: String, name: String) -> AppResult<String> {
    state.collections.create_collection(&workspace_id, &name).await
}

#[tauri::command]
pub async fn create_folder(state: State<'_, AppState>, collection_id: String, parent_id: Option<String>, name: String) -> AppResult<String> {
    state.collections.create_folder(&collection_id, parent_id.as_deref(), &name).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn create_request(
    state: State<'_, AppState>,
    collection_id: String,
    folder_id: Option<String>,
    name: String,
    method: String,
    url: String,
    definition: RequestDefinition,
) -> AppResult<SavedRequest> {
    state.collections.create_request(&collection_id, folder_id.as_deref(), &name, &method, &url, &definition).await
}

#[tauri::command]
pub async fn get_request(state: State<'_, AppState>, id: String) -> AppResult<SavedRequest> {
    state.collections.request(&id).await
}

#[tauri::command]
pub async fn save_request(state: State<'_, AppState>, id: String, patch: RequestPatch) -> AppResult<SavedRequest> {
    state.collections.save_request(&id, &patch).await
}

#[tauri::command]
pub async fn duplicate_request(state: State<'_, AppState>, id: String) -> AppResult<SavedRequest> {
    state.collections.duplicate_request(&id).await
}

#[tauri::command]
pub async fn rename_item(state: State<'_, AppState>, kind: ItemKind, id: String, name: String) -> AppResult<()> {
    state.collections.rename(kind, &id, &name).await
}

#[tauri::command]
pub async fn delete_item(state: State<'_, AppState>, kind: ItemKind, id: String) -> AppResult<()> {
    state.collections.delete(kind, &id).await
}

#[tauri::command]
pub async fn set_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> AppResult<()> {
    state.collections.set_favorite(&id, favorite).await
}

#[tauri::command]
pub async fn move_item(state: State<'_, AppState>, kind: ItemKind, id: String, target: MoveTarget) -> AppResult<()> {
    state.collections.move_item(kind, &id, &target).await
}
