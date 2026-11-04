mod db;
mod logs;
mod notes;
mod open_behaviour;
mod page;

#[cfg(test)]
mod test;

use std::sync::Arc;

use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, get_service, post};
use axum::{Json, Router};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

use crate::db::Db;

const ADDR: &str = "127.0.0.1:47322";
const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
const WEB_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/web");
// the browser routes these paths, so each one returns index.html
pub(crate) const PAGE_PATHS: [&str; 5] = ["/", "/todo", "/open-behaviour", "/notes", "/logs"];

#[tokio::main]
async fn main() -> Result<(), String> {
    std::fs::create_dir_all(DATA_DIR).map_err(|e| format!("failed to create {DATA_DIR}: {e}"))?;
    let db = Db::open(&format!("{DATA_DIR}/app.db"))?;
    let listener = tokio::net::TcpListener::bind(ADDR).await.map_err(|e| format!("failed to bind {ADDR}: {e}"))?;
    println!("listening on http://{ADDR}");
    axum::serve(listener, app(Arc::new(db), WEB_DIR)).await.map_err(|e| format!("server failed: {e}"))
}

pub(crate) fn app(db: Arc<Db>, web_dir: &str) -> Router {
    let index = ServeFile::new(format!("{web_dir}/index.html"));
    let mut router = Router::new()
        .route("/page", get(page::load).put(page::save))
        .route("/projects", get(notes::projects))
        .route("/notes-assign", post(notes::assign))
        .route("/logs-apps", get(logs::apps))
        .route("/logs-events", get(logs::load))
        .route("/open-behaviour-items", get(open_behaviour::load))
        .route("/open-behaviour-resolve", post(open_behaviour::resolve))
        .route("/open-behaviour-progress", get(open_behaviour::progress))
        .route("/open-behaviour-followup", post(open_behaviour::followup));
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
