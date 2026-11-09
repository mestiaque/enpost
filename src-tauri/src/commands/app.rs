use crate::core::error::{AppError, AppResult};
use crate::core::state::AppState;
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use ts_rs::TS;

pub const WEBSITE: &str = "https://mestiaque.com";
pub const EMAIL: &str = "info@mestiaque.com";

/// Version and environment facts for the About page, the status bar and bug reports.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    /// "debug" | "release"
    pub build: String,
    pub tauri_version: String,
    /// WebKitGTK on Linux, WebView2 on Windows; empty when it cannot be read.
    pub webview_version: String,
    /// "linux" | "windows" | "macos"
    pub platform: String,
    pub arch: String,
    pub data_dir: String,
    pub logs_dir: String,
    #[ts(type = "number")]
    pub schema_version: i64,
    pub website: String,
    pub email: String,
}

#[tauri::command]
pub async fn app_info(app: AppHandle, state: State<'_, AppState>) -> AppResult<AppInfo> {
    let info = app.package_info();
    Ok(AppInfo {
        name: info.name.clone(),
        version: info.version.to_string(),
        build: if cfg!(debug_assertions) { "debug" } else { "release" }.into(),
        tauri_version: tauri::VERSION.into(),
        webview_version: tauri::webview_version().unwrap_or_default(),
        platform: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        data_dir: state.data_dir.display().to_string(),
        logs_dir: app.path().app_log_dir().map(|p| p.display().to_string()).unwrap_or_default(),
        schema_version: state.db.schema_version().await?,
        website: WEBSITE.into(),
        email: EMAIL.into(),
    })
}

/// Open one of ENPost's own folders in the file manager (fixed choices, no arbitrary paths).
#[tauri::command]
pub fn open_app_folder(app: AppHandle, state: State<'_, AppState>, which: String) -> AppResult<()> {
    let path = match which.as_str() {
        "data" => state.data_dir.clone(),
        "logs" => app.path().app_log_dir().map_err(|e| AppError::Internal(e.to_string()))?,
        _ => return Err(AppError::Validation(format!("unknown folder {which}"))),
    };
    std::fs::create_dir_all(&path)?;
    app.opener().open_path(path.display().to_string(), None::<&str>).map_err(|e| AppError::Internal(e.to_string()))
}

/// Open the developer's website or a new e-mail (fixed URLs only).
#[tauri::command]
pub fn open_link(app: AppHandle, which: String) -> AppResult<()> {
    let url = match which.as_str() {
        "website" => WEBSITE.to_string(),
        "email" => format!("mailto:{EMAIL}?subject=ENPost"),
        _ => return Err(AppError::Validation(format!("unknown link {which}"))),
    };
    app.opener().open_url(url, None::<&str>).map_err(|e| AppError::Internal(e.to_string()))
}
