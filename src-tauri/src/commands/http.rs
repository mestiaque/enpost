use crate::core::error::{AppError, AppResult};
use crate::core::state::AppState;
use crate::database::now_ms;
use crate::modules::history;
use crate::modules::http::{HttpProgress, HttpRequest, HttpResponse, ProgressSink};
use std::path::Path;
use tauri::{AppHandle, Emitter, State};

pub const EV_PROGRESS: &str = "http-progress";

struct EmitProgress(AppHandle);
impl ProgressSink for EmitProgress {
    fn progress(&self, p: HttpProgress) {
        let _ = self.0.emit(EV_PROGRESS, p);
    }
}

/// Send a request and record it in the history (failed sends are recorded too).
#[tauri::command]
pub async fn send_request(app: AppHandle, state: State<'_, AppState>, request: HttpRequest) -> AppResult<HttpResponse> {
    let settings = state.settings.get().await;
    let history_id = uuid::Uuid::new_v4().to_string();
    let sent_at = now_ms();
    let result = state.http.send(&request, &settings.proxy_url, &history_id, &EmitProgress(app)).await;
    // A cancelled request is not history.
    if !matches!(result, Err(AppError::Cancelled)) {
        let msg = result.as_ref().err().map(|e| e.to_string());
        let rec = history::record(&state.db, &history_id, &request, sent_at, result.as_ref().map_err(|_| msg.as_deref().unwrap_or(""))).await;
        if let Err(e) = rec {
            log::warn!("could not record history: {e}");
        }
        let _ = history::prune(&state.db, settings.history_retention_days).await;
    }
    result
}

#[tauri::command]
pub fn cancel_request(state: State<'_, AppState>, request_id: String) -> bool {
    state.http.cancel(&request_id)
}

/// Copy a stored response body to a file the user chose in the save dialog.
#[tauri::command]
pub async fn save_response_body(state: State<'_, AppState>, history_id: String, destination: String) -> AppResult<()> {
    let dest = Path::new(&destination);
    if !dest.is_absolute() || dest.is_dir() {
        return Err(AppError::Validation("choose a file to save to".into()));
    }
    // Only bodies ENPost stored itself can be copied: the path comes from the database.
    let src = crate::database::repositories::history_repo::body_path(&state.db.pool, &history_id)
        .await?
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| state.http.body_path(&history_id));
    if !src.starts_with(&state.bodies_dir) || !src.is_file() {
        return Err(AppError::NotFound("the response body is no longer stored".into()));
    }
    tokio::fs::copy(&src, dest).await?;
    Ok(())
}
