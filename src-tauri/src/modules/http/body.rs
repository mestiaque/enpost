//! Request bodies. Files come from paths the user picked; they are read at send time.

use super::dto::{FieldKind, FormField, RequestBody};
use crate::core::error::{AppError, AppResult};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest::RequestBuilder;
use std::path::Path;

async fn read_file(path: &str) -> AppResult<(Vec<u8>, String)> {
    let p = Path::new(path);
    if !p.is_absolute() {
        return Err(AppError::InvalidRequest(format!("\"{path}\" is not a full file path")));
    }
    let meta = tokio::fs::metadata(p).await.map_err(|e| AppError::InvalidRequest(format!("cannot read {path}: {e}")))?;
    if !meta.is_file() {
        return Err(AppError::InvalidRequest(format!("{path} is not a file")));
    }
    let bytes = tokio::fs::read(p).await.map_err(|e| AppError::InvalidRequest(format!("cannot read {path}: {e}")))?;
    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
    Ok((bytes, name))
}

fn pairs(fields: &[FormField]) -> AppResult<Vec<(String, String)>> {
    fields
        .iter()
        .filter(|f| !f.name.is_empty())
        .map(|f| match f.kind {
            FieldKind::Text => Ok((f.name.clone(), f.value.clone())),
            FieldKind::File => Err(AppError::InvalidRequest("files can only be sent as form data".into())),
        })
        .collect()
}

pub async fn build_body(rb: RequestBuilder, body: &RequestBody, user_headers: &HeaderMap) -> AppResult<RequestBuilder> {
    let has_ct = user_headers.contains_key(CONTENT_TYPE);
    Ok(match body {
        RequestBody::None => rb,
        RequestBody::Raw { content_type, text } => {
            let rb = if has_ct || content_type.is_empty() { rb } else { rb.header(CONTENT_TYPE, content_type.as_str()) };
            rb.body(text.clone())
        }
        RequestBody::UrlEncoded { fields } => {
            let encoded = form_urlencoded(&pairs(fields)?);
            let rb = if has_ct { rb } else { rb.header(CONTENT_TYPE, "application/x-www-form-urlencoded") };
            rb.body(encoded)
        }
        RequestBody::FormData { fields } => {
            // reqwest sets the multipart Content-Type with its boundary; a user-set one would
            // lack the boundary and break the request, so it is replaced.
            let mut form = Form::new();
            for f in fields.iter().filter(|f| !f.name.is_empty()) {
                form = match f.kind {
                    FieldKind::Text => form.text(f.name.clone(), f.value.clone()),
                    FieldKind::File => {
                        let (bytes, name) = read_file(&f.value).await?;
                        form.part(f.name.clone(), Part::bytes(bytes).file_name(name))
                    }
                };
            }
            rb.multipart(form)
        }
        RequestBody::Binary { path } => {
            let (bytes, _) = read_file(path).await?;
            let rb = if has_ct { rb } else { rb.header(CONTENT_TYPE, "application/octet-stream") };
            rb.body(bytes)
        }
    })
}

/// application/x-www-form-urlencoded (spaces as +, as browsers send forms).
pub fn form_urlencoded(pairs: &[(String, String)]) -> String {
    fn enc(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'*' => out.push(b as char),
                b' ' => out.push('+'),
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }
    pairs.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect::<Vec<_>>().join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoding() {
        assert_eq!(form_urlencoded(&[("a b".into(), "x&y=z".into()), ("ü".into(), "".into())]), "a+b=x%26y%3Dz&%C3%BC=");
    }
}
