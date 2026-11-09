//! Redaction of sensitive values before anything is logged or stored in history.
//! Names are matched case-insensitively; values are replaced, never partially kept.

pub const REDACTED: &str = "[redacted]";

/// Header names whose values are secrets.
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization", "proxy-authorization", "cookie", "set-cookie", "x-api-key", "api-key", "x-auth-token", "x-access-token",
    "x-csrf-token", "x-xsrf-token",
];

/// Substrings that mark a query parameter, form field or variable name as secret.
const SENSITIVE_PARTS: &[&str] = &["token", "secret", "password", "passwd", "apikey", "api_key", "api-key", "auth", "signature", "session", "key"];

pub fn is_sensitive_header(name: &str) -> bool {
    let n = name.trim().to_ascii_lowercase();
    SENSITIVE_HEADERS.contains(&n.as_str()) || is_sensitive_name(&n)
}

pub fn is_sensitive_name(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    SENSITIVE_PARTS.iter().any(|p| n.contains(p))
}

/// Header pairs with secret values replaced.
pub fn headers(pairs: &[(String, String)]) -> Vec<(String, String)> {
    pairs.iter().map(|(k, v)| (k.clone(), if is_sensitive_header(k) { REDACTED.into() } else { v.clone() })).collect()
}

/// A URL with user info and secret-looking query values replaced. Works on plain strings so
/// that half-typed URLs (and `{{variables}}`) are handled too.
pub fn url(raw: &str) -> String {
    let (before_fragment, fragment) = match raw.split_once('#') {
        Some((a, f)) => (a, Some(f)),
        None => (raw, None),
    };
    let (base, query) = match before_fragment.split_once('?') {
        Some((b, q)) => (b, Some(q)),
        None => (before_fragment, None),
    };
    // scheme://user:pass@host → scheme://[redacted]@host
    let base = match base.find("://") {
        Some(i) => {
            let (scheme, rest) = base.split_at(i + 3);
            let host_end = rest.find('/').unwrap_or(rest.len());
            match rest[..host_end].rfind('@') {
                Some(at) => format!("{scheme}{REDACTED}{}", &rest[at..]),
                None => base.to_string(),
            }
        }
        None => base.to_string(),
    };
    let mut out = base;
    if let Some(q) = query {
        let parts: Vec<String> = q
            .split('&')
            .map(|kv| match kv.split_once('=') {
                Some((k, _)) if is_sensitive_name(k) => format!("{k}={REDACTED}"),
                _ => kv.to_string(),
            })
            .collect();
        out.push('?');
        out.push_str(&parts.join("&"));
    }
    if let Some(f) = fragment {
        out.push('#');
        out.push_str(f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secret_headers() {
        let h = headers(&[
            ("Authorization".into(), "Bearer abc".into()),
            ("Content-Type".into(), "application/json".into()),
            ("X-API-Key".into(), "k".into()),
            ("Cookie".into(), "sid=1".into()),
            ("X-Session-Id".into(), "s".into()),
        ]);
        assert_eq!(h[0].1, REDACTED);
        assert_eq!(h[1].1, "application/json");
        assert!(h[2..].iter().all(|(_, v)| v == REDACTED));
    }

    #[test]
    fn redacts_urls() {
        assert_eq!(url("https://u:p@api.example.com/v1?page=2&access_token=xyz#top"), "https://[redacted]@api.example.com/v1?page=2&access_token=[redacted]#top");
        assert_eq!(url("{{base_url}}/users?apiKey={{key}}&q=a"), "{{base_url}}/users?apiKey=[redacted]&q=a");
        assert_eq!(url("http://localhost:3000/health"), "http://localhost:3000/health");
        // An @ in the path is not user info.
        assert_eq!(url("https://example.com/@user"), "https://example.com/@user");
    }
}
