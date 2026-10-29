mod db;
mod page;

use std::sync::Arc;

use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{Json, Router};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

use crate::db::Db;

const ADDR: &str = "127.0.0.1:47322";
const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
const WEB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/web");
const CONF_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conf");
// the browser routes these paths, so each one returns index.html
const PAGE_PATHS: [&str; 3] = ["/", "/todo", "/product-decision"];

#[tokio::main]
async fn main() -> Result<(), String> {
    std::fs::create_dir_all(DATA_DIR).map_err(|e| format!("failed to create {DATA_DIR}: {e}"))?;
    let db = Db::open(&format!("{DATA_DIR}/app.db"))?;
    let listener = tokio::net::TcpListener::bind(ADDR).await.map_err(|e| format!("failed to bind {ADDR}: {e}"))?;
    println!("listening on http://{ADDR}");
    axum::serve(listener, app(Arc::new(db), WEB_DIR, CONF_DIR)).await.map_err(|e| format!("server failed: {e}"))
}

fn app(db: Arc<Db>, web_dir: &str, conf_dir: &str) -> Router {
    let index = ServeFile::new(format!("{web_dir}/index.html"));
    let mut router = Router::new()
        .route("/page", get(page::load).put(page::save))
        .route("/decisions", get_service(ServeFile::new(format!("{conf_dir}/decisions.json"))));
    for path in PAGE_PATHS {
        router = router.route(path, get_service(index.clone()));
    }
    router
        // the files are read from the disk on each request, so a web edit needs no restart
        .fallback_service(ServeDir::new(web_dir))
        .layer(axum::middleware::map_response(json_error))
        .with_state(db)
}

// axum and tower-http answer their own errors, such as 404, 415 and 422, in plain text
async fn json_error(res: Response) -> Response {
    let status = res.status();
    let is_json = res.headers().get(CONTENT_TYPE).is_some_and(|v| v == "application/json");
    if !(status.is_client_error() || status.is_server_error()) || is_json {
        return res;
    }
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap_or_default();
    let mut msg = String::from_utf8_lossy(&body).trim_end().to_string();
    if msg.is_empty() {
        msg = status.canonical_reason().unwrap_or("Error").to_string();
    }
    (status, Json(json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    fn test_app(name: &str) -> Router {
        let dir = std::env::temp_dir().join(format!("agentic-workspace-test-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "<!doctype html>").unwrap();
        std::fs::write(dir.join("app.js"), "export {}").unwrap();
        std::fs::write(dir.join("decisions.json"), "[]").unwrap();
        let dir = dir.to_str().unwrap();
        app(Arc::new(Db::open(":memory:").unwrap()), dir, dir)
    }

    async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    fn get(uri: &str) -> Request<Body> {
        Request::get(uri).body(Body::empty()).unwrap()
    }

    fn put_json(body: &str) -> Request<Body> {
        Request::put("/page").header(CONTENT_TYPE, "application/json").body(Body::from(body.to_string())).unwrap()
    }

    fn error_of(body: &str) -> Value {
        serde_json::from_str::<Value>(body).unwrap()["error"].clone()
    }

    #[tokio::test]
    async fn page_paths_return_index_html() {
        let app = test_app("pages");
        for path in PAGE_PATHS {
            assert_eq!(send(&app, get(path)).await, (StatusCode::OK, "<!doctype html>".to_string()));
        }
    }

    #[tokio::test]
    async fn other_get_returns_web_file() {
        let app = test_app("file");
        assert_eq!(send(&app, get("/app.js")).await, (StatusCode::OK, "export {}".to_string()));
        let (status, body) = send(&app, get("/missing.js")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(error_of(&body).is_string());
    }

    #[tokio::test]
    async fn decisions_returns_conf_file() {
        let app = test_app("decisions");
        assert_eq!(send(&app, get("/decisions")).await, (StatusCode::OK, "[]".to_string()));
    }

    #[tokio::test]
    async fn save_then_load_page() {
        let app = test_app("save");
        assert_eq!(send(&app, get("/page?name=todo-items")).await, (StatusCode::OK, "null".to_string()));
        let saved = send(&app, put_json(r#"{"name":"todo-items","data":{"items":[{"text":"a"}]}}"#)).await;
        assert_eq!(saved, (StatusCode::OK, "null".to_string()));
        let (status, body) = send(&app, get("/page?name=todo-items")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(serde_json::from_str::<Value>(&body).unwrap(), json!({ "items": [{ "text": "a" }] }));
    }

    #[tokio::test]
    async fn bad_requests_return_json_errors() {
        let app = test_app("errors");
        let (status, body) = send(&app, get("/page?name=a_b")).await;
        assert_eq!((status, error_of(&body)), (StatusCode::BAD_REQUEST, json!("Bad name.")));

        let (status, body) = send(&app, get("/page")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(error_of(&body).is_string());

        let text = Request::put("/page").body(Body::from(r#"{"name":"a","data":1}"#)).unwrap();
        let (status, body) = send(&app, text).await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert!(error_of(&body).is_string());

        let (status, body) = send(&app, put_json(r#"{"name":"a"}"#)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(error_of(&body).is_string());
    }
}
