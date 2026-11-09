use crate::core::error::AppResult;
use crate::core::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, State};
use ts_rs::TS;

/// Version and environment facts for the About page and the status bar.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    /// "linux" | "windows" | "macos"
    pub platform: String,
    pub arch: String,
    pub data_dir: String,
    #[ts(type = "number")]
    pub schema_version: i64,
}

#[tauri::command]
pub async fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let info = app.package_info();
    Ok(AppInfo {
        name: info.name.clone(),
        version: info.version.to_string(),
        platform: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        data_dir: state.data_dir.display().to_string(),
        schema_version: state.db.schema_version().await?,
    })
}
