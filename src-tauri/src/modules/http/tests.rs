//! Real requests against a local test server (no internet needed).

use super::engine::{PREVIEW_LIMIT, IMAGE_PREVIEW_LIMIT};
use super::*;
use crate::core::error::AppError;
use axum::body::Body;
use axum::extract::{Path as AxPath, Request};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{any, get};
use axum::Router;
use std::sync::Arc;
use std::time::Duration;

async fn echo(req: Request) -> impl IntoResponse {
    let method = req.method().to_string();
    let headers: serde_json::Map<String, serde_json::Value> =
        req.headers().iter().map(|(k, v)| (k.to_string(), serde_json::Value::from(v.to_str().unwrap_or("")))).collect();
    let body = axum::body::to_bytes(req.into_body(), 10_000_000).await.unwrap();
    axum::Json(serde_json::json!({ "method": method, "headers": headers, "body": String::from_utf8_lossy(&body) }))
}

async fn server() -> String {
    let app = Router::new()
        .route(
            "/json",
            get(|| async { ([(header::CONTENT_TYPE, "application/json"), (header::SET_COOKIE, "sid=abc; Path=/; HttpOnly")], r#"{"hello":"world"}"#) }),
        )
        .route("/echo", any(echo))
        .route(
            "/redirect/{n}",
            get(|AxPath(n): AxPath<u32>| async move { if n == 0 { Redirect::temporary("/json") } else { Redirect::temporary(&format!("/redirect/{}", n - 1)) } }),
        )
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                "late"
            }),
        )
        .route("/big", get(|| async { ([(header::CONTENT_TYPE, "text/plain")], "x".repeat(5 * 1024 * 1024)) }))
        .route("/png", get(|| async { ([(header::CONTENT_TYPE, "image/png")], vec![0x89u8, b'P', b'N', b'G', 0, 1, 2]) }))
        .route("/bin", get(|| async { ([(header::CONTENT_TYPE, "application/octet-stream")], vec![0u8, 1, 2, 3]) }))
        .route("/empty", get(|| async { StatusCode::NO_CONTENT }))
        .route("/status/{code}", get(|AxPath(code): AxPath<u16>| async move { Response::builder().status(code).body(Body::from("s")).unwrap() }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

fn req(id: &str, method: &str, url: String) -> HttpRequest {
    HttpRequest {
        request_id: id.into(),
        method: method.into(),
        url,
        headers: vec![],
        body: RequestBody::None,
        options: RequestOptions { timeout_ms: 10_000, follow_redirects: true, max_redirects: 10, verify_tls: true },
    }
}

fn engine() -> (Arc<HttpEngine>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    (Arc::new(HttpEngine::new(dir.path().join("bodies"))), dir)
}

#[tokio::test]
async fn get_json_with_cookies_and_timing() {
    let base = server().await;
    let (e, _d) = engine();
    let r = e.send(&req("1", "GET", format!("{base}/json")), "", "h1", &NoProgress).await.unwrap();
    assert_eq!((r.status, r.status_text.as_str()), (200, "OK"));
    assert_eq!(r.body.kind, BodyKind::Text);
    assert_eq!(r.body.text.as_deref(), Some(r#"{"hello":"world"}"#));
    assert_eq!(r.body.bytes, 17);
    assert_eq!(r.cookies[0].name, "sid");
    assert!(r.cookies[0].http_only);
    assert!(r.timing.total_ms >= r.timing.ttfb_ms);
    assert_eq!(std::fs::read(r.body.file_path.unwrap()).unwrap(), br#"{"hello":"world"}"#);
    assert!(!e.is_running("1"));
}

#[tokio::test]
async fn bodies_and_headers_reach_the_server() {
    let base = server().await;
    let (e, d) = engine();
    let echo = |r: HttpResponse| serde_json::from_str::<serde_json::Value>(r.body.text.as_deref().unwrap()).unwrap();

    let mut q = req("raw", "post", format!("{base}/echo"));
    q.headers = vec![HeaderPair { name: "X-Trace".into(), value: "42".into() }, HeaderPair { name: "".into(), value: "ignored".into() }];
    q.body = RequestBody::Raw { content_type: "application/json".into(), text: r#"{"a":1}"#.into() };
    let v = echo(e.send(&q, "", "h-raw", &NoProgress).await.unwrap());
    assert_eq!(v["method"], "POST");
    assert_eq!(v["headers"]["content-type"], "application/json");
    assert_eq!(v["headers"]["x-trace"], "42");
    assert_eq!(v["body"], r#"{"a":1}"#);

    // A Content-Type the user set wins over the body type's default.
    q.headers = vec![HeaderPair { name: "Content-Type".into(), value: "application/vnd.api+json".into() }];
    let v = echo(e.send(&q, "", "h-raw2", &NoProgress).await.unwrap());
    assert_eq!(v["headers"]["content-type"], "application/vnd.api+json");

    q.headers = vec![];
    q.body = RequestBody::UrlEncoded { fields: vec![FormField { name: "q".into(), value: "a b&c".into(), kind: FieldKind::Text }] };
    let v = echo(e.send(&q, "", "h-form", &NoProgress).await.unwrap());
    assert_eq!(v["body"], "q=a+b%26c");
    assert_eq!(v["headers"]["content-type"], "application/x-www-form-urlencoded");

    let file = d.path().join("upload.txt");
    std::fs::write(&file, "file-content").unwrap();
    q.body = RequestBody::FormData {
        fields: vec![
            FormField { name: "title".into(), value: "hello".into(), kind: FieldKind::Text },
            FormField { name: "doc".into(), value: file.display().to_string(), kind: FieldKind::File },
        ],
    };
    let v = echo(e.send(&q, "", "h-multi", &NoProgress).await.unwrap());
    let body = v["body"].as_str().unwrap();
    assert!(v["headers"]["content-type"].as_str().unwrap().starts_with("multipart/form-data; boundary="));
    assert!(body.contains("name=\"title\"") && body.contains("hello"));
    assert!(body.contains("filename=\"upload.txt\"") && body.contains("file-content"));

    q.body = RequestBody::Binary { path: file.display().to_string() };
    let v = echo(e.send(&q, "", "h-bin", &NoProgress).await.unwrap());
    assert_eq!(v["body"], "file-content");
    assert_eq!(v["headers"]["content-type"], "application/octet-stream");

    q.body = RequestBody::Binary { path: "relative.txt".into() };
    assert!(matches!(e.send(&q, "", "h-bad", &NoProgress).await, Err(AppError::InvalidRequest(_))));
}

#[tokio::test]
async fn redirects_follow_stop_and_limit() {
    let base = server().await;
    let (e, _d) = engine();
    let r = e.send(&req("a", "GET", format!("{base}/redirect/2")), "", "r1", &NoProgress).await.unwrap();
    assert_eq!(r.status, 200);
    assert!(r.url.ends_with("/json"));

    let mut q = req("b", "GET", format!("{base}/redirect/2"));
    q.options.follow_redirects = false;
    let r = e.send(&q, "", "r2", &NoProgress).await.unwrap();
    assert_eq!(r.status, 307);
    assert!(r.headers.iter().any(|h| h.name == "location"));

    let mut q = req("c", "GET", format!("{base}/redirect/5"));
    q.options.max_redirects = 2;
    assert!(matches!(e.send(&q, "", "r3", &NoProgress).await, Err(AppError::Redirect(_))));
}

#[tokio::test]
async fn timeout_and_cancel() {
    let base = server().await;
    let (e, _d) = engine();
    let mut q = req("t", "GET", format!("{base}/slow"));
    q.options.timeout_ms = 300;
    assert!(matches!(e.send(&q, "", "t1", &NoProgress).await, Err(AppError::Timeout(300))));

    let q = req("c", "GET", format!("{base}/slow"));
    let e2 = e.clone();
    let task = tokio::spawn(async move { e2.send(&q, "", "c1", &NoProgress).await });
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(e.is_running("c"));
    assert!(e.cancel("c"));
    let started = std::time::Instant::now();
    assert!(matches!(task.await.unwrap(), Err(AppError::Cancelled)));
    assert!(started.elapsed() < Duration::from_secs(1), "cancel must not wait for the server");
    assert!(!e.is_running("c"));
    assert!(!e.body_path("c1").exists(), "a cancelled request leaves no body file");
    assert!(!e.cancel("c"), "nothing left to cancel");
}

#[tokio::test]
async fn large_binary_image_and_empty_bodies() {
    let base = server().await;
    let (e, _d) = engine();
    let r = e.send(&req("big", "GET", format!("{base}/big")), "", "b1", &NoProgress).await.unwrap();
    assert_eq!(r.body.bytes, 5 * 1024 * 1024);
    assert!(r.body.truncated);
    assert_eq!(r.body.text.as_ref().unwrap().len(), PREVIEW_LIMIT);
    assert_eq!(std::fs::metadata(r.body.file_path.unwrap()).unwrap().len(), 5 * 1024 * 1024, "the full body is on disk");

    let r = e.send(&req("png", "GET", format!("{base}/png")), "", "b2", &NoProgress).await.unwrap();
    assert_eq!(r.body.kind, BodyKind::Binary);
    assert!(r.body.image_data_url.as_deref().unwrap().starts_with("data:image/png;base64,iVBORw"));
    let _ = IMAGE_PREVIEW_LIMIT;

    let r = e.send(&req("bin", "GET", format!("{base}/bin")), "", "b3", &NoProgress).await.unwrap();
    assert_eq!((r.body.kind, r.body.text.is_none(), r.body.image_data_url.is_none()), (BodyKind::Binary, true, true));

    let r = e.send(&req("e", "GET", format!("{base}/empty")), "", "b4", &NoProgress).await.unwrap();
    assert_eq!((r.status, r.body.kind), (204, BodyKind::Empty));

    let r = e.send(&req("s", "GET", format!("{base}/status/503")), "", "b5", &NoProgress).await.unwrap();
    assert_eq!((r.status, r.status_text.as_str()), (503, "Service Unavailable"));
}

#[tokio::test]
async fn connection_errors_are_classified() {
    // Bind and drop: nothing listens on this port any more.
    let port = std::net::TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let (e, _d) = engine();
    let r = e.send(&req("x", "GET", format!("http://127.0.0.1:{port}/")), "", "x1", &NoProgress).await;
    assert!(matches!(r, Err(AppError::Connect(_))), "{r:?}");
    let r = e.send(&req("y", "GET", "{{base_url}}/users".into()), "", "y1", &NoProgress).await;
    assert!(matches!(r, Err(AppError::InvalidRequest(_))));
    let mut q = req("z", "GET", "http://127.0.0.1:1/".into());
    q.headers = vec![HeaderPair { name: "Bad Header".into(), value: "v".into() }];
    assert!(matches!(e.send(&q, "", "z1", &NoProgress).await, Err(AppError::InvalidRequest(_))));
}

#[tokio::test]
async fn progress_is_reported_for_large_bodies() {
    struct Count(std::sync::Mutex<Vec<String>>);
    impl ProgressSink for Count {
        fn progress(&self, p: HttpProgress) {
            self.0.lock().unwrap().push(p.stage);
        }
    }
    let base = server().await;
    let (e, _d) = engine();
    let sink = Count(Default::default());
    e.send(&req("p", "GET", format!("{base}/big")), "", "p1", &sink).await.unwrap();
    let stages = sink.0.lock().unwrap();
    assert_eq!(stages.first().map(String::as_str), Some("waiting"));
}

#[tokio::test]
async fn history_is_recorded_redacted_and_pruned() {
    use crate::database::Database;
    use crate::modules::history;
    let base = server().await;
    let (e, d) = engine();
    let db = Database::open_in_memory().await.unwrap();
    let mut q = req("h", "GET", format!("{base}/json?access_token=SECRET&page=1"));
    q.headers = vec![HeaderPair { name: "Authorization".into(), value: "Bearer SECRET".into() }];
    let r = e.send(&q, "", "hist1", &NoProgress).await.unwrap();
    history::record(&db, "hist1", &q, 1_000, Ok(&r)).await.unwrap();
    history::record(&db, "hist2", &q, 2_000, Err("Could not connect: refused")).await.unwrap();

    let list = history::list(&db, 10).await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, "hist2", "newest first");
    assert_eq!(list[0].error.as_deref(), Some("Could not connect: refused"));
    assert_eq!(list[1].status, Some(200));
    for entry in &list {
        assert!(!entry.url.contains("SECRET") && !entry.snapshot.contains("SECRET"), "history must be redacted: {entry:?}");
    }
    let stored_headers: String = sqlx::query_scalar("SELECT headers FROM response_metadata WHERE history_id = 'hist1'").fetch_one(&db.pool).await.unwrap();
    assert!(!stored_headers.contains("sid=abc"), "Set-Cookie values are redacted");

    // Everything sent "in 1970" is older than 30 days: pruning removes rows and body files.
    let body = e.body_path("hist1");
    assert!(body.exists());
    history::prune(&db, 30).await.unwrap();
    assert!(history::list(&db, 10).await.unwrap().is_empty());
    assert!(!body.exists());
    drop(d);
}
