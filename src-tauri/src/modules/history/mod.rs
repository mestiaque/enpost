//! Request history: every sent request is recorded, redacted, and pruned by age.

use crate::core::error::AppResult;
use crate::database::repositories::history_repo::{self, HistoryEntry, NewHistory};
use crate::database::{now_ms, Database};
use crate::modules::http::{HttpRequest, HttpResponse, RequestBody};
use crate::security::redact;
use serde_json::json;

const DAY_MS: i64 = 86_400_000;

/// What is kept of a request: method, redacted URL and headers, and the body *type* only
/// (bodies often carry passwords or tokens).
pub fn snapshot(req: &HttpRequest) -> serde_json::Value {
    let headers: Vec<(String, String)> = req.headers.iter().map(|h| (h.name.clone(), h.value.clone())).collect();
    let body_type = match &req.body {
        RequestBody::None => "none",
        RequestBody::Raw { content_type, .. } if content_type.contains("json") => "json",
        RequestBody::Raw { content_type, .. } if content_type.contains("xml") => "xml",
        RequestBody::Raw { .. } => "text",
        RequestBody::UrlEncoded { .. } => "urlencoded",
        RequestBody::FormData { .. } => "form-data",
        RequestBody::Binary { .. } => "binary",
    };
    json!({
        "method": req.method,
        "url": redact::url(&req.url),
        "headers": redact::headers(&headers).into_iter().map(|(name, value)| json!({ "name": name, "value": value })).collect::<Vec<_>>(),
        "bodyType": body_type,
    })
}

pub async fn record(db: &Database, history_id: &str, req: &HttpRequest, sent_at: i64, result: Result<&HttpResponse, &str>) -> AppResult<()> {
    let snap = snapshot(req).to_string();
    let url = redact::url(&req.url);
    let (status, status_text, duration, size, headers, body_path, error) = match result {
        Ok(r) => {
            let pairs: Vec<(String, String)> = r.headers.iter().map(|h| (h.name.clone(), h.value.clone())).collect();
            let hs = redact::headers(&pairs).into_iter().map(|(name, value)| json!({ "name": name, "value": value })).collect::<Vec<_>>();
            (Some(r.status as i64), Some(r.status_text.as_str()), Some(r.timing.total_ms as i64), Some(r.body.bytes as i64), serde_json::Value::from(hs).to_string(), r.body.file_path.as_deref(), None)
        }
        Err(e) => (None, None, None, None, "[]".to_string(), None, Some(e)),
    };
    history_repo::insert(
        &db.pool,
        &NewHistory { id: history_id, method: &req.method, url: &url, snapshot: &snap, sent_at, status, status_text, duration_ms: duration, size_bytes: size, headers: &headers, body_path, error },
    )
    .await
}

pub async fn list(db: &Database, limit: i64) -> AppResult<Vec<HistoryEntry>> {
    history_repo::list(&db.pool, limit.clamp(1, 1000)).await
}

async fn remove_files(files: Vec<String>) {
    for f in files {
        let _ = tokio::fs::remove_file(f).await;
    }
}

/// Apply the retention setting (0 = keep forever).
pub async fn prune(db: &Database, retention_days: u32) -> AppResult<()> {
    if retention_days == 0 {
        return Ok(());
    }
    let files = history_repo::delete_older_than(&db.pool, now_ms() - retention_days as i64 * DAY_MS).await?;
    remove_files(files).await;
    Ok(())
}

pub async fn clear(db: &Database) -> AppResult<()> {
    let files = history_repo::delete_all(&db.pool).await?;
    remove_files(files).await;
    Ok(())
}
