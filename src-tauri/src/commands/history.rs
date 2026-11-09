use crate::core::error::AppResult;
use crate::core::state::AppState;
use crate::database::repositories::history_repo::HistoryEntry;
use crate::modules::history;
use tauri::State;

#[tauri::command]
pub async fn list_history(state: State<'_, AppState>, limit: Option<i64>) -> AppResult<Vec<HistoryEntry>> {
    history::list(&state.db, limit.unwrap_or(200)).await
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> AppResult<()> {
    history::clear(&state.db).await
}
