mod db;
mod logs;
mod notes;
mod open_behaviour;
mod page;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::db::Db;
use crate::{PAGE_PATHS, app};

fn test_app(name: &str) -> Router {
    let dir = std::env::temp_dir().join(format!("agentic-workspace-test-{name}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<!doctype html>").unwrap();
    std::fs::write(dir.join("app.js"), "export {}").unwrap();
    let dir = dir.to_str().unwrap();
    app(Arc::new(Db::open(":memory:").unwrap()), dir)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn put_json(body: &str) -> Request<Body> {
    Request::put("/page")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn error_of(body: &str) -> Value {
    serde_json::from_str::<Value>(body).unwrap()["error"].clone()
}

#[tokio::test]
async fn page_paths_return_index_html() {
    let app = test_app("pages");
    for path in PAGE_PATHS {
        assert_eq!(
            send(&app, get(path)).await,
            (StatusCode::OK, "<!doctype html>".to_string())
        );
    }
}

#[tokio::test]
async fn other_get_returns_web_file() {
    let app = test_app("file");
    assert_eq!(
        send(&app, get("/app.js")).await,
        (StatusCode::OK, "export {}".to_string())
    );
    let (status, body) = send(&app, get("/missing.js")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(error_of(&body).is_string());
}

#[tokio::test]
async fn save_then_load_page() {
    let app = test_app("save");
    assert_eq!(
        send(&app, get("/page?name=todo-items")).await,
        (StatusCode::OK, "null".to_string())
    );
    let saved = send(
        &app,
        put_json(r#"{"name":"todo-items","data":{"items":[{"text":"a"}]}}"#),
    )
    .await;
    assert_eq!(saved, (StatusCode::OK, "null".to_string()));
    let (status, body) = send(&app, get("/page?name=todo-items")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&body).unwrap(),
        json!({ "items": [{ "text": "a" }] })
    );
}

#[tokio::test]
async fn bad_requests_return_json_errors() {
    let app = test_app("errors");
    let (status, body) = send(&app, get("/page?name=a_b")).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad name."))
    );

    let (status, body) = send(&app, get("/page")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error_of(&body).is_string());

    let (status, body) = send(&app, get("/open-behaviour-progress?chat=..%2Fapp")).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad chat id."))
    );

    let followup = Request::post("/open-behaviour-followup")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            r#"{"chat":"../app","text":"Also fix the test."}"#,
        ))
        .unwrap();
    let (status, body) = send(&app, followup).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad chat id."))
    );

    let empty_followup = Request::post("/open-behaviour-followup")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"chat":"local-1","text":"  "}"#))
        .unwrap();
    let (status, body) = send(&app, empty_followup).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("The follow-up has no text."))
    );

    let empty_note = Request::post("/notes-assign")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"tab":"my-app","text":"  "}"#))
        .unwrap();
    let (status, body) = send(&app, empty_note).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("The note has no text."))
    );

    let (status, body) = send(&app, get("/logs-events?app=my-app&date=2026-10-31&page=0")).await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad page."))
    );

    let (status, body) = send(
        &app,
        get("/logs-events?app=my-app&date=2026-10-31&page=1&page_size=10"),
    )
    .await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad page size."))
    );

    let (status, body) = send(
        &app,
        get("/logs-events?app=my-app&date=2026-10-31&page=1&filters=not-json"),
    )
    .await;
    assert_eq!(
        (status, error_of(&body)),
        (StatusCode::BAD_REQUEST, json!("Bad filters."))
    );

    let text = Request::put("/page")
        .body(Body::from(r#"{"name":"a","data":1}"#))
        .unwrap();
    let (status, body) = send(&app, text).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(error_of(&body).is_string());

    let (status, body) = send(&app, put_json(r#"{"name":"a"}"#)).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(error_of(&body).is_string());
}
