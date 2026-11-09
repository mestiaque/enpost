use crate::core::error::AppResult;
use crate::core::state::AppState;
use crate::modules::settings::{Settings, SettingsPatch};
use tauri::State;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    Ok(state.settings.get().await)
}

#[tauri::command]
pub async fn update_settings(state: State<'_, AppState>, patch: SettingsPatch) -> AppResult<Settings> {
    state.settings.update(&patch).await
}

#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    state.settings.reset().await
}
