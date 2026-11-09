//! One error type for every IPC command. It reaches the frontend as
//! `{ kind, message }` (see `src/types/bindings/AppErrorPayload.ts`), never as a bare string,
//! so the UI can react to the kind and show the message.

use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("{0}")]
    Validation(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("File error: {0}")]
    Io(String),
    #[error("{0}")]
    Internal(String),
    // Sending requests (Phase 2). Each kind gets its own message in the response panel.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Could not connect: {0}")]
    Connect(String),
    #[error("The request timed out after {0} ms")]
    Timeout(u64),
    #[error("TLS/certificate problem: {0}")]
    Tls(String),
    #[error("Too many redirects: {0}")]
    Redirect(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Request cancelled")]
    Cancelled,
}

pub type AppResult<T> = Result<T, AppError>;

/// What the frontend receives when a command fails.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "AppErrorPayload")]
pub struct ErrorPayload {
    /// "database" | "validation" | "notFound" | "io" | "internal" | "invalidRequest" | "connect"
    /// | "timeout" | "tls" | "redirect" | "network" | "cancelled"
    pub kind: String,
    pub message: String,
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Database(_) => "database",
            AppError::Validation(_) => "validation",
            AppError::NotFound(_) => "notFound",
            AppError::Io(_) => "io",
            AppError::Internal(_) => "internal",
            AppError::InvalidRequest(_) => "invalidRequest",
            AppError::Connect(_) => "connect",
            AppError::Timeout(_) => "timeout",
            AppError::Tls(_) => "tls",
            AppError::Redirect(_) => "redirect",
            AppError::Network(_) => "network",
            AppError::Cancelled => "cancelled",
        }
    }

    pub fn payload(&self) -> ErrorPayload {
        ErrorPayload { kind: self.kind().into(), message: self.to_string() }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.payload().serialize(s)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("record".into()),
            other => AppError::Database(other.to_string()),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for AppError {
    fn from(e: sqlx::migrate::MigrateError) -> Self {
        AppError::Database(format!("migration failed: {e}"))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal(format!("invalid data: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_kind_and_message() {
        let v = serde_json::to_value(AppError::Validation("Accent must be a #rrggbb colour".into())).unwrap();
        assert_eq!(v, serde_json::json!({ "kind": "validation", "message": "Accent must be a #rrggbb colour" }));
        assert_eq!(AppError::from(sqlx::Error::RowNotFound).kind(), "notFound");
    }
}
