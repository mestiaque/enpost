//! `Set-Cookie` parsing for the response viewer (RFC 6265 attributes).

use super::dto::ResponseCookie;

pub fn parse_set_cookie(header: &str) -> Option<ResponseCookie> {
    let mut parts = header.split(';');
    let (name, value) = parts.next()?.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut c = ResponseCookie {
        name: name.to_string(),
        value: value.trim().trim_matches('"').to_string(),
        domain: None,
        path: None,
        expires: None,
        max_age: None,
        secure: false,
        http_only: false,
        same_site: None,
    };
    for attr in parts {
        let (k, v) = match attr.split_once('=') {
            Some((k, v)) => (k.trim(), Some(v.trim())),
            None => (attr.trim(), None),
        };
        match k.to_ascii_lowercase().as_str() {
            "domain" => c.domain = v.map(|d| d.trim_start_matches('.').to_string()),
            "path" => c.path = v.map(str::to_string),
            "expires" => c.expires = v.map(str::to_string),
            "max-age" => c.max_age = v.and_then(|x| x.parse().ok()),
            "secure" => c.secure = true,
            "httponly" => c.http_only = true,
            "samesite" => c.same_site = v.map(str::to_string),
            _ => {}
        }
    }
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attributes() {
        let c = parse_set_cookie("sid=abc123; Path=/; Domain=.example.com; Max-Age=3600; Secure; HttpOnly; SameSite=Lax").unwrap();
        assert_eq!((c.name.as_str(), c.value.as_str()), ("sid", "abc123"));
        assert_eq!(c.domain.as_deref(), Some("example.com"));
        assert_eq!((c.max_age, c.secure, c.http_only, c.same_site.as_deref()), (Some(3600), true, true, Some("Lax")));
        assert!(parse_set_cookie("novalue").is_none());
    }
}
