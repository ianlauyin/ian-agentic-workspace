mod page;

use std::sync::{Arc, Mutex};

use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service};
use axum::{Json, Router};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

const ADDR: &str = "127.0.0.1:47322";

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

// axum answers its own errors, such as 415 and 422, in plain text
async fn json_error(res: Response) -> Response {
    let status = res.status();
    let json = res.headers().get(CONTENT_TYPE).is_some_and(|v| v == "application/json");
    if !(status.is_client_error() || status.is_server_error()) || json {
        return res;
    }
    let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap_or_default();
    let mut msg = String::from_utf8_lossy(&body).trim_end().to_string();
    if msg.is_empty() {
        msg = status.canonical_reason().unwrap_or("Error").to_string();
    }
    (status, Json(json!({ "error": msg }))).into_response()
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let db = page::open()?;
    let state = Arc::new(AppState { db: Mutex::new(db) });

    // index.html routes the pages in the browser, so each page path answers it on GET
    let web = concat!(env!("CARGO_MANIFEST_DIR"), "/web");
    let index = ServeFile::new(format!("{web}/index.html"));
    let app = Router::new()
        .route("/", get_service(index.clone()))
        .route("/todo", get_service(index))
        .route("/page", get(page::load::handler).put(page::save::handler))
        // pages are read from disk on each request, so a page edit needs no rebuild
        .fallback_service(ServeDir::new(web))
        .layer(axum::middleware::map_response(json_error))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(ADDR).await.map_err(|e| format!("failed to bind {ADDR}: {e}"))?;
    println!("listening on http://{ADDR}");
    axum::serve(listener, app).await.map_err(|e| format!("server failed: {e}"))
}
