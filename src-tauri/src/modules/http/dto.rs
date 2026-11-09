//! Request and response DTOs shared with the frontend (TypeScript bindings via ts-rs).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FieldKind {
    Text,
    /// `value` is an absolute path the user picked.
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FormField {
    pub name: String,
    pub value: String,
    pub kind: FieldKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
#[ts(export)]
pub enum RequestBody {
    None,
    /// JSON, XML or plain text; `content_type` is sent unless the user set one.
    Raw { content_type: String, text: String },
    UrlEncoded { fields: Vec<FormField> },
    FormData { fields: Vec<FormField> },
    /// A file sent as the whole body.
    Binary { path: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RequestOptions {
    /// 0 = no timeout.
    pub timeout_ms: u32,
    pub follow_redirects: bool,
    pub max_redirects: u32,
    pub verify_tls: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HttpRequest {
    /// Chosen by the frontend, so it can cancel before the call returns.
    pub request_id: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<HeaderPair>,
    pub body: RequestBody,
    pub options: RequestOptions,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResponseCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<String>,
    #[ts(type = "number | null")]
    pub max_age: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Timing {
    /// Until the response headers arrived (includes DNS, connect, TLS and server time).
    #[ts(type = "number")]
    pub ttfb_ms: u64,
    /// Receiving the body.
    #[ts(type = "number")]
    pub download_ms: u64,
    #[ts(type = "number")]
    pub total_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum BodyKind {
    Empty,
    Text,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResponseBody {
    pub kind: BodyKind,
    /// Text bodies, up to the preview limit (UTF-8; invalid bytes replaced).
    pub text: Option<String>,
    /// The text was cut at the preview limit; the full body is in `file_path`.
    pub truncated: bool,
    /// `data:` URL for images up to the image preview limit.
    pub image_data_url: Option<String>,
    /// The full body on disk (save/export reads it from here).
    pub file_path: Option<String>,
    #[ts(type = "number")]
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HttpResponse {
    pub request_id: String,
    pub history_id: String,
    /// Final URL after redirects.
    pub url: String,
    pub status: u16,
    pub status_text: String,
    pub http_version: String,
    pub headers: Vec<HeaderPair>,
    pub cookies: Vec<ResponseCookie>,
    pub content_type: Option<String>,
    pub timing: Timing,
    #[ts(type = "number")]
    pub headers_bytes: u64,
    pub body: ResponseBody,
}

/// Sent while a request runs (event `http-progress`).
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HttpProgress {
    pub request_id: String,
    /// "waiting" (sent, no headers yet) | "receiving"
    pub stage: String,
    #[ts(type = "number")]
    pub received_bytes: u64,
    #[ts(type = "number | null")]
    pub total_bytes: Option<u64>,
}
