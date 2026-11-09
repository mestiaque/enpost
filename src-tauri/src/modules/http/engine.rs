//! Executes HTTP requests with reqwest. The webview never makes API calls itself (CSP), so
//! this is the only place requests leave the app.
//!
//! * every request has an id; `cancel` stops it at any point (connect, wait, download);
//! * the body streams to `<data>/bodies/<history id>` while the first part is kept for the
//!   viewer, so a 1 GB response never sits in memory or in the UI;
//! * errors are classified (timeout, connect, TLS, redirect…) for a clear message.

use super::body::build_body;
use super::cookies::parse_set_cookie;
use super::dto::*;
use crate::core::error::{AppError, AppResult};
use base64::Engine as _;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use reqwest::{redirect, Method, Url};
use std::collections::HashMap;
use std::error::Error as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

/// Text kept in memory and sent to the viewer; the rest stays on disk.
pub const PREVIEW_LIMIT: usize = 2 * 1024 * 1024;
/// Images up to this size are previewed inline.
pub const IMAGE_PREVIEW_LIMIT: usize = 8 * 1024 * 1024;

pub trait ProgressSink: Send + Sync {
    fn progress(&self, p: HttpProgress);
}

/// Receives no progress (tests, callers that don't need it).
pub struct NoProgress;
impl ProgressSink for NoProgress {
    fn progress(&self, _: HttpProgress) {}
}

pub struct HttpEngine {
    bodies_dir: PathBuf,
    inflight: Mutex<HashMap<String, CancellationToken>>,
}

/// Removes the request from the in-flight table however `send` ends.
struct InflightGuard<'a> {
    engine: &'a HttpEngine,
    id: String,
}
impl Drop for InflightGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut m) = self.engine.inflight.lock() {
            m.remove(&self.id);
        }
    }
}

impl HttpEngine {
    pub fn new(bodies_dir: PathBuf) -> Self {
        Self { bodies_dir, inflight: Mutex::new(HashMap::new()) }
    }

    pub fn body_path(&self, history_id: &str) -> PathBuf {
        self.bodies_dir.join(format!("{history_id}.body"))
    }

    /// Stop a running request. Returns false when it is not (or no longer) running.
    pub fn cancel(&self, request_id: &str) -> bool {
        match self.inflight.lock().ok().and_then(|m| m.get(request_id).cloned()) {
            Some(t) => {
                t.cancel();
                true
            }
            None => false,
        }
    }

    pub fn is_running(&self, request_id: &str) -> bool {
        self.inflight.lock().map(|m| m.contains_key(request_id)).unwrap_or(false)
    }

    /// Send `req`. `proxy` is the user's proxy setting ("" = system proxy variables).
    pub async fn send(&self, req: &HttpRequest, proxy: &str, history_id: &str, sink: &dyn ProgressSink) -> AppResult<HttpResponse> {
        let token = CancellationToken::new();
        {
            let mut m = self.inflight.lock().map_err(|_| AppError::Internal("request table poisoned".into()))?;
            if m.contains_key(&req.request_id) {
                return Err(AppError::InvalidRequest("a request with this id is already running".into()));
            }
            m.insert(req.request_id.clone(), token.clone());
        }
        let _guard = InflightGuard { engine: self, id: req.request_id.clone() };
        tokio::select! {
            biased;
            _ = token.cancelled() => {
                let _ = tokio::fs::remove_file(self.body_path(history_id)).await;
                Err(AppError::Cancelled)
            }
            r = self.execute(req, proxy, history_id, sink) => {
                if r.is_err() {
                    let _ = tokio::fs::remove_file(self.body_path(history_id)).await;
                }
                r
            }
        }
    }

    async fn execute(&self, req: &HttpRequest, proxy: &str, history_id: &str, sink: &dyn ProgressSink) -> AppResult<HttpResponse> {
        let method = parse_method(&req.method)?;
        let url = parse_url(&req.url)?;
        let opts = &req.options;
        let client = build_client(opts, proxy)?;

        let mut headers = HeaderMap::new();
        for h in &req.headers {
            if h.name.trim().is_empty() {
                continue;
            }
            let name = HeaderName::from_bytes(h.name.trim().as_bytes()).map_err(|_| AppError::InvalidRequest(format!("\"{}\" is not a valid header name", h.name)))?;
            let value = HeaderValue::from_str(&h.value).map_err(|_| AppError::InvalidRequest(format!("the value of header \"{}\" has characters a header cannot contain", h.name)))?;
            headers.append(name, value);
        }
        let mut rb = client.request(method, url).headers(headers.clone());
        if opts.timeout_ms > 0 {
            rb = rb.timeout(Duration::from_millis(opts.timeout_ms as u64));
        }
        rb = build_body(rb, &req.body, &headers).await?;

        sink.progress(HttpProgress { request_id: req.request_id.clone(), stage: "waiting".into(), received_bytes: 0, total_bytes: None });
        let started = Instant::now();
        let resp = rb.send().await.map_err(|e| classify(e, opts.timeout_ms))?;
        let ttfb = started.elapsed();

        let status = resp.status();
        let final_url = resp.url().to_string();
        let http_version = format!("{:?}", resp.version());
        let resp_headers: Vec<HeaderPair> = resp
            .headers()
            .iter()
            .map(|(k, v)| HeaderPair { name: k.as_str().to_string(), value: String::from_utf8_lossy(v.as_bytes()).into_owned() })
            .collect();
        let headers_bytes: u64 = resp_headers.iter().map(|h| (h.name.len() + h.value.len() + 4) as u64).sum();
        let cookies = resp_headers.iter().filter(|h| h.name.eq_ignore_ascii_case("set-cookie")).filter_map(|h| parse_set_cookie(&h.value)).collect();
        let content_type = resp.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).map(str::to_string);
        let total = resp.content_length();

        // Stream: everything to disk, the first PREVIEW/IMAGE limit to memory.
        tokio::fs::create_dir_all(&self.bodies_dir).await?;
        let path = self.body_path(history_id);
        let mut file = tokio::fs::File::create(&path).await?;
        let is_image = content_type.as_deref().is_some_and(|c| c.trim().to_ascii_lowercase().starts_with("image/"));
        let keep = if is_image { IMAGE_PREVIEW_LIMIT + 1 } else { PREVIEW_LIMIT };
        let mut head: Vec<u8> = Vec::new();
        let mut received: u64 = 0;
        let mut last_report = Instant::now();
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| classify(e, opts.timeout_ms))?;
            file.write_all(&chunk).await?;
            if head.len() < keep {
                let take = (keep - head.len()).min(chunk.len());
                head.extend_from_slice(&chunk[..take]);
            }
            received += chunk.len() as u64;
            if last_report.elapsed() >= Duration::from_millis(100) {
                last_report = Instant::now();
                sink.progress(HttpProgress { request_id: req.request_id.clone(), stage: "receiving".into(), received_bytes: received, total_bytes: total });
            }
        }
        file.flush().await?;
        drop(file);
        let total_time = started.elapsed();

        let body = response_body(&head, received, content_type.as_deref(), is_image, &path);
        Ok(HttpResponse {
            request_id: req.request_id.clone(),
            history_id: history_id.to_string(),
            url: final_url,
            status: status.as_u16(),
            status_text: status.canonical_reason().unwrap_or("").to_string(),
            http_version,
            headers: resp_headers,
            cookies,
            content_type,
            timing: Timing { ttfb_ms: ttfb.as_millis() as u64, download_ms: (total_time - ttfb).as_millis() as u64, total_ms: total_time.as_millis() as u64 },
            headers_bytes,
            body,
        })
    }
}

fn parse_method(m: &str) -> AppResult<Method> {
    let m = m.trim().to_ascii_uppercase();
    if m.is_empty() {
        return Err(AppError::InvalidRequest("no method".into()));
    }
    Method::from_bytes(m.as_bytes()).map_err(|_| AppError::InvalidRequest(format!("\"{m}\" is not a valid method")))
}

/// Accepts `host/path` (adds http://), refuses unresolved `{{variables}}` and other schemes.
pub fn parse_url(raw: &str) -> AppResult<Url> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::InvalidRequest("enter a URL".into()));
    }
    if let Some(start) = raw.find("{{") {
        let name: String = raw[start + 2..].chars().take_while(|c| *c != '}').collect();
        return Err(AppError::InvalidRequest(format!("the variable {{{{{name}}}}} has no value (environments arrive in Phase 3)")));
    }
    let with_scheme = if raw.contains("://") { raw.to_string() } else { format!("http://{raw}") };
    let url = Url::parse(&with_scheme).map_err(|e| AppError::InvalidRequest(format!("\"{raw}\" is not a valid URL ({e})")))?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        s => Err(AppError::InvalidRequest(format!("{s}:// URLs are not HTTP (WebSocket and others arrive in Phase 5)"))),
    }
}

fn build_client(opts: &RequestOptions, proxy: &str) -> AppResult<reqwest::Client> {
    let mut b = reqwest::Client::builder()
        .user_agent(concat!("ENPost/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(30))
        .redirect(if opts.follow_redirects { redirect::Policy::limited(opts.max_redirects as usize) } else { redirect::Policy::none() })
        .danger_accept_invalid_certs(!opts.verify_tls);
    let proxy = proxy.trim();
    if !proxy.is_empty() {
        b = b.proxy(reqwest::Proxy::all(proxy).map_err(|e| AppError::InvalidRequest(format!("proxy: {e}")))?);
    }
    b.build().map_err(|e| AppError::Internal(format!("HTTP client: {e}")))
}

/// The message of an error and its causes, e.g. "error sending request: … certificate expired".
fn chain(e: &reqwest::Error) -> String {
    let mut parts = vec![e.without_url_ref()];
    let mut src = e.source();
    while let Some(s) = src {
        let t = s.to_string();
        if !parts.iter().any(|p| p.contains(&t)) {
            parts.push(t);
        }
        src = s.source();
    }
    parts.join(": ")
}

trait WithoutUrl {
    fn without_url_ref(&self) -> String;
}
impl WithoutUrl for reqwest::Error {
    fn without_url_ref(&self) -> String {
        // The URL may hold secrets (tokens in the query); the UI already shows it.
        let s = self.to_string();
        match self.url() {
            Some(u) => s.replace(&format!(" for url ({u})"), "").replace(&format!(" ({u})"), ""),
            None => s,
        }
    }
}

pub fn classify(e: reqwest::Error, timeout_ms: u32) -> AppError {
    let msg = chain(&e);
    let lower = msg.to_ascii_lowercase();
    if e.is_timeout() {
        AppError::Timeout(timeout_ms as u64)
    } else if e.is_redirect() {
        AppError::Redirect(msg)
    } else if lower.contains("certificate") || lower.contains("tls") || lower.contains("handshake") || lower.contains("unknownissuer") {
        AppError::Tls(msg)
    } else if e.is_connect() {
        AppError::Connect(msg)
    } else if e.is_builder() {
        AppError::InvalidRequest(msg)
    } else {
        AppError::Network(msg)
    }
}

fn is_textual(content_type: Option<&str>, head: &[u8]) -> bool {
    match content_type.map(|c| c.split(';').next().unwrap_or("").trim().to_ascii_lowercase()) {
        Some(ct) if !ct.is_empty() => {
            ct.starts_with("text/")
                || ct.ends_with("+json")
                || ct.ends_with("+xml")
                || matches!(ct.as_str(), "application/json" | "application/xml" | "application/javascript" | "application/ecmascript" | "application/x-www-form-urlencoded" | "application/graphql" | "application/x-ndjson" | "image/svg+xml")
        }
        // No content type: sniff. Text has no NUL bytes in its first KB.
        _ => !head.iter().take(1024).any(|b| *b == 0),
    }
}

fn response_body(head: &[u8], bytes: u64, content_type: Option<&str>, is_image: bool, path: &Path) -> ResponseBody {
    let file_path = Some(path.display().to_string());
    if bytes == 0 {
        return ResponseBody { kind: BodyKind::Empty, text: None, truncated: false, image_data_url: None, file_path, bytes };
    }
    let image_data_url = (is_image && (bytes as usize) <= IMAGE_PREVIEW_LIMIT).then(|| {
        let mime = content_type.unwrap_or("image/png").split(';').next().unwrap_or("image/png").trim();
        format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(head))
    });
    if is_textual(content_type, head) {
        let cut = head.len().min(PREVIEW_LIMIT);
        ResponseBody { kind: BodyKind::Text, text: Some(String::from_utf8_lossy(&head[..cut]).into_owned()), truncated: bytes as usize > cut, image_data_url, file_path, bytes }
    } else {
        ResponseBody { kind: BodyKind::Binary, text: None, truncated: false, image_data_url, file_path, bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls() {
        assert_eq!(parse_url("localhost:3000/health").unwrap().as_str(), "http://localhost:3000/health");
        assert!(matches!(parse_url("{{base_url}}/x"), Err(AppError::InvalidRequest(m)) if m.contains("{{base_url}}")));
        assert!(parse_url("ws://x").is_err());
        assert!(parse_url("").is_err());
        assert!(parse_method("get").is_ok());
        assert!(parse_method("GE T").is_err());
    }

    #[test]
    fn text_detection() {
        assert!(is_textual(Some("application/json; charset=utf-8"), b""));
        assert!(is_textual(Some("application/problem+json"), b""));
        assert!(!is_textual(Some("application/octet-stream"), b"abc"));
        assert!(is_textual(None, b"plain"));
        assert!(!is_textual(None, b"\x89PNG\0\0"));
    }
}
