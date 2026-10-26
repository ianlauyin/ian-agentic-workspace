pub mod load;
pub mod save;

use std::sync::{Mutex, MutexGuard};

use axum::Json;
use axum::http::StatusCode;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{Value, json};

pub type Fail = (StatusCode, Json<Value>);

pub fn bad(msg: &str) -> Fail {
    (StatusCode::BAD_REQUEST, Json(json!({ "error": msg })))
}

pub fn fail(msg: impl std::fmt::Display) -> Fail {
    (StatusCode::BAD_GATEWAY, Json(json!({ "error": msg.to_string() })))
}

pub fn check_name(name: &str) -> Result<(), Fail> {
    let ok = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    if ok { Ok(()) } else { Err(bad("Bad name.")) }
}

// a panic in a different request poisons the lock, but the connection stays usable
pub fn lock(db: &Mutex<Connection>) -> MutexGuard<'_, Connection> {
    db.lock().unwrap_or_else(|e| e.into_inner())
}

// outside src/, so a save does not make cargo watch restart the server
pub fn open() -> Result<Connection, String> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/data");
    std::fs::create_dir_all(dir).map_err(|e| format!("failed to create {dir}: {e}"))?;
    let db = Connection::open(format!("{dir}/app.db")).map_err(|e| format!("failed to open {dir}/app.db: {e}"))?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS page (name TEXT PRIMARY KEY, data TEXT NOT NULL, saved_at INTEGER NOT NULL)")
        .map_err(|e| format!("failed to create table page: {e}"))?;
    Ok(db)
}

pub fn get(db: &Connection, name: &str) -> Result<Value, String> {
    let data: Option<String> =
        db.query_row("SELECT data FROM page WHERE name = ?1", [name], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
    let Some(data) = data else { return Ok(Value::Null) };
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

pub fn put(db: &Connection, name: &str, data: &Value) -> Result<(), String> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    db.execute(
        "INSERT INTO page (name, data, saved_at) VALUES (?1, ?2, ?3) ON CONFLICT (name) DO UPDATE SET data = ?2, saved_at = ?3",
        rusqlite::params![name, data.to_string(), now],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
